use crate::{
    attrs,
    evaluator::{
        environment::{EnvironmentEvaluator, EnvironmentEvaluatorComponents},
        litedown::LitedownEvaluator,
    },
    litedown_element::EnvironmentElement,
};

pub struct Section;

impl Section {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Section {})
    }
}

impl EnvironmentEvaluatorComponents for Section {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        _element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.writer
            .open_element("section", attrs! {"class" => "section"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element("section")
    }

    fn eval_child_environment(
        &self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        match element.name.as_str() {
            _ => lde.eval_environment(element),
        }
    }
}
