use nom::{
    character::complete::{alphanumeric1, char, multispace0},
    error::{VerboseError, VerboseErrorKind},
};
use std::collections::HashMap;

use crate::{
    environment::EnvironmentHeader, nom_utility::IResultV,
    parser::command_parameter::parse_command_parameter, verror,
};

pub fn parse_environment_header(str: &str) -> IResultV<&str, EnvironmentHeader> {
    let (str, _) = char('@')(str)?;
    let (str, name) = alphanumeric1(str)?;

    let (str, parameters) = match char::<&str, VerboseError<&str>>('[')(str) {
        Ok((mut str, _)) => {
            let mut result = HashMap::new();

            loop {
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

                str = multispace0(str)?.0;

                // trailing comma
                if let Ok(tmp) = char::<&str, VerboseError<&str>>(',')(str) {
                    str = tmp.0;

                    if let Ok(tmp) = char::<&str, VerboseError<&str>>(']')(str) {
                        str = tmp.0;
                        break;
                    }

                    str = multispace0(str)?.0;
                } else {
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
            parse_environment_header("@headername[2.4]@"),
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
            parse_environment_header(
                "@headername[string=\"aa\\\"あ\",number= 1.1, pixel =5px, M = -7.8em]@"
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
    }
}
