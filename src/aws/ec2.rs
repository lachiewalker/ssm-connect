use crate::aws::types::{EC2Instance, InstanceState};
use crate::error::Result;

pub struct Ec2Manager {
    client: aws_sdk_ec2::Client,
}

impl Ec2Manager {
    pub fn new(config: &aws_config::SdkConfig) -> Self {
        Self {
            client: aws_sdk_ec2::Client::new(config),
        }
    }

    pub async fn list_instances(&self) -> Result<Vec<EC2Instance>> {
        let response = self.client.describe_instances().send().await
            .map_err(crate::error::AppError::from_ec2_error)?;

        let mut instances = Vec::new();

        for reservation in response.reservations() {
            for instance in reservation.instances() {
                let instance_id = instance.instance_id().unwrap_or("unknown").to_string();

                // Extract Name tag
                let name = instance
                    .tags()
                    .iter()
                    .find(|tag| tag.key() == Some("Name"))
                    .and_then(|tag| tag.value())
                    .unwrap_or(&instance_id)
                    .to_string();

                let state_name = instance
                    .state()
                    .and_then(|s| s.name())
                    .map(|n| n.as_str())
                    .unwrap_or("unknown");

                let state = InstanceState::from_aws_state(state_name);

                let instance_type = instance.instance_type()
                    .map(|t| t.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                let private_ip = instance.private_ip_address().map(|s| s.to_string());
                let public_ip = instance.public_ip_address().map(|s| s.to_string());

                let availability_zone = instance
                    .placement()
                    .and_then(|p| p.availability_zone())
                    .unwrap_or("unknown")
                    .to_string();

                instances.push(EC2Instance {
                    id: instance_id,
                    name,
                    instance_type,
                    state,
                    private_ip,
                    public_ip,
                    availability_zone,
                });
            }
        }

        Ok(instances)
    }

    pub async fn start_instance(&self, instance_id: &str) -> Result<()> {
        self.client
            .start_instances()
            .instance_ids(instance_id)
            .send()
            .await
            .map_err(crate::error::AppError::from_ec2_error)?;

        Ok(())
    }
}
