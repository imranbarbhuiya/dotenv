//! Regression coverage for byte scanning and borrowed expansion fast paths.
use dotenv::{EnvMap, ExpandOptions, expand_with, parse};

#[test]
fn byte_scans_preserve_unicode_and_line_boundaries() {
    let long = "🦀é漢字".repeat(1024);
    for newline in ["\n", "\r\n", "\r"] {
        let input = format!(
            "\u{feff}# {long}{newline}INVALID LINE {long}{newline}\tA={long}  # comment{newline}B=尾{newline}C=last"
        );
        let values = parse(&input);
        assert_eq!(values.len(), 3);
        assert_eq!(values["A"], long);
        assert_eq!(values["B"], "尾");
        assert_eq!(values["C"], "last");
    }
}

#[test]
fn literal_fast_path_preserves_backslashes_and_empty_values() {
    let parsed = parse("PATH_VALUE=C:\\Users\\test\nEMPTY=\nUNICODE=🦀é漢字\n");
    for command_substitution in [false, true] {
        let result = expand_with(
            &parsed,
            &EnvMap::new(),
            ExpandOptions {
                command_substitution,
                process_env_precedence: true,
            },
        )
        .unwrap();
        assert_eq!(result, parsed);
    }
}

#[test]
fn recursive_scratch_state_does_not_leak_between_entries_or_operands() {
    let parsed = parse(
        "FIRST=$EXT\nSECOND=${EXT}\nTHIRD=${MISSING:-$EXT}\nFOURTH=${FIRST:+${SECOND}}\nCYCLE=$LOOP\nAFTER=$EXT\n",
    );
    let inherited = parse("EXT=${BASE}\nBASE=🦀\nLOOP=$LOOP\n");
    let result = expand_with(&parsed, &inherited, ExpandOptions::default()).unwrap();
    for key in ["FIRST", "SECOND", "THIRD", "FOURTH", "AFTER"] {
        assert_eq!(result[key], "🦀", "{key}");
    }
    assert_eq!(result["CYCLE"], "");
}
