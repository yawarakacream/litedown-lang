use std::{collections::HashMap, error::Error};

use crate::utility::tree_string_builder::TreeStringBuilder;

#[derive(Debug)]
pub enum Element {
    Environment {
        name: String,
        parameters: HashMap<String, CommandParameterValue>,
        children: Vec<Element>,
    },
    Passage(Vec<Line>),
}

pub type Line = Vec<LineContent>;

#[derive(Debug)]
pub enum LineContent {
    Text(String),
    Function {
        name: String,
        parameters: HashMap<String, CommandParameterValue>,
        body: Option<String>,
    },
}

#[derive(PartialEq, Eq, Debug)]
pub enum NumberUnit {
    None,
    Px,
    Em,
}

#[derive(Debug)]
pub enum CommandParameterValue {
    String(String),
    Number(NumberUnit, f64),
}

#[derive(Debug)]
pub struct EnvironmentHeader {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
}

impl Element {
    pub fn stringify_as_tree(&self) -> Result<String, Box<dyn Error>> {
        let mut builder = TreeStringBuilder::new();
        self.stringify_as_tree_internal(&mut builder, 0)?;
        Ok(builder.build())
    }

    fn stringify_as_tree_internal(
        &self,
        builder: &mut TreeStringBuilder,
        level: usize,
    ) -> Result<(), Box<dyn Error>> {
        match &self {
            Element::Environment {
                name,
                parameters,
                children,
            } => {
                builder.add_node(
                    level,
                    format!(
                        "Environment(name = {:?}, parameters = {:?})",
                        name, parameters
                    ),
                );
                for c in children {
                    c.stringify_as_tree_internal(builder, level + 1)?;
                }
            }
            Element::Passage(lines) => {
                builder.add_node(level, "Passage");
                for line in lines {
                    for c in line {
                        match c {
                            LineContent::Text(content) => {
                                builder.add_node(level + 1, format!("Text({:?})", content))
                            }
                            LineContent::Function {
                                name,
                                parameters,
                                body,
                            } => {
                                builder.add_node(
                                    level + 1,
                                    format!(
                                        "Function(name = {:?}, parameters = {:?})",
                                        name, parameters
                                    ),
                                );
                                if let Some(body) = body {
                                    builder.add_node(level + 2, format!("Body({:?})", body));
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
