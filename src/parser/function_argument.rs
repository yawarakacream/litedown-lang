use anyhow::{bail, Context, Result};

use crate::{
    parser::litedown::parse_name,
    tree::function_argument::{FunctionArgument, FunctionArgumentValue},
    utility::indented_string::IndentedStringIterator,
};

pub(super) fn parse_function_argument(
    iter: &mut IndentedStringIterator,
) -> Result<FunctionArgument> {
    iter.parse(|iter| match parse_value_with_key(iter) {
        Ok(tmp) => Ok(tmp),
        Err(_) => Ok(FunctionArgument {
            name: None,
            value: parse_value(iter)?,
        }),
    })
}

fn parse_value(iter: &mut IndentedStringIterator) -> Result<FunctionArgumentValue> {
    fn parse_number(iter: &mut IndentedStringIterator) -> Result<FunctionArgumentValue> {
        iter.parse(|iter| {
            let mut number = String::new();
            let mut has_point = false;
            let mut unit = String::new();

            if let Ok(_) = iter.next_char_as('-') {
                number.push('-');
            }

            while let Some(c) = iter.next_char() {
                if c == '.' {
                    if has_point {
                        bail!("'.' is allowed only once");
                    }
                    number.push(c);
                    has_point = true;
                } else if c.is_ascii_digit() {
                    number.push(c);
                } else {
                    iter.back_char().unwrap();
                    break;
                }
            }

            while let Some(c) = iter.next_char() {
                if c.is_ascii_alphabetic() || c == '%' {
                    unit.push(c);
                } else {
                    iter.back_char().unwrap();
                    break;
                }
            }

            if number.is_empty() {
                bail!("failed to parse");
            }

            if has_point {
                Ok(FunctionArgumentValue::Float {
                    number: number.parse().unwrap(),
                    unit,
                })
            } else {
                Ok(FunctionArgumentValue::Integer {
                    number: number.parse().unwrap(),
                    unit,
                })
            }
        })
    }

    fn parse_string(iter: &mut IndentedStringIterator) -> Result<FunctionArgumentValue> {
        iter.parse(|iter| {
            let delimiter = iter.next_char().context("Empty string")?;

            if !(delimiter == '"' || delimiter == '\'') {
                bail!("expected delimiter, found: {}", delimiter);
            }

            let mut value = String::new();
            let mut escaped = false;
            while let Some(c) = iter.next_char() {
                if escaped {
                    value.push(c);
                    escaped = false;
                } else {
                    if c == delimiter {
                        return Ok(FunctionArgumentValue::String { value });
                    }
                    if c == '\\' {
                        escaped = true
                    } else {
                        value.push(c)
                    }
                }
            }
            bail!("EOL while scanning string literal")
        })
    }

    parse_number(iter).or_else(|_| parse_string(iter))
}

fn parse_value_with_key(iter: &mut IndentedStringIterator) -> Result<FunctionArgument> {
    iter.parse(|iter| {
        let name = parse_name(iter)?;
        iter.pass_whitespaces();
        iter.next_char_as('=')?;
        iter.pass_whitespaces();
        let value = parse_value(iter)?;

        let parameter = FunctionArgument {
            name: Some(name),
            value,
        };
        Ok(parameter)
    })
}
