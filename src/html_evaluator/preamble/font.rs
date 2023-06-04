use anyhow::{bail, Result};

use crate::tree::function::LitedownFunction;

#[derive(Debug)]
pub struct Font {
    pub family: FontFamily,
    pub size: String,
}

#[derive(Debug)]
pub enum FontFamily {
    Serif,
    SansSerif,
}

pub(super) fn evaluate_font(
    function: &LitedownFunction,
) -> Result<(Option<FontFamily>, Option<String>)> {
    let family = match function.parameters.get_by_name("family") {
        Some(family) => match family.try_into_string()?.as_str() {
            "serif" => Some(FontFamily::Serif),
            "sans-serif" => Some(FontFamily::SansSerif),
            _ => bail!("unknown font family"),
        },
        None => None,
    };
    let size = match function.parameters.get_by_name("size") {
        Some(size) => {
            let (number, unit) = size.try_into_float()?;
            if unit.is_empty() {
                bail!("font size must have unit");
            }
            Some(format!("{}{}", number, unit))
        }
        None => None,
    };
    Ok((family, size))
}
