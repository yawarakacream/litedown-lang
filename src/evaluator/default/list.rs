use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    litedown_element::{CommandParameterValue, EnvironmentElement, LitedownElement},
    utility::html::HtmlElement,
};

enum ListKind {
    Dot,
    Number,
}
impl ListKind {
    fn to_html_tag(&self) -> &str {
        match self {
            ListKind::Dot => "ul",
            ListKind::Number => "ol",
        }
    }
}

pub struct List;

impl List {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(List {})
    }
}

impl EnvironmentEvaluator for List {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let kind = match &element.parameters.get("type") {
            Some(p) => match p {
                CommandParameterValue::String(p) => match p.as_str() {
                    "dot" => ListKind::Dot,
                    "number" => ListKind::Number,
                    _ => bail!("Illegal type: {}", p),
                },
                _ => bail!("Illegal type: {}", p),
            },
            None => ListKind::Dot,
        };

        let mut list = HtmlElement::new(kind.to_html_tag());

        for child in &element.children {
            match child {
                LitedownElement::Environment(child_environment) => {
                    match child_environment.name.as_str() {
                        "item" => {
                            let mut li = HtmlElement::new("li");
                            eval_with_litedown!(child_environment to li with lde);
                            list.append(li);
                        }
                        _ => bail!("Unknown environment: {}", child_environment.name),
                    }
                }
                _ => bail!("Only environment @item@ is allowed"),
            }
        }

        Ok(list)
    }
}
