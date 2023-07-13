use std::{collections::HashMap, path::PathBuf};

use anyhow::{bail, Context, Result};

use crate::{
    html_evaluator::{
        document::document::evaluate_document,
        preamble::{math::Math, preamble::evaluate_preamble},
        presentation::presentation::evaluate_presentation,
    },
    tree::{function::LitedownFunction, litedown::LitedownAst},
    utility::html::{Html, HtmlElement},
};

use super::{
    common::{
        code::evaluate_code,
        decorators::{evaluate_attention, evaluate_divider, evaluate_link, evaluate_strong},
        figure::evaluate_figure,
        grid::evaluate_grid,
        image::evaluate_image,
        list::evaluate_list,
        math::evaluate_math,
    },
    document::pagebreak::evaluate_pagebreak,
    preamble::{font::FontFamily, preamble::Preamble, theme::Theme},
    presentation::absolute_block::evaluate_absolute_block,
};

enum ContentMode {
    Document,
    Presentation,
}

impl ContentMode {
    fn get_function_evaluators(&self) -> HashMap<String, Ld2HtmlFunctionEvaluator> {
        let mut function_evaluators: HashMap<String, Ld2HtmlFunctionEvaluator> = HashMap::new();
        function_evaluators.insert("math".to_string(), evaluate_math);
        function_evaluators.insert("strong".to_string(), evaluate_strong);
        function_evaluators.insert("attention".to_string(), evaluate_attention);
        function_evaluators.insert("list".to_string(), evaluate_list);
        function_evaluators.insert("figure".to_string(), evaluate_figure);
        function_evaluators.insert("image".to_string(), evaluate_image);
        function_evaluators.insert("code".to_string(), evaluate_code);
        function_evaluators.insert("divider".to_string(), evaluate_divider);
        function_evaluators.insert("link".to_string(), evaluate_link);
        function_evaluators.insert("grid".to_string(), evaluate_grid);

        match &self {
            ContentMode::Document => {
                function_evaluators.insert("pagebreak".to_string(), evaluate_pagebreak);
            }
            ContentMode::Presentation => {
                function_evaluators.insert("absolute".to_string(), evaluate_absolute_block);
            }
        }

        function_evaluators
    }

    fn get_evaluator(
        &self,
    ) -> fn(
        evaluator: &Ld2HtmlEvaluator,
        preamble: &Preamble,
        function: &LitedownFunction,
    ) -> Result<(Vec<HtmlElement>, Vec<HtmlElement>)> {
        match &self {
            ContentMode::Document => evaluate_document,
            ContentMode::Presentation => evaluate_presentation,
        }
    }
}

pub struct Ld2HtmlInput {
    pub ast: LitedownAst,
    pub source_path: Option<PathBuf>,
}

type Ld2HtmlFunctionEvaluator =
    fn(evaluator: &Ld2HtmlEvaluator, function: &LitedownFunction) -> Result<Option<HtmlElement>>;

pub fn evaluate_litedown_to_html(input: Ld2HtmlInput) -> Result<Html> {
    let mut evaluator = Ld2HtmlEvaluator::new(input);
    evaluator.evaluate()
}

pub struct Ld2HtmlEvaluator {
    input: Ld2HtmlInput,
    function_evaluators: HashMap<String, Ld2HtmlFunctionEvaluator>,
}

impl Ld2HtmlEvaluator {
    pub fn new(input: Ld2HtmlInput) -> Ld2HtmlEvaluator {
        Ld2HtmlEvaluator {
            input,
            function_evaluators: HashMap::new(),
        }
    }

    pub fn get_source_path(&self) -> Option<&PathBuf> {
        match &self.input.source_path {
            Some(source_path) => Some(source_path),
            None => None,
        }
    }

    pub fn evaluate(&mut self) -> Result<Html> {
        let preamble_function = self
            .input
            .ast
            .body
            .get(0)
            .context("preamble not found: first function must be 'preamble'")?;
        let preamble = match preamble_function.name.as_str() {
            "preamble" => {
                evaluate_preamble(preamble_function).context("failed to evaluate preamble")?
            }
            _ => bail!("invalid preamble found: first function must be 'preamble'"),
        };
        println!("preamble: {:?}", preamble);

        let content_function =
            self.input.ast.body.get(1).context(
                "content not found: second function must be 'document' or 'presentation'",
            )?;
        let content_mode = match content_function.name.as_str() {
            "document" => ContentMode::Document,
            "presentation" => ContentMode::Presentation,
            _ => {
                bail!("invalid content found: second function must be 'document' or 'presentation'")
            }
        };
        self.function_evaluators
            .extend(content_mode.get_function_evaluators());
        let (content_head, content_body) =
            content_mode.get_evaluator()(&self, &preamble, content_function)?;

        let mut html = Html::new();

        for element in Self::get_main_head(&preamble) {
            html.append_head(element);
        }
        for element in content_head {
            html.append_head(element);
        }

        html.append_body({
            let mut root = HtmlElement::new("div");
            root.set_attr("id", "root");
            for element in content_body {
                root.append(element);
            }
            root
        });

        Ok(html)
    }

    fn get_main_head(preamble: &Preamble) -> Vec<HtmlElement> {
        let mut result = Vec::new();

        // common.less
        result.push({
            let mut element = HtmlElement::new("style");
            element.set_attr("type", "text/less");
            element.append_raw_text(include_str!("./common/common.less"));
            element
        });

        // theme variable
        result.push({
            let mut element = HtmlElement::new("style");
            element.set_attr("type", "text/less");
            element.append_raw_text(&format!(
                r#"
                html {{
                    --strong-color: {strong_color};
                }}
                "#,
                strong_color = match preamble.theme {
                    Theme::Default => "royalblue",
                    Theme::Paper => "black",
                }
            ));
            element
        });

        // font
        result.push({
            let mut element = HtmlElement::new("style");
            element.append_raw_text(&format!(
                r#"
                html {{
                    --main-font-family: {font_family};
                    --main-font-size: {font_size};
    
                    font-family: var(--main-font-family);
                    font-size: var(--main-font-size);
                }}
                "#,
                font_size = preamble.font.size,
                font_family = match preamble.font.family {
                    FontFamily::Serif => "Georgia, 'Times New Roman', Times, serif",
                    FontFamily::SansSerif => "Arial, Helvetica, sans-serif",
                }
            ));
            element
        });

        // less.js
        result.push({
            let mut element = HtmlElement::new("script");
            element.set_attr("src", "https://cdn.jsdelivr.net/npm/less");
            element.set_attr("defer", "true");
            element
        });

        // highlight.js
        result.push({
            let mut highlight_style = HtmlElement::new_void("link");
            highlight_style.set_attr("rel", "stylesheet");
            highlight_style.set_attr(
                "href",
                "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/styles/default.min.css",
            );
            highlight_style
        });
        result.push({
            let mut highlight_script = HtmlElement::new("script");
            highlight_script.set_attr(
                "src",
                "https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.7.0/highlight.min.js",
            );
            highlight_script.set_attr("onload", "hljs.highlightAll()");
            highlight_script
        });

        // math
        if let Some(math) = &preamble.math {
            match math {
                Math::Katex => {
                    result.push({
                        let mut math_style = HtmlElement::new_void("link");
                        math_style.set_attr("rel", "stylesheet");
                        math_style.set_attr(
                            "href",
                            "https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.css",
                        );
                        math_style.set_attr(
                            "integrity",
                            "sha384-vKruj+a13U8yHIkAyGgK1J3ArTLzrFGBbBc0tDp4ad/EyewESeXE/Iv67Aj8gKZ0",
                        );
                        math_style.set_attr("crossorigin", "anonymous");
                        math_style
                    });

                    result.push({
                        let mut math_script = HtmlElement::new("script");
                        math_script.set_attr(
                            "src",
                            "https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.js",
                        );
                        math_script.set_attr(
                            "integrity",
                            "sha384-PwRUT/YqbnEjkZO0zZxNqcxACrXe+j766U2amXcgMg5457rve2Y7I6ZJSm2A0mS4",
                        );
                        math_script.set_attr("crossorigin", "anonymous");
                        math_script
                    });

                    result.push({
                        let mut math_load_script = HtmlElement::new("script");
                        math_load_script.set_attr("defer", "true");
                        math_load_script.append_raw_text(
                            r#"
                            window.addEventListener("DOMContentLoaded", () => {
                                const macros = {};
                                Array.from(document.getElementsByClassName("display-math")).forEach((el) => {
                                    console.log(el.innerText);
                                    katex.render(el.innerText, el, {
                                        throwOnError: false,
                                        displayMode: true,
                                        macros,
                                    });
                                });
                                Array.from(document.getElementsByClassName("inline-math")).forEach((el) => {
                                    katex.render(el.innerText, el, {
                                        throwOnError: false,
                                        displayMode: false,
                                        macros,
                                    });
                                });
                            });
                            "#,
                        );
                        math_load_script
                    });

                    result.push({
                        let mut math_load_style = HtmlElement::new("style");
                        math_load_style.append_raw_text(&format!(
                            r#"
                            @font-face {{
                                font-family: litedown-math;
                                src: url("https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/fonts/KaTeX_Main-Regular.woff2") format("woff2");
                                unicode-range: U+0030-0039;
                                size-adjust: 112%;
                            }}
                            body {{
                                font-family: litedown-math, var(--main-font-family);
                            }}
                            .katex .cjk_fallback {{
                                font-family: var(--main-font-family);
                                font-size: calc(100% / 1.21);
                            }}
                            "#,
                        ));
                        math_load_style
                    });
                }
                Math::Mathjax => {
                    result.push({
                        let mut math_prepare_script = HtmlElement::new("script");
                        math_prepare_script.append_raw_text(
                            r#"
                            window.mathJaxTriggers = {
                                inline: ["\\mathjax(", "\\mathjax)"],
                                display: ["\\mathjax[", "\\mathjax]"],
                            };
            
                            window.MathJax = {
                                tex: {
                                    inlineMath: [window.mathJaxTriggers.inline],
                                    displayMath: [window.mathJaxTriggers.display],
                                },
                            };
                            "#,
                        );
                        math_prepare_script
                    });

                    result.push({
                        let mut math_script = HtmlElement::new("script");
                        math_script.set_attr("id", "MathJax-script");
                        math_script.set_attr("defer", "true");
                        math_script.set_attr(
                            "src",
                            "https://cdn.jsdelivr.net/npm/mathjax@3.0.1/es5/tex-mml-chtml.js",
                        );
                        math_script.set_attr("crossorigin", "anonymous");
                        math_script
                    });

                    result.push({
                        let mut math_load_script = HtmlElement::new("script");
                        math_load_script.append_raw_text(
                            r#"
                            window.addEventListener("DOMContentLoaded", () => {
                                Array.from(document.getElementsByClassName("inline-math")).forEach((el) => {
                                    el.innerHTML = window.mathJaxTriggers.inline.join(el.innerHTML);
                                });
            
                                Array.from(document.getElementsByClassName("display-math")).forEach((el) => {
                                    el.innerHTML = window.mathJaxTriggers.display.join(el.innerHTML);
                                });
            
                                MathJax.typeset();
                            });
                            "#,
                        );
                        math_load_script
                    });

                    result.push({
                        let mut math_load_style = HtmlElement::new("style");
                        math_load_style.append_raw_text(&format!(
                            r#"
                            @font-face {{
                                font-family: litedown-math;
                                src: url("https://cdn.jsdelivr.net/npm/mathjax@3.0.1/es5/output/chtml/fonts/woff-v2/MathJax_Main-Regular.woff") format("woff");
                                unicode-range: U+0030-0039;
                                size-adjust: 112%;
                            }}
                            body {{
                                font-family: litedown-math, var(--main-font-family);
                            }}
                            mjx-container mjx-utext {{
                                font-family: var(--main-font-family) !important;
                            }}
                            "#,
                        ));
                        math_load_style
                    });
                }
            }
        }

        result
    }

    pub(crate) fn evaluate_main_function(
        &self,
        function: &LitedownFunction,
    ) -> Result<Option<HtmlElement>> {
        let name = &function.name;
        match name.as_str() {
            _ => match self.function_evaluators.get(name) {
                Some(ev) => ev(self, function),
                None => bail!("unknown function: {}", name),
            },
        }
    }
}
