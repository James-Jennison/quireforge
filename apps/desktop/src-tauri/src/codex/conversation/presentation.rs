use std::path::Path;

use url::Url;

const REDACTED: &str = "[redacted]";
const OUTSIDE_PROJECT: &str = "[outside project]";

pub(crate) fn sanitize_display_text(value: &str, cwd: &Path, max_bytes: usize) -> String {
    let without_controls = strip_terminal_controls(value);
    let redacted = without_controls
        .split('\n')
        .map(|line| redact_tokens(line, Some(cwd)))
        .collect::<Vec<_>>()
        .join("\n");
    truncate_utf8(redacted.trim(), max_bytes)
}

pub(crate) fn sanitize_label(value: &str, max_bytes: usize) -> String {
    let stripped = strip_terminal_controls(value);
    let redacted = redact_tokens(&stripped, None);
    truncate_utf8(redacted.trim(), max_bytes)
}

pub(crate) fn present_path(value: &str, cwd: &Path) -> String {
    let path = Path::new(value);
    if path.is_relative()
        && !value.is_empty()
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return sanitize_label(&format!("./{}", path.to_string_lossy()), 1024);
    }
    if path == cwd {
        return ".".to_owned();
    }
    if let Ok(relative) = path.strip_prefix(cwd) {
        if relative.components().any(|component| {
            matches!(
                component,
                std::path::Component::ParentDir
                    | std::path::Component::RootDir
                    | std::path::Component::Prefix(_)
            )
        }) {
            return OUTSIDE_PROJECT.to_owned();
        }
        let relative = relative.to_string_lossy();
        if relative.is_empty() {
            ".".to_owned()
        } else {
            sanitize_label(&format!("./{relative}"), 1024)
        }
    } else {
        OUTSIDE_PROJECT.to_owned()
    }
}

fn strip_terminal_controls(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut characters = value.chars().peekable();
    while let Some(character) = characters.next() {
        if character == '\u{1b}' {
            match characters.peek().copied() {
                Some('[') => {
                    characters.next();
                    for next in characters.by_ref() {
                        if ('\u{40}'..='\u{7e}').contains(&next) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    characters.next();
                    let mut saw_escape = false;
                    for next in characters.by_ref() {
                        if next == '\u{7}' || (saw_escape && next == '\\') {
                            break;
                        }
                        saw_escape = next == '\u{1b}';
                    }
                }
                Some(_) => {
                    characters.next();
                }
                None => {}
            }
            continue;
        }
        if matches!(
            character,
            '\u{200B}'..='\u{200F}'
                | '\u{202A}'..='\u{202E}'
                | '\u{2060}'..='\u{206F}'
                | '\u{FEFF}'
        ) {
            continue;
        }
        if character == '\r' {
            output.push('\n');
        } else if !character.is_control() || matches!(character, '\n' | '\t') {
            output.push(character);
        }
    }
    output
}

fn redact_tokens(value: &str, cwd: Option<&Path>) -> String {
    let mut output = Vec::new();
    let mut redact_next = false;
    for token in value.split_whitespace() {
        let trimmed = token.trim_matches(|character: char| {
            matches!(character, '\'' | '"' | ',' | ';' | '(' | ')' | '[' | ']')
        });
        let lower = trimmed.to_ascii_lowercase();
        if redact_next {
            if lower == "bearer" {
                output.push("Bearer".to_owned());
            } else {
                output.push(REDACTED.to_owned());
                redact_next = false;
            }
            continue;
        }
        if let Some((header, value)) = lower.split_once(':') {
            if is_sensitive_header(header) {
                output.push(if value.is_empty() {
                    format!("{header}:")
                } else {
                    format!("{header}:{REDACTED}")
                });
                redact_next = value.is_empty();
                continue;
            }
        }
        if is_sensitive_flag(&lower) || is_sensitive_header(&lower) {
            output.push(token.to_owned());
            redact_next = !lower.contains('=') && !lower.ends_with(':');
            if lower.contains('=') {
                let key = token.split_once('=').map_or(token, |(key, _)| key);
                output.pop();
                output.push(format!("{key}={REDACTED}"));
            } else if lower.ends_with(':') {
                redact_next = true;
            }
            continue;
        }
        if let Some((key, _)) = trimmed.split_once('=') {
            if is_sensitive_name(key) {
                output.push(format!("{key}={REDACTED}"));
                continue;
            }
        }
        if lower.starts_with("bearer") {
            output.push("Bearer".to_owned());
            redact_next = true;
            continue;
        }
        if let Some(url) = sanitize_url(trimmed) {
            output.push(url);
            continue;
        }
        if looks_like_absolute_path(trimmed) {
            output.push(cwd.map_or_else(
                || OUTSIDE_PROJECT.to_owned(),
                |cwd| present_path(trimmed, cwd),
            ));
            continue;
        }
        output.push(token.to_owned());
    }
    output.join(" ")
}

fn is_sensitive_flag(value: &str) -> bool {
    let name = value
        .trim_start_matches('-')
        .split_once('=')
        .map_or(value.trim_start_matches('-'), |(name, _)| name);
    is_sensitive_name(name)
}

fn is_sensitive_header(value: &str) -> bool {
    matches!(
        value.trim_end_matches(':'),
        "authorization" | "proxy-authorization" | "cookie" | "set-cookie"
    )
}

fn is_sensitive_name(value: &str) -> bool {
    let normalized = value
        .trim_start_matches('-')
        .to_ascii_lowercase()
        .replace('-', "_");
    matches!(
        normalized.as_str(),
        "token"
            | "access_token"
            | "refresh_token"
            | "api_key"
            | "apikey"
            | "secret"
            | "client_secret"
            | "password"
            | "passwd"
            | "credential"
            | "credentials"
            | "authorization"
            | "cookie"
    ) || normalized.ends_with("_token")
        || normalized.ends_with("_api_key")
        || normalized.ends_with("_apikey")
        || normalized.ends_with("_secret")
        || normalized.ends_with("_password")
        || normalized.ends_with("_passwd")
        || normalized.ends_with("_credential")
        || normalized.ends_with("_credentials")
        || normalized.ends_with("_private_key")
        || normalized == "aws_secret_access_key"
}

fn sanitize_url(value: &str) -> Option<String> {
    let mut url = Url::parse(value).ok()?;
    if !matches!(url.scheme(), "http" | "https") {
        return None;
    }
    if !url.username().is_empty() || url.password().is_some() {
        let _ = url.set_username("");
        let _ = url.set_password(None);
    }
    let pairs = url
        .query_pairs()
        .map(|(key, value)| {
            let value = if is_sensitive_name(&key) {
                REDACTED.to_owned()
            } else {
                value.into_owned()
            };
            (key.into_owned(), value)
        })
        .collect::<Vec<_>>();
    if !pairs.is_empty() {
        url.query_pairs_mut().clear().extend_pairs(pairs);
    }
    Some(url.to_string())
}

fn looks_like_absolute_path(value: &str) -> bool {
    let value = value.trim_matches(|character| matches!(character, '\'' | '"'));
    if !value.starts_with('/') {
        return false;
    }
    if let Some(command) = value
        .strip_prefix("/usr/bin/")
        .or_else(|| value.strip_prefix("/usr/local/bin/"))
        .or_else(|| value.strip_prefix("/bin/"))
    {
        return command.contains('/');
    }
    true
}

fn truncate_utf8(value: &str, max_bytes: usize) -> String {
    if value.len() <= max_bytes {
        return value.to_owned();
    }
    let suffix = "…";
    let mut boundary = max_bytes.saturating_sub(suffix.len()).min(value.len());
    while boundary > 0 && !value.is_char_boundary(boundary) {
        boundary -= 1;
    }
    format!("{}{}", &value[..boundary], suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_terminal_controls_and_redacts_credentials_and_external_paths() {
        let cwd = Path::new("/workspace/project");
        let value = "\u{1b}[31mcurl https://user:secret@example.test/api?token=private\u{1b}[0m --password hunter2 OPENAI_API_KEY=sk-private Authorization: Bearer auth-secret /home/private/file /workspace/project/src/main.rs";
        let sanitized = sanitize_display_text(value, cwd, 4096);

        assert!(!sanitized.contains("\u{1b}"));
        assert!(!sanitized.contains("secret"));
        assert!(!sanitized.contains("private"));
        assert!(!sanitized.contains("hunter2"));
        assert!(!sanitized.contains("sk-private"));
        assert!(!sanitized.contains("auth-secret"));
        assert!(!sanitized.contains("/home"));
        assert!(sanitized.contains(REDACTED));
        assert!(sanitized.contains("./src/main.rs"));
    }

    #[test]
    fn presents_only_project_relative_paths() {
        let cwd = Path::new("/workspace/project");
        assert_eq!(present_path("/workspace/project", cwd), ".");
        assert_eq!(
            present_path("/workspace/project/src/lib.rs", cwd),
            "./src/lib.rs"
        );
        assert_eq!(present_path("/etc/passwd", cwd), OUTSIDE_PROJECT);
        assert_eq!(present_path("src/lib.rs", cwd), "./src/lib.rs");
        assert_eq!(present_path("../private", cwd), OUTSIDE_PROJECT);
        assert_eq!(
            present_path("/workspace/project/../private", cwd),
            OUTSIDE_PROJECT
        );
        assert_eq!(
            sanitize_display_text("/workspace/project-secret/file", cwd, 4096),
            OUTSIDE_PROJECT
        );
    }

    #[test]
    fn truncates_on_a_utf8_boundary() {
        assert_eq!(truncate_utf8("abc😀def", 7), "abc…");
    }
}
