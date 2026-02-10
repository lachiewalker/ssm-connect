#[derive(Debug, Clone)]
pub struct ParsedCredentials {
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub session_token: Option<String>,
}

/// Parse AWS credentials from various formats
pub fn parse_aws_credentials(text: &str) -> Option<ParsedCredentials> {
    let mut access_key = None;
    let mut secret_key = None;
    let mut session_token = None;

    // Track if we found any credentials at all
    let mut found_any = false;

    // Try to parse each line
    for line in text.lines() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Try to match export format: export AWS_ACCESS_KEY_ID=...
        if let Some(stripped) = line.strip_prefix("export ") {
            if let Some((key, value)) = parse_key_value(stripped) {
                match key.as_str() {
                    "AWS_ACCESS_KEY_ID" => {
                        access_key = Some(value);
                        found_any = true;
                    }
                    "AWS_SECRET_ACCESS_KEY" => {
                        secret_key = Some(value);
                        found_any = true;
                    }
                    "AWS_SESSION_TOKEN" => {
                        session_token = Some(value);
                        found_any = true;
                    }
                    _ => {}
                }
            }
        }
        // Try to match key=value format without export
        else if let Some((key, value)) = parse_key_value(line) {
            match key.as_str() {
                "AWS_ACCESS_KEY_ID" => {
                    access_key = Some(value);
                    found_any = true;
                }
                "AWS_SECRET_ACCESS_KEY" => {
                    secret_key = Some(value);
                    found_any = true;
                }
                "AWS_SESSION_TOKEN" => {
                    session_token = Some(value);
                    found_any = true;
                }
                _ => {}
            }
        }
        // Try plain value format (for copy-paste of values only)
        else if !line.contains('=') && !line.contains(' ') {
            // Heuristic: AWS access keys start with AKIA, ASIA, etc.
            // Secret keys and session tokens are longer base64-like strings
            if line.starts_with("AKIA") || line.starts_with("ASIA") {
                if access_key.is_none() {
                    access_key = Some(line.to_string());
                    found_any = true;
                }
            } else if line.len() > 30 {
                // Longer strings are likely secret keys or session tokens
                if secret_key.is_none() {
                    secret_key = Some(line.to_string());
                    found_any = true;
                } else if session_token.is_none() {
                    session_token = Some(line.to_string());
                    found_any = true;
                }
            }
        }
    }

    // Return parsed credentials if we found at least access key and secret key
    if found_any && access_key.is_some() && secret_key.is_some() {
        Some(ParsedCredentials {
            access_key,
            secret_key,
            session_token,
        })
    } else {
        None
    }
}

/// Parse a key=value pair, handling quotes and whitespace
fn parse_key_value(line: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() != 2 {
        return None;
    }

    let key = parts[0].trim().to_string();
    let mut value = parts[1].trim().to_string();

    // Remove surrounding quotes if present
    if (value.starts_with('"') && value.ends_with('"'))
        || (value.starts_with('\'') && value.ends_with('\''))
    {
        value = value[1..value.len() - 1].to_string();
    }

    Some((key, value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_export_format() {
        let input = r#"
export AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
export AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
export AWS_SESSION_TOKEN=IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample
"#;

        let result = parse_aws_credentials(input);
        assert!(result.is_some());

        let creds = result.unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("AKIAIOSFODNN7EXAMPLE"));
        assert_eq!(
            creds.secret_key.as_deref(),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
        );
        assert_eq!(
            creds.session_token.as_deref(),
            Some("IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample")
        );
    }

    #[test]
    fn test_parse_key_value_format() {
        let input = r#"
AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE
AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
"#;

        let result = parse_aws_credentials(input);
        assert!(result.is_some());

        let creds = result.unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("AKIAIOSFODNN7EXAMPLE"));
        assert_eq!(
            creds.secret_key.as_deref(),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
        );
    }

    #[test]
    fn test_parse_plain_values() {
        let input = r#"
AKIAIOSFODNN7EXAMPLE
wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample
"#;

        let result = parse_aws_credentials(input);
        assert!(result.is_some());

        let creds = result.unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("AKIAIOSFODNN7EXAMPLE"));
        assert_eq!(
            creds.secret_key.as_deref(),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
        );
        assert_eq!(
            creds.session_token.as_deref(),
            Some("IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample")
        );
    }

    #[test]
    fn test_parse_with_quotes() {
        let input = r#"
AWS_ACCESS_KEY_ID="AKIAIOSFODNN7EXAMPLE"
AWS_SECRET_ACCESS_KEY='wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY'
"#;

        let result = parse_aws_credentials(input);
        assert!(result.is_some());

        let creds = result.unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("AKIAIOSFODNN7EXAMPLE"));
        assert_eq!(
            creds.secret_key.as_deref(),
            Some("wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY")
        );
    }

    #[test]
    fn test_parse_incomplete_credentials() {
        let input = "AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE";
        let result = parse_aws_credentials(input);
        assert!(result.is_none()); // Missing secret key
    }

    #[test]
    fn test_parse_aws_console_format() {
        // This is the exact format from AWS console's "Copy credentials" button
        let input = r#"export AWS_ACCESS_KEY_ID="ASIAR7HWYIBKXXXXXXXX"
export AWS_SECRET_ACCESS_KEY="XXXXXXXX0owZ234XXXXXXXXWE1y6Eq2EXXXXXXXX"
export AWS_SESSION_TOKEN="IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample""#;

        let result = parse_aws_credentials(input);
        assert!(result.is_some(), "Failed to parse AWS console format");

        let creds = result.unwrap();
        assert_eq!(creds.access_key.as_deref(), Some("ASIAR7HWYIBKXXXXXXXX"));
        assert_eq!(
            creds.secret_key.as_deref(),
            Some("XXXXXXXX0owZ234XXXXXXXXWE1y6Eq2EXXXXXXXX")
        );
        assert_eq!(
            creds.session_token.as_deref(),
            Some("IQoJb3JpZ2luX2VjEMz//////////wEaCXVzLWVhc3QtMSJIMEYCIQCvLkExample")
        );
    }
}
