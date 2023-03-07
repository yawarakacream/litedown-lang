use anyhow::Result;

use crate::{litedown_element::PassageContentFunction, utility::html::HtmlElement};

use super::litedown::LitedownEvaluator;

pub trait FunctionEvaluator {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>>;
}
