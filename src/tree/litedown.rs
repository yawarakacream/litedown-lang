use super::function::LitedownFunction;

#[derive(Debug)]
pub struct LitedownAst {
    pub body: Vec<LitedownFunction>,
}
