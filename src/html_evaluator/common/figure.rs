use anyhow::{bail, Result};

use crate::{
    evaluate_with_ld2html_evaluator, html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::LitedownFunction, utility::html::HtmlElement,
};

pub fn evaluate_figure(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut figure_content_html = HtmlElement::new("div");
    figure_content_html.set_attr("class", "content");

    let mut figcaption_html = None;

    evaluate_with_ld2html_evaluator!(function to figure_content_html with evaluator;
        function: {
            caption: (child_function) => {
                figcaption_html = {
                    let mut figcaption_html = HtmlElement::new("figcaption");

                    figcaption_html.append({
                        let mut figcaption_tag_html = HtmlElement::new("div");
                        if let Some(raw_tag) = child_function.arguments.get_by_name("raw_tag") {
                            let raw_tag = raw_tag.try_into_string()?;
                            figcaption_tag_html.append_text(&raw_tag);
                        } else {
                            bail!("no tag found");
                        }

                        figcaption_tag_html
                    });

                    if !child_function.body.is_empty() {
                        figcaption_html.append_text("：");
                        figcaption_html.append({
                            let mut figcaption_content_html = HtmlElement::new("div");
                            figcaption_content_html.set_attr("class", "content");
                            evaluate_with_ld2html_evaluator!(child_function to figcaption_content_html with evaluator);

                            figcaption_content_html
                        });
                    }

                    Some(figcaption_html)
                }
            }
        }
    );

    let mut figure_html = HtmlElement::new("figure");

    figure_html.append(figure_content_html);

    match figcaption_html {
        Some(figcaption_html) => figure_html.append(figcaption_html),
        None => bail!("no caption found"),
    };

    Ok(Some(figure_html))
}
