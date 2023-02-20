use crate::{
    attrs,
    evaluator::{function::FunctionEvaluator, litedown::LitedownEvaluator},
    litedown_element::PassageContentFunction,
};

pub struct BoldText;

impl BoldText {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(BoldText {})
    }
}

impl FunctionEvaluator for BoldText {
    fn eval(
        &self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<(), String> {
        match &content.body {
            Some(body) => lde.writer.add_element("strong", attrs! {}, &body),
            None => Err("The body is empty".to_string()),
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
        &self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<(), String> {
        match &content.body {
            Some(body) => lde
                .writer
                .add_element("span", attrs! {"class" => "inline-math"}, &body),
            None => Err("The body is empty".to_string()),
        }
    }
}
