//! Adapted from kermage/dotenv-extra v0.10.0 tests (commit a2d2a72c).

use dotenv::{DotenvEditor, LineEnding};

#[test]
fn parses_dotenv_extra_dialect() {
    let editor = DotenvEditor::from_content(" FOO = bar \nBAZ=qux  # comment\nHASH=bar#baz\nQ=\"abc # def\" # comment\nS='it is # here'\nEMPTY=#only\nexport EXPORTED=value\nBACK=`literal`\nBAD\n=VALUE\n").unwrap();
    let values = editor.dump();
    assert_eq!(values["FOO"], "bar");
    assert_eq!(values["BAZ"], "qux");
    assert_eq!(values["HASH"], "bar#baz");
    assert_eq!(values["Q"], "abc # def");
    assert_eq!(values["S"], "it is # here");
    assert_eq!(values["EMPTY"], "");
    assert_eq!(values["EXPORTED"], "value");
    assert_eq!(values["BACK"], "`literal`");
    assert_eq!(values.len(), 8);
}

#[test]
fn ports_quoted_edge_cases_and_duplicate_rules() {
    let editor = DotenvEditor::from_content(
        "A=\"foo\"bar\nB=\"two\\\\\"\nC=\"unterminated\nD='it\\'s ready'\nE=\"a\\\"b\" # comment\nA=last\nCOLON: value\nPHYSICAL=\"line1\nline2\"\n",
    ).unwrap();
    let values = editor.dump();
    assert_eq!(values["A"], "last");
    assert_eq!(values["B"], "two\\");
    assert_eq!(values["C"], "\"unterminated");
    assert_eq!(values["D"], "it\\'s ready");
    assert_eq!(values["E"], "a\\\"b");
    assert!(!values.contains_key("COLON"));
    assert_eq!(values["PHYSICAL"], "\"line1");
}

#[test]
fn appends_empty_and_equals_values_and_preserves_comments() {
    let mut editor = DotenvEditor::from_content("# top\n\nTARGET=old\n# bottom\n").unwrap();
    editor.upsert("TARGET", "changed").unwrap();
    editor.upsert("EMPTY", "").unwrap();
    editor.upsert("URL", "pg://host/db?opt=1&x=2").unwrap();
    assert_eq!(
        editor.render(),
        "# top\n\nTARGET=changed\n# bottom\nEMPTY=\nURL=pg://host/db?opt=1&x=2\n"
    );
}

#[test]
fn special_key_characters_match_literally() {
    let mut editor = DotenvEditor::from_content(
        "MY.KEY=old\nMY*KEY=old\nMY$KEY=old\nDATABASE_URL=x\nDATABASE=y\n",
    )
    .unwrap();
    editor.upsert("MY.KEY", "dot").unwrap();
    editor.upsert("MY*KEY", "star").unwrap();
    editor.upsert("MY$KEY", "dollar").unwrap();
    editor.upsert("DATABASE", "exact").unwrap();
    assert_eq!(editor.dump()["MY.KEY"], "dot");
    assert_eq!(editor.dump()["MY*KEY"], "star");
    assert_eq!(editor.dump()["MY$KEY"], "dollar");
    assert_eq!(editor.dump()["DATABASE_URL"], "x");
    assert_eq!(editor.dump()["DATABASE"], "exact");
}

#[test]
fn preserves_layout_and_updates_last_active_or_last_commented() {
    let input = "# heading\nFOO=first\n\nFOO=last\n# BAR=old\n# BAR=latest\n";
    let mut editor = DotenvEditor::from_content(input).unwrap();
    editor
        .upsert("FOO", "new")
        .unwrap()
        .upsert("BAR", "active")
        .unwrap();
    assert_eq!(
        editor.render(),
        "# heading\nFOO=first\n\nFOO=new\n# BAR=old\nBAR=active\n"
    );
    assert_eq!(editor.dump()["FOO"], "new");
    assert_eq!(editor.dump()["BAR"], "active");
}

#[test]
fn preserves_each_line_ending_and_trailing_newline_policy() {
    for (input, ending) in [
        ("A=1\nB=2\n", LineEnding::Lf),
        ("A=1\r\nB=2\r\n", LineEnding::CrLf),
        ("A=1\rB=2\r", LineEnding::Cr),
    ] {
        let mut editor = DotenvEditor::from_content(input).unwrap();
        assert_eq!(editor.line_ending(), ending);
        editor.upsert("B", "updated").unwrap();
        assert!(editor.render().ends_with(match ending {
            LineEnding::Lf => "\n",
            LineEnding::CrLf => "\r\n",
            LineEnding::Cr => "\r",
        }));
    }
    let mut no_newline = DotenvEditor::from_content("A=1\nB=2").unwrap();
    no_newline.upsert("B", "updated").unwrap();
    assert_eq!(no_newline.render(), "A=1\nB=updated");
    assert!(DotenvEditor::from_content("A=1\r\nB=2\n").is_err());
    assert_eq!(DotenvEditor::from_content("\n").unwrap().render(), "\n");
    assert_eq!(DotenvEditor::from_content("\r\n").unwrap().render(), "\r\n");
}

#[test]
fn quotes_values_without_changing_their_meaning() {
    let cases = [
        ("PLAIN", "a#b", "PLAIN=a#b\n"),
        ("COMMENT", "a # b", "COMMENT=\"a # b\"\n"),
        ("PADDED", "  padded  ", "PADDED=\"  padded  \"\n"),
        ("LEADING", "#value", "LEADING=\"#value\"\n"),
        ("SINGLE", "'value'", "SINGLE=\"'value'\"\n"),
        ("DOUBLE", "\"value\"", "DOUBLE='\"value\"'\n"),
        ("SLASH", "a # b\\", "SLASH=\"a # b\\\\\"\n"),
    ];
    for (key, value, rendered) in cases {
        let mut editor = DotenvEditor::from_content("").unwrap();
        editor.upsert(key, value).unwrap();
        assert_eq!(editor.render(), rendered.trim_end_matches('\n'));
        assert_eq!(editor.dump()[key], value);
    }
    let mut editor = DotenvEditor::from_content("").unwrap();
    assert!(editor.upsert("BOTH", "say \"hi\" it's # done").is_err());
}

#[test]
fn rejects_injection_and_invalid_keys() {
    for key in ["", " PAD", "PAD ", "A=B", "export FOO", "BAD\nKEY"] {
        assert!(
            DotenvEditor::from_content("A=1\n")
                .unwrap()
                .upsert(key, "value")
                .is_err(),
            "{key:?}"
        );
    }
    assert!(
        DotenvEditor::from_content("A=1\n")
            .unwrap()
            .upsert("KEY", "a\nEVIL=pwned")
            .is_err()
    );
}

#[test]
fn long_backslash_runs_are_linear() {
    let value = format!("{} ", "\\".repeat(100_000));
    let mut editor = DotenvEditor::from_content("").unwrap();
    editor.upsert("LONG", &value).unwrap();
    assert_eq!(editor.dump()["LONG"], value);
}

#[test]
fn disk_round_trip_and_save_behavior() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join(".env");
    std::fs::write(&path, "# comment\r\nA=1\r\n").unwrap();
    let mut editor = DotenvEditor::open(&path).unwrap();
    editor.upsert("A", "2").unwrap();
    editor.save().unwrap();
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        "# comment\r\nA=2\r\n"
    );
    assert!(DotenvEditor::from_content("A=1").unwrap().save().is_err());
}
