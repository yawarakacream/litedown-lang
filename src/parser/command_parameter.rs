use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, anychar, char},
};

use crate::{
    tree::parameter::{CommandParameter, CommandParameterValue},
    utility::nom::{namestr, parse_f64, ws, IResultV},
};

impl CommandParameterValue {
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
        Ok((str, CommandParameterValue::String(value)))
    }

    fn parse_number(str: &str) -> IResultV<&str, CommandParameterValue> {
        let (str, value) = parse_f64(str)?;
        match Self::parse_number_unit(str) {
            Ok((str, unit)) => Ok((str, CommandParameterValue::Number(Some(unit), value))),
            Err(_) => Ok((str, CommandParameterValue::Number(None, value))),
        }
    }

    fn parse_number_unit(str: &str) -> IResultV<&str, String> {
        let (str, value) = alt((alpha1, tag("%")))(str)?;
        Ok((str, value.to_string()))
    }

    fn parse_some(str: &str) -> IResultV<&str, CommandParameterValue> {
        alt((
            CommandParameterValue::parse_string,
            CommandParameterValue::parse_number,
        ))(str)
    }
}

pub fn parse_command_parameter(str: &str) -> IResultV<&str, CommandParameter> {
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

#[cfg(test)]
mod tests {
    use crate::{
        parser::command_parameter::parse_command_parameter,
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
                    value: Number(None, 1.0)
                }
            ))
        );

        assert_eq!(
            parse_command_parameter("pixel = -1.2px"),
            Ok((
                "",
                CommandParameter {
                    key: "pixel".to_string(),
                    value: Number(Some("px".to_string()), -1.2)
                }
            ))
        );

        assert_eq!(
            parse_command_parameter("hw = 'Hello, world!'"),
            Ok((
                "",
                CommandParameter {
                    key: "hw".to_string(),
                    value: String("Hello, world!".to_string())
                }
            ))
        );

        assert_eq!(
            parse_command_parameter(r#"konnnitiha = "こんにちは。\\ \"Hello\" \\""#),
            Ok((
                "",
                CommandParameter {
                    key: "konnnitiha".to_string(),
                    value: String("こんにちは。\\ \"Hello\" \\".to_string())
                }
            ))
        );
    }
}
