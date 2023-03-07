use anyhow::Result;

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    litedown_element::EnvironmentElement,
    utility::html::HtmlElement,
};

pub struct Section {
    index: usize,
}

impl Section {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Section { index: 1 })
    }
}

impl EnvironmentEvaluator for Section {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut section = HtmlElement::new("section");

        let mut header = HtmlElement::new("div");
        header.set_attr("class", "header");
        header.append_text(&format!("{}.", self.index));
        self.index += 1;
        section.append(header);

        eval_with_litedown!(element to section with lde);
        Ok(section)
    }
}
