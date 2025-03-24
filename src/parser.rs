use std::fmt::Debug;

#[derive(Debug, PartialEq, Eq)]
pub struct Result<'a, T> {
    pub source: &'a str,
    pub value: T,
}

pub trait Parser<'a> {
    type Output;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>>;

    fn or<P>(self, other: P) -> Choice<Self, P>
    where
        Self: Sized,
        P: Parser<'a, Output = Self::Output>,
    {
        Choice(self, other)
    }

    fn bind<U, F, O>(self, function: F) -> Bind<Self, F>
    where
        Self: Sized,
        O: Parser<'a, Output = U>,
        F: Fn(Self::Output) -> O,
    {
        Bind {
            parser: self,
            function,
        }
    }

    fn and<U, P>(self, other: P) -> And<Self, P>
    where
        Self: Sized,
        P: Parser<'a, Output = U>,
    {
        And(self, other)
    }

    fn map<U, F>(self, function: F) -> impl Parser<'a, Output = U>
    where
        Self: Sized,
        U: Clone,
        F: Fn(Self::Output) -> U + 'a,
    {
        self.bind(move |val| Constant::new(function(val)))
    }

    #[allow(clippy::result_unit_err)]
    fn parse_to_completion(&self, source: &'a str) -> std::result::Result<Self::Output, ()> {
        match self.parse(source) {
            None => Err(()),
            Some(Result {
                source: "",
                value: _,
            }) => Err(()),
            Some(Result { source: _, value }) => Ok(value),
        }
    }
}

impl<'a, F, T> Parser<'a> for F
where
    F: Fn(&'a str) -> Option<Result<'a, T>>,
{
    type Output = T;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        self(source)
    }
}

pub struct Constant<T: Clone>(T);

impl<T: Clone> Constant<T> {
    pub fn new(value: T) -> Self {
        Constant(value)
    }
}

impl<'a, T> Parser<'a> for Constant<T>
where
    T: Clone,
{
    type Output = T;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        Some(Result {
            source,
            value: self.0.clone(),
        })
    }
}

pub struct Choice<P1, P2>(P1, P2);

impl<'a, T, P1, P2> Parser<'a> for Choice<P1, P2>
where
    P1: Parser<'a, Output = T>,
    P2: Parser<'a, Output = T>,
{
    type Output = T;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        let res = self.0.parse(source);
        if res.is_some() {
            res
        } else {
            self.1.parse(source)
        }
    }
}

pub struct ZeroOrMore<P>(P);

impl<'a, P> ZeroOrMore<P>
where
    P: Parser<'a>,
{
    pub fn new(parser: P) -> Self {
        ZeroOrMore(parser)
    }
}

impl<'a, T, P> Parser<'a> for ZeroOrMore<P>
where
    P: Parser<'a, Output = T>,
{
    type Output = Vec<T>;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        let mut result = Vec::new();
        let mut remaining = source;

        while let Some(res) = self.0.parse(remaining) {
            result.push(res.value);
            remaining = res.source;
        }

        Some(Result {
            source: remaining,
            value: result,
        })
    }
}

pub struct Bind<P, F> {
    parser: P,
    function: F,
}

impl<'a, P, O, F, T, U> Parser<'a> for Bind<P, F>
where
    P: Parser<'a, Output = T>,
    O: Parser<'a, Output = U>,
    F: Fn(T) -> O,
{
    type Output = U;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        let res = self.parser.parse(source)?;
        let p = (self.function)(res.value);
        p.parse(res.source)
    }
}

pub struct And<P1, P2>(P1, P2);

impl<'a, T, U, P1, P2> Parser<'a> for And<P1, P2>
where
    P1: Parser<'a, Output = T>,
    P2: Parser<'a, Output = U>,
{
    type Output = U;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        let res = self.0.parse(source)?;
        self.1.parse(res.source)
    }
}

pub struct Maybe<P>(P);

impl<'a, P> Maybe<P>
where
    P: Parser<'a>,
{
    pub fn new(parser: P) -> Self {
        Maybe(parser)
    }
}

impl<'a, T, P> Parser<'a> for Maybe<P>
where
    P: Parser<'a, Output = T>,
{
    type Output = Option<T>;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        let res = self.0.parse(source);
        if let Some(res) = res {
            Some(Result {
                source: res.source,
                value: Some(res.value),
            })
        } else {
            Some(Result {
                source,
                value: None,
            })
        }
    }
}

pub fn whitespace(source: &'_ str) -> Option<Result<'_, ()>> {
    if source.is_empty() {
        return None;
    }

    let mut ends_at = None;
    for (idx, ch) in source.char_indices() {
        if !ch.is_whitespace() {
            ends_at = Some(idx);
            break;
        }
    }

    match ends_at {
        None => Some(Result {
            source: "",
            value: (),
        }),
        Some(0) => None,
        Some(idx) => Some(Result {
            source: &source[idx..],
            value: (),
        }),
    }
}

fn single_line_comment(source: &'_ str) -> Option<Result<'_, ()>> {
    if !source.starts_with("//") {
        return None;
    }

    for (idx, ch) in source.char_indices().skip(2) {
        if ch == '\n' {
            return Some(Result {
                source: &source[(idx + 1)..],
                value: (),
            });
        }
    }

    Some(Result {
        source: "",
        value: (),
    })
}

fn multi_line_comment(source: &'_ str) -> Option<Result<'_, ()>> {
    if !source.starts_with("/*") {
        return None;
    }

    for (idx, ch) in source.char_indices().skip(2) {
        if ch == '*' && source.get(idx + 1..idx + 2) == Some("/") {
            return Some(Result {
                source: &source[(idx + 2)..],
                value: (),
            });
        }
    }

    None
}

fn comments(source: &'_ str) -> Option<Result<'_, ()>> {
    single_line_comment.or(multi_line_comment).parse(source)
}

pub fn ignored(source: &'_ str) -> Option<Result<'_, ()>> {
    ZeroOrMore::new(whitespace.or(comments))
        .map(|_| ())
        .parse(source)
    // whitespace.or(comments).parse(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whitespace_empty() {
        assert_eq!(whitespace.parse(""), None)
    }

    #[test]
    fn whitespace_single_space() {
        assert_eq!(
            whitespace.parse(" "),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn whitespace_multiple_spaces() {
        assert_eq!(
            whitespace.parse("   \t\n\t  \n"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn whitespace_no_space() {
        assert_eq!(whitespace.parse("no space!"), None);
    }

    #[test]
    fn single_line_comment_empty() {
        assert_eq!(single_line_comment.parse(""), None);
    }

    #[test]
    fn single_line_comment_single_line() {
        assert_eq!(
            single_line_comment.parse("// single line comment"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn single_line_comment_multiple_lines() {
        assert_eq!(
            single_line_comment.parse("// single line comment\nsomething else"),
            Some(Result {
                source: "something else",
                value: ()
            })
        );
    }

    #[test]
    fn single_line_comment_no_comment() {
        assert_eq!(single_line_comment.parse("no comment"), None);
    }

    #[test]
    fn multi_line_comment_empty() {
        assert_eq!(multi_line_comment.parse(""), None);
    }

    #[test]
    fn multi_line_comment_single_line() {
        assert_eq!(
            multi_line_comment.parse("/* multi line comment */"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn multi_line_comment_multiple_lines() {
        assert_eq!(
            multi_line_comment.parse("/* multi line comment\nsomething else */"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn multi_line_comment_no_comment() {
        assert_eq!(multi_line_comment.parse("no comment"), None);
    }

    #[test]
    fn multi_line_comment_no_end() {
        assert_eq!(multi_line_comment.parse("/* multi line comment"), None);
    }

    #[test]
    fn ignored_empty() {
        assert_eq!(
            ignored.parse(""),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn ignored_whitespace() {
        assert_eq!(
            ignored.parse(" \t\n"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn ignored_comments() {
        assert_eq!(
            ignored("// single line comment\n/* multi line comment */"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }

    #[test]
    fn ignored_some() {
        assert_eq!(
            ignored(" \t\n  // some comment"),
            Some(Result {
                source: "",
                value: ()
            })
        );
    }
}
