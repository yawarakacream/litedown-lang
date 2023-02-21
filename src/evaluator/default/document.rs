use crate::{
    attrs,
    evaluator::environment::EnvironmentEvaluator,
    evaluator::{environment::EnvironmentEvaluatorComponents, litedown::LitedownEvaluator},
    litedown_element::{CommandParameterValue, EnvironmentElement},
};

use super::title::Title;

enum FontFamily {
    Serif,
    SansSerif,
}

enum Math {
    Katex,
    MathJax,
}

pub struct Document {
    font_size: String,
    font_family: FontFamily,
    math: Option<Math>,
}

impl Document {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Document {
            font_size: "11pt".to_string(),
            font_family: FontFamily::SansSerif,
            math: Some(Math::Katex),
        })
    }
}

impl EnvironmentEvaluatorComponents for Document {
    fn open_environment(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        match &element.parameters.get("font-size") {
            Some(font_size) => match font_size {
                CommandParameterValue::Number(unit, number) => {
                    self.font_size = number.to_string();
                    match unit {
                        Some(unit) => self.font_size.push_str(&unit),
                        None => {}
                    }
                }
                _ => {}
            },
            None => {}
        }

        match &element.parameters.get("font-family") {
            Some(font_size) => match font_size {
                CommandParameterValue::String(string) => {
                    let string = string.as_str();
                    self.font_family = match string {
                        "serif" => FontFamily::Serif,
                        "sans-serif" => FontFamily::SansSerif,
                        _ => return Err("Illegal font-family".to_string()),
                    }
                }
                _ => {}
            },
            None => {}
        }

        match &element.parameters.get("math") {
            Some(math) => match math {
                CommandParameterValue::String(string) => {
                    let string = string.to_lowercase();
                    let string = string.as_str();
                    self.math = match string {
                        "katex" => Some(Math::Katex),
                        "mathjax" => Some(Math::MathJax),
                        _ => return Err("Illegal math".to_string()),
                    }
                }
                _ => {}
            },
            None => self.math = None,
        }

        lde.writer
            .open_element("div", attrs! {"class" => "document"})
    }

    fn close_environment(&mut self, lde: &mut LitedownEvaluator) -> Result<(), String> {
        lde.writer.open_element("style", attrs! {})?;
        lde.writer.write_raw_inner(&format!(
            r#"
            .document {{
                font-size: {font_size};
            }}
            "#,
            font_size = self.font_size,
        ))?;
        lde.writer.close_element("style")?;

        match &self.math {
            Some(math) => match math {
                Math::Katex => {
                    lde.writer.add_void_element(
                        "link",
                        attrs! {
                            "rel" => "stylesheet",
                            "href" => "https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.css",
                            "integrity" => "sha384-vKruj+a13U8yHIkAyGgK1J3ArTLzrFGBbBc0tDp4ad/EyewESeXE/Iv67Aj8gKZ0",
                            "crossorigin" => "anonymous"
                        },
                    )?;
                    lde.writer.add_inline_element(
                        "script",
                        attrs! {
                            "src" => "https://cdn.jsdelivr.net/npm/katex@0.16.4/dist/katex.min.js",
                            "integrity" => "sha384-PwRUT/YqbnEjkZO0zZxNqcxACrXe+j766U2amXcgMg5457rve2Y7I6ZJSm2A0mS4",
                            "crossorigin" => "anonymous"
                        },
                        ""
                    )?;

                    lde.writer.open_element("script", attrs! {})?;
                    lde.writer.write_raw_inner(
                        r#"
                        Array.from(document.getElementsByClassName("inline-math")).forEach((el) => {
                            katex.render(el.innerHTML, el, {
                                throwOnError: false
                            });
                        });
                        "#,
                    )?;
                    lde.writer.close_element("script")?;

                    lde.writer.open_element("style", attrs! {})?;
                    lde.writer.write_raw_inner(
                        &format!(
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
                        ),
                    )?;
                    lde.writer.close_element("style")?;
                }
                Math::MathJax => {
                    lde.writer.open_element("script", attrs! {})?;
                    lde.writer.write_raw_inner(
                        r#"
                        const triggers = {
                            inline: ["\\mathjax(", "\\mathjax)"],
                            display: ["\\mathjax[", "\\mathjax]"],
                        };

                        Array.from(document.getElementsByClassName("inline-math")).forEach((el) => {
                            el.innerHTML = triggers.inline.join(el.innerHTML);
                        });

                        Array.from(document.getElementsByClassName("display-math")).forEach((el) => {
                            el.innerHTML = triggers.display.join(el.innerHTML);
                        });

                        window.MathJax = {
                            tex: {
                                inlineMath: [triggers.inline],
                                displayMath: [triggers.display],
                            },
                        };
                        "#
                    )?;
                    lde.writer.close_element("script")?;

                    lde.writer.add_inline_element(
                        "script",
                        attrs! {
                            "id" => "MathJax-script",
                            "src" => "https://cdn.jsdelivr.net/npm/mathjax@3.0.1/es5/tex-mml-chtml.js"
                        },
                        ""
                    )?;

                    lde.writer.open_element("style", attrs! {})?;
                    lde.writer.write_raw_inner(&format!(
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
                    ))?;
                    lde.writer.close_element("style")?;
                }
            },
            None => {}
        }

        lde.writer.close_element("div")
    }

    fn eval_child_environment(
        &self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<(), String> {
        match element.name.as_str() {
            "title" => {
                let mut title = Title::new();
                title.eval(lde, element)
            }
            _ => lde.eval_environment(element),
        }
    }
}
