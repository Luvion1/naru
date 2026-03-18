use std::time::Duration;

pub trait CommandMiddleware: Send + Sync {
    fn before_execute(&self, context: &CommandContext) -> Result<(), MiddlewareError>;
    fn after_execute(&self, context: &CommandContext);
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub command_name: String,
    pub user_identifier: String,
    pub environment: Option<String>,
    pub ip_address: Option<String>,
    pub timestamp: std::time::Instant,
}

impl CommandContext {
    pub fn new(command_name: &str) -> Self {
        Self {
            command_name: command_name.to_string(),
            user_identifier: std::env::var("USER")
                .or_else(|_| std::env::var("USERNAME"))
                .unwrap_or_else(|_| "unknown".to_string()),
            environment: None,
            ip_address: None,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn with_environment(mut self, env: &str) -> Self {
        self.environment = Some(env.to_string());
        self
    }

    pub fn rate_limit_key(&self) -> String {
        format!("{}:{}", self.command_name, self.user_identifier)
    }
}

#[derive(Debug, Clone)]
pub enum MiddlewareError {
    RateLimited(Duration),
    PermissionDenied(String),
    ValidationFailed(String),
    Other(String),
}

impl std::fmt::Display for MiddlewareError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MiddlewareError::RateLimited(duration) => {
                write!(
                    f,
                    "Rate limited. Try again in {:.0}s",
                    duration.as_secs_f64()
                )
            }
            MiddlewareError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            MiddlewareError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            MiddlewareError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for MiddlewareError {}

pub struct MiddlewareChain {
    middlewares: Vec<Box<dyn CommandMiddleware>>,
}

impl MiddlewareChain {
    pub fn new() -> Self {
        Self {
            middlewares: Vec::new(),
        }
    }

    pub fn add<M: CommandMiddleware + 'static>(mut self, middleware: M) -> Self {
        self.middlewares.push(Box::new(middleware));
        self
    }

    pub fn before_execute(&self, context: &CommandContext) -> Result<(), MiddlewareError> {
        for middleware in &self.middlewares {
            middleware.before_execute(context)?;
        }
        Ok(())
    }

    pub fn execute<F, R>(&self, context: &CommandContext, mut f: F) -> Result<R, MiddlewareError>
    where
        F: FnMut() -> Result<R, Box<dyn std::error::Error>>,
    {
        self.before_execute(context)?;

        let result = f();

        // Call after_execute on each middleware
        for middleware in &self.middlewares {
            middleware.after_execute(context);
        }

        result.map_err(|e| MiddlewareError::Other(e.to_string()))
    }
}

impl Default for MiddlewareChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_context_creation() {
        let ctx = CommandContext::new("test_command");
        assert_eq!(ctx.command_name, "test_command");
    }

    #[test]
    fn test_command_context_with_environment() {
        let ctx = CommandContext::new("set").with_environment("production");

        assert_eq!(ctx.environment, Some("production".to_string()));
    }

    #[test]
    fn test_rate_limit_key() {
        let ctx = CommandContext::new("get").with_environment("production");

        let key = ctx.rate_limit_key();
        assert!(key.contains("get"));
    }

    #[test]
    fn test_middleware_chain_empty() {
        let chain = MiddlewareChain::new();
        let ctx = CommandContext::new("test");

        let result = chain.execute(&ctx, || Ok(42));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
