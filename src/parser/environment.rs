use nom::{
    character::complete::{line_ending, space0},
    combinator::eof,
    error::VerboseError,
};

use crate::{
    litedown_element::{
        Element, EnvironmentElement, LitedownAst, PassageContent, PassageContentText,
        PassageElement,
    },
    parser::{environment_header::parse_environment_header, passage_line::parse_passage_line},
    utility::nom::{any_to_line_ending, count_indent, pass_blank_lines0, IResultV},
    verror,
};

pub(crate) fn parse_environment(indent: usize) -> impl FnMut(&str) -> IResultV<&str, Element> {
    move |str: &str| {
        let (str, header) = parse_environment_header(indent)(str)?;

        let mut children = Vec::<Element>::new();

        // pass spaces
        let (str, _) = space0(str)?;
        let str = match line_ending::<&str, VerboseError<&str>>(str) {
            // multi lines
            Ok((str, _)) => {
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
                                children.push(Element::Passage(PassageElement(buffer)));
                                buffer = Vec::new();
                            }
                            str = tmp.0;
                            children.push(tmp.1);
                        }
                        Err(_) => {
                            if tmp.1 > 0 && !buffer.is_empty() {
                                children.push(Element::Passage(PassageElement(buffer)));
                                buffer = Vec::new();
                            }

                            let tmp = count_indent(str)?;
                            let here_indent = tmp.1;
                            if here_indent < children_indent {
                                break; // pass to parent environment
                            }
                            str = tmp.0;

                            if children_indent < here_indent {
                                return Err(verror!("parse_environment", str, "invalid indent"));
                            }

                            if let Ok(tmp) = eof::<&str, VerboseError<&str>>(str) {
                                str = tmp.0;
                                break;
                            }

                            let tmp = any_to_line_ending(str)?;
                            str = tmp.0;
                            let line = tmp.1;
                            assert!(line.len() > 0);
                            let (_, mut line) = parse_passage_line(&line)?;
                            if 0 < buffer.len() {
                                buffer.push(PassageContent::Text(PassageContentText(
                                    "\n".to_string(),
                                )));
                            }
                            buffer.append(&mut line);
                        }
                    }
                }
                if !buffer.is_empty() {
                    children.push(Element::Passage(PassageElement(buffer)));
                }
                str
            }

            // inline
            Err(_) => {
                if let Ok((str, _)) = eof::<&str, VerboseError<&str>>(str) {
                    return Err(verror!("parse_environment", str, "no children"));
                }

                let (str, line) = any_to_line_ending(str)?;
                let (_, line) = parse_passage_line(&line)?;

                children.push(Element::Passage(PassageElement(line)));
                str
            }
        };

        if children.is_empty() {
            return Err(verror!("parse_environment", str, "no children"));
        }

        let environment = Element::Environment(EnvironmentElement {
            name: header.name,
            parameters: header.parameters,
            children,
        });
        Ok((str, environment))
    }
}

pub fn parse_litedown(str: &str) -> Result<LitedownAst, nom::Err<VerboseError<&str>>> {
    let (_, environment) = parse_environment(0)(str)?;
    // let environment = match environment {
    //     Element::Environment(environment) => environment,
    //     Element::Passage(_) => unreachable!(),
    // };
    Ok(LitedownAst { root: environment })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use nom::error::{VerboseError, VerboseErrorKind};

    use crate::{
        command_params,
        litedown_element::{
            CommandParameterValue::*, Element, EnvironmentElement, PassageContent,
            PassageContentFunction, PassageContentText, PassageElement,
        },
        parser::{environment::parse_environment, passage_line::parse_passage_line},
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
            self.0 == other.0
        }
    }

    impl PartialEq for Element {
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
            parse_passage_line("left @func{body} right"),
            Ok((
                "",
                vec![
                    PassageContent::Text(PassageContentText("left ".to_string())),
                    PassageContent::Function(PassageContentFunction {
                        name: "func".to_string(),
                        parameters: HashMap::new(),
                        body: Some("body".to_string())
                    }),
                    PassageContent::Text(PassageContentText(" right".to_string()))
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
                Element::Environment(EnvironmentElement {
                    name: "name".to_string(),
                    parameters: command_params! {
                        "string" => String("あいうえお".to_string()),
                        "number" => Number(None, 1.1)
                    },
                    children: vec![Element::Passage(PassageElement(vec![
                        PassageContent::Text(PassageContentText("aaa".to_string())),
                        PassageContent::Text(PassageContentText("\n".to_string())),
                        PassageContent::Text(PassageContentText("bbb".to_string()))
                    ]))]
                })
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
                Element::Environment(EnvironmentElement {
                    name: "ev".to_string(),
                    parameters: HashMap::new(),
                    children: vec![
                        Element::Passage(PassageElement(vec![
                            PassageContent::Text(PassageContentText("line 1".to_string())),
                            PassageContent::Text(PassageContentText("\n".to_string())),
                            PassageContent::Text(PassageContentText("line 2".to_string()))
                        ])),
                        Element::Passage(PassageElement(vec![PassageContent::Text(
                            PassageContentText("line 3".to_string())
                        )]))
                    ]
                })
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
                Element::Environment(EnvironmentElement {
                    name: "env1".to_string(),
                    parameters: HashMap::new(),
                    children: vec![
                        Element::Passage(PassageElement(vec![
                            PassageContent::Text(PassageContentText("aaa".to_string())),
                            PassageContent::Text(PassageContentText("\n".to_string())),
                            PassageContent::Text(PassageContentText("bbb".to_string()))
                        ])),
                        Element::Passage(PassageElement(vec![PassageContent::Text(
                            PassageContentText("ccc".to_string())
                        )])),
                        Element::Environment(EnvironmentElement {
                            name: "env2".to_string(),
                            parameters: HashMap::new(),
                            children: vec![
                                Element::Passage(PassageElement(vec![PassageContent::Text(
                                    PassageContentText("xxx".to_string())
                                )])),
                                Element::Passage(PassageElement(vec![
                                    PassageContent::Text(PassageContentText("yyy".to_string())),
                                    PassageContent::Text(PassageContentText("\n".to_string())),
                                    PassageContent::Text(PassageContentText("zzz".to_string()))
                                ])),
                            ]
                        }),
                        Element::Passage(PassageElement(vec![PassageContent::Text(
                            PassageContentText("ddd".to_string())
                        )]))
                    ]
                })
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
                    "bbb",
                    VerboseErrorKind::Context("parse_environment (invalid indent)")
                )]
            }))
        );
    }
}
