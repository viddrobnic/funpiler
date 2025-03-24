#[cfg(test)]
mod tests;

mod ast;

#[derive(Debug, PartialEq, Eq)]
struct Result<'a, T> {
    source: &'a str,
    value: T,
}

trait Parser<'a> {
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

struct Constant<T: Clone>(T);

impl<T: Clone> Constant<T> {
    fn new(value: T) -> Self {
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

struct Choice<P1, P2>(P1, P2);

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

struct ZeroOrMore<P>(P);

impl<'a, P> ZeroOrMore<P>
where
    P: Parser<'a>,
{
    fn new(parser: P) -> Self {
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

struct Bind<P, F> {
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

struct And<P1, P2>(P1, P2);

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

struct Maybe<P>(P);

impl<'a, P> Maybe<P>
where
    P: Parser<'a>,
{
    fn new(parser: P) -> Self {
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

fn whitespace(source: &'_ str) -> Option<Result<'_, ()>> {
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

fn ignored(source: &'_ str) -> Option<Result<'_, ()>> {
    ZeroOrMore::new(whitespace.or(comments))
        .map(|_| ())
        .parse(source)
}

struct TokenBase<'a> {
    token: &'a str,
    whitespace_end: bool,
}

impl<'a> TokenBase<'a> {
    fn new(token: &'a str, whitespace_end: bool) -> Self {
        Self {
            token,
            whitespace_end,
        }
    }
}

impl<'a> Parser<'a> for TokenBase<'a> {
    type Output = &'a str;

    fn parse(&self, source: &'a str) -> Option<Result<'a, Self::Output>> {
        if !source.starts_with(self.token) {
            return None;
        }

        if !self.whitespace_end {
            return Some(Result {
                source: &source[self.token.len()..],
                value: self.token,
            });
        }

        let Some((idx, ch)) = source[self.token.len()..].char_indices().next() else {
            return Some(Result {
                source: "",
                value: self.token,
            });
        };

        if ch.is_whitespace() {
            Some(Result {
                source: &source[(idx + ch.len_utf8())..],
                value: self.token,
            })
        } else {
            None
        }
    }
}

fn token(token: &str, whitespace_end: bool) -> impl Parser<'_, Output = &'_ str> {
    TokenBase::new(token, whitespace_end).bind(|tk| ignored.and(Constant::new(tk)))
}

fn function_t(source: &str) -> Option<Result<'_, &str>> {
    token("function", true).parse(source)
}

fn if_t(source: &str) -> Option<Result<'_, &str>> {
    token("if", true).parse(source)
}

fn else_t(source: &str) -> Option<Result<'_, &str>> {
    token("else", true).parse(source)
}

fn return_t(source: &str) -> Option<Result<'_, &str>> {
    token("return", true).parse(source)
}

fn var_t(source: &str) -> Option<Result<'_, &str>> {
    token("var", true).parse(source)
}

fn while_t(source: &str) -> Option<Result<'_, &str>> {
    token("while", true).parse(source)
}

fn comma_t(source: &str) -> Option<Result<'_, &str>> {
    token(",", false).parse(source)
}

fn semicolon_t(source: &str) -> Option<Result<'_, &str>> {
    token(";", false).parse(source)
}

fn left_paren_t(source: &str) -> Option<Result<'_, &str>> {
    token("(", false).parse(source)
}

fn right_paren_t(source: &str) -> Option<Result<'_, &str>> {
    token(")", false).parse(source)
}

fn left_brace_t(source: &str) -> Option<Result<'_, &str>> {
    token("{", false).parse(source)
}

fn right_brace_t(source: &str) -> Option<Result<'_, &str>> {
    token("}", false).parse(source)
}

fn number_base(source: &str) -> Option<Result<'_, i64>> {
    let mut end = 0;
    for (idx, ch) in source.char_indices() {
        if !ch.is_ascii_digit() {
            break;
        }

        end = idx + 1;
    }

    if end == 0 {
        None
    } else {
        Some(Result {
            value: source[0..end].parse().unwrap(),
            source: &source[end..],
        })
    }
}

fn number(source: &str) -> Option<Result<'_, i64>> {
    number_base
        .bind(|tk| ignored.and(Constant::new(tk)))
        .parse(source)
}

fn id_base(source: &str) -> Option<Result<'_, &str>> {
    let mut end = 0;
    for (idx, ch) in source.char_indices() {
        if idx == 0 && !ch.is_alphabetic() && ch != '_' {
            return None;
        }

        if !ch.is_alphanumeric() && ch != '_' {
            break;
        }

        end = idx + ch.len_utf8();
    }

    if end == 0 {
        None
    } else {
        Some(Result {
            value: &source[0..end],
            source: &source[end..],
        })
    }
}

fn id(source: &str) -> Option<Result<'_, &str>> {
    id_base
        .bind(|tk| ignored.and(Constant::new(tk)))
        .parse(source)
}

fn not_t(source: &str) -> Option<Result<'_, &str>> {
    token("!", false).parse(source)
}

fn equal_t(source: &str) -> Option<Result<'_, &str>> {
    token("==", false).parse(source)
}

fn not_equal_t(source: &str) -> Option<Result<'_, &str>> {
    token("!=", false).parse(source)
}

fn plus_t(source: &str) -> Option<Result<'_, &str>> {
    token("+", false).parse(source)
}

fn minus_t(source: &str) -> Option<Result<'_, &str>> {
    token("-", false).parse(source)
}

fn star_t(source: &str) -> Option<Result<'_, &str>> {
    token("*", false).parse(source)
}

fn slash_t(source: &str) -> Option<Result<'_, &str>> {
    token("/", false).parse(source)
}

fn assign_t(source: &str) -> Option<Result<'_, &str>> {
    token("=", false).parse(source)
}
