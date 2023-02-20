use crate::litedown_element::PassageContentFunction;

use super::litedown::LitedownEvaluator;

pub trait FunctionEvaluator {
    fn eval(
        &self,
        lde: &mut LitedownEvaluator,
        content: &PassageContentFunction,
    ) -> Result<(), String>;
}
