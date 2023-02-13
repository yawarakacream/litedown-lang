pub struct TreeStringBuilder {
    nodes: Vec<(usize, String)>,
}

impl TreeStringBuilder {
    pub fn new() -> TreeStringBuilder {
        TreeStringBuilder { nodes: vec![] }
    }

    pub fn add_node<T: ToString>(&mut self, level: usize, content: T) {
        self.nodes.push((level, content.to_string()));
    }

    pub fn build(&self) -> String {
        let mut buffer = String::new();
        for (level, content) in &self.nodes {
            Self::write_indent(&mut buffer, *level);
            buffer.push_str(content);
            buffer.push('\n');
        }
        buffer
    }

    fn write_indent(buffer: &mut String, level: usize) {
        if 0 < level {
            for _ in 0..(level - 1) {
                buffer.push_str("   ");
            }
            buffer.push_str("├─ ");
        }
    }
}
