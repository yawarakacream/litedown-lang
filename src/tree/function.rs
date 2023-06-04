use anyhow::{bail, Result};

use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

use super::function_argument::FunctionArgument;

#[derive(Debug)]
pub struct LitedownFunction {
    pub name: String,
    pub parameters: FunctionArgumentContainer,
    pub body: FunctionBody,
}

#[derive(Debug)]
pub struct FunctionArgumentContainer {
    arguments: Vec<FunctionArgument>,
}

impl FunctionArgumentContainer {
    pub fn new(arguments: Vec<FunctionArgument>) -> Result<Self> {
        let mut use_named_parameter = false;
        for param in &arguments {
            if param.name.is_some() {
                use_named_parameter = true;
            } else {
                if use_named_parameter {
                    bail!("Cannot use unnamed parameter after named parameter");
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
        if let Some(parameter) = self.arguments.get(index) {
            if parameter.name.is_none() {
                return Some(parameter);
            }
        }
        None
    }

    pub fn get_by_name(&self, name: &str) -> Option<&FunctionArgument> {
        for parameter in &self.arguments {
            if let Some(key0) = &parameter.name {
                if key0 == name {
                    return Some(parameter);
                }
            }
        }
        None
    }

    pub fn try_get_by_name(&self, name: &str) -> Result<&FunctionArgument> {
        match self.get_by_name(name) {
            Some(value) => Ok(value),
            None => bail!("Parameter '{}' not found", name),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FunctionBodyForm {
    Inline,
    Block,
}

#[derive(Debug)]
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

#[derive(Debug)]
pub struct LitedownPassage {
    pub elements: Vec<PassageElement>,
}

#[derive(Debug)]
// #[serde(tag = "struct")]
pub enum PassageElement {
    String(String),
    Function(LitedownFunction),
}

impl ToTreeString for LitedownFunction {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, format!("Function({:?})", self.name));

        if !self.parameters.is_empty() {
            builder.add_node(level + 1, "Parameter");
            for parameter in &self.parameters.arguments {
                match &parameter.name {
                    Some(key) => {
                        builder.add_node(level + 2, format!("{:?} => {:?}", key, parameter.value));
                    }
                    None => {
                        builder.add_node(level + 2, format!("{:?}", parameter.value));
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