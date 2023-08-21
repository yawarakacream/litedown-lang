use serde::Serialize;

use super::function::LitedownFunction;

#[derive(Debug, Serialize)]
pub struct LitedownAst {
    pub body: Vec<LitedownFunction>,
}
