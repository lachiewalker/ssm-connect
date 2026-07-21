use crate::aws::types::AwsCredentials;
use crate::config::PortForwardingRule;
use crate::error::{AppError, Result};
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::os::unix::process::CommandExt;

pub struct SsmManager {
    #[allow(dead_code)]
    client: aws_sdk_ssm::Client,
    region: String,
}

impl SsmManager {
    pub fn new(config: &aws_config::SdkConfig, region: String) -> Self {
        Self {
            client: aws_sdk_ssm::Client::new(config),
            region,
        }
    }

    /// Returns true if the SSM agent for the given instance is online according to the SSM API.
    pub async fn is_agent_online(&self, instance_id: &str) -> bool {
        use aws_sdk_ssm::types::InstanceInformationStringFilter;

        let filter = match InstanceInformationStringFilter::builder()
            .key("InstanceIds")
            .values(instance_id)
            .build()
        {
            Ok(f) => f,
            Err(_) => return false,
        };

        match self
            .client
            .describe_instance_information()
            .filters(filter)
            .send()
            .await
        {
            Ok(output) => output
                .instance_information_list()
                .iter()
                .any(|info| info.ping_status().is_some_and(|s| s.as_str() == "Online")),
            Err(_) => false,
        }
    }

    pub fn launch_session(&self, instance_id: &str, shell: &str, credentials: Option<&AwsCredentials>, auto_commands: &[String]) -> Result<()> {
        // Check if AWS CLI is installed
        let aws_check = Command::new("which")
            .arg("aws")
            .output()
            .map_err(|e| AppError::SsmSession(format!("Failed to check for AWS CLI: {}", e)))?;

        if !aws_check.status.success() {
            return Err(AppError::SsmSession(
                "AWS CLI not found. Please install AWS CLI with Session Manager plugin.".to_string(),
            ));
        }

        let shell_command = if auto_commands.is_empty() {
            // No auto-commands: just launch shell
            match shell {
                "zsh" => "zsh -l".to_string(),
                _ => "bash -l".to_string(),
            }
        } else {
            // Chain commands then start interactive shell
            let commands_str = auto_commands.join(" && ");
            match shell {
                "zsh" => format!("zsh -l -c '{} && exec zsh -l'", escape_single_quotes(&commands_str)),
                _ => format!("bash -l -c '{} && exec bash -l'", escape_single_quotes(&commands_str)),
            }
        };

        let mut cmd = Command::new("aws");

        // Escape double quotes in the command and wrap the entire value in double quotes
        // This ensures AWS CLI properly parses the parameter even with complex shell commands
        let escaped_command = shell_command.replace('"', r#"\""#);

        cmd.arg("ssm")
            .arg("start-session")
            .arg("--target")
            .arg(instance_id)
            .arg("--region")
            .arg(&self.region)
            .arg("--document-name")
            .arg("AWS-StartInteractiveCommand")
            .arg("--parameters")
            .arg(format!(r#"command="{}""#, escaped_command));

        // Pass credentials as environment variables if provided
        if let Some(creds) = credentials {
            if creds.access_key_id != "default" {
                cmd.env("AWS_ACCESS_KEY_ID", &creds.access_key_id);
                cmd.env("AWS_SECRET_ACCESS_KEY", &creds.secret_access_key);
                if let Some(token) = &creds.session_token {
                    cmd.env("AWS_SESSION_TOKEN", token);
                }
            }
        }

        // On Unix, replace the current process with `aws` so that Ctrl+C inside the
        // remote session is forwarded to the remote host rather than killing this
        // parent process and tearing down the connection.
        #[cfg(unix)]
        {
            let err = cmd.exec();
            return Err(AppError::SsmSession(format!("Failed to start SSM session: {}", err)));
        }

        // Fallback for non-Unix platforms.
        #[cfg(not(unix))]
        {
            let status = cmd
                .status()
                .map_err(|e| AppError::SsmSession(format!("Failed to start SSM session: {}", e)))?;

            if !status.success() {
                return Err(AppError::SsmSession(format!(
                    "SSM session exited with code: {:?}",
                    status.code()
                )));
            }

            Ok(())
        }
    }

    pub fn start_port_forwarding(
        &self,
        instance_id: &str,
        rule: &PortForwardingRule,
        credentials: Option<&AwsCredentials>,
    ) -> Result<()> {
        let mut cmd = Command::new("aws");
        cmd.arg("ssm")
            .arg("start-session")
            .arg("--target")
            .arg(instance_id)
            .arg("--region")
            .arg(&self.region)
            .arg("--document-name")
            .arg("AWS-StartPortForwardingSession")
            .arg("--parameters")
            .arg(format!(
                r#"portNumber=["{}"],localPortNumber=["{}"]"#,
                rule.remote_port, rule.local_port
            ));

        if let Some(creds) = credentials {
            if creds.access_key_id != "default" {
                cmd.env("AWS_ACCESS_KEY_ID", &creds.access_key_id);
                cmd.env("AWS_SECRET_ACCESS_KEY", &creds.secret_access_key);
                if let Some(token) = &creds.session_token {
                    cmd.env("AWS_SESSION_TOKEN", token);
                }
            }
        }

        cmd.stdout(Stdio::null()).stderr(Stdio::piped());

        let alias = rule.alias.clone();
        let mut child = cmd.spawn().map_err(|e| {
            AppError::SsmSession(format!(
                "Failed to start port forwarding for {}: {}",
                rule.alias, e
            ))
        })?;

        // Read stderr from the child process in a background thread so any AWS CLI
        // errors (auth failures, port conflicts, unreachable targets, etc.) are
        // captured in the log file rather than silently discarded.
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                for line in BufReader::new(stderr).lines().flatten() {
                    tracing::warn!(alias = %alias, "port-forward: {}", line);
                }
                // Log the exit status once the process terminates.
                if let Ok(status) = child.wait() {
                    if !status.success() {
                        tracing::error!(alias = %alias, status = %status, "port-forward process exited");
                    }
                }
            });
        }

        Ok(())
    }
}

fn escape_single_quotes(s: &str) -> String {
    s.replace("'", "'\\''")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_single_quotes() {
        assert_eq!(escape_single_quotes("hello"), "hello");
        assert_eq!(escape_single_quotes("test's quote"), "test'\\''s quote");
        assert_eq!(escape_single_quotes("it's a test"), "it'\\''s a test");
        assert_eq!(escape_single_quotes("multiple ' quotes '"), "multiple '\\'' quotes '\\''");
    }

    #[test]
    fn test_command_construction_no_auto_commands() {
        let auto_commands: Vec<String> = vec![];
        let shell = "bash";

        // Simulate the command construction logic
        let shell_command = if auto_commands.is_empty() {
            match shell {
                "zsh" => "zsh -l".to_string(),
                _ => "bash -l".to_string(),
            }
        } else {
            let commands_str = auto_commands.join(" && ");
            match shell {
                "zsh" => format!("zsh -l -c '{} && exec zsh -l'", escape_single_quotes(&commands_str)),
                _ => format!("bash -l -c '{} && exec bash -l'", escape_single_quotes(&commands_str)),
            }
        };

        assert_eq!(shell_command, "bash -l");
    }

    #[test]
    fn test_command_construction_with_auto_commands() {
        let auto_commands = vec!["cd /tmp".to_string(), "pwd".to_string()];
        let shell = "bash";

        let shell_command = if auto_commands.is_empty() {
            match shell {
                "zsh" => "zsh -l".to_string(),
                _ => "bash -l".to_string(),
            }
        } else {
            let commands_str = auto_commands.join(" && ");
            match shell {
                "zsh" => format!("zsh -l -c '{} && exec zsh -l'", escape_single_quotes(&commands_str)),
                _ => format!("bash -l -c '{} && exec bash -l'", escape_single_quotes(&commands_str)),
            }
        };

        assert_eq!(shell_command, "bash -l -c 'cd /tmp && pwd && exec bash -l'");
    }

    #[test]
    fn test_command_construction_with_quotes() {
        let auto_commands = vec!["sudo su - ubuntu".to_string()];
        let shell = "bash";

        let shell_command = if auto_commands.is_empty() {
            match shell {
                "zsh" => "zsh -l".to_string(),
                _ => "bash -l".to_string(),
            }
        } else {
            let commands_str = auto_commands.join(" && ");
            match shell {
                "zsh" => format!("zsh -l -c '{} && exec zsh -l'", escape_single_quotes(&commands_str)),
                _ => format!("bash -l -c '{} && exec bash -l'", escape_single_quotes(&commands_str)),
            }
        };

        // This should be properly wrapped in double quotes when passed to AWS CLI
        assert_eq!(shell_command, "bash -l -c 'sudo su - ubuntu && exec bash -l'");

        // And when escaped for AWS CLI parameter
        let escaped = shell_command.replace('"', r#"\""#);
        assert_eq!(escaped, "bash -l -c 'sudo su - ubuntu && exec bash -l'");
    }
}
