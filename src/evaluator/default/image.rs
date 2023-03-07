use anyhow::{bail, Result};

use crate::{
    evaluator::{function::FunctionEvaluator, litedown::LitedownEvaluator},
    litedown_element::PassageContentFunction,
    utility::html::HtmlElement,
};

pub struct Image;

impl Image {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(Image {})
    }
}

impl FunctionEvaluator for Image {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        match &content.body {
            Some(body) => {
                let mut img = HtmlElement::new_void("img");
                img.set_attr("src", body.as_str());
                Ok(Some(img))
            }
            None => bail!("src is empty"),
        }
    }
}
