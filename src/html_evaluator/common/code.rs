use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};

use crate::{
    html_evaluator::litedown::Ld2HtmlEvaluator,
    tree::function::{FunctionBodyForm, LitedownFunction},
    utility::html::HtmlElement,
};

pub fn evaluate_code(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut lang = match function.arguments.get_by_name("lang") {
        Some(lang) => Some(lang.try_into_string()?),
        None => None,
    };

    let code = match function.arguments.get_by_name("src") {
        Some(src) => {
            let src = src.try_into_string()?;
            let path = if src.starts_with("/") {
                PathBuf::from(src)
            } else {
                let source_path = evaluator
                    .get_source_path()
                    .context("cannot use relative path")?;
                source_path.with_file_name(src)
            };

            let extension = path.extension().and_then(|str| str.to_str()).unwrap_or("");
            if lang.is_none() {
                lang = match extension {
                    "r" => Some("R".to_string()),
                    _ => bail!("unknown extension: {}", extension),
                };
            }

            match fs::read_to_string(&path) {
                Ok(code) => code,
                Err(e) => bail!("could not read code: {}", e),
            }
        }
        None => function.body.try_get_as_string()?,
    };

    let mut code_html = HtmlElement::new("code");
    code_html.append_raw_text(&code);
    if let Some(lang) = lang {
        code_html.set_attr("class", format!("language-{}", lang).as_str());
    }

    match &function.body.form {
        FunctionBodyForm::Inline => Ok(Some(code_html)),
        FunctionBodyForm::Block => {
            let mut pre_html = HtmlElement::new("pre");
            pre_html.append(code_html);
            Ok(Some(pre_html))
        }
    }
}
