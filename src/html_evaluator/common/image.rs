use anyhow::Result;

use crate::{
    html_evaluator::litedown::Ld2HtmlEvaluator, tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_image(
    _: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut img_html = HtmlElement::new_void("img");

    img_html.set_attr("src", &function.body.try_get_as_string()?);

    if let Some(height) = function.arguments.get_by_name("height") {
        let height = height.try_into_float()?;
        let height = format!("{}{}", height.0, height.1);
        img_html.set_attr("height", &height);
    }

    Ok(Some(img_html))
}
