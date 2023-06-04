use anyhow::{bail, Result};

use crate::{
    html_evaluator::litedown::Ld2HtmlEvaluator, tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_pagebreak(
    _: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    if !function.body.is_empty() {
        bail!("cannot write body in 'pagebreak'");
    }

    let mut el = HtmlElement::new("span");
    el.set_attr("class", "page-break");
    Ok(Some(el))
}
