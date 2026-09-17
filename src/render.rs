//! Powerline rendering with a fixed Catppuccin Mocha palette.
//! One fg color (crust) on colored segment backgrounds, rounded caps,
//! no stray resets between segments (that was the upstream bug).

pub const CRUST: Rgb = Rgb(30, 30, 46); // #1e1e2e
pub const PEACH: Rgb = Rgb(250, 179, 135); // #fab387
pub const YELLOW: Rgb = Rgb(249, 226, 175); // #f9e2af
pub const GREEN: Rgb = Rgb(166, 227, 161); // #a6e3a1
pub const SAPPHIRE: Rgb = Rgb(116, 199, 236); // #74c7ec
pub const LAVENDER: Rgb = Rgb(180, 190, 254); // #b4befe
pub const MAUVE: Rgb = Rgb(203, 166, 247); // #cba6f7

const CAP_LEFT: char = '\u{e0b6}'; // rounded left half circle
const CAP_RIGHT: char = '\u{e0b4}'; // rounded right half circle
const RESET: &str = "\x1b[0m";

/// Glyph set. Nerd caps and separators are private-use codepoints, so a
/// terminal without a patched font renders them as tofu boxes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Style {
    Nerd,
    Plain,
}

impl Style {
    /// `STATUSLINE_STYLE=nerd|plain` wins. Otherwise only terminals that ship a
    /// Nerd Font or a Nerd Font symbol fallback get the glyphs.
    pub fn detect() -> Self {
        match std::env::var("STATUSLINE_STYLE").unwrap_or_default().trim() {
            "nerd" => return Self::Nerd,
            "plain" => return Self::Plain,
            _ => {}
        }
        let program = std::env::var("TERM_PROGRAM").unwrap_or_default().to_ascii_lowercase();
        let term = std::env::var("TERM").unwrap_or_default();
        if matches!(program.as_str(), "ghostty" | "wezterm") || term == "xterm-kitty" {
            Self::Nerd
        } else {
            Self::Plain
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rgb(pub u8, pub u8, pub u8);

pub struct Segment {
    pub text: String,
    pub bg: Rgb,
    pub bold: bool,
}

impl Segment {
    pub fn new(text: impl Into<String>, bg: Rgb, bold: bool) -> Self {
        Self { text: text.into(), bg, bold }
    }
}

fn fg(c: Rgb) -> String {
    format!("\x1b[38;2;{};{};{}m", c.0, c.1, c.2)
}

fn fg_bg(f: Rgb, b: Rgb) -> String {
    format!("\x1b[38;2;{};{};{};48;2;{};{};{}m", f.0, f.1, f.2, b.0, b.1, b.2)
}

/// Render segments as one powerline: rounded cap, colored blocks joined by
/// half-circle separators, rounded cap, single trailing reset.
/// `Style::Plain` drops every Nerd glyph and lets the background color changes
/// separate the segments.
pub fn line(segments: &[Segment], style: Style) -> String {
    let mut out = String::new();
    for (i, seg) in segments.iter().enumerate() {
        if style == Style::Nerd {
            if i == 0 {
                out.push_str(&fg(seg.bg));
                out.push(CAP_LEFT);
            } else {
                out.push_str(&fg_bg(segments[i - 1].bg, seg.bg));
                out.push(CAP_RIGHT);
            }
        }
        out.push_str(&fg_bg(CRUST, seg.bg));
        if seg.bold {
            out.push_str("\x1b[1m");
        }
        out.push(' ');
        out.push_str(&seg.text);
        out.push(' ');
        if seg.bold {
            out.push_str("\x1b[22m");
        }
    }
    if let (Some(last), Style::Nerd) = (segments.last(), style) {
        out.push_str(RESET);
        out.push_str(&fg(last.bg));
        out.push(CAP_RIGHT);
    }
    out.push_str(RESET);
    out
}

/// A percentage as a squares bar, `width` cells.
pub fn bar(pct: f64, width: usize) -> String {
    let clamped = pct.clamp(0.0, 100.0);
    let filled = ((clamped / 100.0) * width as f64).round() as usize;
    let filled = filled.min(width);
    "\u{25fc}".repeat(filled) + &"\u{25fb}".repeat(width - filled)
}

/// Seconds until reset as "2d0h", "4h53m" or "12m".
pub fn until(secs: i64) -> String {
    let secs = secs.max(0);
    let d = secs / 86_400;
    let h = (secs % 86_400) / 3_600;
    let m = (secs % 3_600) / 60;
    if d > 0 {
        format!("{d}d{h}h")
    } else if h > 0 {
        format!("{h}h{m}m")
    } else {
        format!("{m}m")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bar_rounds_and_clamps() {
        assert_eq!(bar(0.0, 5), "◻◻◻◻◻");
        assert_eq!(bar(23.0, 8), "◼◼◻◻◻◻◻◻");
        assert_eq!(bar(89.0, 5), "◼◼◼◼◻");
        assert_eq!(bar(100.0, 5), "◼◼◼◼◼");
        assert_eq!(bar(250.0, 5), "◼◼◼◼◼");
        assert_eq!(bar(-5.0, 5), "◻◻◻◻◻");
    }

    #[test]
    fn until_formats_all_ranges() {
        assert_eq!(until(2 * 86_400 + 600), "2d0h");
        assert_eq!(until(4 * 3_600 + 53 * 60), "4h53m");
        assert_eq!(until(12 * 60), "12m");
        assert_eq!(until(-30), "0m");
    }

    #[test]
    fn no_reset_between_segments() {
        let segs = [
            Segment::new("+725", GREEN, true),
            Segment::new("Fable 5", SAPPHIRE, true),
        ];
        let s = line(&segs, Style::Nerd);
        // exactly one reset before the closing cap and one at the end
        assert_eq!(s.matches("\x1b[0m").count(), 2);
        assert!(s.ends_with(RESET));
    }

    #[test]
    fn plain_style_has_no_private_use_glyphs() {
        let segs = [
            Segment::new("~/D/smoobu", PEACH, false),
            Segment::new("master", YELLOW, false),
        ];
        let s = line(&segs, Style::Plain);
        assert!(!s.chars().any(|c| ('\u{e000}'..='\u{f8ff}').contains(&c)), "got: {s:?}");
        assert!(s.contains(" ~/D/smoobu ") && s.contains(" master "), "got: {s:?}");
        assert_eq!(s.matches(RESET).count(), 1);
        assert!(s.ends_with(RESET));
    }
}
