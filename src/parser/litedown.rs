use anyhow::{bail, Context, Result};

use crate::{
    parser::function::parse_function,
    tree::litedown::LitedownAst,
    utility::{
        indented_string::IndentedStringIterator,
        tree_string_builder::{ToTreeString, TreeStringBuilder},
    },
};

pub fn parse_litedown(source_code: &str) -> Result<LitedownAst> {
    let mut iter = IndentedStringIterator::new(&source_code);
    let ast = iter
        .parse(move |iter| {
            let mut body = Vec::new();
            while !iter.is_finished() {
                iter.pass_blank_lines();
                body.push(parse_function(iter)?);
            }
            Ok(LitedownAst { body })
        })
        .with_context(|| {
            let indices = iter.last_consumed_indices().unwrap();
            format!(
                "Failed to parse litedown: line {}, column {}",
                indices.line_index + 1,
                indices.indent_level + indices.char_index + 1
            )
        })?;
    Ok(ast)
}

pub(super) fn parse_name(iter: &mut IndentedStringIterator) -> Result<String> {
    iter.parse(|iter| {
        let mut ret = String::new();
        while let Some(c) = iter.next_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                ret.push(c);
            } else {
                iter.back_char().unwrap();
                break;
            }
        }
        if ret.len() == 0 {
            bail!("empty name");
        }
        Ok(ret)
    })
}

impl ToTreeString for LitedownAst {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize) {
        builder.add_node(level, "LitedownAst");
        builder.add_node(level + 1, "Body");
        for function in &self.body {
            function.write_tree_string(builder, level + 2);
        }
    }
}
