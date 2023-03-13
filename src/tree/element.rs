use std::collections::HashMap;

use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

use super::parameter::CommandParameterValue;

#[derive(Debug)]
pub enum LitedownElement {
    Environment(EnvironmentElement),
    Passage(PassageElement),
}

#[derive(Debug)]
pub struct EnvironmentElement {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
    pub children: Vec<LitedownElement>,
}

#[derive(Debug)]
pub struct PassageElement {
    pub contents: Vec<PassageContent>,
}

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
pub struct EnvironmentHeader {
    pub name: String,
    pub parameters: HashMap<String, CommandParameterValue>,
}

impl ToTreeString for LitedownElement {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        match self {
            LitedownElement::Environment(environment) => {
                environment.write_tree_string(builder, level)
            }
            LitedownElement::Passage(passage) => passage.write_tree_string(builder, level),
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
        for c in &self.contents {
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
