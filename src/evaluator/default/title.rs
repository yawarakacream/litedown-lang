use crate::{
    attrs,
    evaluator::{
        environment::{EnvironmentEvaluator, EnvironmentEvaluatorComponents},
        litedown::LitedownEvaluator,
    },
    litedown_element::EnvironmentElement,
};

pub struct Title;

impl Title {
    pub fn new() -> Title {
        Title {}
    }
}

impl EnvironmentEvaluatorComponents for Title {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        _element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.writer.open_element("div", attrs! {"class" => "title"})
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
            "author" => {
                let mut author = Author {};
                author.eval(lde, element)
            }
            _ => lde.eval_environment(element),
        }
    }
}

struct Author;

impl EnvironmentEvaluatorComponents for Author {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        _element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.writer.open_element("div", attrs! {"class" => "author"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element("div")
    }

    fn open_passage(&mut self, _lde: &mut LitedownEvaluator) -> Result<(), String> {
        Ok(())
    }

    fn close_passage(&mut self, _lde: &mut LitedownEvaluator) -> Result<(), String> {
        Ok(())
    }
}
