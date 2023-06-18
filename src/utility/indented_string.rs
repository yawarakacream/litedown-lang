use anyhow::{bail, Result};

use super::whitespace;

#[derive(Debug, Clone)]
pub struct IndentedStringLine {
    is_blank: bool,
    max_level: usize,
    chars: Vec<char>,
}
impl IndentedStringLine {
    pub fn is_blank(&self) -> bool {
        self.is_blank
    }

    pub fn max_level(&self) -> usize {
        self.max_level
    }

    fn get_char(&self, level: usize, index: usize) -> Option<&char> {
        if self.max_level < level {
            return None;
        }
        let mut level = level as isize;
        let mut index = index;
        for &c in self.chars.iter() {
            if level == 0 {
                break;
            }
            if level < 0 {
                return None;
            }
            if let Some(len) = whitespace::len_as_whitespace(c) {
                level -= len as isize;
                index += 1;
            } else {
                break;
            }
        }
        self.chars.get(index)
    }
}

#[derive(Debug, Clone)]
pub struct IndentedStringIteratorIndices {
    pub indent_level: usize,
    pub line_index: usize,
    pub char_index: usize,
}

#[derive(Debug, Clone)]
pub struct IndentedStringIterator {
    lines: Vec<IndentedStringLine>,
    current_indices: IndentedStringIteratorIndices,
    last_consumed_indices: Option<IndentedStringIteratorIndices>,
}

impl IndentedStringIterator {
    pub fn new(text: &str) -> Self {
        let mut vec = Vec::new();
        for line in text.split("\n") {
            let mut level = 0;
            let mut i = 0;
            for e in line.chars() {
                if let Some(len) = whitespace::len_as_whitespace(e) {
                    level += len;
                } else {
                    break;
                }
                i += 1;
            }
            let is_blank = i == line.len();
            let max_level = if is_blank { 0 } else { level };
            let chars = if is_blank {
                vec![]
            } else {
                line.chars().collect()
            };
            vec.push(IndentedStringLine {
                is_blank,
                max_level,
                chars,
            });
        }
        IndentedStringIterator {
            lines: vec,
            current_indices: IndentedStringIteratorIndices {
                indent_level: 0,
                line_index: 0,
                char_index: 0,
            },
            last_consumed_indices: None,
        }
    }

    pub fn current_indices(&self) -> IndentedStringIteratorIndices {
        self.current_indices.clone()
    }

    pub fn last_consumed_indices(&self) -> Option<IndentedStringIteratorIndices> {
        self.last_consumed_indices.clone()
    }

    pub fn is_finished(&mut self) -> bool {
        self.current_indices.line_index == self.lines.len() && !self.has_next_char()
    }

    pub fn peek_line(&self) -> Option<&IndentedStringLine> {
        if let Some(line) = self.lines.get(self.current_indices.line_index) {
            if line.is_blank || self.current_indices.indent_level <= line.max_level {
                Some(line)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn peek_line_max_indent(&self) -> Option<usize> {
        self.peek_line().map(|line| line.max_level)
    }

    pub fn next_line(&mut self) -> bool {
        if self.peek_line().is_none() {
            return false;
        }

        if self.has_next_char() {
            panic!("this line still has character");
        }

        if self.current_indices.line_index == self.lines.len() {
            false
        } else {
            self.current_indices.char_index = 0;
            self.current_indices.line_index += 1;
            if let Some(line) = self.peek_line() {
                line.is_blank || self.current_indices.indent_level <= line.max_level
            } else {
                false
            }
        }
    }

    pub fn pass_blank_lines(&mut self) -> usize {
        assert_eq!(
            self.current_indices.char_index, 0,
            "Must start line from first character"
        );
        let mut ret = 0;
        while let Some(line) = self.peek_line() {
            if line.is_blank {
                self.current_indices.line_index += 1;
                ret += 1;
            } else {
                break;
            }
        }
        return ret;
    }

    pub fn set_line_indent(&mut self, new_level: usize) {
        assert_eq!(
            self.current_indices.char_index, 0,
            "Cannot set indent during scanning this line"
        );
        self.current_indices.indent_level = new_level;
    }

    pub fn peek_char(&mut self) -> Option<char> {
        let line = self.peek_line()?;
        let char = *line.get_char(
            self.current_indices.indent_level,
            self.current_indices.char_index,
        )?;
        self.last_consumed_indices = Some(self.current_indices.clone());
        Some(char)
    }

    fn peek_char_as(&mut self, char: char) -> Result<char> {
        match self.peek_char() {
            Some(ret) => {
                if ret == char {
                    Ok(ret)
                } else {
                    bail!("expected '{char}', found '{ret}'")
                }
            }
            None => bail!("EOL while scanning, expected '{char}'"),
        }
    }

    pub fn next_char(&mut self) -> Option<char> {
        let ret = self.peek_char()?;
        self.current_indices.char_index += 1;
        Some(ret)
    }

    pub fn next_char_as(&mut self, char: char) -> Result<char> {
        let ret = self.peek_char_as(char)?;
        self.current_indices.char_index += 1;
        Ok(ret)
    }

    pub fn has_next_char(&mut self) -> bool {
        self.peek_char().is_some()
    }

    pub fn back_char(&mut self) -> Result<char> {
        if 0 < self.current_indices.char_index {
            self.current_indices.char_index -= 1;
            Ok(self.peek_char().unwrap())
        } else {
            bail!("Cannot go back anymore")
        }
    }

    pub fn next_str_as<'a>(&mut self, str: &'a str) -> Result<&'a str> {
        self.parse(|iter| {
            for c in str.chars() {
                iter.next_char_as(c)?;
            }
            Ok(str)
        })
    }

    pub fn collect_until_line_ending(&mut self) -> String {
        let mut ret = String::new();
        while let Some(c) = self.next_char() {
            ret.push(c)
        }
        ret
    }

    pub fn pass_whitespaces(&mut self) -> usize {
        let mut i = 0;
        while let Some(c) = self.peek_char() {
            if let Some(l) = whitespace::len_as_whitespace(c) {
                i += l;
                self.current_indices.char_index += 1;
            } else {
                break;
            }
        }
        i
    }

    pub fn parse<T, F>(&mut self, parser: F) -> Result<T>
    where
        F: FnOnce(&mut IndentedStringIterator) -> Result<T>,
    {
        let indices = self.current_indices.clone();
        let ret = parser(self);
        if ret.is_err() {
            self.current_indices = indices;
        }
        ret
    }
}

// pub struct IndentedStringWriter {
//     written: String,
//     line_buffer: String,
//     current_level: usize,
//     level_size: usize,
// }

// impl IndentedStringWriter {
//     pub fn new() -> IndentedStringWriter {
//         IndentedStringWriter {
//             written: String::new(),
//             line_buffer: String::new(),
//             current_level: 0,
//             level_size: 2,
//         }
//     }

//     pub fn get_level(&self) -> usize {
//         self.current_level
//     }

//     pub fn set_level(&mut self, level: usize) {
//         self.written.push_str(&self.line_buffer.as_str());
//         self.line_buffer.clear();
//         self.current_level = level;
//     }

//     pub fn write(&mut self, str: &str) {
//         self.line_buffer.push_str(str);
//     }

//     pub fn writeln(&mut self, str: &str) {
//         for _ in 0..(self.current_level * self.level_size) {
//             self.written.push(' ');
//         }
//         self.write(str);
//         self.written.push_str(&self.line_buffer.as_str());
//         self.line_buffer.clear();
//         self.written.push('\n');
//     }

//     pub fn build(&mut self) -> String {
//         let mut written = self.written.clone();
//         written.push_str(&self.line_buffer.as_str());
//         self.line_buffer.clear();
//         written
//     }
// }
