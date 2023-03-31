use std::{collections::HashMap, fmt};

use anyhow::{bail, Result};

use serde::{ser::SerializeMap, Serialize as SerdeSerialize, Serializer as SerdeSerializer};

#[derive(Clone, Debug, SerdeSerialize)]
#[serde(tag = "__struct")]
pub enum CommandParameterValue {
    String { value: String },
    Number { number: f64, unit: Option<String> },
}

impl fmt::Display for CommandParameterValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CommandParameterValue::String { value } => write!(f, "{}", value),
            CommandParameterValue::Number { number, unit } => match unit {
                Some(unit) => write!(f, "{}{}", number, unit),
                None => write!(f, "{}", number),
            },
        }
    }
}

pub fn stringify_number_parameter(unit: &Option<String>, number: f64) -> String {
    match unit {
        Some(unit) => format!("{number}{unit}"),
        None => number.to_string(),
    }
}

#[derive(Clone, Debug)]
pub struct CommandParameter {
    pub key: String,
    pub value: CommandParameterValue,
}

impl CommandParameter {
    pub fn try_into_str(&self) -> Result<&str> {
        match &self.value {
            CommandParameterValue::String { value } => Ok(&value.as_str()),
            _ => bail!("Invalid parameter '{}': {}", self.key, self.value),
        }
    }

    pub fn try_into_string(&self) -> Result<&String> {
        match &self.value {
            CommandParameterValue::String { value } => Ok(&value),
            _ => bail!("Invalid parameter '{}': {}", self.key, self.value),
        }
    }

    pub fn try_into_number(&self) -> Result<(&Option<String>, f64)> {
        match &self.value {
            CommandParameterValue::Number { unit, number } => Ok((unit, *number)),
            _ => bail!("Invalid parameter '{}': {}", self.key, self.value),
        }
    }

    pub fn try_into_bare_number(&self) -> Result<f64> {
        let (unit, number) = self.try_into_number()?;
        match unit {
            Some(_) => bail!(
                "Invalid parameter '{}': {} is not bare",
                self.key,
                self.value
            ),
            None => Ok(number),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CommandParameterContainer {
    parameters: HashMap<String, CommandParameter>,
}

impl CommandParameterContainer {
    pub fn new() -> CommandParameterContainer {
        CommandParameterContainer {
            parameters: HashMap::new(),
        }
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.parameters.contains_key(key)
    }

    pub fn insert(&mut self, key: &str, value: CommandParameterValue) {
        self.parameters.insert(
            key.to_string(),
            CommandParameter {
                key: key.to_string(),
                value,
            },
        );
    }

    pub fn get(&self, key: &str) -> Option<&CommandParameter> {
        self.parameters.get(key)
    }

    pub fn try_get(&self, key: &str) -> Result<&CommandParameter> {
        match self.get(key) {
            Some(value) => Ok(value),
            None => bail!("Parameter '{}' not found", key),
        }
    }
}

impl SerdeSerialize for CommandParameterContainer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: SerdeSerializer,
    {
        let mut state = serializer.serialize_map(Some(self.parameters.len()))?;
        for CommandParameter { key, value } in self.parameters.values() {
            state.serialize_entry(key, value)?;
        }
        state.end()
    }
}

mod tests {
    use super::{CommandParameter, CommandParameterContainer, CommandParameterValue};

    impl PartialEq for CommandParameterValue {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::String { value: l_value }, Self::String { value: r_value }) => {
                    l_value == r_value
                }
                (
                    Self::Number {
                        number: l_number,
                        unit: l_unit,
                    },
                    Self::Number {
                        number: r_number,
                        unit: r_unit,
                    },
                ) => l_number == r_number && l_unit == r_unit,
                _ => false,
            }
        }
    }

    impl PartialEq for CommandParameter {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key && self.value == other.value
        }
    }

    impl PartialEq for CommandParameterContainer {
        fn eq(&self, other: &Self) -> bool {
            self.parameters == other.parameters
        }
    }
}
