use anyhow::{bail, Result};

use crate::{deconstruct_required_arguments, tree::function::LitedownFunction};

#[derive(Debug)]
pub struct PagePadding {
    pub horizontal: String,
    pub vertical: String,
}

pub(super) fn evaluate_page_padding(function: &LitedownFunction) -> Result<PagePadding> {
    match function.arguments.len() {
        2 => {
            deconstruct_required_arguments!((horizontal, vertical) from function);
            let horizontal = horizontal.try_into_string()?.clone();
            let vertical = vertical.try_into_string()?.clone();
            Ok(PagePadding {
                horizontal,
                vertical,
            })
        }
        _ => bail!("invalid size"),
    }
}
