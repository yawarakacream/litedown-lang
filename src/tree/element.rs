use super::parameter::CommandParameterContainer;

use serde::Serialize as SerdeSerialize;

#[derive(Clone, Debug, SerdeSerialize)]
#[serde(tag = "struct")]
pub enum LitedownElement {
    Environment(EnvironmentElement),
    Passage(PassageElement),
}

#[derive(Clone, Debug, SerdeSerialize)]
pub struct EnvironmentElement {
    pub name: String,
    pub parameters: CommandParameterContainer,
    pub children: Vec<LitedownElement>,
}

#[derive(Clone, Debug, SerdeSerialize)]
pub struct PassageElement {
    pub contents: Vec<PassageContent>,
}

#[derive(Clone, Debug, SerdeSerialize)]
#[serde(tag = "struct")]
pub enum PassageContent {
    Text(PassageContentText),
    Function(PassageContentFunction),
}

#[derive(Clone, Debug, SerdeSerialize)]
pub struct PassageContentText {
    pub value: String,
}

#[derive(Clone, Debug, SerdeSerialize)]
pub struct PassageContentFunction {
    pub name: String,
    pub parameters: CommandParameterContainer,
    pub body: Option<String>,
}

#[derive(Debug)]
pub struct EnvironmentHeader {
    pub name: String,
    pub parameters: CommandParameterContainer,
}

mod tree_string {
    use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

    use super::{
        EnvironmentElement, LitedownElement, PassageContent, PassageContentFunction,
        PassageContentText, PassageElement,
    };

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
                    PassageContent::Text(PassageContentText { value: text }) => {
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
                            builder.add_node(level + 2, format!("Body({:?})", body));
                        }
                    }
                }
            }
        }
    }
}
