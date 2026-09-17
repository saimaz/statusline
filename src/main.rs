//! Minimal powerline status line for Claude Code.
//! Reads the statusline JSON payload from stdin, prints one ANSI line.
//! Never fails: bad or missing input renders whatever segments it can.

mod git;
mod input;
mod render;
mod segments;

use std::io::Read;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let mut raw = String::new();
    let _ = std::io::stdin().read_to_string(&mut raw);
    let payload: input::Payload = serde_json::from_str(&raw).unwrap_or_default();

    let dir = payload
        .workspace
        .as_ref()
        .and_then(|w| w.current_dir.clone())
        .or_else(|| payload.cwd.clone());
    let git_info = dir.as_deref().and_then(git::read);

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let style = render::Style::detect();
    let segs = segments::build(&payload, git_info.as_ref(), now, style);
    println!("{}", render::line(&segs, style));
}
