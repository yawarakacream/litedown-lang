use nom::{
    character::complete::{line_ending, space0},
    combinator::eof,
    error::VerboseError,
};

use crate::{
    litedown_element::Element,
    parser::{environment_header::parse_environment_header, passage_line::parse_passage_line},
    utility::nom::{any_to_line_ending, count_indent, pass_blank_lines0, IResultV},
    verror,
};

pub fn parse_environment(indent: usize) -> impl FnMut(&str) -> IResultV<&str, Element> {
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
                                children.push(Element::Passage(buffer));
                                buffer = Vec::new();
                            }
                            str = tmp.0;
                            children.push(tmp.1);
                        }
                        Err(_) => {
                            if tmp.1 > 0 && !buffer.is_empty() {
                                children.push(Element::Passage(buffer));
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
                            let (_, line) = parse_passage_line(&line)?;
                            buffer.push(line);
                        }
                    }
                }
                if !buffer.is_empty() {
                    children.push(Element::Passage(buffer));
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

                children.push(Element::Passage(vec![line]));
                str
            }
        };

        if children.is_empty() {
            return Err(verror!("parse_environment", str, "no children"));
        }

        let environment = Element::Environment {
            name: header.name,
            parameters: header.parameters,
            children,
        };
        Ok((str, environment))
    }
}
