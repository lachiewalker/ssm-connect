use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[allow(dead_code)]
    #[error("Credential error: {0}")]
    Credentials(String),

    #[error("AWS API error: {0}")]
    AwsApi(String),

    #[allow(dead_code)]
    #[error("Instance not found: {0}")]
    InstanceNotFound(String),

    #[error("SSM session error: {0}")]
    SsmSession(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("Terminal error: {0}")]
    Terminal(String),

    #[allow(dead_code)]
    #[error("Configuration error: {0}")]
    Config(String),
}

// Helper methods for converting AWS SDK errors
impl AppError {
    pub fn from_ec2_error<E: std::fmt::Display>(err: aws_sdk_ec2::error::SdkError<E>) -> Self {
        AppError::AwsApi(err.to_string())
    }

    pub fn from_sts_error<E: std::fmt::Display>(err: aws_sdk_sts::error::SdkError<E>) -> Self {
        AppError::AwsApi(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
