use anyhow::{bail, Result};

use crate::{
    deconstruct_required_arguments, evaluate_litedown_function, evaluate_with_ld2html_evaluator,
    html_evaluator::litedown::Ld2HtmlEvaluator, tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

enum Marker {
    Dot,
    Number,
}

impl Marker {
    fn to_html_tag(&self) -> &str {
        match self {
            Marker::Dot => "ul",
            Marker::Number => "ol",
        }
    }
}

pub fn evaluate_list(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let marker = if function.parameters.is_empty() {
        Marker::Dot
    } else {
        deconstruct_required_arguments!((marker) from function);
        let marker = marker.try_into_string()?;
        match marker.as_str() {
            "dot" => Marker::Dot,
            "number" => Marker::Number,
            _ => bail!("unknown marker: {}", marker),
        }
    };

    let mut list_html = HtmlElement::new(marker.to_html_tag());
    evaluate_litedown_function!(function;
        item: (child_function) => {
            let mut li_html = HtmlElement::new("li");
            evaluate_with_ld2html_evaluator!(child_function to li_html with evaluator);
            list_html.append(li_html);
        }
    );
    Ok(Some(list_html))
}
