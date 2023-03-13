use anyhow::Result;

use crate::{tree::element::PassageContentFunction, utility::html::HtmlElement};

use super::litedown::LitedownEvaluator;

pub trait FunctionEvaluator {
    fn eval(
        &mut self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<Option<HtmlElement>>;
}
