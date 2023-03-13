use std::fmt;

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
