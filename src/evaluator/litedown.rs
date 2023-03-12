use std::collections::HashMap;

use anyhow::{bail, Result};

use crate::{
    litedown_element::{EnvironmentElement, LitedownAst, PassageContentFunction},
    utility::html::{Html, HtmlElement, HtmlString},
};

use super::{environment::EnvironmentEvaluator, function::FunctionEvaluator};

pub struct LitedownEvaluator {
    ast: Option<LitedownAst>,
    environments: HashMap<String, Box<dyn EnvironmentEvaluator>>,
    functions: HashMap<String, Box<dyn FunctionEvaluator>>,
}

impl LitedownEvaluator {
    pub fn new() -> Self {
        LitedownEvaluator {
            ast: None,
            environments: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_ast(&self) -> &LitedownAst {
        match &self.ast {
            Some(ast) => ast,
            None => panic!("LitedownEvaluator is not evaluating anything"),
        }
    }

    pub fn eval(mut self, ast: &LitedownAst) -> Result<HtmlString> {
        let mut html = Html::new();

        let mut root = HtmlElement::new("div");
        root.set_attr("id", "root");

        for environment in &ast.roots {
            root.append(self.eval_environment(environment)?);
        }
        html.append_body(root);

        // less.js
        let mut less_script = HtmlElement::new("script");
        less_script.set_attr("src", "https://cdn.jsdelivr.net/npm/less");
        less_script.set_attr("defer", "true");
        html.append_head(less_script);

        for environment in self.environments.values() {
            for head in environment.get_heads()? {
                html.append_head(head);
            }
        }

        Ok(html.to_string())
    }

    pub fn set_environment(&mut self, key: &str, value: Box<dyn EnvironmentEvaluator>) {
        let key = key.to_string();
        if self.environments.contains_key(&key) {
            panic!("Already exists: {}", key);
        }
        self.environments.insert(key, value);
    }

    pub fn eval_environment(&mut self, element: &EnvironmentElement) -> Result<HtmlElement> {
        let key = element.name.clone();
        let environment = self.environments.remove(&key);
        match environment {
            Some(mut environment) => {
                let result = environment.eval(self, element);
                self.environments.insert(key, environment);
                result
            }
            None => bail!("Unknown environment: {}", element.name),
        }
    }

    pub fn set_function(&mut self, key: &str, value: Box<dyn FunctionEvaluator>) {
        let key = key.to_string();
        if self.functions.contains_key(&key) {
            panic!("Already exists: {}", key);
        }
        self.functions.insert(key, value);
    }

    pub fn eval_function(
        &mut self,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        let key = content.name.clone();
        let function = self.functions.remove(&key);
        match function {
            Some(mut function) => {
                let result = function.eval(self, content);
                self.functions.insert(key, function);
                result
            }
            None => bail!("Unknown function: {}", content.name),
        }
    }
}
