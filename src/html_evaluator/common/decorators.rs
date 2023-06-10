use anyhow::{Context, Result};

use crate::{
    deconstruct_required_arguments, evaluate_with_ld2html_evaluator,
    html_evaluator::litedown::Ld2HtmlEvaluator, tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_strong(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut strong_html = HtmlElement::new("strong");
    evaluate_with_ld2html_evaluator!(function to strong_html with evaluator);
    Ok(Some(strong_html))
}

pub fn evaluate_divider(_: &Ld2HtmlEvaluator, _: &LitedownFunction) -> Result<Option<HtmlElement>> {
    let hr = HtmlElement::new_void("hr");
    Ok(Some(hr))
}

pub fn evaluate_link(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut anchor_html = HtmlElement::new("a");

    if function.parameters.is_empty() {
        let body = function
            .body
            .try_get_as_string()
            .context("href not found")?;
        anchor_html.append_raw_text(&body);

        anchor_html.set_attr("href", &body);
    } else {
        deconstruct_required_arguments!((href) from function);
        let href = href.try_into_string()?;

        evaluate_with_ld2html_evaluator!(function to anchor_html with evaluator);

        anchor_html.set_attr("href", &href);
    }

    Ok(Some(anchor_html))
}
