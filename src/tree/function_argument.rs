use std::fmt;

use anyhow::{bail, Result};

#[derive(Debug)]
pub enum FunctionArgumentValue {
    String { value: String },
    Integer { number: isize, unit: String },
    Float { number: f64, unit: String },
}

impl fmt::Display for FunctionArgumentValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionArgumentValue::String { value } => write!(f, "{}", value),
            FunctionArgumentValue::Integer { number, unit } => write!(f, "{}{}", number, unit),
            FunctionArgumentValue::Float { number, unit } => write!(f, "{}{}", number, unit),
        }
    }
}

#[derive(Debug)]
pub struct FunctionArgument {
    pub name: Option<String>,
    pub value: FunctionArgumentValue,
}

impl FunctionArgument {
    pub fn try_into_string(&self) -> Result<String> {
        Ok(format!("{}", self.value))
    }

    pub fn try_into_integer(&self) -> Result<(isize, &String)> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            return Ok((*number, unit));
        }
        bail!("invalid argument: {} is not Integer", self.value);
    }

    pub fn try_into_bare_integer(&self) -> Result<(isize, &String)> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            if unit.is_empty() {
                return Ok((*number, unit));
            }
        }
        bail!("invalid argument: {} is not bare Integer", self.value);
    }

    pub fn try_into_float(&self) -> Result<(f64, &String)> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            return Ok(((*number as f64), &unit));
        }
        if let FunctionArgumentValue::Float { number, unit } = &self.value {
            return Ok((*number, &unit));
        }
        bail!("invalid argument: {} is not Float", self.value);
    }

    pub fn try_into_bare_float(&self) -> Result<f64> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            if unit.is_empty() {
                return Ok(*number as f64);
            }
        }
        if let FunctionArgumentValue::Float { number, unit } = &self.value {
            if unit.is_empty() {
                return Ok(*number);
            }
        }
        bail!("invalid argument: {} is not bare Float", self.value);
    }
}
