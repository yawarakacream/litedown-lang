use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    litedown_element::EnvironmentElement,
    utility::html::HtmlElement,
};

pub struct Title {}

impl Title {
    pub fn new() -> Title {
        Title {}
    }
}

impl EnvironmentEvaluator for Title {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut title = HtmlElement::new("div");
        title.set_attr("class", "title");

        let mut author = None;

        eval_with_litedown!(
            element to title with lde
            @author@ (child_environment) {
                if author.is_some() {
                    bail!("Environment @author@ is already written");
                }
                let mut author_ = HtmlElement::new("div");
                author_.set_attr("class", "author");
                eval_with_litedown!(child_environment to author_ with lde);
                author = Some(author_);
            }
        );

        if let Some(author) = author {
            title.append(author);
        }

        Ok(title)
    }
}
