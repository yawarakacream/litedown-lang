use std::{collections::HashMap, error::Error, fmt};

use crate::utility::tree_string_builder::TreeStringBuilder;

#[derive(Debug)]
pub struct LitedownAst {
    pub root: Element,
}

#[derive(Debug)]
pub enum Element {
    Environment(EnvironmentElement),
    Passage(PassageElement),
}

#[derive(Debug)]
pub struct EnvironmentElement {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
    pub children: Vec<Element>,
}

#[derive(Debug)]
pub struct PassageElement(pub Vec<PassageContent>);

#[derive(Debug)]
pub enum PassageContent {
    Text(PassageContentText),
    Function(PassageContentFunction),
}

#[derive(Debug)]
pub struct PassageContentText(pub String);

#[derive(Debug)]
pub struct PassageContentFunction {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
    pub body: Option<String>,
}

#[derive(Debug)]
pub enum CommandParameterValue {
    String(String),
    Number(Option<String>, f64),
}

impl fmt::Display for CommandParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandParameterValue::String(s) => write!(f, "{}", s),
            CommandParameterValue::Number(u, n) => match u {
                Some(u) => write!(f, "{}{}", n, u),
                None => write!(f, "{}", n),
            },
        }
    }
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
            Element::Environment(EnvironmentElement {
                name,
                parameters,
                children,
            }) => {
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
            Element::Passage(PassageElement(contents)) => {
                builder.add_node(level, "Passage");
                for c in contents {
                    match c {
                        PassageContent::Text(PassageContentText(text)) => {
                            builder.add_node(level + 1, format!("Text({:?})", text))
                        }
                        PassageContent::Function(PassageContentFunction {
                            name,
                            parameters,
                            body,
                        }) => {
                            builder.add_node(
                                level + 1,
                                format!(
                                    "Function(name = {:?}, parameters = {:?})",
                                    name, parameters
                                ),
                            );
                            if let Some(body) = body {
                                builder.add_node(level + 3, format!("Body({:?})", body));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
