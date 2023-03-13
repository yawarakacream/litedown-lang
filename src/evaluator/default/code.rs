use std::{fs, path::PathBuf};

use anyhow::{bail, Result};

use crate::{
    evaluator::{
        environment::EnvironmentEvaluator, function::FunctionEvaluator, litedown::LitedownEvaluator,
    },
    tree::element::{
        EnvironmentElement, LitedownElement, PassageContent, PassageContentFunction, PassageElement,
    },
    utility::html::HtmlElement,
};

pub struct CodeBlock;

impl CodeBlock {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(CodeBlock {})
    }
}

impl EnvironmentEvaluator for CodeBlock {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut lang = match &element.parameters.get("lang") {
            Some(lang) => Some(lang.try_into_string()?.clone()),
            None => None,
        };

        let mut inner = None;

        for child in &element.children {
            match child {
                LitedownElement::Environment(child_environment) => {
                    bail!("Unknown environment: {}", child_environment.name)
                }
                LitedownElement::Passage(PassageElement { contents }) => {
                    for content in contents {
                        match content {
                            PassageContent::Text(content) => {
                                if inner.is_some() {
                                    bail!("Only 1 passage is allowed");
                                }
                                inner = Some(content.0.clone());
                            }
                            PassageContent::Function(content) => match content.name.as_str() {
                                "src" => match &content.body {
                                    Some(body) => {
                                        if inner.is_some() {
                                            bail!("Only 1 passage is allowed");
                                        }

                                        let path = if body.starts_with(".") {
                                            match &lde.get_source_path() {
                                                Some(ld_path) => ld_path.with_file_name(body),
                                                None => bail!("Cannot use relative path"),
                                            }
                                        } else {
                                            PathBuf::from(body)
                                        };

                                        if !path.exists() {
                                            bail!("File not found: {:?}", path);
                                        }

                                        let extension = path
                                            .extension()
                                            .and_then(|str| str.to_str())
                                            .unwrap_or("");
                                        lang = match extension {
                                            "r" => Some("R".to_string()),
                                            _ => bail!("Unknown extension: {}", extension),
                                        };
                                        match fs::read_to_string(&path) {
                                            Ok(code) => inner = Some(code),
                                            Err(e) => bail!("Could not read code: {}", e),
                                        }
                                    }
                                    None => bail!("Illegal src"),
                                },
                                _ => bail!("Illegal function: {}", content.name),
                            },
                        }
                    }
                }
            }
        }

        let lang = match lang {
            Some(lang) => lang,
            None => bail!("Cannot determine lang"),
        };

        let inner = match inner {
            Some(inner) => inner,
            None => bail!("Empty code block is not allowed"),
        };

        let mut pre = HtmlElement::new("pre");
        let mut code = HtmlElement::new("code");
        code.set_attr("class", format!("language-{}", lang).as_str());
        code.append_raw_text(&inner);
        pre.append(code);
        Ok(pre)
    }

    fn get_heads(&self) -> Result<Vec<HtmlElement>> {
        let mut highlight_style = HtmlElement::new_void("link");
        highlight_style.set_attr("rel", "stylesheet");
        highlight_style.set_attr(
            "href",
            "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/default.min.css",
        );

        let mut highlight_script = HtmlElement::new("script");
        highlight_script.set_attr(
            "src",
            "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js",
        );
        highlight_script.set_attr("onload", "hljs.highlightAll()");

        Ok(vec![highlight_style, highlight_script])
    }
}

pub struct InlineCode;

impl InlineCode {
    pub fn new() -> Box<dyn FunctionEvaluator> {
        Box::new(InlineCode {})
    }
}

impl FunctionEvaluator for InlineCode {
    fn eval(
        &mut self,
        _: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>> {
        let mut el = HtmlElement::new("code");
        match &content.body {
            Some(body) => el.append_text(body),
            None => bail!("body is empty"),
        };
        Ok(Some(el))
    }
}
