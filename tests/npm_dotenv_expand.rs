//! Adapted from dotenvx/dotenv-expand v1000.0.0 tests (commit 0d8c9260).

use dotenv::{EnvMap, ExpandError, ExpandOptions, expand_with, parse};

fn expanded(input: &str) -> EnvMap {
    expand_with(
        &parse(input),
        &EnvMap::new(),
        ExpandOptions {
            command_substitution: false,
            process_env_precedence: true,
        },
    )
    .unwrap()
}

#[test]
fn expands_braced_simple_missing_self_and_escaped_variables() {
    let values = expanded(
        "BASIC=basic\nBRACED=${BASIC}\nSIMPLE=$BASIC\nMISSING=$UNDEFINED\nSELF=$SELF\nSELF_DEFAULT=${SELF_DEFAULT:-default}\nESCAPED=\\$ESCAPED\nPASSWORD=pa\\$\\$word\nMIXED=\\$this$BASIC\\$is${BASIC}\n",
    );
    assert_eq!(values["BRACED"], "basic");
    assert_eq!(values["SIMPLE"], "basic");
    assert_eq!(values["MISSING"], "");
    assert_eq!(values["SELF"], "");
    assert_eq!(values["SELF_DEFAULT"], "default");
    assert_eq!(values["ESCAPED"], "$ESCAPED");
    assert_eq!(values["PASSWORD"], "pa$$word");
    assert_eq!(values["MIXED"], "$thisbasic$isbasic");
}

#[test]
fn expands_defaults_and_alternates_with_set_vs_nonempty_rules() {
    let values = expanded(
        "EMPTY=\nSET=value\nCOLON_DEFAULT=${EMPTY:-default}\nUNSET_DEFAULT=${EMPTY-default}\nMISSING_DEFAULT=${MISSING-default}\nCOLON_ALT=${EMPTY:+alternate}\nSET_ALT=${EMPTY+alternate}\nMISSING_ALT=${MISSING+alternate}\nPRESENT_ALT=${SET:+alternate}\nNESTED=${MISSING:-${ALSO_MISSING:-nested}}\nSPECIAL=${MISSING:-/default/path:with/colon}\n",
    );
    assert_eq!(values["COLON_DEFAULT"], "default");
    assert_eq!(values["UNSET_DEFAULT"], "");
    assert_eq!(values["MISSING_DEFAULT"], "default");
    assert_eq!(values["COLON_ALT"], "");
    assert_eq!(values["SET_ALT"], "alternate");
    assert_eq!(values["MISSING_ALT"], "");
    assert_eq!(values["PRESENT_ALT"], "alternate");
    assert_eq!(values["NESTED"], "nested");
    assert_eq!(values["SPECIAL"], "/default/path:with/colon");
}

#[test]
fn progressively_expands_recursive_values_and_preserves_order_semantics() {
    let values = expanded(
        "USER=username\nPASSWORD=password\nHOST=example.test\nAUTH=${USER}:${PASSWORD}\nURL=https://${AUTH}@${HOST}\nPLAIN=https://$USER:$PASSWORD@$HOST\n",
    );
    assert_eq!(values["AUTH"], "username:password");
    assert_eq!(values["URL"], "https://username:password@example.test");
    assert_eq!(values["PLAIN"], "https://username:password@example.test");

    let reverse = expanded("URL=${HOST}/health\nHOST=http://localhost:${PORT}\nPORT=8090\n");
    assert_eq!(reverse["URL"], "/health");
    assert_eq!(reverse["HOST"], "http://localhost:");
    assert_eq!(reverse["PORT"], "8090");
}

#[test]
fn process_environment_has_precedence_and_supports_forward_literals() {
    let parsed = parse("MACHINE=file\nUSE=$Mululo\nMACHINE_USE=$MACHINE\n");
    let process_env = parse("MACHINE=machine\nMululo=forward\n");
    let values = expand_with(
        &parsed,
        &process_env,
        ExpandOptions {
            command_substitution: false,
            process_env_precedence: true,
        },
    )
    .unwrap();
    assert_eq!(values["MACHINE"], "machine");
    assert_eq!(values["MACHINE_USE"], "machine");
    assert_eq!(values["USE"], "forward");
}

#[test]
fn handles_dotted_names_adjacent_variables_and_literal_dollars() {
    let values = expanded(
        "POSTGRESQL.BASE.USER=postgres\nPOSTGRESQL.MAIN.USER=${POSTGRESQL.BASE.USER}\nONE=one\nTWO=two\nBOTH=${ONE}$TWO\nDOLLAR=$\nTWO_DOLLARS=abcd$$1234\nSOURCE=12345\nJOINED=ab-$SOURCE-cd-ef\n",
    );
    assert_eq!(values["POSTGRESQL.MAIN.USER"], "postgres");
    assert_eq!(values["BOTH"], "onetwo");
    assert_eq!(values["DOLLAR"], "$");
    assert_eq!(values["TWO_DOLLARS"], "abcd$$1234");
    assert_eq!(values["JOINED"], "ab-12345-cd-ef");
}

#[test]
fn required_forms_report_missing_values() {
    let parsed = parse("EMPTY=\nA=${EMPTY:?set A}\n");
    let error = expand_with(
        &parsed,
        &EnvMap::new(),
        ExpandOptions {
            command_substitution: false,
            process_env_precedence: false,
        },
    )
    .unwrap_err();
    assert!(matches!(error, ExpandError::RequiredVariable { .. }));
}

#[test]
fn evaluates_command_substitution_when_enabled() {
    let parsed = parse("VALUE=$(printf evaluated)\n");
    let values = expand_with(&parsed, &EnvMap::new(), ExpandOptions::default()).unwrap();
    assert_eq!(values["VALUE"], "evaluated");

    let literal = expand_with(
        &parsed,
        &EnvMap::new(),
        ExpandOptions {
            command_substitution: false,
            process_env_precedence: false,
        },
    )
    .unwrap();
    assert_eq!(literal["VALUE"], "$(printf evaluated)");
}

#[test]
fn decrypts_encrypted_values_before_expansion() {
    let values = expanded("SECRET=encrypted:opaque\n");
    assert_eq!(values["SECRET"], "encrypted:opaque");

    let keypair = dotenvx_primitives::keypair(None).unwrap();
    let ciphertext = dotenvx_primitives::encrypt(&keypair.public_key, "$BASIC").unwrap();
    let parsed = parse(&format!("BASIC=basic\nSECRET={ciphertext}\n"));
    let process_env = parse(&format!("DOTENV_PRIVATE_KEY={}\n", keypair.private_key));
    let decrypted = expand_with(
        &parsed,
        &process_env,
        ExpandOptions {
            command_substitution: false,
            process_env_precedence: true,
        },
    )
    .unwrap();
    assert_eq!(decrypted["SECRET"], "basic");
}
