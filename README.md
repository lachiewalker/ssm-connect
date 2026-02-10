# SSM-Connect CLI

A modern terminal UI for connecting to EC2 instances via AWS Systems Manager (SSM).

## Features

- 🔐 Automatic AWS credential discovery with manual fallback
- 📊 Visual instance browser with real-time SSM status
- 🚀 Quick instance starting for stopped instances
- 🌍 Easy region switching
- ⌨️ Keyboard-driven interface
- 🎨 Clean, modern TUI built with ratatui

## Prerequisites

- [AWS CLI](https://aws.amazon.com/cli/) with [Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html)
- Valid AWS credentials with appropriate IAM permissions
- EC2 instances with SSM agent installed and configured

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
./target/release/ssm-connect
```

## Usage

Simply run the application:

```bash
ssm-connect
```

### Keyboard Shortcuts

**Instance List:**
- `↑/↓` or `j/k` - Navigate through instances
- `Enter` - Connect to selected instance
- `s` - Start a stopped instance
- `r` - Change AWS region
- `?` - Toggle help screen
- `q` or `Esc` - Quit

**Credential Input:**
- `Tab` - Switch between fields
- `Shift+Tab` - Switch backwards
- `Enter` - Submit credentials
- `Ctrl+D` - Try default credentials
- `Ctrl+Q` - Quit

## Configuration

Configuration is stored in `~/.config/ssm-connect/config.json`:

```json
{
  "default_region": "us-east-1",
  "default_shell": "bash",
  "auto_refresh_interval_seconds": 30,
  "theme": "dark"
}
```

## SSM Status Indicators

- `✓` (Green) - Online - Ready for connections
- `⚠` (Yellow) - Connection Lost - Agent offline
- `✗` (Red) - Not Installed - SSM agent not available
- `?` (Gray) - Unknown - Status unavailable

## IAM Permissions Required

Your AWS credentials need the following permissions:

```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ec2:DescribeInstances",
        "ec2:StartInstances",
        "ssm:DescribeInstanceInformation",
        "ssm:StartSession"
      ],
      "Resource": "*"
    }
  ]
}
```

## Logs

Logs are stored in `~/.local/share/ssm-connect/logs/`

## License

MIT
