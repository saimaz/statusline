# statusline

Minimal powerline status line for Claude Code. Reads the statusline JSON
payload from stdin, prints one line: directory, git branch, lines changed,
model, context bar, and 5h / 7d usage bars. Catppuccin Mocha, no config file.

We wrote it because the statusline we used before needed a wrapper script to
work around two bugs (a stray color reset drawing a black gap, and `git status`
creating `.git/index.lock` mid-session). Owning ~500 lines of Rust turned out
to be simpler than maintaining workarounds. Everything renders from data Claude
Code already pipes in (`rate_limits` needs Claude Code 2.1.80+), plus one
`git --no-optional-locks status` call, so there is no state, no API, no lock.

Any segment without data is skipped, and bad input renders an empty line
instead of failing, so the status line never breaks your session.

## Install

```sh
brew tap saimaz/tap
brew install statusline
```

Wire it up in `~/.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "statusline"
  }
}
```

## Glyphs

The powerline caps and the branch glyph are Nerd Font codepoints, so a terminal
without a patched font draws them as boxes. The status line picks a style on
its own: Ghostty, WezTerm and kitty get the glyphs, everything else (Terminal.app,
the VS Code and JetBrains terminals, anything unknown) gets a plain style where
the colored blocks separate the segments and no Nerd glyph is printed.

Override it per terminal with `STATUSLINE_STYLE`:

```sh
export STATUSLINE_STYLE=nerd   # force the glyphs (iTerm2 with a patched font)
export STATUSLINE_STYLE=plain  # force the plain style
```

We use JetBrainsMono Nerd Font.

## Releasing

Bump `version` in `Cargo.toml`, then tag and push:

```sh
git tag v0.2.0 && git push --tags
```

The release workflow builds macOS binaries (arm64 + x86_64), attaches them to
a GitHub Release, and bumps the formula in
[saimaz/homebrew-tap](https://github.com/saimaz/homebrew-tap). Users get it
with a plain `brew upgrade`.

## Development

To hack on it and test a local build without touching the brew install:

```sh
git clone git@github.com:saimaz/statusline.git
cd statusline
cargo test
cargo build --release
```

Try it without Claude Code:

```sh
printf '{"model":{"display_name":"Fable 5"},"workspace":{"current_dir":"'$PWD'"},"cost":{"total_lines_added":7,"total_lines_removed":2},"context_window":{"used_percentage":23},"rate_limits":{"five_hour":{"used_percentage":5,"resets_at":'$(($(date +%s)+17580))'}}}' | target/release/statusline
```

Point `statusLine.command` at the absolute path of `target/release/statusline`
while testing, switch back to `statusline` when done.

Layout and colors are constants in `src/render.rs` and `src/segments.rs`,
change them there. `STATUSLINE_STYLE=nerd|plain` also works when testing, and
`Style::detect` in `src/render.rs` holds the terminal list.
