use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::{environment::EnvironmentEvaluator, litedown::LitedownEvaluator},
    tree::{
        element::{EnvironmentElement, LitedownElement},
        parameter::stringify_number_parameter,
    },
    utility::html::HtmlElement,
};

enum Alignment {
    Center,
    Space,
    Left,
}
impl Alignment {
    fn to_css_class(&self) -> &str {
        match self {
            Alignment::Center => "center",
            Alignment::Space => "space",
            Alignment::Left => "left",
        }
    }
}

pub struct MiniPages;

impl MiniPages {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(MiniPages {})
    }
}

impl EnvironmentEvaluator for MiniPages {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        let mut minipages = HtmlElement::new("div");

        let columns = match element.parameters.get("columns") {
            Some(parameter) => {
                let number = parameter.try_into_bare_number()?;
                let number = number as isize;
                if 0 < number {
                    bail!("Invalid parameter 'columns': must be positive");
                }
                Some(number as usize)
            }
            None => None,
        };

        let alignment = match &element.parameters.get("alignment") {
            Some(parameter) => match parameter.try_into_str()? {
                "center" => Alignment::Center,
                "space" => Alignment::Space,
                _ => bail!("Invalid parameter 'alignment'"),
            },
            None => match columns {
                Some(_) => Alignment::Left,
                None => Alignment::Center,
            },
        };

        minipages.set_attr("class", &format!("minipages {}", alignment.to_css_class()));

        let padding = match element.parameters.get("padding") {
            Some(parameter) => {
                let (unit, number) = parameter.try_into_number()?;
                stringify_number_parameter(unit, number)
            }
            None => "0px".to_string(),
        };

        let gap = match element.parameters.get("gap") {
            Some(parameter) => {
                if columns.is_none() {
                    bail!("Cannot use 'gap' without 'columns'");
                }
                let (unit, number) = parameter.try_into_number()?;
                stringify_number_parameter(unit, number)
            }
            None => "0px".to_string(),
        };

        minipages.set_attr("style", &format!("padding: 0 {padding}; gap: {gap};"));

        for child in &element.children {
            match child {
                LitedownElement::Environment(child_environment) => {
                    match child_environment.name.as_str() {
                        "page" => {
                            let mut page = HtmlElement::new("div");
                            page.set_attr("class", "page");

                            let width = match child_environment.parameters.get("width") {
                                Some(parameter) => {
                                    match columns {
                                        Some(_) => bail!("Cannot specify 'width' on @page@ with 'columns' on @minipages@"),
                                        None => {
                                            let (unit, number) = parameter.try_into_number()?;
                                            stringify_number_parameter(unit, number)
                                        }
                                    }
                                }
                                None => {
                                    match columns {
                                        Some(columns) => format!("calc((100% - {} * {}) / {})", gap, columns - 1, columns),
                                        None => bail!("Either 'columns' on @minipages@ or 'width' on @page@ is needed")
                                    }
                                }
                            };
                            page.set_attr("style", &format!("width: {width};"));

                            eval_with_litedown!(child_environment to page with lde);
                            minipages.append(page);
                        }
                        _ => bail!("Unknown environment: {}", child_environment.name),
                    }
                }
                _ => bail!("Only environment @page@ is allowed"),
            }
        }

        Ok(minipages)
    }
}
