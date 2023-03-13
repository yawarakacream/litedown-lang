use anyhow::Result;

use crate::{
    evaluator::{function::FunctionEvaluator, litedown::LitedownEvaluator},
    tree::element::PassageContentFunction,
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
        let mut el = HtmlElement::new("span");
        el.set_attr("class", "page-break");
        Ok(Some(el))
    }
}
