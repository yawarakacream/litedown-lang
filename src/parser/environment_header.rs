use nom::{
    character::complete::{char, line_ending, space0},
    error::VerboseError,
};
use std::collections::HashMap;

use crate::{
    environment::EnvironmentHeader,
    nom_utility::{count_indent, namestr, pass_blank_lines0, IResultV},
    parser::command_parameter::parse_command_parameter,
    verror,
};

fn pass_spaces_in_environment_header(indent: usize, str: &str) -> IResultV<&str, usize> {
    let (str, _) = space0(str)?;
    match line_ending::<&str, VerboseError<&str>>(str) {
        Ok((str, _)) => {
            let (str, _) = pass_blank_lines0(str)?;

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

        let (str, parameters) = match char::<&str, VerboseError<&str>>('[')(str) {
            Ok((mut str, _)) => {
                let mut result = HashMap::new();

                loop {
                    str = pass_spaces_in_environment_header(indent, str)?.0;

                    let tmp = parse_command_parameter(str)?;
                    let (key, value) = tmp.1;
                    if result.contains_key(&key) {
                        return Err(verror!(
                            "parse_environment_header",
                            str,
                            "duplicate parameter"
                        ));
                    }
                    str = tmp.0;
                    result.insert(key, value);

                    str = space0(str)?.0;

                    if let Ok(tmp) = char::<&str, VerboseError<&str>>(',')(str) {
                        str = tmp.0;
                        str = pass_spaces_in_environment_header(indent, str)?.0;

                        // support trailing comma
                        if let Ok(tmp) = char::<&str, VerboseError<&str>>(']')(str) {
                            str = tmp.0;
                            break;
                        }
                    } else {
                        str = pass_spaces_in_environment_header(indent, str)?.0;
                        str = char(']')(str)?.0;
                        break;
                    }
                }

                (str, result)
            }
            Err(_) => (str, HashMap::new()),
        };

        let (str, _) = char('@')(str)?;

        let result = EnvironmentHeader {
            name: name.to_string(),
            parameters,
        };
        Ok((str, result))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        environment::{CommandParameterValue::*, EnvironmentHeader, NumberUnit},
        param,
        parser::environment_header::parse_environment_header,
    };

    macro_rules! params {
        ($($name:expr => $value:expr),*) => {
            vec![ $( param!($name => $value), )* ].into_iter().collect()
        };
    }

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
                    parameters: params![
                        "" => Number(NumberUnit::None, 2.4)
                    ]
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
                    parameters: params![
                        "string" => String("aa\"あ".to_string()),
                        "number" => Number(NumberUnit::None, 1.1),
                        "pixel" => Number(NumberUnit::Px, 5.0),
                        "M" => Number(NumberUnit::Em, -7.8)
                    ]
                }
            ))
        );

        assert_eq!(
            parse_environment_header(4)(
                "    @name[
        aiueo = `あいうえお`,
        iti_ni = 12
    ]@"
            ),
            Ok((
                "",
                EnvironmentHeader {
                    name: "name".to_string(),
                    parameters: params![
                        "aiueo" => String("あいうえお".to_string()),
                        "iti_ni" => Number(NumberUnit::None, 12.0)
                    ]
                }
            ))
        )
    }
}
