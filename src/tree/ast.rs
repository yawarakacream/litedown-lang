use std::path::PathBuf;

use crate::utility::tree_string_builder::{ToTreeString, TreeStringBuilder};

use super::element::EnvironmentElement;

#[derive(Debug)]
pub struct LitedownAst {
    pub source_path: Option<PathBuf>,
    pub roots: Vec<EnvironmentElement>,
}

impl ToTreeString for LitedownAst {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, "LitedownAst");
        for root in &self.roots {
            root.write_tree_string(builder, level + 1);
        }
    }
}
