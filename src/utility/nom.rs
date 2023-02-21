use nom::{
    branch::alt,
    bytes::complete::{is_not, take_while1},
    character::complete::{alpha1, char, digit1, line_ending, space0},
    combinator::{eof, map, opt, recognize},
    error::{ParseError, VerboseError},
    sequence::{delimited, pair, terminated, tuple},
    AsChar, Finish, IResult, IResult as NomIResult,
};

pub type IResultV<I, O> = NomIResult<I, O, VerboseError<I>>;

pub fn print_nom<'a, O: std::fmt::Debug, F: nom::Parser<&'a str, O, VerboseError<&'a str>>>(
    input: &'a str,
    mut parser: F,
) -> Option<O> {
    eprintln!("\x1B[42minput: {:?}\x1B[0m", input);
    let result = parser.parse(input).finish();
    let mut ret = None;
    match result {
        Ok((_, a)) => {
            eprintln!("\x1B[42mOK!\x1B[0m");
            eprintln!("{a:?}");
            ret = Some(a);
        }
        Err(err) => {
            eprintln!("\x1B[41mError!\x1B[0m");
            eprintln!("{}", nom::error::convert_error(input, err.clone()));
        }
    };
    eprintln!();
    ret
}

pub fn is_whitespace(c: char) -> bool {
    c == ' ' || c == '\t'
}

pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(space0, inner, space0)
}

pub fn namestr<'a, E: ParseError<&'a str>>(str: &'a str) -> IResult<&'a str, String, E> {
    let (str, a) = alpha1(str)?;
    let (str, b) = opt(take_while1(|x: char| {
        x.is_alphabetic() || x.is_dec_digit() || x == '-'
    }))(str)?;
    let mut result = a.to_string();
    if let Some(b) = b {
        result.push_str(b);
    }
    Ok((str, result))
}

pub fn parse_f64<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, f64, E> {
    map(
        recognize(tuple((
            opt(alt((char('+'), char('-')))),
            alt((
                map(tuple((digit1, opt(pair(char('.'), opt(digit1))))), |_| ()),
                map(tuple((char('.'), digit1)), |_| ()),
            )),
        ))),
        |x: &str| x.parse().unwrap(),
    )(input)
}

pub fn pass_blank_line(str: &str) -> IResultV<&str, usize> {
    map(terminated(space0, line_ending), |s: &str| s.len())(str)
}

pub fn pass_blank_lines0(str: &str) -> IResultV<&str, usize> {
    let mut str = str;
    let mut i = 0;
    loop {
        match pass_blank_line(str) {
            Ok(tmp) => {
                str = tmp.0;
                i += 1;
            }
            Err(_) => break,
        }
    }
    return Ok((str, i));
}

pub fn any_to_line_ending(str: &str) -> IResultV<&str, &str> {
    // map(
    //     many_till(
    //         anychar,
    //         alt((eof::<&str, VerboseError<&str>>, alt((line_ending, eof)))),
    //     ),
    //     |(v, _)| (v.into_iter().collect()),
    // )(str)
    let (str, res) = is_not("\r\n")(str)?;
    let (str, _) = alt((line_ending, eof))(str)?;
    Ok((str, res))
}

pub fn count_indent(str: &str) -> IResultV<&str, usize> {
    map(space0, |str: &str| str.len())(str)
}

#[macro_export]
macro_rules! verror {
    // return Err(nom::Err::Error(VerboseError {
    //     errors: vec![(str, VerboseErrorKind::Context("aaa"))],
    // }));
    // return context("duplicate parameter", fail)(str);
    ($caller:expr, $input:expr, $cause:expr) => {
        nom::Err::Error(nom::error::VerboseError {
            errors: vec![(
                $input,
                nom::error::VerboseErrorKind::Context(concat!($caller, " (", $cause, ")")),
            )],
        })
    };
}
