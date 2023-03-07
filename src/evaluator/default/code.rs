use std::{fs, path::PathBuf};

use anyhow::{bail, Result};

use crate::{
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    litedown_element::{
        CommandParameterValue, Element, EnvironmentElement, PassageContent, PassageElement,
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
            Some(CommandParameterValue::String(lang)) => Some(lang.clone()),
            None => None,
            _ => bail!("Illegal lang"),
        };

        let mut inner = None;

        for child in &element.children {
            match child {
                Element::Environment(child_environment) => {
                    bail!("Unknown environment: {}", child_environment.name)
                }
                Element::Passage(PassageElement(contents)) => {
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
                                            match lde.get_source() {
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
}
