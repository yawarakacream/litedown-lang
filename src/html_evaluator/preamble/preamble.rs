use anyhow::{bail, Context, Result};

use crate::{evaluate_litedown_function, tree::function::LitedownFunction};

use super::{
    font::{evaluate_font, Font, FontFamily},
    math::{evaluate_math, Math},
    page_padding::{evaluate_page_padding, PagePadding},
    page_size::{evaluate_page_size, PageSize},
    theme::{evaluate_theme, Theme},
};

#[derive(Debug)]
pub struct Preamble {
    pub page_size: PageSize,
    pub page_padding: PagePadding,
    pub theme: Theme,
    pub font: Font,
    pub math: Option<Math>,
}

pub fn evaluate_preamble(function: &LitedownFunction) -> Result<Preamble> {
    let mut page_size = None;
    let mut page_padding = PagePadding {
        horizontal: "2em".to_string(),
        vertical: "1em".to_string(),
    };
    let mut theme = Theme::Default;
    let mut font = Font {
        family: FontFamily::SansSerif,
        size: "10.5pt".to_string(),
    };
    let mut math = Some(Math::Katex);

    if function.body.is_empty() {
        bail!("preamble must have body");
    }
    evaluate_litedown_function!(function;
        page_size: (child_function) => {
            page_size = Some(evaluate_page_size(child_function)?);
        }
        page_padding: (child_function) => {
            page_padding = evaluate_page_padding(child_function)?
        }
        theme: (child_function) => {
            theme = evaluate_theme(child_function)?;
        }
        font: (child_function) => {
            let new_font = evaluate_font(child_function)?;
            if let Some(family) = new_font.0 {
                font.family = family;
            }
            if let Some(size) = new_font.1 {
                font.size = size;
            }
        }
        math: (child_function) => {
            math = evaluate_math(child_function)?;
        }
    );

    let page_size = page_size.context("page-size not found")?;

    Ok(Preamble {
        page_size,
        page_padding,
        theme,
        font,
        math,
    })
}
