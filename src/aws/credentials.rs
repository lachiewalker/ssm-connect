use crate::aws::types::{AwsCredentials, CallerIdentity};
use crate::error::{AppError, Result};
use aws_config::BehaviorVersion;
use aws_credential_types::provider::SharedCredentialsProvider;
use aws_credential_types::Credentials;

pub struct CredentialManager;

impl CredentialManager {
    pub async fn discover_credentials() -> Result<Option<aws_config::SdkConfig>> {
        // Try to load from default credential chain
        let config = aws_config::defaults(BehaviorVersion::latest())
            .load()
            .await;

        // Try to get credentials to see if they exist
        match config.credentials_provider() {
            Some(_provider) => {
                // If a provider exists, assume credentials are available
                // We'll validate them with STS later
                Ok(Some(config))
            }
            None => Ok(None),
        }
    }

    pub async fn validate_credentials(config: &aws_config::SdkConfig) -> Result<CallerIdentity> {
        let sts_client = aws_sdk_sts::Client::new(config);

        let response = sts_client
            .get_caller_identity()
            .send()
            .await
            .map_err(|e| AppError::from_sts_error(e))?;

        Ok(CallerIdentity {
            user_id: response.user_id().unwrap_or("unknown").to_string(),
            account: response.account().unwrap_or("unknown").to_string(),
            arn: response.arn().unwrap_or("unknown").to_string(),
        })
    }

    pub async fn build_config(creds: &AwsCredentials, region: &str) -> Result<aws_config::SdkConfig> {
        let credentials = if let Some(ref token) = creds.session_token {
            Credentials::new(
                &creds.access_key_id,
                &creds.secret_access_key,
                Some(token.clone()),
                None,
                "manual-input",
            )
        } else {
            Credentials::new(
                &creds.access_key_id,
                &creds.secret_access_key,
                None,
                None,
                "manual-input",
            )
        };

        let credentials_provider = SharedCredentialsProvider::new(credentials);

        let config = aws_config::defaults(BehaviorVersion::latest())
            .credentials_provider(credentials_provider)
            .region(aws_config::Region::new(region.to_string()))
            .load()
            .await;

        Ok(config)
    }
}
