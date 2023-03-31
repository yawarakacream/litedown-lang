use nom::{
    branch::alt,
    bytes::complete::{tag, take_until1},
    character::complete::{alpha1, anychar, char, space0},
    error::VerboseError,
};

use crate::{
    tree::parameter::{CommandParameter, CommandParameterContainer, CommandParameterValue},
    utility::nom::{namestr, parse_f64, ws, IResultV},
    verror,
};

use super::environment_header::pass_indent;

impl CommandParameterValue {
    fn parse_number(str: &str) -> IResultV<&str, CommandParameterValue> {
        let (str, value) = parse_f64(str)?;
        match Self::parse_number_unit(str) {
            Ok((str, unit)) => Ok((
                str,
                CommandParameterValue::Number {
                    number: value,
                    unit: Some(unit),
                },
            )),
            Err(_) => Ok((
                str,
                CommandParameterValue::Number {
                    number: value,
                    unit: None,
                },
            )),
        }
    }

    fn parse_number_unit(str: &str) -> IResultV<&str, String> {
        let (str, value) = alt((alpha1, tag("%")))(str)?;
        Ok((str, value.to_string()))
    }

    fn parse_string(str: &str) -> IResultV<&str, CommandParameterValue> {
        let (str, delimiter) = alt((char('"'), char('\''), char('`')))(str)?;

        let mut value = String::new();
        let mut escaped = false;
        let mut str = str;
        loop {
            if escaped {
                let tmp = alt((char(delimiter), char('\\')))(str)?;
                str = tmp.0;
                value.push(tmp.1);
                escaped = false;
            } else {
                let tmp = anychar(str)?;
                str = tmp.0;
                let c = tmp.1;

                if c == delimiter {
                    break;
                }
                if c == '\\' {
                    escaped = true;
                } else {
                    value.push(c);
                }
            }
        }
        Ok((str, CommandParameterValue::String { value }))
    }

    fn parse_some(str: &str) -> IResultV<&str, CommandParameterValue> {
        alt((
            CommandParameterValue::parse_number,
            CommandParameterValue::parse_string,
        ))(str)
    }
}

fn parse_command_parameter(str: &str) -> IResultV<&str, CommandParameter> {
    match CommandParameterValue::parse_some(str) {
        Ok((str, value)) => Ok((
            str,
            CommandParameter {
                key: "".to_string(),
                value,
            },
        )),
        Err(_) => {
            let (str, key) = namestr(str)?;
            let (str, _) = ws(char('='))(str)?;
            let (str, value) = CommandParameterValue::parse_some(str)?;
            Ok((str, CommandParameter { key, value }))
        }
    }
}

pub fn parse_command_parameter_container(
    indent: usize,
) -> impl FnMut(&str) -> IResultV<&str, CommandParameterContainer> {
    move |str: &str| {
        fn normal(indent: usize) -> impl FnMut(&str) -> IResultV<&str, CommandParameterContainer> {
            move |str: &str| {
                let (mut str, _) = char('[')(str)?;
                let mut parameters = CommandParameterContainer::new();

                loop {
                    str = pass_indent(indent, str)?.0;

                    let tmp = parse_command_parameter(str)?;
                    let CommandParameter { key, value } = tmp.1;
                    if parameters.contains_key(&key) {
                        return Err(verror!(
                            "parse_command_parameter_container",
                            str,
                            "duplicate parameter"
                        ));
                    }
                    str = tmp.0;
                    parameters.insert(&key, value);

                    str = space0(str)?.0;

                    if let Ok(tmp) = char::<&str, VerboseError<&str>>(',')(str) {
                        str = tmp.0;
                        str = pass_indent(indent, str)?.0;

                        // support trailing comma
                        if let Ok(tmp) = char::<&str, VerboseError<&str>>(']')(str) {
                            str = tmp.0;
                            break;
                        }
                    } else {
                        str = pass_indent(indent, str)?.0;
                        str = char(']')(str)?.0;
                        break;
                    }
                }

                Ok((str, parameters))
            }
        }

        fn anonymous_string(str: &str) -> IResultV<&str, CommandParameterContainer> {
            let (str, _) = char('[')(str)?;
            let (str, value) = take_until1("]")(str)?;
            let (str, _) = char(']')(str)?;
            if value.contains('\n') {
                return Err(verror!(
                    "parse_command_parameter_container anonymous_string",
                    str,
                    "Cannot contain line break"
                ));
            }
            let mut parameters = CommandParameterContainer::new();
            parameters.insert(
                "",
                CommandParameterValue::String {
                    value: value.to_string(),
                },
            );
            Ok((str, parameters))
        }

        alt((normal(indent), anonymous_string))(str)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        parser::command_parameter::{parse_command_parameter, parse_command_parameter_container},
        tree::parameter::{CommandParameter, CommandParameterValue::*},
    };

    #[macro_export]
    macro_rules! command_params {
        () => ($crate::tree::parameter::CommandParameterContainer::new());

        ($($key:expr => $value:expr),+) => {{
            let mut container = $crate::tree::parameter::CommandParameterContainer::new();
            $( container.insert(&$key.to_string(), $value); )*
            container
        }};
    }

    #[test]
    fn test() {
        assert_eq!(
            parse_command_parameter("number = 1"),
            Ok((
                "",
                CommandParameter {
                    key: "number".to_string(),
                    value: Number {
                        number: 1.0,
                        unit: None
                    }
                }
            ))
        );

        assert_eq!(
            parse_command_parameter("pixel = -1.2px"),
            Ok((
                "",
                CommandParameter {
                    key: "pixel".to_string(),
                    value: Number {
                        number: -1.2,
                        unit: Some("px".to_string())
                    }
                }
            ))
        );

        assert_eq!(
            parse_command_parameter("hw = \"Hello, world!\""),
            Ok((
                "",
                CommandParameter {
                    key: "hw".to_string(),
                    value: String {
                        value: "Hello, world!".to_string()
                    }
                }
            ))
        );

        assert_eq!(
            parse_command_parameter(r#"konnnitiha = "こんにちは。\\ \"Hello\" \\""#),
            Ok((
                "",
                CommandParameter {
                    key: "konnnitiha".to_string(),
                    value: String {
                        value: "こんにちは。\\ \"Hello\" \\".to_string()
                    }
                }
            ))
        );

        assert_eq!(
            parse_command_parameter_container(0)(r#"[strstr]"#),
            Ok((
                "",
                command_params! {
                    "" => String{ value: "strstr".to_string() }
                }
            ))
        );
    }
}
