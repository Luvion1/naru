use crate::core::middleware::{CommandContext, CommandMiddleware, MiddlewareError};
use crate::core::rate_limiter::{RateLimitConfig, RateLimiter};
use std::sync::Arc;

pub struct RateLimitMiddleware {
    limiter: Arc<RateLimiter>,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: Arc::new(RateLimiter::new(config)),
        }
    }

    pub fn with_default() -> Self {
        Self {
            limiter: Arc::new(RateLimiter::with_default_config()),
        }
    }

    pub fn strict() -> Self {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 3;
        config.window_duration = std::time::Duration::from_secs(60);
        config.lockout_duration = std::time::Duration::from_secs(300);
        Self::new(config)
    }

    pub fn lenient() -> Self {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 10;
        config.window_duration = std::time::Duration::from_secs(60);
        config.lockout_duration = std::time::Duration::from_secs(60);
        Self::new(config)
    }
}

impl CommandMiddleware for RateLimitMiddleware {
    fn before_execute(&self, context: &CommandContext) -> Result<(), MiddlewareError> {
        let key = context.rate_limit_key();
        match self.limiter.check_rate_limit(&key) {
            Ok(()) => Ok(()),
            Err(e) => match e {
                crate::core::rate_limiter::RateLimitError::LockedOut(duration) => {
                    Err(MiddlewareError::RateLimited(duration))
                }
            },
        }
    }

    fn after_execute(&self, _context: &CommandContext) {}
}

pub struct AuditMiddleware {
    #[allow(dead_code)]
    log_path: String,
}

impl AuditMiddleware {
    pub fn new(log_path: &str) -> Self {
        Self {
            log_path: log_path.to_string(),
        }
    }
}

impl CommandMiddleware for AuditMiddleware {
    fn before_execute(&self, _context: &CommandContext) -> Result<(), MiddlewareError> {
        Ok(())
    }

    fn after_execute(&self, _context: &CommandContext) {}
}

pub struct ValidationMiddleware {
    #[allow(dead_code)]
    strict: bool,
}

impl ValidationMiddleware {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }
}

impl CommandMiddleware for ValidationMiddleware {
    fn before_execute(&self, _context: &CommandContext) -> Result<(), MiddlewareError> {
        Ok(())
    }

    fn after_execute(&self, _context: &CommandContext) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_middleware() {
        let middleware = RateLimitMiddleware::strict();
        let ctx = CommandContext::new("test");
        let result = middleware.before_execute(&ctx);
        assert!(result.is_ok());
    }

    #[test]
    fn test_audit_middleware() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let log_path = temp_dir.path().join("audit.log");
        let middleware = AuditMiddleware::new(log_path.to_str().unwrap());
        let ctx = CommandContext::new("test");
        middleware.after_execute(&ctx);
    }
}
