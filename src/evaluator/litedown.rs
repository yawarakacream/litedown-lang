use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, Result};

use crate::{
    tree::{
        ast::LitedownAst,
        element::{EnvironmentElement, PassageContentFunction},
    },
    utility::html::{Html, HtmlElement, HtmlString},
};

use super::{environment::EnvironmentEvaluator, function::FunctionEvaluator};

pub struct LitedownEvaluator {
    source_path: Option<PathBuf>,
    root_environments: HashMap<String, Box<dyn EnvironmentEvaluator>>,
    environments: HashMap<String, Box<dyn EnvironmentEvaluator>>,
    functions: HashMap<String, Box<dyn FunctionEvaluator>>,
}

macro_rules! define_evaluator_function {
    ($name:expr; $setfn:ident, $evalfn:ident; self.$evrmap:ident: $evrtype:ident, $eltype:ty => $rettype:ty) => {
        pub fn $setfn(&mut self, key: &str, value: Box<dyn $evrtype>) {
            let key = key.to_string();
            if self.$evrmap.contains_key(&key) {
                panic!("Already exists: {}", key);
            }
            self.$evrmap.insert(key, value);
        }

        pub fn $evalfn(&mut self, element: &$eltype) -> Result<$rettype> {
            let key = element.name.clone();
            let evaluator = self.$evrmap.remove(&key);
            match evaluator {
                Some(mut evaluator) => {
                    let result = evaluator.eval(self, element);
                    self.$evrmap.insert(key, evaluator);
                    result
                }
                None => bail!("Unknown {}: {}", $name, element.name),
            }
        }
    };
}

impl LitedownEvaluator {
    pub fn new() -> Self {
        LitedownEvaluator {
            source_path: None,
            root_environments: HashMap::new(),
            environments: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    pub fn get_source_path(&self) -> &Option<PathBuf> {
        &self.source_path
    }

    pub fn eval(mut self, source_path: PathBuf, ast: LitedownAst) -> Result<HtmlString> {
        self.source_path = Some(source_path);

        let mut html = Html::new();

        let mut root = HtmlElement::new("div");
        root.set_attr("id", "root");

        for environment in &ast.roots {
            root.append(self.eval_root_environment(environment)?);
        }
        html.append_body(root);

        // less.js
        let mut less_script = HtmlElement::new("script");
        less_script.set_attr("src", "https://cdn.jsdelivr.net/npm/less");
        less_script.set_attr("defer", "true");
        html.append_head(less_script);

        for environment in self.root_environments.values() {
            for head in environment.get_heads()? {
                html.append_head(head);
            }
        }

        for environment in self.environments.values() {
            for head in environment.get_heads()? {
                html.append_head(head);
            }
        }

        Ok(html.to_string())
    }

    define_evaluator_function!("root environment"; set_root_environment, eval_root_environment;
        self.root_environments: EnvironmentEvaluator, EnvironmentElement => HtmlElement);

    define_evaluator_function!("environment"; set_environment, eval_environment;
        self.environments: EnvironmentEvaluator, EnvironmentElement => HtmlElement);

    define_evaluator_function!("function"; set_function, eval_function;
        self.functions: FunctionEvaluator, PassageContentFunction => Option<HtmlElement>);
}
