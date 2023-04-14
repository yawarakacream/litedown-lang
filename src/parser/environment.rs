use std::path::PathBuf;

use anyhow::{anyhow, Result};

use nom::{
    character::complete::{line_ending, space0},
    combinator::eof,
    error::VerboseError,
    Finish,
};

use crate::{
    parser::{environment_header::parse_environment_header, passage_line::parse_passage_line},
    tree::{
        ast::LitedownAst,
        element::{
            EnvironmentElement, LitedownElement, PassageContent, PassageContentText, PassageElement,
        },
    },
    utility::nom::{
        any_to_line_ending, count_indent, pass_blank_line, pass_blank_lines0, IResultV,
    },
    verror,
};

pub(crate) fn parse_environment(
    indent: usize,
) -> impl FnMut(&str) -> IResultV<&str, EnvironmentElement> {
    move |str: &str| {
        let (str, header) = parse_environment_header(indent)(str)?;

        let mut children = Vec::<LitedownElement>::new();

        // pass spaces
        let (str, _) = space0(str)?;
        let str = match line_ending::<&str, VerboseError<&str>>(str) {
            // multi lines
            Ok((str, _)) => {
                if header.raw_body {
                    let mut str = str;
                    let mut buffer = Vec::new();
                    let mut children_indent = None;
                    loop {
                        if let Ok(tmp) = pass_blank_line(str) {
                            str = tmp.0;
                            buffer.push(PassageContent::Text(PassageContentText {
                                value: "\n".to_string(),
                            }));
                            continue;
                        }

                        if children_indent.is_none() {
                            let tmp = count_indent(str)?;
                            if tmp.1 <= indent {
                                return Err(verror!("parse_environment", str, "invalid indent"));
                            }
                            children_indent = Some(tmp.1);
                        }
                        let children_indent = children_indent.unwrap();

                        let tmp = count_indent(str)?;
                        let here_indent = tmp.1;
                        if here_indent < children_indent {
                            break; // pass to parent environment
                        }

                        let tmp = any_to_line_ending(str)?;
                        str = tmp.0;

                        if !buffer.is_empty() {
                            buffer.push(PassageContent::Text(PassageContentText {
                                value: "\n".to_string(),
                            }))
                        }
                        buffer.push(PassageContent::Text(PassageContentText {
                            value: tmp.1[children_indent..].to_string(),
                        }))
                    }
                    if !buffer.is_empty() {
                        while !buffer.is_empty() {
                            if let Some(PassageContent::Text(PassageContentText { value })) =
                                buffer.last()
                            {
                                if value == "\n" {
                                    buffer.pop();
                                    continue;
                                }
                            }
                            break;
                        }
                        children.push(LitedownElement::Passage(PassageElement {
                            contents: buffer,
                        }));
                    }
                    str
                } else {
                    let (str, _) = pass_blank_lines0(str)?;

                    if let Ok(_) = eof::<&str, VerboseError<&str>>(str) {
                        return Err(verror!("parse_environment", str, "no children"));
                    }

                    let (_, children_indent) = count_indent(str)?;

                    if children_indent <= indent {
                        return Err(verror!("parse_environment", str, "invalid indent"));
                    }

                    let mut str = str;
                    let mut buffer = Vec::new();
                    loop {
                        let tmp = pass_blank_lines0(str)?;
                        str = tmp.0;

                        match parse_environment(children_indent)(str) {
                            Ok(tmp) => {
                                if !buffer.is_empty() {
                                    children.push(LitedownElement::Passage(PassageElement {
                                        contents: buffer,
                                    }));
                                    buffer = Vec::new();
                                }
                                str = tmp.0;
                                children.push(LitedownElement::Environment(tmp.1));
                            }
                            Err(_) => {
                                if tmp.1 > 0 && !buffer.is_empty() {
                                    children.push(LitedownElement::Passage(PassageElement {
                                        contents: buffer,
                                    }));
                                    buffer = Vec::new();
                                }

                                let (_, here_indent) = count_indent(str)?;
                                if here_indent < children_indent {
                                    break; // pass to parent environment
                                }
                                if children_indent < here_indent {
                                    return Err(verror!(
                                        "parse_environment",
                                        str,
                                        "invalid indent"
                                    ));
                                }

                                let tmp = parse_passage_line(here_indent)(str)?;
                                str = tmp.0;
                                let mut line = tmp.1;

                                assert!(line.len() > 0);
                                if 0 < buffer.len() {
                                    buffer.push(PassageContent::Text(PassageContentText {
                                        value: "\n".to_string(),
                                    }));
                                }
                                buffer.append(&mut line);

                                if let Ok(tmp) = eof::<&str, VerboseError<&str>>(str) {
                                    str = tmp.0;
                                    break;
                                }
                            }
                        }
                    }
                    if !buffer.is_empty() {
                        children.push(LitedownElement::Passage(PassageElement {
                            contents: buffer,
                        }));
                    }
                    str
                }
            }

            // inline
            Err(_) => {
                if header.raw_body {
                    if let Ok((str, _)) = eof::<&str, VerboseError<&str>>(str) {
                        return Err(verror!("parse_environment", str, "no children"));
                    }

                    let (str, line) = any_to_line_ending(str)?;

                    children.push(LitedownElement::Passage(PassageElement {
                        contents: vec![PassageContent::Text(PassageContentText {
                            value: line.to_string(),
                        })],
                    }));
                    str
                } else {
                    if let Ok((str, _)) = eof::<&str, VerboseError<&str>>(str) {
                        return Err(verror!("parse_environment", str, "no children"));
                    }

                    let (str, line) = any_to_line_ending(str)?;
                    let (_, line) = parse_passage_line(0)(&line)?;

                    children.push(LitedownElement::Passage(PassageElement { contents: line }));
                    str
                }
            }
        };

        if children.is_empty() {
            return Err(verror!("parse_environment", str, "no children"));
        }

        let environment = EnvironmentElement {
            name: header.name,
            parameters: header.parameters,
            children,
        };
        Ok((str, environment))
    }
}

pub fn parse_litedown(source_path: Option<PathBuf>, source_code: &str) -> Result<LitedownAst> {
    let mut source_code = source_code;
    let mut roots = Vec::new();
    while !source_code.is_empty() {
        match parse_environment(0)(source_code).finish() {
            Ok(tmp) => {
                source_code = tmp.0;
                roots.push(tmp.1);
            }
            Err(err) => {
                return Err(anyhow!(
                    "{}",
                    nom::error::convert_error(source_code, err.clone())
                ));
            }
        }
    }
    Ok(LitedownAst { source_path, roots })
}

#[cfg(test)]
mod tests {
    use nom::error::{VerboseError, VerboseErrorKind};

    use crate::{
        command_params,
        parser::{environment::parse_environment, passage_line::parse_passage_line},
        tree::element::{
            EnvironmentElement, LitedownElement, PassageContent, PassageContentFunction,
            PassageContentText, PassageElement,
        },
        tree::parameter::CommandParameterValue::*,
    };

    impl PartialEq for EnvironmentElement {
        fn eq(&self, other: &Self) -> bool {
            self.name == other.name
                && self.parameters == other.parameters
                && self.children == other.children
        }
    }

    impl PartialEq for PassageElement {
        fn eq(&self, other: &Self) -> bool {
            self.contents == other.contents
        }
    }

    impl PartialEq for LitedownElement {
        fn eq(&self, other: &Self) -> bool {
            match (self, other) {
                (Self::Environment(l0), Self::Environment(r0)) => l0 == r0,
                (Self::Passage(l0), Self::Passage(r0)) => l0 == r0,
                _ => false,
            }
        }
    }

    #[test]
    fn test() {
        assert_eq!(
            parse_passage_line(0)("left @func{body} right"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText {
                        value: "left ".to_string()
                    }),
                    PassageContent::Function(PassageContentFunction {
                        name: "func".to_string(),
                        parameters: command_params! {},
                        body: Some("body".to_string())
                    }),
                    PassageContent::Text(PassageContentText {
                        value: " right".to_string()
                    })
                ]
            ))
        );

        assert_eq!(
            parse_environment(0)(
                "\
        @name[
            string = `あいうえお`,
            number = 1.1
        ]@
            aaa
            bbb"
            ),
            Ok((
                "",
                EnvironmentElement {
                    name: "name".to_string(),
                    parameters: command_params! {
                        "string" => String { value: "あいうえお".to_string() },
                        "number" => Number { number: 1.1, unit: None }
                    },
                    children: vec![LitedownElement::Passage(PassageElement {
                        contents: vec![
                            PassageContent::Text(PassageContentText {
                                value: "aaa".to_string()
                            }),
                            PassageContent::Text(PassageContentText {
                                value: "\n".to_string()
                            }),
                            PassageContent::Text(PassageContentText {
                                value: "bbb".to_string()
                            })
                        ]
                    })]
                }
            ))
        );

        assert_eq!(
            parse_environment(0)(
                "@ev@
    line 1
    line 2
    
     
    line 3
    "
            ),
            Ok((
                "",
                EnvironmentElement {
                    name: "ev".to_string(),
                    parameters: command_params! {},
                    children: vec![
                        LitedownElement::Passage(PassageElement {
                            contents: vec![
                                PassageContent::Text(PassageContentText {
                                    value: "line 1".to_string()
                                }),
                                PassageContent::Text(PassageContentText {
                                    value: "\n".to_string()
                                }),
                                PassageContent::Text(PassageContentText {
                                    value: "line 2".to_string()
                                })
                            ]
                        }),
                        LitedownElement::Passage(PassageElement {
                            contents: vec![PassageContent::Text(PassageContentText {
                                value: "line 3".to_string()
                            })]
                        })
                    ]
                }
            ))
        );

        assert_eq!(
            parse_environment(0)(
                "\
@env1@
    aaa
    bbb

    ccc

    @env2@

        xxx

        yyy
        zzz

    ddd
    "
            ),
            Ok((
                "",
                EnvironmentElement {
                    name: "env1".to_string(),
                    parameters: command_params! {},
                    children: vec![
                        LitedownElement::Passage(PassageElement {
                            contents: vec![
                                PassageContent::Text(PassageContentText {
                                    value: "aaa".to_string()
                                }),
                                PassageContent::Text(PassageContentText {
                                    value: "\n".to_string()
                                }),
                                PassageContent::Text(PassageContentText {
                                    value: "bbb".to_string()
                                })
                            ]
                        }),
                        LitedownElement::Passage(PassageElement {
                            contents: vec![PassageContent::Text(PassageContentText {
                                value: "ccc".to_string()
                            })]
                        }),
                        LitedownElement::Environment(EnvironmentElement {
                            name: "env2".to_string(),
                            parameters: command_params! {},
                            children: vec![
                                LitedownElement::Passage(PassageElement {
                                    contents: vec![PassageContent::Text(PassageContentText {
                                        value: "xxx".to_string()
                                    })]
                                }),
                                LitedownElement::Passage(PassageElement {
                                    contents: vec![
                                        PassageContent::Text(PassageContentText {
                                            value: "yyy".to_string()
                                        }),
                                        PassageContent::Text(PassageContentText {
                                            value: "\n".to_string()
                                        }),
                                        PassageContent::Text(PassageContentText {
                                            value: "zzz".to_string()
                                        })
                                    ]
                                }),
                            ]
                        }),
                        LitedownElement::Passage(PassageElement {
                            contents: vec![PassageContent::Text(PassageContentText {
                                value: "ddd".to_string()
                            })]
                        })
                    ]
                }
            ))
        );

        assert_eq!(
            parse_environment(0)("@abcabc@"),
            Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    "",
                    VerboseErrorKind::Context("parse_environment (no children)")
                )]
            }))
        );

        assert_eq!(
            parse_environment(0)(
                "@ev@
         "
            ),
            Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    "",
                    VerboseErrorKind::Context("parse_environment (no children)")
                )]
            }))
        );

        assert_eq!(
            parse_environment(0)(
                "\
            @name@
                aaa
                 bbb"
            ),
            Err(nom::Err::Error(VerboseError {
                errors: vec![(
                    "                 bbb",
                    VerboseErrorKind::Context("parse_environment (invalid indent)")
                )]
            }))
        );
    }
}
