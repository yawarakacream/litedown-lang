use anyhow::{bail, Result};

use crate::{deconstruct_required_arguments, tree::function::LitedownFunction};

#[derive(Debug)]
pub struct PageSize {
    pub width: String,
    pub height: String,
}

pub(super) fn evaluate_page_size(function: &LitedownFunction) -> Result<PageSize> {
    match function.arguments.len() {
        1 => {
            deconstruct_required_arguments!((size) from function);
            let size = size.try_into_string()?.to_lowercase();
            match size.as_str() {
                "a4" | "a4-portrait" => Ok(PageSize {
                    width: "210mm".to_string(),
                    height: "297mm".to_string(),
                }),
                "a4-landscape" => Ok(PageSize {
                    width: "297mm".to_string(),
                    height: "210mm".to_string(),
                }),
                "powerpoint-16:9" => Ok(PageSize {
                    width: "33.867cm".to_string(),
                    height: "19.05cm".to_string(),
                }),
                _ => bail!("unknown built-in size"),
            }
        }
        2 => {
            deconstruct_required_arguments!((width, height) from function);
            let width = width.try_into_string()?.clone();
            let height = height.try_into_string()?.clone();
            Ok(PageSize { width, height })
        }
        _ => bail!("invalid size"),
    }
}
