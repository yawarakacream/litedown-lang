use anyhow::{bail, Result};

use crate::{
    evaluate_with_ld2html_evaluator, html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::LitedownFunction, utility::html::HtmlElement,
};

pub fn evaluate_footer(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<HtmlElement> {
    let mut footer_html = HtmlElement::new("div");
    footer_html.set_attr("class", "footer");
    evaluate_with_ld2html_evaluator!(function to footer_html with evaluator);
    if footer_html.is_child_empty() {
        bail!("function 'footer' must have body");
    }
    Ok(footer_html)
}
