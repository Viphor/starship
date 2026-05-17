# Spec: Powerline Separator

## Objective

Add a global `[separator]` configuration section to starship that automatically injects
powerline-style connector strings between prompt segments. The separator's foreground color
is taken from the previous segment's background, and its background color is taken from the
next segment's background. When no next-segment background exists (end of prompt), the
separator's background falls back to the terminal default (cleared). This feature is disabled
by default to preserve all existing configurations.

**User story:** As a user who wants a powerline prompt aesthetic, I want to set a global
separator symbol so that adjacent background-colored segments flow into each other visually
without manually configuring prefix/suffix strings on each module.

**Acceptance criteria:**
- `[separator]` section in `starship.toml` with `left_symbol`, `right_symbol`, and `disabled` fields.
- When enabled, a separator glyph is automatically injected between every pair of adjacent
  non-empty segments where the left segment has a background color.
- Separator foreground = left segment's background color.
- Separator background = right segment's background color (or cleared if none).
- `left_symbol` is used for the left prompt; `right_symbol` is used for the right prompt.
- Feature is `disabled = true` by default (no change to existing behavior).
- Config is documented; schema is updated.

## Tech Stack

- Language: Rust (edition 2024)
- Config: TOML via `serde` + `toml` crate, schema via `schemars`
- Styling: `nu_ansi_term` for ANSI string construction

## Commands

```sh
# Build
cargo build

# Test (all)
cargo test

# Test (specific)
cargo test separator

# Lint
cargo clippy -- -D warnings

# Format
cargo fmt
```

## Project Structure

```
src/
  configs/
    separator.rs     # NEW: SeparatorConfig struct
    mod.rs           # Add separator module + FullConfig field
    starship_root.rs # Add separator field to StarshipRootConfig
  print.rs           # Add inject_separators() post-processing pass
  segment.rs         # (unchanged — injection happens at AnsiString level)
docs/
  config/README.md   # Document [separator] section
  spec/separator.md  # This file
```

## Implementation Approach

Separator injection happens **after** `root_module.ansi_strings_for_width()` and **before**
`AnsiStrings(...).to_string()` in `get_prompt()`. At that point, each `AnsiString` carries a
resolved `nu_ansi_term::Style` with concrete colors (no `prev_fg`/`prev_bg` references left).

The `inject_separators()` function:
1. Iterates through `Vec<AnsiString>`.
2. Tracks the `background` color of the last non-empty `AnsiString`.
3. When it finds a new non-empty `AnsiString` and the previous segment had a background color,
   inserts a separator `AnsiString` with `fg = prev_bg` and `bg = current_bg` (or `None`).
4. Returns the expanded `Vec<AnsiString>`.

This approach requires **no changes** to `Segment`, `Module`, or the formatter pipeline.

## Config Shape

```toml
[separator]
disabled = true          # opt-in, default disabled
left_symbol  = ""      # used in left prompt
right_symbol = ""      # used in right prompt
```

### SeparatorConfig struct

```rust
#[derive(Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct SeparatorConfig<'a> {
    pub disabled: bool,
    pub left_symbol: &'a str,
    pub right_symbol: &'a str,
}

impl Default for SeparatorConfig<'_> {
    fn default() -> Self {
        Self {
            disabled: true,
            left_symbol: "",
            right_symbol: "",
        }
    }
}
```

### Loading convention

Follows the same `'a`-lifetime pattern as `FillConfig<'a>` — strings borrow from the parsed
TOML `Value` tree owned by `context.config`. Loaded in `get_prompt()` as:

```rust
let sep_config = SeparatorConfig::try_load(context.config.get_module_config("separator"));
```

No changes to `StarshipRootConfig` are needed. `separator` is added to `FullConfig` (for
schema generation) following the same pattern as `fill`.

## Code Style

Follow the existing pattern in `src/modules/fill.rs`:

```rust
pub fn inject_separators<'a>(
    strings: Vec<AnsiString<'a>>,
    symbol: &str,
    target: Target,
    config: &SeparatorConfig,
) -> Vec<AnsiString<'a>> {
    if config.disabled {
        return strings;
    }
    // ... injection logic
}
```

- No `unwrap()` on user-supplied config values; use `unwrap_or_default()`.
- Prefer early returns for the disabled case.

## Testing Strategy

- Framework: Rust's built-in `#[test]` + `crate::test::ModuleRenderer`
- Test file: inline `#[cfg(test)]` mod inside `src/print.rs` (same pattern as `ansi_line`)
- Coverage:
  - `inject_separators` is disabled by default → no output change
  - Two segments with backgrounds → separator injected with correct fg/bg
  - Last segment with background, next segment has no background → separator bg is cleared
  - First segment with no background → no separator before it
  - Right prompt uses `right_symbol` instead of `left_symbol`
  - Separator tracking resets across `LineTerm` — no separator injected between segments on different lines

## Boundaries

- **Always:** Run `cargo test` and `cargo clippy` before marking done. Keep `disabled = true` as
  the default so no existing config is affected.
- **Ask first:** Changes to `StarshipRootConfig` struct fields (affects serialization/schema),
  adding new `Segment` variants (breaks match exhaustiveness everywhere).
- **Never:** Change default behavior of any existing module; remove or rename existing config keys.

## Success Criteria

1. `cargo test` passes with no regressions.
2. `cargo clippy -- -D warnings` is clean.
3. A user who adds the following to `starship.toml` sees powerline separators:
   ```toml
   [separator]
   disabled = false
   left_symbol = ""
   ```
4. A user with no `[separator]` section sees no change in their prompt.
5. Docs in `docs/config/README.md` describe the `[separator]` section with all fields.

## Open Questions

1. **Separator between segments on different lines:** Resolved — tracking state resets on
   `LineTerm` segments, so separators are never injected across line boundaries.

2. **`$fill` interaction:** Suppressing separator injection adjacent to fill segments is out of
   scope for now. Flagged for future consideration: fill segments don't have a meaningful
   resolved background color at the injection point, so naively injecting a separator next to
   one will produce incorrect colors.
