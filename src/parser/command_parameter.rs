use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{anychar, char},
};

use crate::{
    environment::{CommandParameterValue, NumberUnit},
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
            Ok((str, unit)) => Ok((str, CommandParameterValue::Number(unit, value))),
            Err(_) => Ok((str, CommandParameterValue::Number(NumberUnit::None, value))),
        }
    }

    fn parse_number_unit(str: &str) -> IResultV<&str, NumberUnit> {
        let (str, value) = alt((tag("px"), tag("em")))(str)?;
        Ok((
            str,
            match value {
                "px" => NumberUnit::Px,
                "em" => NumberUnit::Em,
                _ => unreachable!(),
            },
        ))
    }

    fn parse_some(str: &str) -> IResultV<&str, CommandParameterValue> {
        alt((
            CommandParameterValue::parse_string,
            CommandParameterValue::parse_number,
        ))(str)
    }
}

pub fn parse_command_parameter(str: &str) -> IResultV<&str, (String, CommandParameterValue)> {
    match CommandParameterValue::parse_some(str) {
        Ok((str, value)) => Ok((str, ("".to_string(), value))),
        Err(_) => {
            let (str, key) = namestr(str)?;
            let (str, _) = ws(char('='))(str)?;
            let (str, value) = CommandParameterValue::parse_some(str)?;
            Ok((str, (key, value)))
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        environment::{
            CommandParameterValue::{self, *},
            NumberUnit,
        },
        parser::command_parameter::parse_command_parameter,
    };

    #[macro_export]
    macro_rules! command_param {
        ($name:expr => $value:expr) => {
            ($name.to_string(), $value)
        };
    }

    #[macro_export]
    macro_rules! command_params {
        ($($name:expr => $value:expr),*) => {
            vec![ $( crate::command_param!($name => $value), )* ].into_iter().collect()
        };
    }

    impl PartialEq for CommandParameterValue {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::String(l0), Self::String(r0)) => l0 == r0,
                (Self::Number(l0, l1), Self::Number(r0, r1)) => l0 == r0 && (l1 - r1).abs() < 1e-8,
                _ => false,
            }
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            parse_command_parameter("number = 1"),
            Ok((
                "",
                command_param!("number" => Number(NumberUnit::None, 1.0))
            ))
        );

        assert_eq!(
            parse_command_parameter("pixel = -1.2px"),
            Ok(("", command_param!("pixel" => Number(NumberUnit::Px, -1.2))))
        );

        assert_eq!(
            parse_command_parameter("hw = 'Hello, world!'"),
            Ok((
                "",
                command_param!("hw" => String("Hello, world!".to_string()))
            ))
        );

        assert_eq!(
            parse_command_parameter(r#"konnnitiha = "こんにちは。\\ \"Hello\" \\""#),
            Ok((
                "",
                command_param!("konnnitiha" => String("こんにちは。\\ \"Hello\" \\".to_string()))
            ))
        )
    }
}
