use crate::{
    attrs,
    evaluator::environment::EnvironmentEvaluator,
    evaluator::{environment::EnvironmentEvaluatorComponents, litedown::LitedownEvaluator},
    litedown_element::EnvironmentElement,
};

use super::title::Title;

pub struct Document;

impl Document {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Document {})
    }
}

impl EnvironmentEvaluatorComponents for Document {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        _element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.writer
            .open_element("div", attrs! {"class" => "document"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element("div")
    }

    fn eval_child_environment(
        &self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        match element.name.as_str() {
            "title" => {
                let mut title = Title::new();
                title.eval(lde, element)
            }
            _ => lde.eval_environment(element),
        }
    }
}
