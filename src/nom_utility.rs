use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alpha1, char, digit1, multispace0},
    combinator::{map, opt, recognize},
    error::{ParseError, VerboseError},
    sequence::{delimited, pair, tuple},
    AsChar, Finish, IResult, IResult as NomIResult,
};

pub type IResultV<I, O> = NomIResult<I, O, VerboseError<I>>;

pub fn print_nom<'a, O: std::fmt::Debug, F: nom::Parser<&'a str, O, VerboseError<&'a str>>>(
    input: &'a str,
    mut parser: F,
) {
    eprintln!("\x1B[42minput: {:?}\x1B[0m", input);
    let result = parser.parse(input).finish();
    match result {
        Ok((_, a)) => {
            eprintln!("\x1B[42mOK!\x1B[0m");
            eprintln!("{:?}", a);
        }
        Err(err) => {
            eprintln!("\x1B[41mError!\x1B[0m");
            eprintln!("{}", nom::error::convert_error(input, err));
        }
    }
    eprintln!();
}

pub fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(
    inner: F,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    F: Fn(&'a str) -> IResult<&'a str, O, E>,
{
    delimited(multispace0, inner, multispace0)
}

pub fn namestr<'a, E: ParseError<&'a str>>(str: &'a str) -> IResult<&'a str, String, E> {
    let (str, a) = alpha1(str)?;
    let (str, b) = opt(take_while1(|x: char| {
        x.is_alphabetic() || x.is_dec_digit() || x == '_'
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

#[macro_export]
macro_rules! verror {
    // return Err(nom::Err::Error(VerboseError {
    //     errors: vec![(str, VerboseErrorKind::Context("aaa"))],
    // }));
    // return context("duplicate parameter", fail)(str);
    ($caller:expr, $input:expr, $cause:expr) => {
        nom::Err::Error(VerboseError {
            errors: vec![(
                $input,
                VerboseErrorKind::Context(concat!($caller, " (", $cause, ")")),
            )],
        })
    };
}
