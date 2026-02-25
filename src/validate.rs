use crate::error::{Error, Result};
use crate::fs as afs;
use std::path::Path;

pub fn parse_assignment(input: &str) -> Result<(String, u32)> {
    let s = input.trim();
    if s.is_empty() {
        return Err(Error::Validation("assignment cannot be empty".into()));
    }

    let lower = s.to_ascii_lowercase();
    let digits = if let Some(rest) = lower.strip_prefix("assignment") {
        let rest = rest.trim();
        if rest.is_empty() {
            return Err(Error::Validation(
                "incomplete assignment label, use '7' or 'Assignment7'".into(),
            ));
        }
        rest
    } else {
        s
    };

    if !digits.chars().all(|c| c.is_ascii_digit()) {
        return Err(Error::Validation(
            "assignment must be a number (e.g. 7) or label (e.g. Assignment7)".into(),
        ));
    }

    let n: u32 = digits
        .parse()
        .map_err(|_| Error::Validation("assignment number too large".into()))?;

    if n == 0 {
        return Err(Error::Validation(
            "assignment number must be greater than 0".into(),
        ));
    }

    Ok((format!("Assignment{n}"), n))
}

pub fn clean_name(input: &str, label: &str) -> Result<String> {
    let compact: String = input.split_whitespace().collect();
    if compact.is_empty() {
        return Err(Error::Validation(format!("{label} cannot be empty")));
    }
    for ch in compact.chars() {
        if matches!(ch, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || ch.is_control() {
            return Err(Error::Validation(format!(
                "invalid character in {label}: '{ch}'"
            )));
        }
    }
    Ok(compact)
}

pub fn render_display_command(
    tpl: Option<&str>,
    assignment: &str,
    num: u32,
    name: &str,
    student_id: &str,
    c_file: &Path,
) -> Result<String> {
    let default = if cfg!(windows) {
        format!("{assignment}.exe")
    } else {
        assignment.to_string()
    };

    let tpl = match tpl {
        Some(t) => {
            let t = t.trim();
            if t.is_empty() {
                return Err(Error::Validation(
                    "run-display-template cannot be blank".into(),
                ));
            }
            t
        }
        None => return Ok(default),
    };

    let c_name = afs::file_name(c_file)?;
    let c_stem = c_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");

    let out = tpl
        .replace("{assignment}", assignment)
        .replace("{assignment_number}", &num.to_string())
        .replace("{name}", name)
        .replace("{id}", student_id)
        .replace("{c_file}", c_name)
        .replace("{c_stem}", c_stem);

    let out = out.trim().to_string();
    if out.is_empty() {
        return Err(Error::Validation(
            "run-display-template produced an empty result".into(),
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bare_number() {
        let (label, n) = parse_assignment("7").unwrap();
        assert_eq!(label, "Assignment7");
        assert_eq!(n, 7);
    }

    #[test]
    fn parse_prefixed_label() {
        let (label, n) = parse_assignment("Assignment12").unwrap();
        assert_eq!(label, "Assignment12");
        assert_eq!(n, 12);
    }

    #[test]
    fn parse_case_insensitive() {
        let (_, n) = parse_assignment("ASSIGNMENT3").unwrap();
        assert_eq!(n, 3);
    }

    #[test]
    fn parse_zero_rejected() {
        assert!(parse_assignment("0").is_err());
    }

    #[test]
    fn parse_empty_rejected() {
        assert!(parse_assignment("").is_err());
        assert!(parse_assignment("  ").is_err());
    }

    #[test]
    fn parse_alpha_rejected() {
        assert!(parse_assignment("abc").is_err());
    }

    #[test]
    fn parse_incomplete_prefix() {
        assert!(parse_assignment("Assignment").is_err());
    }

    #[test]
    fn clean_name_collapses_whitespace() {
        assert_eq!(clean_name("Joe Bloggs", "name").unwrap(), "JoeBloggs");
    }

    #[test]
    fn clean_name_rejects_empty() {
        assert!(clean_name("", "name").is_err());
        assert!(clean_name("   ", "name").is_err());
    }

    #[test]
    fn clean_name_rejects_bad_chars() {
        assert!(clean_name("foo/bar", "name").is_err());
        assert!(clean_name("a:b", "name").is_err());
        assert!(clean_name("a*b", "name").is_err());
    }

    #[test]
    fn clean_name_allows_normal_text() {
        assert_eq!(clean_name("Alice", "name").unwrap(), "Alice");
        assert_eq!(clean_name("Bob123", "name").unwrap(), "Bob123");
    }

    #[test]
    fn display_cmd_default_no_template() {
        let result =
            render_display_command(None, "Assignment7", 7, "Alice", "123", Path::new("main.c"))
                .unwrap();
        if cfg!(windows) {
            assert_eq!(result, "Assignment7.exe");
        } else {
            assert_eq!(result, "Assignment7");
        }
    }

    #[test]
    fn display_cmd_template_substitution() {
        let result = render_display_command(
            Some("./{c_stem}"),
            "Assignment7",
            7,
            "Alice",
            "123",
            Path::new("main.c"),
        )
        .unwrap();
        assert_eq!(result, "./main");
    }

    #[test]
    fn display_cmd_all_placeholders() {
        let result = render_display_command(
            Some("{assignment} {assignment_number} {name} {id} {c_file} {c_stem}"),
            "Assignment7",
            7,
            "Alice",
            "123",
            Path::new("main.c"),
        )
        .unwrap();
        assert_eq!(result, "Assignment7 7 Alice 123 main.c main");
    }

    #[test]
    fn display_cmd_blank_template_rejected() {
        assert!(
            render_display_command(
                Some("  "),
                "Assignment7",
                7,
                "Alice",
                "123",
                Path::new("main.c"),
            )
            .is_err()
        );
    }
}
