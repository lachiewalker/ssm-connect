use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
}

#[derive(Clone, Debug)]
pub struct EC2Instance {
    pub id: String,
    pub name: String,
    pub instance_type: String,
    pub state: InstanceState,
    pub private_ip: Option<String>,
    #[allow(dead_code)]
    pub public_ip: Option<String>,
    #[allow(dead_code)]
    pub availability_zone: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InstanceState {
    Pending,
    Running,
    Stopping,
    Stopped,
    ShuttingDown,
    Terminated,
    Unknown,
}

impl InstanceState {
    pub fn from_aws_state(state: &str) -> Self {
        match state {
            "pending" => InstanceState::Pending,
            "running" => InstanceState::Running,
            "stopping" => InstanceState::Stopping,
            "stopped" => InstanceState::Stopped,
            "shutting-down" => InstanceState::ShuttingDown,
            "terminated" => InstanceState::Terminated,
            _ => InstanceState::Unknown,
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            InstanceState::Running => Color::Green,
            InstanceState::Stopped => Color::Red,
            InstanceState::Pending | InstanceState::Stopping => Color::Yellow,
            InstanceState::Terminated | InstanceState::ShuttingDown => Color::DarkGray,
            InstanceState::Unknown => Color::Gray,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            InstanceState::Pending => "Pending",
            InstanceState::Running => "Running",
            InstanceState::Stopping => "Stopping",
            InstanceState::Stopped => "Stopped",
            InstanceState::ShuttingDown => "Shutting Down",
            InstanceState::Terminated => "Terminated",
            InstanceState::Unknown => "Unknown",
        }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct CallerIdentity {
    pub user_id: String,
    pub account: String,
    pub arn: String,
}
