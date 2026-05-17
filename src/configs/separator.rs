use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize)]
#[cfg_attr(
    feature = "config-schema",
    derive(schemars::JsonSchema),
    schemars(deny_unknown_fields)
)]
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
