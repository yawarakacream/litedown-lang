use anyhow::{bail, Result};

use crate::{deconstruct_required_arguments, tree::function::LitedownFunction};

#[derive(Debug)]
pub enum Math {
    Katex,
    Mathjax,
}

pub(super) fn evaluate_math(function: &LitedownFunction) -> Result<Option<Math>> {
    deconstruct_required_arguments!((mode) from function);
    let mode = mode.try_into_string()?.to_lowercase();
    match mode.as_str() {
        "katex" => Ok(Some(Math::Katex)),
        "mathjax" => Ok(Some(Math::Mathjax)),
        "none" => Ok(None),
        _ => bail!("unknown math mode"),
    }
}
