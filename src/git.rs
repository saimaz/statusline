//! Git segment data via one `git status --porcelain=v2 --branch` call.
//! --no-optional-locks so status never creates .git/index.lock.

use std::process::Command;

#[derive(Debug, Default, PartialEq)]
pub struct GitInfo {
    pub branch: String,
    pub dirty: bool,
    pub ahead: i64,
    pub behind: i64,
}

pub fn read(dir: &str) -> Option<GitInfo> {
    let out = Command::new("git")
        .args([
            "--no-optional-locks",
            "-C",
            dir,
            "status",
            "--porcelain=v2",
            "--branch",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse(&String::from_utf8_lossy(&out.stdout))
}

pub fn parse(porcelain: &str) -> Option<GitInfo> {
    let mut info = GitInfo::default();
    let mut oid = String::new();
    for line in porcelain.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            info.branch = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.oid ") {
            oid = rest.trim().to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            for part in rest.split_whitespace() {
                if let Some(n) = part.strip_prefix('+') {
                    info.ahead = n.parse().unwrap_or(0);
                } else if let Some(n) = part.strip_prefix('-') {
                    info.behind = n.parse().unwrap_or(0);
                }
            }
        } else if !line.starts_with('#') && !line.is_empty() {
            info.dirty = true;
        }
    }
    if info.branch.is_empty() {
        return None;
    }
    if info.branch == "(detached)" {
        info.branch = format!("@{}", oid.chars().take(7).collect::<String>());
    }
    Some(info)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clean_branch() {
        let s = "# branch.oid abc123\n# branch.head main\n# branch.ab +0 -0\n";
        let g = parse(s).unwrap();
        assert_eq!(g.branch, "main");
        assert!(!g.dirty);
        assert_eq!((g.ahead, g.behind), (0, 0));
    }

    #[test]
    fn parses_dirty_ahead_behind() {
        let s = "# branch.oid abc\n# branch.head test\n# branch.ab +2 -1\n1 .M N... 100644 100644 100644 x y src/main.rs\n? new.txt\n";
        let g = parse(s).unwrap();
        assert!(g.dirty);
        assert_eq!((g.ahead, g.behind), (2, 1));
    }

    #[test]
    fn detached_head_shows_short_oid() {
        let s = "# branch.oid deadbeefcafe\n# branch.head (detached)\n";
        let g = parse(s).unwrap();
        assert_eq!(g.branch, "@deadbee");
    }

    #[test]
    fn no_branch_line_is_none() {
        assert_eq!(parse("? foo\n"), None);
    }
}
