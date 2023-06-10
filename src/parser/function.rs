use anyhow::{bail, Context, Result};

use crate::{
    parser::{function_argument::parse_function_argument, litedown::parse_name},
    tree::{
        function::{FunctionArgumentContainer, FunctionBody, FunctionBodyForm, LitedownFunction},
        function_argument::{FunctionArgument, FunctionArgumentValue},
    },
    utility::indented_string::IndentedStringIterator,
};

use super::function_body::parse_function_body;

pub(crate) fn parse_function(iter: &mut IndentedStringIterator) -> Result<LitedownFunction> {
    iter.parse(|iter| {
        let started_at_first_character = iter.current_indices().char_index == 0;

        iter.next_char_as('@')
            .context("The start character '@' not found")?;

        let name = parse_name(iter).context("Failed to parse function name")?;
        let arguments = parse_argument_container(iter)?;
        let body = parse_function_body(iter, started_at_first_character)?.unwrap_or_else(|| {
            FunctionBody {
                form: FunctionBodyForm::Inline,
                value: Vec::new(),
            }
        });

        let function = LitedownFunction {
            name,
            arguments,
            body,
        };
        Ok(function)
    })
}

pub(super) fn parse_argument_container(
    iter: &mut IndentedStringIterator,
) -> Result<FunctionArgumentContainer> {
    iter.parse(|iter| {
        if iter.next_char_as('[').is_err() {
            return Ok(FunctionArgumentContainer::new(Vec::new())?);
        }

        let mut arguments = Vec::new();
        let mut trailing_comma_used = false;
        loop {
            iter.pass_whitespaces();
            if !iter.has_next_char() {
                iter.next_line();
                iter.pass_blank_lines();
                iter.pass_whitespaces();
            }

            match parse_function_argument(iter) {
                Ok(tmp) => {
                    iter.pass_whitespaces();
                    arguments.push(tmp);
                }
                Err(_) => {
                    // 不正な形式の場合，そのまま文字列のパラメータと解釈する
                    let mut value = String::new();
                    while let Some(c) = iter.next_char() {
                        if c == ']' || c == ',' {
                            iter.back_char().unwrap();
                            break;
                        }
                        value.push(c);
                    }

                    if value.is_empty() {
                        if trailing_comma_used {
                            bail!("Cannot write trailing comma multi times");
                        }
                        trailing_comma_used = true;
                    } else {
                        arguments.push(FunctionArgument {
                            name: None,
                            value: FunctionArgumentValue::String { value },
                        })
                    }
                }
            }

            if let Ok(_) = iter.next_char_as(',') {
                continue;
            }

            iter.pass_whitespaces();
            if !iter.has_next_char() {
                iter.next_line();
                iter.pass_blank_lines();
            }

            if let Ok(_) = iter.next_char_as(']') {
                return Ok(FunctionArgumentContainer::new(arguments)?);
            }

            break;
        }

        bail!("The finish character ']' not found");
    })
}
