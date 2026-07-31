//! Claude Code statusline stdin payload (subset we render).
//! Schema: https://code.claude.com/docs/en/statusline.md

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
pub struct Payload {
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default)]
    pub model: Option<Model>,
    #[serde(default)]
    pub workspace: Option<Workspace>,
    #[serde(default)]
    pub cost: Option<Cost>,
    #[serde(default)]
    pub context_window: Option<ContextWindow>,
    #[serde(default)]
    pub rate_limits: Option<RateLimits>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Model {
    #[serde(default)]
    pub display_name: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Workspace {
    #[serde(default)]
    pub current_dir: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Cost {
    #[serde(default)]
    pub total_lines_added: i64,
    #[serde(default)]
    pub total_lines_removed: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct ContextWindow {
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub context_window_size: Option<i64>,
    #[serde(default)]
    pub current_usage: Option<CurrentUsage>,
}

#[derive(Debug, Default, Deserialize)]
pub struct CurrentUsage {
    #[serde(default)]
    pub input_tokens: i64,
    #[serde(default)]
    pub cache_creation_input_tokens: i64,
    #[serde(default)]
    pub cache_read_input_tokens: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateLimits {
    #[serde(default)]
    pub five_hour: Option<RateLimitWindow>,
    #[serde(default)]
    pub seven_day: Option<RateLimitWindow>,
}

#[derive(Debug, Default, Deserialize)]
pub struct RateLimitWindow {
    #[serde(default)]
    pub used_percentage: Option<f64>,
    #[serde(default)]
    pub resets_at: Option<i64>,
}

impl ContextWindow {
    /// used_percentage straight from the payload, or recomputed from
    /// current_usage with the same input-only formula Claude Code uses.
    pub fn used_pct(&self) -> Option<f64> {
        if let Some(p) = self.used_percentage {
            return Some(p);
        }
        let usage = self.current_usage.as_ref()?;
        let size = self.context_window_size.filter(|s| *s > 0)?;
        let input = usage.input_tokens
            + usage.cache_creation_input_tokens
            + usage.cache_read_input_tokens;
        Some(input as f64 / size as f64 * 100.0)
    }
}
