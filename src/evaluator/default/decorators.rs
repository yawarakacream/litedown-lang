use anyhow::{bail, Result};

use crate::{
    evaluator::{function::FunctionEvaluator, litedown::LitedownEvaluator},
    tree::element::PassageContentFunction,
    utility::html::HtmlElement,
};

pub struct BoldText;

impl BoldText {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(BoldText {})
    }
}

impl FunctionEvaluator for BoldText {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        match &content.body {
            Some(body) => {
                let mut strong = HtmlElement::new("strong");
                strong.append_text(body);
                Ok(Some(strong))
            }
            None => bail!("The body is empty"),
        }
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

pub struct Link;

impl Link {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(Link {})
    }
}

impl FunctionEvaluator for Link {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        let mut anchor = HtmlElement::new("a");
        let body = match &content.body {
            Some(body) => body,
            None => bail!("Body is empty"),
        };

        let href = match &content.parameters.get("") {
            Some(href) => href.try_into_string()?,
            None => body,
        };

        anchor.append_raw_text(body);
        anchor.set_attr("href", href);
        Ok(Some(anchor))
    }
}
