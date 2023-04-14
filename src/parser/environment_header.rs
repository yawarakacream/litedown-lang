use nom::{
    character::complete::{anychar, char, line_ending, space0},
    error::VerboseError,
};

use crate::{
    tree::{element::EnvironmentHeader, parameter::CommandParameterContainer},
    utility::nom::{count_indent, namestr, IResultV},
    verror,
};

use super::command_parameter::parse_command_parameter_container;

pub fn pass_indent(indent: usize, str: &str) -> IResultV<&str, usize> {
    let (str, _) = space0(str)?;
    match line_ending::<&str, VerboseError<&str>>(str) {
        Ok((str, _)) => {
            let (str, here_indent) = count_indent(str)?;
            if here_indent < indent {
                Err(verror!(
                    "pass_spaces_in_environment_header",
                    str,
                    "invalid indent"
                ))
            } else {
                Ok((str, here_indent))
            }
        }
        Err(_) => Ok((str, 0)),
    }
}

pub fn parse_environment_header(
    indent: usize,
) -> impl FnMut(&str) -> IResultV<&str, EnvironmentHeader> {
    move |str: &str| {
        let (str, here_indent) = count_indent(str)?;
        if indent != here_indent {
            return Err(verror!("parse_environment_header", str, "invalid indent"));
        }

        let (str, _) = char('@')(str)?;
        let (str, name) = namestr(str)?;
        let (str, parameters) = parse_command_parameter_container(indent)(str)
            .unwrap_or((str, CommandParameterContainer::new()));

        let (str, c) = anychar(str)?;
        let raw_body = match c {
            '@' => false,
            ':' => true,
            _ => {
                return Err(verror!(
                    "parse_environment_header",
                    str,
                    "invalid last splitter"
                ))
            }
        };
        let result = EnvironmentHeader {
            name: name.to_string(),
            parameters,
            raw_body,
        };
        Ok((str, result))
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        command_params, parser::environment_header::parse_environment_header,
        tree::element::EnvironmentHeader, tree::parameter::CommandParameterValue::*,
    };

    impl PartialEq for EnvironmentHeader {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name && self.parameters == other.parameters
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            parse_environment_header(0)("@headername[2.4]@"),
            Ok((
                "",
                EnvironmentHeader {
                    name: "headername".to_string(),
                    parameters: command_params! {
                        ""=>Number{number:2.4, unit:None}
                    },
                    raw_body: false,
                }
            ))
        );

        assert_eq!(
            parse_environment_header(0)(
                r#"@headername[string="aa\"あ",number= 1.1, pixel =5px, M = -7.8em]@"#
            ),
            Ok((
                "",
                EnvironmentHeader {
                    name: "headername".to_string(),
                    parameters: command_params! {
                        "string" => String { value: "aa\"あ".to_string() },
                        "number" => Number { number: 1.1, unit: None },
                        "pixel" => Number { number: 5.0, unit: Some("px".to_string()) },
                        "M" => Number { number: -7.8, unit: Some("em".to_string()) }
                    },
                    raw_body: false,
                }
            ))
        );

        assert_eq!(
            parse_environment_header(4)(
                "    @name[
        aiueo = `あいうえお`,
        iti-ni = 12
    ]@"
            ),
            Ok((
                "",
                EnvironmentHeader {
                    name: "name".to_string(),
                    parameters: command_params! {
                        "aiueo" => String { value: "あいうえお".to_string() },
                        "iti-ni" => Number { number: 12.0, unit: None }
                    },
                    raw_body: false,
                }
            ))
        )
    }
}
