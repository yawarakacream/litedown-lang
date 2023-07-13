use anyhow::Result;

use crate::{
    evaluate_with_ld2html_evaluator, html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::LitedownFunction, utility::html::HtmlElement,
};

pub fn evaluate_absolute_block(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut style = "position: absolute;".to_string();
    for name in &["top", "bottom", "left", "right", "width", "height"] {
        if let Some(arg) = function.arguments.get_by_name(name) {
            let arg = arg.try_into_float()?;
            style.push_str(&format!("{}: {}{};", name, arg.0, arg.1));
        }
    }

    let mut block_html = HtmlElement::new("div");
    block_html.set_attr("style", &style);
    evaluate_with_ld2html_evaluator!(function to block_html with evaluator);
    Ok(Some(block_html))
}
