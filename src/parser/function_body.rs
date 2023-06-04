use anyhow::{bail, Result};

use crate::{
    tree::function::{FunctionBody, FunctionBodyForm, LitedownPassage, PassageElement},
    utility::indented_string::IndentedStringIterator,
};

use super::function::parse_function;

pub fn parse_function_body(
    iter: &mut IndentedStringIterator,
    started_at_first_character: bool,
) -> Result<Option<FunctionBody>> {
    iter.parse(|iter| {
        // inline body
        if let Ok(_) = iter.next_char_as('{') {
            let mut elements = Vec::new();
            let mut string_body = String::new();
            loop {
                if let Ok(function) = parse_function(iter) {
                    if !string_body.is_empty() {
                        elements.push(PassageElement::String(string_body));
                        string_body = String::new();
                    }
                    elements.push(PassageElement::Function(function));
                    continue;
                }

                match iter.next_char() {
                    Some(char) => {
                        if char == '}' {
                            if !string_body.is_empty() {
                                elements.push(PassageElement::String(string_body));
                            }
                            return Ok(Some(FunctionBody {
                                form: FunctionBodyForm::Inline,
                                value: vec![LitedownPassage { elements }],
                            }));
                        } else {
                            string_body.push(char);
                        }
                    }
                    None => bail!("EOL while scanning inline function"),
                }
            }
        }

        // inline raw string body
        if let Ok(_) = iter.next_char_as('$') {
            // let mut body = String::new();
            // let mut escaped = false;
            // while let Some(char) = iter.next_char() {
            //     if escaped {
            //         body.push(char)
            //     } else {
            //         if char == '$' {
            //             return Ok(Some(FunctionBody {
            //                 form: FunctionBodyForm::Inline,
            //                 value: vec![LitedownPassage {
            //                     elements: vec![PassageElement::String(body)],
            //                 }],
            //             }));
            //         }
            //         if char == '\\' {
            //             escaped = true;
            //         } else {
            //             body.push(char);
            //         }
            //     }
            // }
            // bail!("The finish character '$' not found");
            let mut body = String::new();
            while let Some(char) = iter.next_char() {
                if char == '$' {
                    return Ok(Some(FunctionBody {
                        form: FunctionBodyForm::Inline,
                        value: vec![LitedownPassage {
                            elements: vec![PassageElement::String(body)],
                        }],
                    }));
                }
                body.push(char);
            }
            bail!("The finish character '$' not found");
        }

        if started_at_first_character {
            // block body
            if let Ok(_) = iter.next_char_as('@') {
                let header_indent = iter.peek_line_max_indent().unwrap();

                iter.pass_whitespaces();
                if iter.has_next_char() {
                    bail!(
                        "Cannot write after '@', found {:?}",
                        iter.collect_until_line_ending()
                    );
                }
                iter.next_line();
                iter.pass_blank_lines();

                let body_indent = match iter.peek_line_max_indent() {
                    Some(body_indent) => {
                        if body_indent <= header_indent {
                            None
                        } else {
                            Some(body_indent)
                        }
                    }
                    None => None,
                };
                let body_indent = match body_indent {
                    Some(body_indent) => {
                        iter.set_line_indent(body_indent);
                        body_indent
                    }
                    None => {
                        // bail!("block body function must have body");
                        return Ok(Some(FunctionBody {
                            form: FunctionBodyForm::Block,
                            value: Vec::new(),
                        }));
                    }
                };

                let mut passages = Vec::new();
                let mut elements = Vec::new();
                let mut string_body = String::new();
                loop {
                    match iter.next_char() {
                        Some(char) => {
                            if char == '@' {
                                if !string_body.is_empty() {
                                    elements.push(PassageElement::String(string_body));
                                    string_body = String::new();
                                }
                                iter.back_char().unwrap();
                                let function = parse_function(iter)?;
                                elements.push(PassageElement::Function(function));
                            } else {
                                string_body.push(char);
                            }
                        }
                        None => {
                            if !iter.next_line() {
                                break;
                            }

                            let passed_blank_lines = iter.pass_blank_lines();
                            if passed_blank_lines == 0 {
                                string_body.push('\n');
                            } else {
                                if !string_body.is_empty() {
                                    elements.push(PassageElement::String(string_body));
                                    string_body = String::new();
                                }
                                if !elements.is_empty() {
                                    passages.push(LitedownPassage { elements });
                                    elements = Vec::new();
                                }
                            }

                            if let Some(here_indent) = iter.peek_line_max_indent() {
                                if body_indent < here_indent {
                                    bail!("invalid indent");
                                }
                                if here_indent < body_indent {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                if !string_body.is_empty() {
                    elements.push(PassageElement::String(string_body));
                }
                if !elements.is_empty() {
                    passages.push(LitedownPassage { elements });
                }

                iter.set_line_indent(header_indent);

                return Ok(Some(FunctionBody {
                    form: FunctionBodyForm::Block,
                    value: passages,
                }));
            }

            // block raw string body
            if let Ok(_) = iter.next_char_as(':') {
                let header_indent = iter.peek_line_max_indent().unwrap();

                iter.pass_whitespaces();
                if iter.has_next_char() {
                    bail!(
                        "Cannot write after ':', found {:?}",
                        iter.collect_until_line_ending()
                    );
                }
                if !iter.next_line() {
                    bail!("block raw string body function must have body");
                }

                let mut body = String::new();
                let mut is_body_indent_fitted = false;
                loop {
                    if !is_body_indent_fitted && iter.has_next_char() {
                        is_body_indent_fitted = true;
                        let body_indent = iter.peek_line_max_indent().unwrap();
                        if body_indent <= header_indent {
                            bail!("block raw string body function must have body");
                        }
                        iter.set_line_indent(body_indent);
                    }
                    while let Some(char) = iter.next_char() {
                        body.push(char);
                    }
                    if iter.next_line() {
                        body.push('\n');
                    } else {
                        iter.set_line_indent(header_indent);
                        break;
                    }
                }
                if body.is_empty() {
                    bail!("block body function must have body");
                }

                return Ok(Some(FunctionBody {
                    form: FunctionBodyForm::Block,
                    value: vec![LitedownPassage {
                        elements: vec![PassageElement::String(body)],
                    }],
                }));
            }
        }
        Ok(None)
    })
}
