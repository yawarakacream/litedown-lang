use anyhow::{bail, Result};

use crate::{
    deconstruct_required_arguments, evaluate_litedown_function, evaluate_with_ld2html_evaluator,
    html_evaluator::litedown::Ld2HtmlEvaluator, tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_grid(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<Option<HtmlElement>> {
    let mut container_html = HtmlElement::new("div");
    container_html.set_attr("class", "grid");

    let mut rows_style = "grid-auto-rows: 1fr".to_string();
    let mut columns_style = "grid-auto-columns: 1fr".to_string();
    let mut gap_style = None;

    evaluate_litedown_function!(function;
        rows: (child_function) => {
            let mut tmp = String::new();
            for i in 0..(child_function.arguments.len()) {
                if 0 < i {
                    tmp.push_str(" ");
                }
                match child_function.arguments.get_by_index(i) {
                    Some(arg) => {
                        tmp.push_str(&arg.try_into_string()?);
                    }
                    None => break,
                }
            }
            rows_style = format!("grid-template-rows: {}", tmp);
        }
        columns: (child_function) => {
            let mut tmp = String::new();
            for i in 0..(child_function.arguments.len()) {
                if 0 < i {
                    tmp.push_str(" ");
                }
                match child_function.arguments.get_by_index(i) {
                    Some(arg) => {
                        tmp.push_str(&arg.try_into_string()?);
                    }
                    None => break,
                }
            }
            columns_style = format!("grid-template-columns: {}", tmp);
        }
        gap: (child_function) => {
            let gap = match child_function.arguments.len() {
                1 => {
                    deconstruct_required_arguments!((gap) from child_function);
                    gap
                }
                _ => bail!("Invalid argument"),
            };
            gap_style = Some(format!("gap: {}", gap.try_into_string()?));
        }
        item: (child_function) => {
            // zero-indexed
            let (row_start, row_end, column_start, column_end) = match child_function.arguments.len() {
                2 => {
                    deconstruct_required_arguments!((row, column) from child_function);
                    let row = row.try_into_bare_unsigned_integer()?;
                    let column = column.try_into_bare_unsigned_integer()?;
                    (row, row + 1, column, column + 1)
                }
                4 => {
                    deconstruct_required_arguments!((row_start, row_end, column_start, column_end) from child_function);
                    let row_start = row_start.try_into_bare_unsigned_integer()?;
                    let column_start = column_start.try_into_bare_unsigned_integer()?;
                    let row_end = row_end.try_into_bare_unsigned_integer()?;
                    let column_end = column_end.try_into_bare_unsigned_integer()?;
                    (row_start, row_end, column_start, column_end)
                }
                _ => bail!("Invalid argument"),
            };

            let mut item_html = HtmlElement::new("div");
            item_html.set_attr("class", "item");
            item_html.set_attr("style", &format!(
                "grid-row: {} / {}; grid-column: {} / {}",
                row_start + 1, row_end + 1, column_start + 1, column_end + 1
            ));
            evaluate_with_ld2html_evaluator!(child_function to item_html with evaluator);
            container_html.append(item_html);
        }
    );

    let mut style = String::new();
    style.push_str(&rows_style);
    style.push(';');
    style.push_str(&columns_style);
    style.push(';');
    if let Some(gap_style) = &gap_style {
        style.push_str(&gap_style);
        style.push(';');
    }
    container_html.set_attr("style", &style);

    Ok(Some(container_html))
}
