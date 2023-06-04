use anyhow::Result;

use crate::{
    evaluate_with_ld2html_evaluator,
    html_evaluator::{
        document::title::evaluate_title, litedown::Ld2HtmlEvaluator, preamble::preamble::Preamble,
    },
    tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_document(
    evaluator: &Ld2HtmlEvaluator,
    preamble: &Preamble,
    function: &LitedownFunction,
) -> Result<(Vec<HtmlElement>, Vec<HtmlElement>)> {
    let mut head: Vec<HtmlElement> = Vec::new();

    // document.less
    head.push({
        let mut style = HtmlElement::new("style");
        style.set_attr("type", "text/less");
        style.append_raw_text(include_str!("./document.less"));
        style
    });

    // 大きさ設定
    // TODO: よりよいサイズ指定方法を探す
    head.push({
        let mut style = HtmlElement::new("style");
        style.set_attr("type", "text/less");
        style.append_raw_text(&format!(
            r#"
            @page {{
                size: {width} {height};
                margin: {padding_vertical} 0;
                padding: 0;
                border-width: 0;
            }}

            #root {{
                @media screen {{
                    width: calc({width} - 2 * {padding_horizontal});
                    min-height: calc({height} - 2 * {padding_vertical});
                    padding: {padding_vertical} {padding_horizontal};
                }}

                @media print {{
                    width: calc({width} - 2 * {padding_horizontal});
                    margin: 0 {padding_horizontal};
                }}
            }}
            "#,
            width = preamble.page_size.width,
            height = preamble.page_size.height,
            padding_horizontal = preamble.page_padding.horizontal,
            padding_vertical = preamble.page_padding.vertical,
        ));
        style
    });

    let mut body = HtmlElement::new("div");
    body.set_attr("class", "document");

    let mut section_index = 1;

    evaluate_with_ld2html_evaluator!(function to body with evaluator;
        function: {
            title: (child_function) => {
                body.append(evaluate_title(evaluator, child_function)?);
            }
            section: (child_function) => {
                let mut section_html = HtmlElement::new("section");
                section_html.append({
                    let mut header_html = HtmlElement::new("div");
                    header_html.set_attr("class", "header");
                    header_html.append_text(&format!("{}.", section_index));
                    section_index += 1;
                    header_html
                });
                evaluate_with_ld2html_evaluator!(child_function to section_html with evaluator);
                body.append(section_html);
            }
        }
    );

    let body = vec![body];

    Ok((head, body))
}
