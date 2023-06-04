use anyhow::{bail, Result};

use crate::{
    evaluate_litedown_function, evaluate_with_ld2html_evaluator,
    html_evaluator::{
        litedown::Ld2HtmlEvaluator, preamble::preamble::Preamble,
        presentation::header::evaluate_header,
    },
    tree::function::LitedownFunction,
    utility::html::HtmlElement,
};

pub fn evaluate_presentation(
    evaluator: &Ld2HtmlEvaluator,
    preamble: &Preamble,
    function: &LitedownFunction,
) -> Result<(Vec<HtmlElement>, Vec<HtmlElement>)> {
    let mut head: Vec<HtmlElement> = Vec::new();

    // presentation.less
    head.push({
        let mut style = HtmlElement::new("style");
        style.set_attr("type", "text/less");
        style.append_raw_text(include_str!("./presentation.less"));
        style
    });

    // 大きさ設定
    head.push({
        //TODO よりよいサイズ指定方法を探す
        let mut style = HtmlElement::new("style");
        style.set_attr("type", "text/less");
        style.append_raw_text(&format!(
            r#"
            body {{
                @media print {{
                    width: {width};
                }}
            }}
            
            @page {{
                size: {width} {height};
                margin: 0;
                padding: 0;
                box-sizing: border-box;
                border-width: 0;
            }}

            .presentation {{
                & > .slide-wrapper {{
                    position: relative;
                    width: {width};
                    height: {height};

                    & > .slide {{
                        position: absolute;
                        width: calc({width} - 2 * {padding_horizontal});
                        min-height: calc({height} - 1 * {padding_vertical});
                        transform: translateX({padding_horizontal}) translateY({padding_vertical});
                    }}
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
    body.set_attr("class", "presentation");

    evaluate_litedown_function!(function;
        slide: (child_function) => {
            let mut slide_wrapper_html = HtmlElement::new("div");
            slide_wrapper_html.set_attr("class", "slide-wrapper");
            slide_wrapper_html.append(evaluate_slide(evaluator, child_function)?);
            body.append(slide_wrapper_html);
        }
        title: (child_function) => {
            let mut slide_wrapper_html = HtmlElement::new("div");
            slide_wrapper_html.set_attr("class", "slide-wrapper");
            slide_wrapper_html.append(evaluate_title(evaluator, child_function)?);
            body.append(slide_wrapper_html);
        }
    );

    let body = vec![body];

    Ok((head, body))
}

fn evaluate_slide(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<HtmlElement> {
    let mut slide_html = HtmlElement::new("div");
    slide_html.set_attr("class", "slide");

    evaluate_with_ld2html_evaluator!(function to slide_html with evaluator;
        function: {
            header: (function) => {
                slide_html.append(evaluate_header(evaluator, function)?);
            }
        }
    );
    Ok(slide_html)
}

fn evaluate_title(
    evaluator: &Ld2HtmlEvaluator,
    function: &LitedownFunction,
) -> Result<HtmlElement> {
    let mut title_html = HtmlElement::new("div");
    title_html.set_attr("class", "slide title");

    let mut subtitle_html = None;
    let mut author_html = None;

    evaluate_with_ld2html_evaluator!(function to title_html with evaluator;
        function: {
            subtitle: (function) => {
                if subtitle_html.is_some() {
                    bail!("'title' got multiple 'subtitle'");
                }
                let mut subtitle_html_0 = HtmlElement::new("div");
                subtitle_html_0.set_attr("class", "subtitle");
                evaluate_with_ld2html_evaluator!(function to subtitle_html_0 with evaluator);
                subtitle_html = Some(subtitle_html_0);
            }
            author: (function) => {
                if author_html.is_some() {
                    bail!("'title' got multiple 'author'");
                }
                let mut author_html_0 = HtmlElement::new("div");
                author_html_0.set_attr("class", "subtitle");
                evaluate_with_ld2html_evaluator!(function to author_html_0 with evaluator);
                author_html = Some(author_html_0);
            }
        }
    );

    if let Some(subtitle_html) = subtitle_html {
        title_html.append(subtitle_html);
    }
    if let Some(author_html) = author_html {
        title_html.append(author_html);
    }

    Ok(title_html)
}
