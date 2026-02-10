# SSM-Connect TUI

A modern terminal UI for connecting to EC2 instances via AWS Systems Manager (SSM).

## Features

- 🔐 Automatic AWS credential discovery with manual fallback
- 📊 Visual instance browser
- 🚀 Quick instance starting for stopped instances
- 🌍 Easy region switching
- ⌨️ Keyboard-driven interface
- 🎨 Clean, modern TUI built with [ratatui](https://github.com/ratatui/ratatui)

## Prerequisites

- [AWS CLI](https://aws.amazon.com/cli/) with [Session Manager Plugin](https://docs.aws.amazon.com/systems-manager/latest/userguide/session-manager-working-with-install-plugin.html)
- Valid AWS credentials with appropriate IAM permissions
- EC2 instance with SSM agent installed and configured

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
