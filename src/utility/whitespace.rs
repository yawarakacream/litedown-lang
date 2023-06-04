use std::slice::Iter;

pub enum Whitespace {
    SingleByteSpace,
    Tab,
}
impl Whitespace {
    fn to_char(&self) -> char {
        match self {
            Self::SingleByteSpace => ' ',
            Self::Tab => '\t',
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::SingleByteSpace => 1,
            Self::Tab => 8,
        }
    }

    fn iter() -> Iter<'static, Whitespace> {
        static ALL: [Whitespace; 2] = [Whitespace::SingleByteSpace, Whitespace::Tab];
        ALL.iter()
    }
}

pub fn is_blank(str: &str) -> bool {
    str.trim().is_empty()
}

pub fn is_whitespace(c: char) -> bool {
    Whitespace::iter().any(|ws| ws.to_char() == c)
}

// pub fn is_all_whitespaces(str: &str) -> bool {
//     str.chars().all(Self::is_whitespace)
// }

pub fn len_as_whitespace(c: char) -> Option<usize> {
    Whitespace::iter()
        .find(|ws| ws.to_char() == c)
        .map(|ws| ws.len())
}
