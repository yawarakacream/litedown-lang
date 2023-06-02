use std::mem::swap;

use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::environment::EnvironmentEvaluator,
    evaluator::{
        default::{
            decorators::{Separator, StrongText},
            document::title::Title,
            math::{DisplayMath, InlineMath},
        },
        litedown::LitedownEvaluator,
    },
    tree::{element::EnvironmentElement, parameter::stringify_number_parameter},
    utility::html::HtmlElement,
};

use crate::evaluator::default::{
    code::{CodeBlock, InlineCode},
    decorators::{BoldText, Link},
    figure::Figure,
    image::Image,
    list::List,
    minipages::MiniPages,
    section::Section,
};

use super::decorators::PageBreak;

static DEFAULT_STYLE: &str = include_str!("../default.less");

static DOCUMENT_STYLE: &str = include_str!("./document.less");

struct Size {
    width: String,
    height: String,
}

enum FontFamily {
    Serif,
    SansSerif,
}

enum Math {
    Katex,
    MathJax,
}

pub struct Document {
    size: Size,
    padding: Size,
    font_size: String,
    font_family: FontFamily,
    math: Option<Math>,
}

impl Document {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Document {
            size: Size {
                width: "210mm".to_string(),
                height: "297mm".to_string(),
            },
            font_size: "10.5pt".to_string(),
            font_family: FontFamily::SansSerif,
            padding: Size {
                width: "2em".to_string(),
                height: "1em".to_string(),
            },
            math: Some(Math::Katex),
        })
    }

    pub fn new_evaluator() -> LitedownEvaluator {
        let mut evaluator = LitedownEvaluator::new();

        evaluator.set_root_environment("document", Document::new());

        evaluator.set_environment("section", Section::new());
        evaluator.set_environment("list", List::new());
        evaluator.set_environment("code", CodeBlock::new());
        evaluator.set_environment("figure", Figure::new());
        evaluator.set_environment("minipages", MiniPages::new());
        evaluator.set_environment("math", DisplayMath::new());

        evaluator.set_function("link", Link::new());
        evaluator.set_function("pagebreak", PageBreak::new());
        evaluator.set_function("code", InlineCode::new());
        evaluator.set_function("math", InlineMath::new());
        evaluator.set_function("bold", BoldText::new());
        evaluator.set_function("strong", StrongText::new());
        evaluator.set_function("image", Image::new());
        evaluator.set_function("separator", Separator::new());

        evaluator
    }
}

impl EnvironmentEvaluator for Document {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        if let Some(string) = &element.parameters.get("size") {
            let string = string.try_into_string()?;
            let string = string.to_lowercase();
            let (size, orientation) = match string.find('-') {
                Some(hyphen) => (&string[..hyphen], Some(&string[(hyphen + 1)..])),
                None => (string.as_str(), None),
            };
            self.size = match size {
                "a4" => Size {
                    width: "210mm".to_string(),
                    height: "297mm".to_string(),
                },
                _ => bail!("Invalid size"),
            };
            if let Some(orientation) = orientation {
                match orientation {
                    "portrait" => {}
                    "landscape" => swap(&mut self.size.width, &mut self.size.height),
                    _ => bail!("Invalid size orientation: {}", orientation),
                }
            }
        }

        if let Some(font_size) = &element.parameters.get("font-size") {
            let (u, n) = font_size.try_into_number()?;
            self.font_size = stringify_number_parameter(u, n);
        }

        if let Some(font_family) = &element.parameters.get("font-family") {
            self.font_family = match font_family.try_into_str()? {
                "serif" => FontFamily::Serif,
                "sans-serif" => FontFamily::SansSerif,
                _ => bail!("Invalid font-family"),
            };
        }

        if let Some(string) = &element.parameters.get("padding") {
            let splitted = string.try_into_string()?.split(' ').collect::<Vec<_>>();
            self.padding = match splitted.len() {
                1 => Size {
                    width: splitted[0].to_string(),
                    height: splitted[0].to_string(),
                },
                2 => Size {
                    width: splitted[1].to_string(),
                    height: splitted[0].to_string(),
                },
                _ => bail!("Invalid padding"),
            }
        }

        if let Some(math) = &element.parameters.get("math") {
            self.math = match math.try_into_str()? {
                "katex" => Some(Math::Katex),
                "mathjax" => Some(Math::MathJax),
                "none" => None,
                _ => bail!("Illegal math"),
            }
        }

        let mut document = HtmlElement::new("div");
        document.set_attr("class", "document");

        eval_with_litedown!(
            element to document with lde;
            environment: {
                title: (child_environment) => {
                    let mut title = Title::new();
                    document.append(title.eval(lde, child_environment)?);
                }
            }
        );

        Ok(document)
    }

    fn get_heads(&self) -> Result<Vec<HtmlElement>> {
        let mut result = Vec::new();

        // main style
        let mut default_style = HtmlElement::new("style");
        default_style.set_attr("type", "text/less");
        default_style.append_raw_text(DEFAULT_STYLE);
        result.push(default_style);

        // document style
        let mut document_style = HtmlElement::new("style");
        document_style.set_attr("type", "text/less");
        document_style.append_raw_text(DOCUMENT_STYLE);
        result.push(document_style);

        //TODO よりよいサイズ指定方法を探す
        let mut main_style = HtmlElement::new("style");
        main_style.set_attr("type", "text/less");
        main_style.append_raw_text(&format!(
            r#"
            @page {{
                size: {width} {height};
                margin: {padding_height} 0;
                padding: 0;
                border-width: 0;
            }}

            #root {{
                font-size: {font_size};

                @media screen {{
                    width: calc({width} - 2 * {padding_width});
                    padding: {padding_height} {padding_width};
                }}

                @media print {{
                    width: calc({width} - 2 * {padding_width});
                    margin: 0 {padding_width};
                }}
            }}
            "#,
            width = self.size.width,
            height = self.size.height,
            font_size = self.font_size,
            padding_width = self.padding.width,
            padding_height = self.padding.height,
        ));
        result.push(main_style);

        // math
        if let Some(math) = &self.math {
            match math {
                Math::Katex => {
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
                    result.push(math_style);

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
                    result.push(math_script);

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
                    result.push(math_load_script);

                    let mut math_load_style = HtmlElement::new("style");
                    math_load_style.append_raw_text(&format!(
                        r#"
                        @font-face {{
                            font-family: litedown-math;
                            src: url("https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/fonts/KaTeX_Main-Regular.woff2") format("woff2");
                            unicode-range: U+0030-0039;
                            size-adjust: 121%;
                        }}
                        #root {{
                            font-family: litedown-math, {font_family};
                        }}
                        .katex .cjk_fallback {{
                            font-family: {font_family};
                            font-size: calc(100% / 1.21);
                        }}
                        "#,
                        font_family = match self.font_family {
                            FontFamily::Serif => "Georgia, 'Times New Roman', Times, serif",
                            FontFamily::SansSerif => "Arial, Helvetica, sans-serif",
                        }
                    ));
                    result.push(math_load_style);
                }
                Math::MathJax => {
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
                    result.push(math_prepare_script);

                    let mut math_script = HtmlElement::new("script");
                    math_script.set_attr("id", "MathJax-script");
                    math_script.set_attr("defer", "true");
                    math_script.set_attr(
                        "src",
                        "https://cdn.jsdelivr.net/npm/mathjax@3.0.1/es5/tex-mml-chtml.js",
                    );
                    math_script.set_attr("crossorigin", "anonymous");
                    result.push(math_script);

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
                        "#
                    );
                    result.push(math_load_script);

                    let mut math_load_style = HtmlElement::new("style");
                    math_load_style.append_raw_text(&format!(
                        r#"
                        @font-face {{
                            font-family: litedown-math;
                            src: url("https://cdn.jsdelivr.net/npm/mathjax@3.0.1/es5/output/chtml/fonts/woff-v2/MathJax_Main-Regular.woff") format("woff");
                            unicode-range: U+0030-0039;
                            size-adjust: 121%;
                        }}
                        #root {{
                            font-family: litedown-math, {font_family};
                        }}
                        mjx-container mjx-utext {{
                            font-family: {font_family} !important;
                        }}
                        "#,
                        font_family = match self.font_family {
                            FontFamily::Serif => "Georgia, 'Times New Roman', Times, serif",
                            FontFamily::SansSerif => "Arial, Helvetica, sans-serif",
                        }
                    ));
                    result.push(math_load_style);
                }
            }
        }

        Ok(result)
    }
}
