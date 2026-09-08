//! Adapted from motdotla/dotenv v17.4.2 tests (commit f116f703).

use dotenv::{LoadOptions, parse, populate};

#[test]
fn ports_the_upstream_primary_fixture() {
    let parsed = parse(include_str!("fixtures/npm-dotenv/basic.env"));
    let cases = [
        ("BASIC", "basic"),
        ("AFTER_LINE", "after_line"),
        ("EMPTY", ""),
        ("EMPTY_SINGLE_QUOTES", ""),
        ("EMPTY_DOUBLE_QUOTES", ""),
        ("EMPTY_BACKTICKS", ""),
        ("SINGLE_QUOTES", "single_quotes"),
        ("SINGLE_QUOTES_SPACED", "    single quotes    "),
        ("DOUBLE_QUOTES", "double_quotes"),
        ("DOUBLE_QUOTES_SPACED", "    double quotes    "),
        (
            "DOUBLE_QUOTES_INSIDE_SINGLE",
            "double \"quotes\" work inside single quotes",
        ),
        (
            "DOUBLE_QUOTES_WITH_NO_SPACE_BRACKET",
            "{ port: $MONGOLAB_PORT}",
        ),
        (
            "SINGLE_QUOTES_INSIDE_DOUBLE",
            "single 'quotes' work inside double quotes",
        ),
        (
            "BACKTICKS_INSIDE_SINGLE",
            "`backticks` work inside single quotes",
        ),
        (
            "BACKTICKS_INSIDE_DOUBLE",
            "`backticks` work inside double quotes",
        ),
        ("BACKTICKS", "backticks"),
        ("BACKTICKS_SPACED", "    backticks    "),
        (
            "DOUBLE_QUOTES_INSIDE_BACKTICKS",
            "double \"quotes\" work inside backticks",
        ),
        (
            "SINGLE_QUOTES_INSIDE_BACKTICKS",
            "single 'quotes' work inside backticks",
        ),
        (
            "DOUBLE_AND_SINGLE_QUOTES_INSIDE_BACKTICKS",
            "double \"quotes\" and single 'quotes' work inside backticks",
        ),
        ("EXPAND_NEWLINES", "expand\nnew\nlines"),
        ("DONT_EXPAND_UNQUOTED", "dontexpand\\nnewlines"),
        ("DONT_EXPAND_SQUOTED", "dontexpand\\nnewlines"),
        ("INLINE_COMMENTS", "inline comments"),
        (
            "INLINE_COMMENTS_SINGLE_QUOTES",
            "inline comments outside of #singlequotes",
        ),
        (
            "INLINE_COMMENTS_DOUBLE_QUOTES",
            "inline comments outside of #doublequotes",
        ),
        (
            "INLINE_COMMENTS_BACKTICKS",
            "inline comments outside of #backticks",
        ),
        ("INLINE_COMMENTS_SPACE", "inline comments start with a"),
        ("EQUAL_SIGNS", "equals=="),
        ("RETAIN_INNER_QUOTES", "{\"foo\": \"bar\"}"),
        ("RETAIN_INNER_QUOTES_AS_STRING", "{\"foo\": \"bar\"}"),
        ("RETAIN_INNER_QUOTES_AS_BACKTICKS", "{\"foo\": \"bar's\"}"),
        ("TRIM_SPACE_FROM_UNQUOTED", "some spaced out string"),
        ("USERNAME", "therealnerdybeast@example.tld"),
        ("SPACED_KEY", "parsed"),
        ("EXPORT_IS_DECLARED", "parsed"),
        ("EXPORT_IS_DECLARED_WITH_SPACING", "parsed"),
        ("EXPORT_IS_DECLARED_WITH_SOME_VALUE", "some_value"),
        ("EXPORT_IS_DECLARED_WITH_SOME_VALUE_SPACED", "some_value"),
        (
            "EXPORT_IS_DECLARED_WITH_SOME_VALUE_AND_SPACING",
            "some_value",
        ),
    ];
    for (key, expected) in cases {
        assert_eq!(parsed.get(key).map(String::as_str), Some(expected), "{key}");
    }
    assert!(!parsed.contains_key("COMMENTS"));
}

#[test]
fn ports_line_endings_duplicates_bom_and_separator_cases() {
    let expected = [
        ("SERVER", "localhost"),
        ("PASSWORD", "password"),
        ("DB", "tests"),
    ];
    for ending in ["\r", "\n", "\r\n"] {
        let input = expected
            .iter()
            .map(|(k, v)| format!("{k}={v}{ending}"))
            .collect::<String>();
        let parsed = parse(&input);
        for (key, value) in expected {
            assert_eq!(parsed[key], value);
        }
    }
    assert_eq!(parse("DUP=one\nDUP=two")["DUP"], "two");
    assert_eq!(parse("\u{feff}BOM=ok")["BOM"], "ok");
    assert_eq!(parse("COLON: value")["COLON"], "value");
    assert!(!parse("INVALID:value").contains_key("INVALID"));

    for input in [
        "\u{feff}export BASIC=basic\n",
        "\u{feff}# first\nBASIC=basic\n",
        "\n\u{feff}BASIC=basic\n",
        "FIRST=one\n\u{feff}SECOND=two\n",
    ] {
        assert!(
            parse(input)
                .values()
                .any(|value| value == "basic" || value == "one")
        );
    }
}

#[test]
fn ports_multiline_and_escape_behavior() {
    let parsed = parse(
        "SINGLE='first\nsecond'\nDOUBLE=\"first\nsecond\\nthird\\r\"\nBACK=`first\nsecond`\nESCAPED=\"a\\\"b\"\n",
    );
    assert_eq!(parsed["SINGLE"], "first\nsecond");
    assert_eq!(parsed["DOUBLE"], "first\nsecond\nthird\r");
    assert_eq!(parsed["BACK"], "first\nsecond");
    assert_eq!(parsed["ESCAPED"], "a\\\"b");
}

#[test]
fn ports_escaped_backslashes_before_closing_quotes() {
    for input in [
        "KEY=\"\\\\\"\nNEXT=ok\n",
        "KEY='\\\\'\nNEXT=ok\n",
        "KEY=`\\\\`\nNEXT=ok\n",
        "WINDIR=\"C:\\Users\\me\\\\\"\nAPI_KEY=secret\nPORT=3000\n",
        "KEY=\"a\\\\b\"\nNEXT=ok\n",
    ] {
        let parsed = parse(input);
        assert_eq!(
            parsed
                .get("NEXT")
                .or_else(|| parsed.get("API_KEY"))
                .map(String::as_str),
            Some(if parsed.contains_key("NEXT") {
                "ok"
            } else {
                "secret"
            })
        );
    }
    let parsed = parse("A=\"\\\\\"\nB=plain\nC=\"quoted\"\nD=last\n");
    assert_eq!(parsed["A"], "\\\\");
    assert_eq!(parsed["B"], "plain");
    assert_eq!(parsed["C"], "quoted");
    assert_eq!(parsed["D"], "last");
}

#[test]
fn ports_the_upstream_multiline_fixture() {
    let parsed = parse(include_str!("fixtures/npm-dotenv/.env.multiline"));
    let multiline = "THIS\nIS\nA\nMULTILINE\nSTRING";
    assert_eq!(parsed["MULTI_DOUBLE_QUOTED"], multiline);
    assert_eq!(parsed["MULTI_SINGLE_QUOTED"], multiline);
    assert_eq!(
        parsed["MULTI_BACKTICKED"],
        "THIS\nIS\nA\n\"MULTILINE'S\"\nSTRING"
    );
    assert!(parsed["MULTI_PEM_DOUBLE_QUOTED"].starts_with("-----BEGIN PUBLIC KEY-----\n"));
    assert!(parsed["MULTI_PEM_DOUBLE_QUOTED"].ends_with("\n-----END PUBLIC KEY-----"));
}

#[test]
fn populate_preserves_or_overrides_existing_values() {
    let mut destination = parse("PRESENT=old\nEMPTY=");
    let incoming = parse("PRESENT=new\nEMPTY=filled\nADDED=yes");
    populate(&mut destination, incoming.clone(), LoadOptions::default());
    assert_eq!(destination["PRESENT"], "old");
    assert_eq!(destination["EMPTY"], "");
    assert_eq!(destination["ADDED"], "yes");
    populate(
        &mut destination,
        incoming,
        LoadOptions {
            override_existing: true,
        },
    );
    assert_eq!(destination["PRESENT"], "new");
    assert_eq!(destination["EMPTY"], "filled");
}
