use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, Result};

use crate::{
    litedown_element::{EnvironmentElement, LitedownAst, PassageContentFunction},
    utility::html::{Html, HtmlElement, HtmlString},
};

use super::{
    default::{
        code::{CodeBlock, InlineCode},
        decorators::{BoldText, InlineMath, Link, PageBreak},
        document::Document,
        figure::Figure,
        image::Image,
        list::List,
        minipages::MiniPages,
        section::Section,
    },
    environment::EnvironmentEvaluator,
    function::FunctionEvaluator,
};

static STYLE: &str = include_str!("./default/style.less");

pub struct LitedownEvaluator {
    source: Option<PathBuf>,
    environments: HashMap<String, Box<dyn EnvironmentEvaluator>>,
    functions: HashMap<String, Box<dyn FunctionEvaluator>>,
}

impl LitedownEvaluator {
    pub fn new(source: Option<PathBuf>) -> Self {
        let mut instance = LitedownEvaluator {
            source,
            environments: HashMap::new(),
            functions: HashMap::new(),
        };

        instance.init_default();

        instance
    }

    fn init_default(&mut self) {
        self.set_environment("section", Section::new());
        self.set_environment("list", List::new());
        self.set_environment("code", CodeBlock::new());
        self.set_environment("figure", Figure::new());
        self.set_environment("minipages", MiniPages::new());

        self.set_function("link", Link::new());
        self.set_function("pagebreak", PageBreak::new());
        self.set_function("code", InlineCode::new());
        self.set_function("math", InlineMath::new());
        self.set_function("bold", BoldText::new());
        self.set_function("image", Image::new());
    }

    pub fn get_source(&self) -> &Option<PathBuf> {
        &self.source
    }

    pub fn eval(mut self, ast: &LitedownAst) -> Result<HtmlString> {
        let mut root = Html::new();

        // main style
        let mut less_style = HtmlElement::new("style");
        less_style.set_attr("type", "text/less");
        less_style.append_raw_text(STYLE);
        root.append_head(less_style);

        // less.js
        let mut less_script = HtmlElement::new("script");
        less_script.set_attr("src", "https://cdn.jsdelivr.net/npm/less");
        less_script.set_attr("defer", "true");
        root.append_head(less_script);

        for environment in &ast.roots {
            match environment.name.as_str() {
                "document" => {
                    let mut document = Document::new();
                    root.append_body(document.eval(&mut self, &environment)?);
                    for head in document.get_heads()? {
                        root.append_head(head);
                    }
                }
                _ => bail!("Unknown environment: {}", environment.name),
            }
        }

        for environment in self.environments.values() {
            for head in environment.get_heads()? {
                root.append_head(head);
            }
        }

        Ok(root.to_string())
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
