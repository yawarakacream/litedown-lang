use serde::Serialize;

use super::function::LitedownFunction;

#[derive(Clone, Debug, Serialize)]
pub struct LitedownAst {
    pub body: Vec<LitedownFunction>,
}
