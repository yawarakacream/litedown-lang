use nom::{
    character::complete::{anychar, char, line_ending},
    combinator::eof,
    error::VerboseError,
};

use crate::{
    tree::{
        element::{PassageContent, PassageContentFunction, PassageContentText},
        parameter::CommandParameterContainer,
    },
    utility::nom::{count_indent, namestr, IResultV},
    verror,
};

use super::command_parameter::parse_command_parameter_container;

pub fn parse_passage_line(
    indent: usize,
) -> impl FnMut(&str) -> IResultV<&str, Vec<PassageContent>> {
    move |str: &str| {
        let (mut str, here_indent) = count_indent(str)?;
        if here_indent != indent {
            return Err(verror!("parse_passage_line", str, "invalid indent"));
        }

        let mut ret = Vec::new();

        let mut text_buffer = String::new();
        loop {
            if let Ok(tmp) = eof::<&str, VerboseError<&str>>(str) {
                str = tmp.0;
                break;
            }

            if let Ok(tmp) = line_ending::<&str, VerboseError<&str>>(str) {
                str = tmp.0;
                break;
            }
            let tmp = anychar(str)?;
            str = tmp.0;
            let c = tmp.1;

            if c == '@' {
                if !text_buffer.is_empty() {
                    ret.push(PassageContent::Text(PassageContentText {
                        value: text_buffer,
                    }));
                    text_buffer = String::new();
                }

                let tmp = namestr(str)?;
                str = tmp.0;
                let name = tmp.1;

                let tmp = parse_command_parameter_container(indent)(str)
                    .unwrap_or((str, CommandParameterContainer::new()));
                str = tmp.0;
                let parameters = tmp.1;

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

        if !text_buffer.is_empty() {
            ret.push(PassageContent::Text(PassageContentText {
                value: text_buffer,
            }));
        }
        Ok((str, ret))
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
                (
                    Self::Text(PassageContentText { value: l_value }),
                    Self::Text(PassageContentText { value: r_value }),
                ) => l_value == r_value,
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
            parse_passage_line(0)("@aaa"),
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
            parse_passage_line(0)("@aaa[p = 1]"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {
                        "p" => Number { number: 1.0, unit: None }
                    },
                    body: None,
                })]
            ))
        );

        assert_eq!(
            parse_passage_line(0)("@aaa{bbb}"),
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
            parse_passage_line(0)("@aaa[p = 1]{bbb}"),
            Ok((
                "",
                vec![PassageContent::Function(PassageContentFunction {
                    name: "aaa".to_string(),
                    parameters: command_params! {
                        "p" => Number { number: 1.0, unit: None }
                    },
                    body: Some("bbb".to_string())
                })]
            ))
        );

        assert_eq!(
            parse_passage_line(0)("left @func right"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText {
                        value: "left ".to_string()
                    }),
                    PassageContent::Function(PassageContentFunction {
                        name: "func".to_string(),
                        parameters: command_params! {},
                        body: None,
                    }),
                    PassageContent::Text(PassageContentText {
                        value: " right".to_string()
                    })
                ]
            ))
        );

        assert_eq!(
            parse_passage_line(0)("おはようございます @konnnitiha[16px]{}こんばんは"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText {
                        value: "おはようございます ".to_string()
                    }),
                    PassageContent::Function(PassageContentFunction {
                        name: "konnnitiha".to_string(),
                        parameters: command_params! {
                            "" => Number { number: 16.0, unit: Some("px".to_string()) }
                        },
                        body: Some("".to_string()),
                    }),
                    PassageContent::Text(PassageContentText {
                        value: "こんばんは".to_string()
                    })
                ]
            ))
        );

        assert_eq!(
            parse_passage_line(0)(
                "\
abc @func[
    p = 'q'
] def"
            ),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText {
                        value: "abc ".to_string()
                    }),
                    PassageContent::Function(PassageContentFunction {
                        name: "func".to_string(),
                        parameters: command_params! {
                            "p" => String { value: "q".to_string() }
                        },
                        body: None,
                    }),
                    PassageContent::Text(PassageContentText {
                        value: " def".to_string()
                    })
                ]
            ))
        );
    }
}
