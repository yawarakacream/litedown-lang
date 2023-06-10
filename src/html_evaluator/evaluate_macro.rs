#[macro_export]
macro_rules! deconstruct_required_arguments {
    (($($argname:ident),*) from $element:ident) => {
        let arguments = &$element.arguments;
        let mut argument_index = 0;
        $(
            let $argname = {
                let argname = stringify!($argname);
                let from_index = arguments.get_by_index(argument_index);
                let from_name = arguments.get_by_name(argname);
                if from_index.is_some() && from_name.is_some() {
                    anyhow::bail!(
                        "function '{}' got multiple values for argument '{}'",
                        $element.name, argname
                    );
                }
                if from_index.is_none() && from_name.is_none() {
                    anyhow::bail!(
                        "function '{}' missing required positional argument '{}'",
                        $element.name, argname
                    );
                }
                from_index.or(from_name).unwrap()
            };
            argument_index += 1;
        )*
        if arguments.get_by_index(argument_index).is_some() {
            let needed = argument_index;
            while arguments.get_by_index(argument_index).is_some() {
                argument_index += 1;
            }
            anyhow::bail!(
                "function '{}' takes {} positional argument but {} were given",
                $element.name,
                needed,
                argument_index
            );
        }
    };
}

#[macro_export]
macro_rules! evaluate_with_ld2html_evaluator {
    ($function:ident to $html_element:ident with $evaluator:ident;
        function: { $($func_name:ident: ($func_element:ident) => $func_block:block)* }
    ) => {
        for passage in &$function.body.value {
            let mut passage_html = match $function.body.form {
                crate::tree::function::FunctionBodyForm::Block => HtmlElement::new("p"),
                crate::tree::function::FunctionBodyForm::Inline => HtmlElement::new("span"),
            };

            for passage_element in &passage.elements {
                match &passage_element {
                    crate::tree::function::PassageElement::String(string) => {
                        passage_html.append_text(&string);
                    }

                    crate::tree::function::PassageElement::Function(child_function) => {
                        if child_function.body.form == crate::tree::function::FunctionBodyForm::Block {
                            if !passage_html.is_child_empty() {
                                $html_element.append(passage_html);
                                passage_html = match $function.body.form {
                                    crate::tree::function::FunctionBodyForm::Block => HtmlElement::new("p"),
                                    crate::tree::function::FunctionBodyForm::Inline => HtmlElement::new("span"),
                                };
                            }
                        }
                        match child_function.name.as_str() {
                            $(
                                stringify!($func_name) => {
                                    let $func_element = child_function;
                                    $func_block;
                                }
                            )*
                            _ => {
                                let evaluated = $evaluator.evaluate_main_function(child_function)?;
                                if let Some(evaluated) = evaluated {
                                    match &child_function.body.form {
                                        crate::tree::function::FunctionBodyForm::Block => {
                                            $html_element.append(evaluated);
                                        }
                                        crate::tree::function::FunctionBodyForm::Inline => {
                                            passage_html.append(evaluated);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !passage_html.is_child_empty() {
                $html_element.append(passage_html);
            }
        }
    };

    ($function:ident to $html_element:ident with $evaluator:ident) => {
        evaluate_with_ld2html_evaluator!($function to $html_element with $evaluator; function: {});
    };

}

#[macro_export]
macro_rules! evaluate_litedown_function {
    ($function:ident;
        $($func_name:ident: ($func_element:ident) => $func_block:block)*
    ) => {
        for passage in &$function.body.value {
            for passage_element in &passage.elements {
                match &passage_element {
                    crate::tree::function::PassageElement::String(string) => {
                        if !crate::utility::whitespace::is_blank(string) {
                            anyhow::bail!("cannot write string in function '{}'", $function.name);
                        }
                    }

                    crate::tree::function::PassageElement::Function(child_function) => {
                        match child_function.name.as_str() {
                            $(
                                stringify!($func_name) => {
                                    let $func_element = child_function;
                                    $func_block;
                                }
                            )*
                            _ => {
                                anyhow::bail!("unknown function: {}", child_function.name);
                            }
                        }
                    }
                }
            }
        }
    };
}
