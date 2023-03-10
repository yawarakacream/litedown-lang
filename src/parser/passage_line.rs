use nom::{
    character::complete::{anychar, char, line_ending, space0},
    combinator::eof,
    error::VerboseError,
};

use crate::{
    parser::command_parameter::parse_command_parameter,
    tree::{
        element::{PassageContent, PassageContentFunction, PassageContentText},
        parameter::{CommandParameter, CommandParameterContainer},
    },
    utility::nom::{namestr, IResultV},
    verror,
};

pub fn parse_passage_line(str: &str) -> IResultV<&str, Vec<PassageContent>> {
    let mut ret = Vec::new();

    let mut text_buffer = String::new();
    let mut str = str;
    loop {
        if let Ok((str, _)) = eof::<&str, VerboseError<&str>>(str) {
            if !text_buffer.is_empty() {
                ret.push(PassageContent::Text(PassageContentText(text_buffer)));
            }

            return Ok((str, ret));
        }

        if let Ok(tmp) = line_ending::<&str, VerboseError<&str>>(str) {
            return Err(verror!(
                "parse_passage_line",
                tmp.0,
                "cannot contain line_ending"
            ));
        }
        let tmp = anychar(str)?;
        str = tmp.0;
        let c = tmp.1;

        if c == '@' {
            if !text_buffer.is_empty() {
                ret.push(PassageContent::Text(PassageContentText(text_buffer)));
                text_buffer = String::new();
            }

            let tmp = namestr(str)?;
            str = tmp.0;
            let name = tmp.1;

            let parameters = match char::<&str, VerboseError<&str>>('[')(str) {
                Ok(tmp) => {
                    str = tmp.0;
                    let mut parameters = CommandParameterContainer::new();

                    loop {
                        str = space0(str)?.0;

                        let tmp = parse_command_parameter(str)?;
                        let CommandParameter { key, value } = tmp.1;
                        if parameters.contains_key(&key) {
                            return Err(verror!("parse_passage_line", str, "duplicate parameter"));
                        }
                        str = tmp.0;
                        parameters.insert(&key, value);

                        str = space0(str)?.0;

                        if let Ok(tmp) = char::<&str, VerboseError<&str>>(',')(str) {
                            str = tmp.0;
                            str = space0(str)?.0;

                            // support trailing comma
                            if let Ok(tmp) = char::<&str, VerboseError<&str>>(']')(str) {
                                str = tmp.0;
                                break;
                            }
                        } else {
                            str = space0(tmp.0)?.0;
                            str = char(']')(str)?.0;
                            break;
                        }
                    }

                    parameters
                }
                Err(_) => CommandParameterContainer::new(),
            };

            let body = match char::<&str, VerboseError<&str>>('{')(str) {
                Ok(tmp) => {
                    str = tmp.0;
                    let mut body = String::new();
                    let mut escaped = false;

                    loop {
                        let tmp = anychar(str)?;
                        str = tmp.0;
                        let c = tmp.1;

                        if escaped {
                            body.push(c);
                        } else if c == '\\' {
                            escaped = true;
                        } else if c == '}' {
                            break;
                        } else {
                            body.push(c);
                        }
                    }

                    Some(body)
                }
                Err(_) => match char::<&str, VerboseError<&str>>('$')(str) {
                    Ok(tmp) => {
                        str = tmp.0;
                        let mut body = String::new();

                        loop {
                            let tmp = anychar(str)?;
                            str = tmp.0;
                            let c = tmp.1;

                            if c == '$' {
                                break;
                            } else {
                                body.push(c);
                            }
                        }

                        Some(body)
                    }
                    Err(_) => None,
                },
            };

            ret.push(PassageContent::Function(PassageContentFunction {
                name,
                parameters,
                body,
            }));
        } else {
            text_buffer.push(c);
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        command_params,
        parser::passage_line::parse_passage_line,
        tree::element::{PassageContent, PassageContentFunction, PassageContentText},
        tree::parameter::CommandParameterValue::*,
    };

    impl PartialEq for PassageContent {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Text(PassageContentText(l0)), Self::Text(PassageContentText(r0))) => {
                    l0 == r0
                }
                (
                    Self::Function(PassageContentFunction {
                        name: l_name,
                        parameters: l_parameters,
                        body: l_body,
                    }),
                    Self::Function(PassageContentFunction {
                        name: r_name,
                        parameters: r_parameters,
                        body: r_body,
                    }),
                ) => l_name == r_name && *l_parameters == *r_parameters && l_body == r_body,
                _ => false,
            }
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            parse_passage_line("@aaa"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {},
                    body: None,
                })]
            ))
        );

        assert_eq!(
            parse_passage_line("@aaa[p = 1]"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {
                        "p" => Number(None, 1.0)
                    },
                    body: None,
                })]
            ))
        );

        assert_eq!(
            parse_passage_line("@aaa{bbb}"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {},
                    body: Some("bbb".to_string())
                })]
            ))
        );

        assert_eq!(
            parse_passage_line("@aaa[p = 1]{bbb}"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {
                        "p" => Number(None, 1.0)
                    },
                    body: Some("bbb".to_string())
                })]
            ))
        );

        assert_eq!(
            parse_passage_line("left @func right"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText("left ".to_string())),
                    PassageContent::Function(PassageContentFunction {
                        name: "func".to_string(),
                        parameters: command_params! {},
                        body: None,
                    }),
                    PassageContent::Text(PassageContentText(" right".to_string()))
                ]
            ))
        );

        assert_eq!(
            parse_passage_line("??????????????????????????? @konnnitiha[16px]{}???????????????"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText("??????????????????????????? ".to_string())),
                    PassageContent::Function(PassageContentFunction {
                        name: "konnnitiha".to_string(),
                        parameters: command_params! {
                            "" => Number(Some("px".to_string()), 16.0)
                        },
                        body: Some("".to_string()),
                    }),
                    PassageContent::Text(PassageContentText("???????????????".to_string()))
                ]
            ))
        );

        // assert_eq!(parse_passage_line("str"))
    }
}
