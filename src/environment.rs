use std::collections::HashMap;

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
