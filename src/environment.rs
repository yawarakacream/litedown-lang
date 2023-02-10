use std::collections::HashMap;

#[derive(PartialEq, Eq, Debug)]
pub enum NumberUnit {
    None,
    Px,
    Em,
}

pub enum Element {
    Environment {
        name: String,
        parameters: HashMap<String, EnvironmentParameterValue>,
        children: Box<Vec<Element>>,
    },
    Text(String),
}

#[derive(Debug)]
pub enum EnvironmentParameterValue {
    String(String),
    Number(NumberUnit, f64),
}

#[derive(Debug)]
pub struct EnvironmentHeader {
    pub name: String,
    pub parameters: HashMap<String, EnvironmentParameterValue>,
}
