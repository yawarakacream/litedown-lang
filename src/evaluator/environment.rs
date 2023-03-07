use anyhow::Result;

use crate::{litedown_element::EnvironmentElement, utility::html::HtmlElement};

use super::litedown::LitedownEvaluator;

pub trait EnvironmentEvaluator {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        element: &EnvironmentElement,
    ) -> Result<HtmlElement>;

    fn get_head(&self) -> Result<Vec<HtmlElement>> {
        Ok(Vec::new())
    }
}

#[macro_export]
macro_rules! eval_with_litedown {
    ($element:ident to $root:ident with $lde:ident $(@$env:ident@ ($child_environment:ident) $envblock:block)*) => {
        for child in &$element.children {
            match child {
                crate::litedown_element::Element::Environment(child_environment) => {
                    match child_environment.name.as_str() {
                        $(
                            stringify!($env) => {
                                let $child_environment = child_environment;
                                $envblock;
                            }
                        )*
                        _ => {
                            $root.append($lde.eval_environment(child_environment)?);
                        }
                    }
                }
                crate::litedown_element::Element::Passage(
                    crate::litedown_element::PassageElement(contents),
                ) => {
                    let mut passage = HtmlElement::new("p");
                    for content in contents {
                        match content {
                            crate::litedown_element::PassageContent::Text(content) => {
                                passage.append_text(&content.0);
                            }
                            crate::litedown_element::PassageContent::Function(content) => {
                                if let Some(el) = $lde.eval_function(content)? {
                                    passage.append(el);
                                }
                            }
                        }
                    }
                    $root.append(passage);
                }
            }
        }
    };
}
