use anyhow::{bail, Result};

use crate::{
    evaluator::{function::FunctionEvaluator, litedown::LitedownEvaluator},
    litedown_element::PassageContentFunction,
    utility::html::HtmlElement,
};

pub struct PageBreak;

impl PageBreak {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(PageBreak {})
    }
}

impl FunctionEvaluator for PageBreak {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        _: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        let mut el = HtmlElement::new("div");
        el.set_attr("class", "page-break");
        Ok(Some(el))
    }
}

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
