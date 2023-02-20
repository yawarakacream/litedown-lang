use std::collections::HashMap;

use crate::{
    litedown_element::{Element, EnvironmentElement, LitedownAst, PassageContentFunction},
    utility::html::HtmlWriter,
};

use super::{
    default::{
        decorators::{BoldText, InlineMath},
        document::Document,
        list::List,
        section::Section,
    },
    environment::EnvironmentEvaluator,
    function::FunctionEvaluator,
};

pub struct LitedownEvaluator<'a> {
    pub writer: HtmlWriter,
    environments: HashMap<&'a str, fn() -> Box<dyn EnvironmentEvaluator>>,
    functions: HashMap<&'a str, fn() -> Box<dyn FunctionEvaluator>>,
}

impl<'a> LitedownEvaluator<'a> {
    pub fn new() -> Self {
        let writer = HtmlWriter::new();

        let mut environments: HashMap<_, fn() -> Box<dyn EnvironmentEvaluator>> = HashMap::new();
        environments.insert("document", Document::new);
        environments.insert("section", Section::new);
        environments.insert("list", List::new);

        let mut functions: HashMap<_, fn() -> Box<dyn FunctionEvaluator>> = HashMap::new();
        functions.insert("bold", BoldText::new);
        functions.insert("inlinemath", InlineMath::new);

        LitedownEvaluator {
            writer,
            environments,
            functions,
        }
    }

    pub fn eval(mut self, ast: &LitedownAst) -> Result<String, String> {
        let root = match &ast.root {
            Element::Environment(environment) => environment,
            Element::Passage(_) => panic!("Illegal element"),
        };

        match self.get_environment(&root.name) {
            Some(mut environment) => {
                environment.eval(&mut self, &root)?;
                // Ok(self.buffer)
                self.writer.build()
            }
            None => Err(format!("Unknown environment: {}", root.name)),
        }
    }

    pub fn get_environment(&self, key: &str) -> Option<Box<dyn EnvironmentEvaluator>> {
        self.environments.get(key).map(|environment| environment())
    }

    pub fn set_environment(&mut self, key: &'a str, value: fn() -> Box<dyn EnvironmentEvaluator>) {
        if self.environments.contains_key(&key) {
            panic!("Already exists: {}", key);
        }
        self.environments.insert(key, value);
    }

    pub fn eval_environment(&mut self, element: &EnvironmentElement) -> Result<(), String> {
        match self.get_environment(&element.name) {
            Some(mut environment) => {
                environment.eval(self, element)?;
                Ok(())
            }
            None => Err(format!("Unknown environment: {}", element.name)),
        }
    }

    pub fn get_function(&self, key: &str) -> Option<Box<dyn FunctionEvaluator>> {
        self.functions.get(key).map(|function| function())
    }

    pub fn set_function(&mut self, key: &'a str, value: fn() -> Box<dyn FunctionEvaluator>) {
        if self.functions.contains_key(&key) {
            panic!("Already exists: {}", key);
        }
        self.functions.insert(key, value);
    }

    pub fn eval_function(&mut self, content: &PassageContentFunction) -> Result<(), String> {
        match self.get_function(&content.name) {
            Some(function) => {
                function.eval(self, content)?;
                Ok(())
            }
            None => Err(format!("Unknown function: {}", content.name)),
        }
    }
}
