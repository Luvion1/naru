/// Rate Limiter for Naru
/// Prevents brute-force attacks by limiting operation frequency
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_attempts: u32,
    pub window_duration: Duration,
    pub lockout_duration: Duration,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts: 5,
            window_duration: Duration::from_secs(60),
            lockout_duration: Duration::from_secs(300),
        }
    }
}

#[derive(Debug, Clone)]
struct RateLimitState {
    attempts: Vec<Instant>,
    lockout_until: Option<Instant>,
}

impl RateLimitState {
    fn new() -> Self {
        Self {
            attempts: Vec::new(),
            lockout_until: None,
        }
    }
}

pub struct RateLimiter {
    config: RateLimitConfig,
    states: Arc<Mutex<HashMap<String, RateLimitState>>>,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            states: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RateLimitConfig::default())
    }

    pub fn check_rate_limit(&self, identifier: &str) -> Result<(), RateLimitError> {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();

        let state = states
            .entry(identifier.to_string())
            .or_insert_with(RateLimitState::new);

        // Check if currently locked out
        if let Some(lockout_until) = state.lockout_until {
            if now < lockout_until {
                let remaining = lockout_until.duration_since(now);
                return Err(RateLimitError::LockedOut(remaining));
            } else {
                // Lockout expired, reset state
                state.attempts.clear();
                state.lockout_until = None;
            }
        }

        // Clean old attempts outside the window
        let window_start = now - self.config.window_duration;
        state.attempts.retain(|&t| t > window_start);

        // Check if max attempts exceeded
        if state.attempts.len() as u32 >= self.config.max_attempts {
            state.lockout_until = Some(now + self.config.lockout_duration);
            return Err(RateLimitError::LockedOut(self.config.lockout_duration));
        }

        // Warn if approaching max attempts (more than 75% of limit)
        let warning_threshold = (self.config.max_attempts as f32 * 0.75) as usize;
        if state.attempts.len() >= warning_threshold {
            return Err(RateLimitError::TooManyAttempts);
        }

        // Record this attempt
        state.attempts.push(now);

        Ok(())
    }

    #[allow(dead_code)]
    pub fn reset(&self, identifier: &str) {
        let mut states = self.states.lock().unwrap();
        if let Some(state) = states.get_mut(identifier) {
            state.attempts.clear();
            state.lockout_until = None;
        }
    }

    pub fn cleanup_expired(&self) {
        let mut states = self.states.lock().unwrap();
        let now = Instant::now();
        let max_age = self.config.window_duration + self.config.lockout_duration;

        states.retain(|_, state| {
            if let Some(lockout) = state.lockout_until {
                if lockout > now {
                    return true;
                }
            }
            state.attempts.iter().any(|&t| now - t < max_age)
        });
    }
}

#[derive(Debug, Clone)]
pub enum RateLimitError {
    LockedOut(Duration),
    TooManyAttempts,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RateLimitError::LockedOut(remaining) => {
                write!(
                    f,
                    "Too many attempts. Try again in {:.0}s",
                    remaining.as_secs_f64()
                )
            }
            RateLimitError::TooManyAttempts => {
                write!(f, "Too many attempts. Please wait before trying again.")
            }
        }
    }
}

impl std::error::Error for RateLimitError {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_rate_limiter_allows_normal_usage() {
        let limiter = RateLimiter::with_default_config();

        for i in 0..3 {
            assert!(
                limiter.check_rate_limit("user1").is_ok(),
                "Attempt {} should succeed",
                i
            );
        }
    }

    #[test]
    fn test_rate_limiter_blocks_after_max_attempts() {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 3;
        config.window_duration = Duration::from_secs(60);
        config.lockout_duration = Duration::from_secs(1);

        let limiter = RateLimiter::new(config);

        for i in 0..3 {
            assert!(
                limiter.check_rate_limit("user2").is_ok(),
                "Attempt {} should succeed",
                i
            );
        }

        let result = limiter.check_rate_limit("user2");
        assert!(result.is_err(), "Attempt 4 should be blocked");
        assert!(matches!(result, Err(RateLimitError::LockedOut(_))));
    }

    #[test]
    fn test_rate_limiter_recovers_after_lockout() {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 2;
        config.window_duration = Duration::from_secs(1);
        config.lockout_duration = Duration::from_millis(500);

        let limiter = RateLimiter::new(config);

        for _ in 0..2 {
            limiter.check_rate_limit("user3").unwrap();
        }

        assert!(limiter.check_rate_limit("user3").is_err());

        thread::sleep(Duration::from_millis(600));

        assert!(limiter.check_rate_limit("user3").is_ok());
    }

    #[test]
    fn test_rate_limiter_different_identifiers() {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 2;

        let limiter = RateLimiter::new(config);

        for _ in 0..2 {
            limiter.check_rate_limit("userA").unwrap();
        }

        assert!(limiter.check_rate_limit("userA").is_err());
        assert!(limiter.check_rate_limit("userB").is_ok());
    }

    #[test]
    fn test_rate_limiter_reset() {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 2;

        let limiter = RateLimiter::new(config);

        for _ in 0..2 {
            limiter.check_rate_limit("userC").unwrap();
        }

        assert!(limiter.check_rate_limit("userC").is_err());

        limiter.reset("userC");

        assert!(limiter.check_rate_limit("userC").is_ok());
    }

    #[test]
    fn test_rate_limiter_window_expiration() {
        let mut config = RateLimitConfig::default();
        config.max_attempts = 2;
        config.window_duration = Duration::from_millis(500);

        let limiter = RateLimiter::new(config);

        limiter.check_rate_limit("userD").unwrap();

        thread::sleep(Duration::from_millis(600));

        limiter.check_rate_limit("userD").unwrap();

        assert!(limiter.check_rate_limit("userD").is_ok());
    }
}
