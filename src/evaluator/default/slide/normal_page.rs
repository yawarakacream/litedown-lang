use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    tree::element::EnvironmentElement,
    utility::html::HtmlElement,
};

use super::page::create_slide_page;

pub struct NormalPage {}

impl NormalPage {
    pub fn new() -> NormalPage {
        NormalPage {}
    }
}

impl EnvironmentEvaluator for NormalPage {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        create_slide_page(|el| {
            eval_with_litedown!(element to el with lde;
                environment: {
                    header: (child_environment) => {
                        let mut header = Header {};
                        el.append(header.eval(lde, child_environment)?);
                    }
                }
            );
            Ok(())
        })
    }
}

struct Header {}

impl EnvironmentEvaluator for Header {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let level = match element.parameters.get("level") {
            Some(level) => {
                let level = level.try_into_str()?;
                if level != "primary" && level != "secondary" {
                    bail!("Invalid level: {}", level);
                }
                level
            }
            None => "primary",
        };

        let mut header = HtmlElement::new("div");
        header.set_attr("class", "header");
        header.set_attr("data-level", level);
        eval_with_litedown!(element to header with lde);
        Ok(header)
    }
}
