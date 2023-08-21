use std::fmt;

use anyhow::{bail, Result};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum FunctionArgumentValue {
    Integer { number: isize, unit: String },
    Float { number: f64, unit: String },
    Boolean { value: bool },
    String { value: String },
    Array { value: Vec<FunctionArgumentValue> },
}

impl fmt::Display for FunctionArgumentValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionArgumentValue::Integer { number, unit } => {
                write!(formatter, "{}{}", number, unit)
            }
            FunctionArgumentValue::Float { number, unit } => {
                write!(formatter, "{}{}", number, unit)
            }
            FunctionArgumentValue::Boolean { value } => {
                write!(formatter, "{}", value)
            }
            FunctionArgumentValue::String { value } => {
                write!(formatter, "{}", value)
            }
            FunctionArgumentValue::Array { value } => {
                write!(formatter, "[")?;
                let mut f = true;
                for v in value {
                    if f {
                        write!(formatter, ", ")?;
                        f = false;
                    }
                    write!(formatter, "{}", v)?;
                }
                write!(formatter, "]")
            }
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct FunctionArgument {
    pub name: Option<String>,
    pub value: FunctionArgumentValue,
}

impl FunctionArgument {
    pub fn try_into_integer(&self) -> Result<(isize, &String)> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            return Ok((*number, unit));
        }
        bail!("invalid argument: {} is not Integer", self.value);
    }

    pub fn try_into_bare_integer(&self) -> Result<isize> {
        if let FunctionArgumentValue::Integer { number, unit } = &self.value {
            if unit.is_empty() {
                return Ok(*number);
            }
        }
        bail!("invalid argument: {} is not bare Integer", self.value);
    }

    pub fn try_into_bare_unsigned_integer(&self) -> Result<usize> {
        let ret = self.try_into_bare_integer()?;
        if let Ok(ret) = usize::try_from(ret) {
            return Ok(ret);
        }
        bail!("invalid argument: {} is not unsigned Integer", self.value);
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

    pub fn try_into_boolean(&self) -> Result<bool> {
        if let FunctionArgumentValue::Boolean { value } = &self.value {
            return Ok(*value);
        }
        bail!("invalid argument: {} is not bare Boolean", self.value);
    }

    pub fn try_into_string(&self) -> Result<String> {
        Ok(format!("{}", self.value))
    }

    pub fn try_into_array(&self) -> Result<&Vec<FunctionArgumentValue>> {
        if let FunctionArgumentValue::Array { value } = &self.value {
            return Ok(value);
        }
        bail!("invalid argument: {} is not Array", self.value);
    }
}
