use std::collections::HashMap;

use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    tree::element::EnvironmentElement,
    utility::html::HtmlElement,
};

pub struct Figure {
    tag_indices: HashMap<String, usize>,
}

impl Figure {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Figure {
            tag_indices: HashMap::new(),
        })
    }
}

impl EnvironmentEvaluator for Figure {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut figure = HtmlElement::new("figure");

        let mut caption = None;

        eval_with_litedown!(
            element to figure with lde;
            environment: {
                caption: (child_environment) => {
                    let mut figcaption_tag = HtmlElement::new("div");
                    figcaption_tag.set_attr("class", "tag");
                    if let Ok(tag) = child_environment.parameters.try_get("tag") {
                        let tag = tag.try_into_string()?;
                        let tag_index = self.tag_indices.entry(tag.to_string()).or_insert(1);
                        figcaption_tag.append_text(&format!("{tag} {tag_index}"));
                        *tag_index += 1;
                    } else if let Ok(raw_tag) = child_environment.parameters.try_get("raw-tag") {
                        let raw_tag = raw_tag.try_into_string()?;
                        figcaption_tag.append_text(&raw_tag);
                    } else {
                        bail!("No tag");
                    }

                    let mut figcaption_content = HtmlElement::new("div");
                    figcaption_content.set_attr("class", "content");
                    eval_with_litedown!(child_environment to figcaption_content with lde);

                    let mut figcaption = HtmlElement::new("figcaption");
                    figcaption.append(figcaption_tag);
                    figcaption.append(figcaption_content);
                    caption = Some(figcaption);
                }
            }
        );

        let caption = match caption {
            Some(caption) => caption,
            None => bail!("No caption found"),
        };

        figure.append(caption);

        Ok(figure)
    }
}
