use thiserror::Error;

#[derive(Error, Debug)]
pub enum NaruError {
    #[error("Project is already initialized")]
    ProjectAlreadyInitialized,
    
    #[error("Project not initialized")]
    ProjectNotInitialized,
    
    #[error("Environment not found: {env}")]
    EnvironmentNotFound { env: String },
    
    #[error("Configuration key not found: {key}")]
    KeyNotFound { key: String },
    
    #[error("Validation failed: {message}")]
    ValidationError { message: String },
    
    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
    
    #[error("JSON error: {source}")]
    JsonError {
        #[from]
        source: serde_json::Error,
    },
}