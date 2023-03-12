use std::{collections::HashMap, fmt, path::PathBuf};

use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

#[derive(Debug)]
pub struct LitedownAst {
    pub source_path: Option<PathBuf>,
    pub roots: Vec<EnvironmentElement>,
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

pub fn stringify_number_parameter(unit: &Option<String>, number: &f64) -> String {
    match unit {
        Some(unit) => format!("{number}{unit}"),
        None => number.to_string(),
    }
}

#[derive(Debug)]
pub struct EnvironmentHeader {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
}

impl ToTreeString for LitedownAst {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, "LitedownAst");
        for root in &self.roots {
            root.write_tree_string(builder, level + 1);
        }
    }
}

impl ToTreeString for Element {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        match self {
            Element::Environment(environment) => environment.write_tree_string(builder, level),
            Element::Passage(passage) => passage.write_tree_string(builder, level),
        }
    }
}

impl ToTreeString for EnvironmentElement {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(
            level,
            format!(
                "Environment(name = {:?}, parameters = {:?})",
                self.name, self.parameters
            ),
        );
        for c in &self.children {
            c.write_tree_string(builder, level + 1);
        }
    }
}

impl ToTreeString for PassageElement {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, "Passage");
        for c in &self.0 {
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
                        format!("Function(name = {:?}, parameters = {:?})", name, parameters),
                    );
                    if let Some(body) = body {
                        builder.add_node(level + 3, format!("Body({:?})", body));
                    }
                }
            }
        }
    }
}
