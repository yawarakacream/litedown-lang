use anyhow::{bail, Result};

use crate::{
    html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::{FunctionBodyForm, LitedownFunction, PassageElement},
    utility::html::HtmlElement,
};

pub fn evaluate_title(_: &Ld2HtmlEvaluator, function: &LitedownFunction) -> Result<HtmlElement> {
    let mut title_html: HtmlElement = HtmlElement::new("div");
    title_html.set_attr("class", "title");

    let mut author = None;

    for passage in &function.body.value {
        for passage_element in &passage.elements {
            match &passage_element {
                PassageElement::String(string) => {
                    title_html.append_text(&string);
                }
                PassageElement::Function(child_function) => match child_function.name.as_str() {
                    "author" => {
                        if child_function.body.form != FunctionBodyForm::Block {
                            bail!("function 'author' must be block");
                        }
                        if author.is_some() {
                            bail!("function 'author' is already written");
                        }
                        author = Some(child_function.body.try_get_as_string()?);
                    }
                    _ => bail!("unknown function: '{}'", child_function.name),
                },
            }
        }
    }

    if let Some(author) = author {
        let mut author_html = HtmlElement::new("div");
        author_html.set_attr("class", "author");
        author_html.append_text(&author);
        title_html.append(author_html);
    }

    Ok(title_html)
}
