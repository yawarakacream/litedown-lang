use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{
        default::slide::page::create_slide_page, environment::EnvironmentEvaluator,
        litedown::LitedownEvaluator,
    },
    tree::element::EnvironmentElement,
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
        create_slide_page(|el| {
            let mut title = HtmlElement::new("div");
            title.set_attr("class", "title");

            let mut subtitle = None;
            let mut author = None;

            eval_with_litedown!(
                element to title with lde;
                environment: {
                    subtitle: (child_environment) => {
                        if author.is_some() {
                            bail!("Environment @subtitle@ is already written");
                        }
                        let mut subtitle_ = HtmlElement::new("div");
                        subtitle_.set_attr("class", "subtitle");
                        eval_with_litedown!(child_environment to subtitle_ with lde);
                        subtitle = Some(subtitle_);
                    }
                    author: (child_environment) => {
                        if author.is_some() {
                            bail!("Environment @author@ is already written");
                        }
                        let mut author_ = HtmlElement::new("div");
                        author_.set_attr("class", "author");
                        eval_with_litedown!(child_environment to author_ with lde);
                        author = Some(author_);
                    }
                }
            );

            if let Some(subtitle) = subtitle {
                title.append(subtitle);
            }

            if let Some(author) = author {
                title.append(author);
            }

            el.append(title);

            Ok(())
        })
    }
}
