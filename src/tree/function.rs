use anyhow::{bail, Result};
use serde::Serialize;

use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

use super::function_argument::FunctionArgument;

#[derive(Clone, Debug, Serialize)]
pub struct LitedownFunction {
    pub name: String,
    pub arguments: FunctionArgumentContainer,
    pub body: FunctionBody,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionArgumentContainer {
    arguments: Vec<FunctionArgument>,
}

impl FunctionArgumentContainer {
    pub fn new(arguments: Vec<FunctionArgument>) -> Result<Self> {
        let mut use_named_argument = false;
        for param in &arguments {
            if param.name.is_some() {
                use_named_argument = true;
            } else {
                if use_named_argument {
                    bail!("Cannot use unnamed argument after named argument");
                }
            }
        }
        Ok(FunctionArgumentContainer { arguments })
    }

    pub fn is_empty(&self) -> bool {
        self.arguments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.arguments.len()
    }

    pub fn get_by_index(&self, index: usize) -> Option<&FunctionArgument> {
        if let Some(argument) = self.arguments.get(index) {
            if argument.name.is_none() {
                return Some(argument);
            }
        }
        None
    }

    pub fn get_by_name(&self, name: &str) -> Option<&FunctionArgument> {
        for argument in &self.arguments {
            if let Some(key0) = &argument.name {
                if key0 == name {
                    return Some(argument);
                }
            }
        }
        None
    }

    pub fn try_get_by_name(&self, name: &str) -> Result<&FunctionArgument> {
        match self.get_by_name(name) {
            Some(value) => Ok(value),
            None => bail!("Argument '{}' not found", name),
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionBodyForm {
    Inline,
    Block,
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionBody {
    pub form: FunctionBodyForm,
    pub value: Vec<LitedownPassage>,
}

impl FunctionBody {
    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }

    pub fn try_get_as_string(&self) -> Result<String> {
        let mut result = String::new();
        for passage in &self.value {
            for passage_element in &passage.elements {
                match passage_element {
                    PassageElement::String(string) => result.push_str(string),
                    PassageElement::Function(_) => bail!("cannot write function"),
                }
            }
        }
        Ok(result)
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct LitedownPassage {
    pub elements: Vec<PassageElement>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", content = "value")]
#[serde(rename_all = "snake_case")]
pub enum PassageElement {
    String(String),
    Function(LitedownFunction),
}

impl ToTreeString for LitedownFunction {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, format!("Function({:?})", self.name));

        if !self.arguments.is_empty() {
            builder.add_node(level + 1, "Parameter");
            for argument in &self.arguments.arguments {
                match &argument.name {
                    Some(key) => {
                        builder.add_node(level + 2, format!("{:?} => {:?}", key, argument.value));
                    }
                    None => {
                        builder.add_node(level + 2, format!("{:?}", argument.value));
                    }
                }
            }
        }

        builder.add_node(level + 1, format!("Body({:?})", self.body.form));
        for passage in &self.body.value {
            builder.add_node(level + 2, "Passage");
            for passage_element in &passage.elements {
                match passage_element {
                    PassageElement::String(string) => {
                        builder.add_node(level + 3, format!("{:?}", string));
                    }
                    PassageElement::Function(function) => {
                        function.write_tree_string(builder, level + 3);
                    }
                }
            }
        }
    }
}
