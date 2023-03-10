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
        let mut max_level = 0;
        for node in &self.nodes {
            max_level = max_level.max(node.0);
        }

        let mut levels_to_below = vec![vec![false; max_level + 1]; self.nodes.len()];
        {
            let i = self.nodes.len() - 1;
            levels_to_below[i][self.nodes[i].0] = true;
        }
        for i in (1..(self.nodes.len() - 1)).rev() {
            let node = &self.nodes[i];
            for l in 1..(node.0) {
                levels_to_below[i][l] = levels_to_below[i + 1][l];
            }
            levels_to_below[i][node.0] = true;
        }

        let mut buffer = String::new();
        for (i, (level, content)) in (&self.nodes).iter().enumerate() {
            if 0 < i {
                for l in 1..*level {
                    if levels_to_below[i][l] {
                        buffer.push_str("│  ");
                    } else {
                        buffer.push_str("   ");
                    }
                }
                if i + 1 < self.nodes.len() && levels_to_below[i + 1][*level] {
                    buffer.push_str("├─ ");
                } else {
                    buffer.push_str("└─ ");
                }
            }

            buffer.push_str(content);
            buffer.push('\n');
        }
        buffer
    }
}

pub trait ToTreeString {
    fn write_tree_string(&self, builder: &mut TreeStringBuilder, level: usize);

    fn to_tree_string(&self) -> String {
        let mut builder = TreeStringBuilder::new();
        self.write_tree_string(&mut builder, 0);
        builder.build()
    }
}
