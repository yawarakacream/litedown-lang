use anyhow::{bail, Context, Result};

use crate::{
    deconstruct_required_arguments, html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::LitedownFunction, utility::html::HtmlElement,
};

pub fn evaluate_header(_: &Ld2HtmlEvaluator, function: &LitedownFunction) -> Result<HtmlElement> {
    let level = if function.parameters.is_empty() {
        "primary".to_string()
    } else {
        deconstruct_required_arguments!((level) from function);
        let level = level.try_into_string()?;
        if !&["primary", "secondary"].contains(&level.as_str()) {
            bail!("unknown level: {}", level);
        }
        level
    };

    let mut header_html = HtmlElement::new("div");
    header_html.set_attr("class", "header");
    header_html.set_attr("data-level", &level);
    let body = function
        .body
        .try_get_as_string()
        .context("function 'header' must have body")?;
    header_html.append_text(&body);
    Ok(header_html)
}
