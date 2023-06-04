use anyhow::{bail, Result};

use crate::{
    html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::{FunctionBodyForm, LitedownFunction, PassageElement},
    utility::html::HtmlElement,
};

pub fn evaluate_math(
    _: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    if function.body.is_empty() {
        bail!("'math' cannot be empty");
    }
    let mut container = match function.body.form {
        FunctionBodyForm::Block => {
            let mut container = HtmlElement::new("div");
            container.set_attr("class", "display-math");
            container
        }
        FunctionBodyForm::Inline => {
            let mut container = HtmlElement::new("span");
            container.set_attr("class", "inline-math");
            container
        }
    };
    for passage in &function.body.value {
        for passage_element in &passage.elements {
            match passage_element {
                PassageElement::String(string) => {
                    container.append_raw_text(string);
                }
                PassageElement::Function(_) => {
                    bail!("cannot write function in function 'math'");
                }
            }
        }
    }
    Ok(Some(container))
}
