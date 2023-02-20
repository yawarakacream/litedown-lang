use crate::{
    attrs,
    evaluator::{
        environment::{EnvironmentEvaluator, EnvironmentEvaluatorComponents},
        litedown::LitedownEvaluator,
    },
    litedown_element::{CommandParameterValue, EnvironmentElement},
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

pub struct List {
    kind: ListKind,
}

impl List {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(List {
            kind: ListKind::Dot,
        })
    }
}

impl EnvironmentEvaluatorComponents for List {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        self.kind = match &element.parameters.get("type") {
            Some(p) => match p {
                CommandParameterValue::String(p) => match p.as_str() {
                    "dot" => ListKind::Dot,
                    "number" => ListKind::Number,
                    _ => return Err(format!("Illegal type: {}", p)),
                },
                _ => return Err(format!("Illegal type: {}", p)),
            },
            None => ListKind::Dot,
        };
        lde.writer
            .open_element(self.kind.to_html_tag(), attrs! {"class" => "list"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element(self.kind.to_html_tag())
    }

    fn eval_child_environment(
        &self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        match element.name.as_str() {
            "item" => {
                let mut author = Item {};
                author.eval(lde, element)
            }
            _ => Err(format!("Unknown environment: {}", element.name)),
        }
    }
}

struct Item;

impl EnvironmentEvaluatorComponents for Item {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        _element: &EnvironmentElement,
    ) -> Result<(), String> {
        lde.writer.open_element("li", attrs! {"class" => "item"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.close_element("li")
    }
}
