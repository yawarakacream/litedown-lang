use std::mem::swap;

use anyhow::{bail, Result};

use crate::{
    eval_with_litedown,
    evaluator::environment::EnvironmentEvaluator,
    evaluator::litedown::LitedownEvaluator,
    litedown_element::{stringify_number_parameter, CommandParameterValue, EnvironmentElement},
    utility::html::HtmlElement,
};

use super::title::Title;

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
}

impl EnvironmentEvaluator for Document {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        if let Some(size) = &element.parameters.get("size") {
            match size {
                CommandParameterValue::String(string) => {
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
                        _ => bail!("Illegal size"),
                    };
                    if let Some(orientation) = orientation {
                        match orientation {
                            "portrait" => {}
                            "landscape" => swap(&mut self.size.width, &mut self.size.height),
                            _ => bail!("Illegal size orientation: {}", orientation),
                        }
                    }
                }
                _ => bail!("Illegal size"),
            }
        }

        if let Some(font_size) = &element.parameters.get("font-size") {
            match font_size {
                CommandParameterValue::Number(u, n) => {
                    self.font_size = stringify_number_parameter(u, n);
                }
                _ => bail!("Illegal font-size"),
            }
        }

        if let Some(font_family) = &element.parameters.get("font-family") {
            match font_family {
                CommandParameterValue::String(string) => {
                    let string = string.as_str();
                    self.font_family = match string {
                        "serif" => FontFamily::Serif,
                        "sans-serif" => FontFamily::SansSerif,
                        _ => bail!("Illegal font-family"),
                    }
                }
                _ => bail!("Illegal font-family"),
            }
        }

        if let Some(padding) = &element.parameters.get("padding") {
            match padding {
                CommandParameterValue::String(string) => {
                    let splitted = string.split(' ').collect::<Vec<_>>();
                    self.padding = match splitted.len() {
                        1 => Size {
                            width: splitted[0].to_string(),
                            height: splitted[0].to_string(),
                        },
                        2 => Size {
                            width: splitted[1].to_string(),
                            height: splitted[0].to_string(),
                        },
                        _ => bail!("Illegal padding"),
                    }
                }
                _ => bail!("Illegal padding"),
            }
        }

        if let Some(math) = &element.parameters.get("math") {
            match math {
                CommandParameterValue::String(string) => {
                    let string = string.to_lowercase();
                    self.math = match string.as_str() {
                        "katex" => Some(Math::Katex),
                        "mathjax" => Some(Math::MathJax),
                        "none" => None,
                        _ => bail!("Illegal math"),
                    }
                }
                _ => bail!("Illegal math"),
            }
        }

        let mut document = HtmlElement::new("div");
        document.set_attr("class", "document");

        eval_with_litedown!(
            element to document with lde
            @title@ (child_environment) {
                let mut title = Title::new();
                document.append(title.eval(lde, child_environment)?);
            }
        );

        Ok(document)
    }

    fn get_heads(&self) -> Result<Vec<HtmlElement>> {
        let mut result = Vec::new();

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

            .document {{
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
                            Array.from(document.getElementsByClassName("inline-math")).forEach((el) => {
                                katex.render(el.innerHTML, el, {
                                    throwOnError: false
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
                        .document {{
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
                        .document {{
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
