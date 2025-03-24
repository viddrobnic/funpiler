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

#[test]
fn token_empty() {
    assert_eq!(TokenBase::new("a", false).parse(""), None);
    assert_eq!(TokenBase::new("a", true).parse(""), None);
}

#[test]
fn token_breakable() {
    assert_eq!(
        token(",", false).parse(","),
        Some(Result {
            source: "",
            value: ",",
        })
    );

    assert_eq!(
        token(",", false).parse(",foo"),
        Some(Result {
            source: "foo",
            value: ",",
        })
    );

    assert_eq!(
        token(",", false).parse(", \n /* comment */ foo"),
        Some(Result {
            source: "foo",
            value: ",",
        })
    );

    assert_eq!(token(",", false).parse("foo,"), None);
}

#[test]
fn token_unbreakable() {
    assert_eq!(
        token(",", true).parse(","),
        Some(Result {
            source: "",
            value: ",",
        })
    );

    assert_eq!(token(",", true).parse(",foo"), None);

    assert_eq!(
        token(",", true).parse(", \n /* comment */ foo"),
        Some(Result {
            source: "foo",
            value: ",",
        })
    );

    assert_eq!(token(",", true).parse("foo,"), None);
}

#[test]
fn number_empty() {
    assert_eq!(number.parse(""), None);
}

#[test]
fn number_valid() {
    assert_eq!(
        number.parse("123"),
        Some(Result {
            source: "",
            value: 123,
        })
    );

    assert_eq!(
        number.parse("123   "),
        Some(Result {
            source: "",
            value: 123,
        })
    );
}

#[test]
fn number_invalid() {
    assert_eq!(number.parse("foo"), None);
}

#[test]
fn id_empty() {
    assert_eq!(id.parse(""), None);
}

#[test]
fn id_valid() {
    assert_eq!(
        id.parse("foo"),
        Some(Result {
            source: "",
            value: "foo",
        })
    );

    assert_eq!(
        id.parse("_foo_123 \n test"),
        Some(Result {
            source: "test",
            value: "_foo_123",
        })
    );

    assert_eq!(
        id.parse("foo,bar"),
        Some(Result {
            source: ",bar",
            value: "foo",
        })
    );
}

#[test]
fn id_invalid() {
    assert_eq!(id.parse("1foo"), None);
}
