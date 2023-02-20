use crate::{
    attrs,
    litedown_element::{
        Element, EnvironmentElement, PassageContent, PassageContentFunction, PassageContentText,
        PassageElement,
    },
};

use super::litedown::LitedownEvaluator;

pub trait EnvironmentEvaluator {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String>;
}

pub trait EnvironmentEvaluatorComponents {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String>;
    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String>;

    fn open_passage(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.open_element("p", attrs! {})
    }
    fn close_passage(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element("p")
    }

    fn eval_text(
        &mut self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentText,
    ) -> Result<(), String> {
        lde.writer.write_inner(&content.0)
    }
    fn eval_function(
        &mut self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<(), String> {
        lde.eval_function(content)
    }

    fn eval_child_environment(
        &self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.eval_environment(element)
    }
}

impl<T: EnvironmentEvaluatorComponents> EnvironmentEvaluator for T {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        self.open_environment(lde, &element)?;

        for child in &element.children {
            match child {
                Element::Environment(child_environment) => {
                    self.eval_child_environment(lde, &child_environment)?;
                }
                Element::Passage(PassageElement(contents)) => {
                    self.open_passage(lde)?;

                    for content in contents {
                        match content {
                            PassageContent::Text(content) => {
                                self.eval_text(lde, content)?;
                            }
                            PassageContent::Function(content) => {
                                self.eval_function(lde, content)?;
                            }
                        }
                    }

                    self.close_passage(lde)?;
                }
            }
        }

        self.close_environment(lde)?;

        Ok(())
    }
}
