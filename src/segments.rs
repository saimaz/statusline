//! Builds the six segments from the payload + git info.
//! Any segment with missing data is silently skipped.

use crate::git::GitInfo;
use crate::input::Payload;
use crate::render::{bar, until, Segment, Style, GREEN, LAVENDER, MAUVE, PEACH, SAPPHIRE, YELLOW};

const BRANCH_GLYPH: char = '\u{e0a0}';
const CONTEXT_BAR_WIDTH: usize = 8;
const USAGE_BAR_WIDTH: usize = 5;
const BRANCH_MAX: usize = 30;

pub fn build(payload: &Payload, git: Option<&GitInfo>, now: i64, style: Style) -> Vec<Segment> {
    let mut segs = Vec::with_capacity(6);

    if let Some(dir) = directory(payload) {
        segs.push(Segment::new(dir, PEACH, false));
    }
    if let Some(g) = git {
        segs.push(Segment::new(branch(g, style), YELLOW, false));
    }
    if let Some(lines) = lines_changed(payload) {
        segs.push(Segment::new(lines, GREEN, true));
    }
    if let Some(m) = payload.model.as_ref().and_then(|m| m.display_name.clone()) {
        if !m.is_empty() {
            segs.push(Segment::new(m, SAPPHIRE, true));
        }
    }
    if let Some(pct) = payload.context_window.as_ref().and_then(|c| c.used_pct()) {
        let text = format!("{} {:.0}%", bar(pct, CONTEXT_BAR_WIDTH), pct);
        segs.push(Segment::new(text, LAVENDER, false));
    }
    if let Some(usage) = usage(payload, now) {
        segs.push(Segment::new(usage, MAUVE, false));
    }

    segs
}

fn directory(payload: &Payload) -> Option<String> {
    let dir = payload
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| payload.cwd.clone())?;
    Some(shorten(&dir, &std::env::var("HOME").unwrap_or_default()))
}

/// Fish-style: home becomes ~, every component except the last two is cut
/// to its first character. `/Users/x/Developer/smoobu/wiki` -> `~/D/smoobu/wiki`.
pub fn shorten(path: &str, home: &str) -> String {
    let path = if !home.is_empty() && path.starts_with(home) {
        format!("~{}", &path[home.len()..])
    } else {
        path.to_string()
    };
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() <= 2 {
        return path;
    }
    let mut out = Vec::with_capacity(parts.len());
    for (i, p) in parts.iter().enumerate() {
        if i + 2 >= parts.len() || *p == "~" {
            out.push(p.to_string());
        } else {
            out.push(p.chars().take(1).collect());
        }
    }
    let joined = out.join("/");
    if path.starts_with('/') {
        format!("/{joined}")
    } else {
        joined
    }
}

fn branch(g: &GitInfo, style: Style) -> String {
    let mut name: String = g.branch.chars().take(BRANCH_MAX).collect();
    if g.branch.chars().count() > BRANCH_MAX {
        name.push('\u{2026}');
    }
    let mut s = match style {
        Style::Nerd => format!("{BRANCH_GLYPH} {name}"),
        Style::Plain => name,
    };
    if g.dirty {
        s.push_str(" *");
    }
    if g.ahead > 0 {
        s.push_str(&format!(" \u{21e1}{}", g.ahead));
    }
    if g.behind > 0 {
        s.push_str(&format!(" \u{21e3}{}", g.behind));
    }
    s
}

fn lines_changed(payload: &Payload) -> Option<String> {
    let cost = payload.cost.as_ref()?;
    if cost.total_lines_added == 0 && cost.total_lines_removed == 0 {
        return None;
    }
    Some(format!(
        "+{} -{}",
        cost.total_lines_added, cost.total_lines_removed
    ))
}

fn usage(payload: &Payload, now: i64) -> Option<String> {
    let limits = payload.rate_limits.as_ref()?;
    let mut parts = Vec::with_capacity(2);
    for (label, window) in [("5h", &limits.five_hour), ("7d", &limits.seven_day)] {
        let Some(w) = window else { continue };
        let Some(pct) = w.used_percentage else { continue };
        let mut s = format!("{label} {} {:.0}%", bar(pct, USAGE_BAR_WIDTH), pct);
        if let Some(resets) = w.resets_at {
            s.push_str(&format!(" {}", until(resets - now)));
        }
        parts.push(s);
    }
    if parts.is_empty() {
        return None;
    }
    Some(parts.join(" \u{2502} "))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::Payload;

    fn payload(json: &str) -> Payload {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn shorten_truncates_middle_components() {
        assert_eq!(
            shorten("/Users/simas/Developer/smoobu/wiki", "/Users/simas"),
            "~/D/smoobu/wiki"
        );
        assert_eq!(shorten("/etc/nginx/conf.d", ""), "/e/nginx/conf.d");
        assert_eq!(shorten("/Users/simas", "/Users/simas"), "~");
        assert_eq!(shorten("/tmp", ""), "/tmp");
    }

    #[test]
    fn zero_lines_changed_is_skipped() {
        let p = payload(r#"{"cost":{"total_lines_added":0,"total_lines_removed":0}}"#);
        assert_eq!(lines_changed(&p), None);
    }

    #[test]
    fn usage_renders_both_windows() {
        let p = payload(
            r#"{"rate_limits":{
                "five_hour":{"used_percentage":5,"resets_at":1000000},
                "seven_day":{"used_percentage":89,"resets_at":2000000}}}"#,
        );
        let s = usage(&p, 1000000 - (4 * 3600 + 53 * 60)).unwrap();
        assert!(s.starts_with("5h ◻◻◻◻◻ 5% 4h53m"), "got: {s}");
        assert!(s.contains("\u{2502} 7d ◼◼◼◼◻ 89%"), "got: {s}");
    }

    #[test]
    fn missing_rate_limits_is_skipped() {
        let p = payload("{}");
        assert_eq!(usage(&p, 0), None);
    }

    #[test]
    fn branch_marks_dirty_and_counts() {
        let g = GitInfo { branch: "main".into(), dirty: true, ahead: 0, behind: 1 };
        assert_eq!(branch(&g, Style::Nerd), "\u{e0a0} main * \u{21e3}1");
    }

    #[test]
    fn plain_branch_drops_the_glyph() {
        let g = GitInfo { branch: "main".into(), dirty: false, ahead: 2, behind: 0 };
        assert_eq!(branch(&g, Style::Plain), "main \u{21e1}2");
    }
}
