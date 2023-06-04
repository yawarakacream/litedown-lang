use anyhow::{bail, Result};

use crate::{deconstruct_required_arguments, tree::function::LitedownFunction};

#[derive(Debug)]
pub enum Theme {
    Default,
    Paper,
}

pub(super) fn evaluate_theme(function: &LitedownFunction) -> Result<Theme> {
    deconstruct_required_arguments!((theme) from function);
    let theme = theme.try_into_string()?.to_lowercase();
    match theme.as_str() {
        "default" => Ok(Theme::Default),
        "paper" => Ok(Theme::Paper),
        _ => bail!("unknown theme"),
    }
}
