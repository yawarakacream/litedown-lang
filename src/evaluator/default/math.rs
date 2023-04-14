use anyhow::{bail, Result};

use crate::{
    evaluator::{
        environment::EnvironmentEvaluator, function::FunctionEvaluator, litedown::LitedownEvaluator,
    },
    tree::element::{EnvironmentElement, LitedownElement, PassageContent, PassageContentFunction},
    utility::html::HtmlElement,
};

pub struct DisplayMath {}

impl DisplayMath {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(DisplayMath {})
    }
}

impl EnvironmentEvaluator for DisplayMath {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut container = HtmlElement::new("div");
        container.set_attr("class", "display-math");
        for child in &element.children {
            match child {
                LitedownElement::Environment(_) => {
                    bail!("Cannot write environment in @math@ environment");
                }
                LitedownElement::Passage(passage) => {
                    for content in &passage.contents {
                        match content {
                            PassageContent::Function(_) => {
                                bail!("Cannot write function in @math@ environment");
                            }
                            PassageContent::Text(text) => {
                                container.append_raw_text(&text.value);
                            }
                        }
                    }
                }
            }
        }
        Ok(container)
    }
}

pub struct InlineMath;

impl InlineMath {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(InlineMath {})
    }
}

impl FunctionEvaluator for InlineMath {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        match &content.body {
            Some(body) => {
                let mut span = HtmlElement::new("span");
                span.set_attr("class", "inline-math");
                span.append_raw_text(body);
                Ok(Some(span))
            }
            None => bail!("The body is empty"),
        }
    }
}
