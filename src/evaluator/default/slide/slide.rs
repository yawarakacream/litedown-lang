use anyhow::{bail, Result};

use crate::{
    evaluator::environment::EnvironmentEvaluator,
    evaluator::{
        default::{
            decorators::{Separator, StrongText},
            math::{DisplayMath, InlineMath},
            slide::normal_page::NormalPage,
        },
        litedown::LitedownEvaluator,
    },
    tree::{
        element::{EnvironmentElement, LitedownElement},
        parameter::stringify_number_parameter,
    },
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

use super::title::Title;

static DEFAULT_STYLE: &str = include_str!("../default.less");

static SLIDE_STYLE: &str = include_str!("./slide.less");

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

pub struct Slide {
    size: Size,
    padding: Size,
    font_size: String,
    font_family: FontFamily,
    math: Option<Math>,
}

impl Slide {
    pub fn new() -> Box<dyn EnvironmentEvaluator> {
        Box::new(Slide {
            size: Size {
                width: "33.867cm".to_string(),
                height: "19.05cm".to_string(),
            },
            font_size: "18pt".to_string(),
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

        evaluator.set_root_environment("slide", Slide::new());

        evaluator.set_environment("section", Section::new());
        evaluator.set_environment("list", List::new());
        evaluator.set_environment("code", CodeBlock::new());
        evaluator.set_environment("figure", Figure::new());
        evaluator.set_environment("minipages", MiniPages::new());
        evaluator.set_environment("math", DisplayMath::new());

        evaluator.set_function("link", Link::new());
        evaluator.set_function("code", InlineCode::new());
        evaluator.set_function("math", InlineMath::new());
        evaluator.set_function("bold", BoldText::new());
        evaluator.set_function("strong", StrongText::new());
        evaluator.set_function("image", Image::new());
        evaluator.set_function("separator", Separator::new());

        evaluator
    }
}

impl EnvironmentEvaluator for Slide {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement> {
        if let Some(size) = &element.parameters.get("size") {
            let size = size.try_into_str()?;
            self.size = match size {
                "16:9" => Size {
                    width: "33.867cm".to_string(),
                    height: "19.05cm".to_string(),
                },
                "a4" => Size {
                    width: "210mm".to_string(),
                    height: "297mm".to_string(),
                },
                _ => bail!("Invalid size"),
            };
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

        let mut slide = HtmlElement::new("div");
        slide.set_attr("class", "slide");

        for child in &element.children {
            match child {
                LitedownElement::Environment(child_environment) => {
                    match child_environment.name.as_str() {
                        "title" => {
                            let mut title = Title::new();
                            slide.append(title.eval(lde, child_environment)?);
                        }
                        "page" => {
                            let mut page = NormalPage::new();
                            slide.append(page.eval(lde, child_environment)?);
                        }
                        _ => bail!("Unknown environment: {}", &child_environment.name),
                    }
                }
                LitedownElement::Passage(_) => bail!("Cannot write passage"),
            }
        }

        Ok(slide)
    }

    fn get_heads(&self) -> Result<Vec<HtmlElement>> {
        let mut result = Vec::new();

        // main style
        let mut default_style = HtmlElement::new("style");
        default_style.set_attr("type", "text/less");
        default_style.append_raw_text(DEFAULT_STYLE);
        result.push(default_style);

        // slide style
        let mut slide_style = HtmlElement::new("style");
        slide_style.set_attr("type", "text/less");
        slide_style.append_raw_text(SLIDE_STYLE);
        result.push(slide_style);

        //TODO よりよいサイズ指定方法を探す
        let mut main_style = HtmlElement::new("style");
        main_style.set_attr("type", "text/less");
        main_style.append_raw_text(&format!(
            r#"
            body {{
                @media print {{
                    width: {width};
                }}
            }}

            @page {{
                size: {width} {height};
            }}

            #root {{
                font-size: {font_size};
            }}

            .slide {{
                & > .page-container {{
                    position: relative;
                    width: {width};
                    height: {height};

                    & > .page {{
                        position: absolute;
                        width: calc({width} - 2 * {padding_width});
                        min-height: calc({height} - 1 * {padding_height});
                        transform: translateX({padding_width}) translateY({padding_height});
                    }}
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
