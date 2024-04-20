mod parse_content;
use std::env;

pub use parse_content::parse_content;

pub fn parse_and_set_env(content: &str, override_env: bool) {
    let parsed = parse_content(content);
    for (key, value) in parsed {
        if !override_env && env::var(&key).is_ok() {
            continue;
        }
        env::set_var(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_and_set_env() {
        let content = r#"
            KEY1="value1"
            KEY2='value2'
            "#;

        assert!(env::var("KEY1").is_err());
        assert!(env::var("KEY2").is_err());

        parse_and_set_env(content, false);

        assert_eq!(env::var("KEY1").unwrap(), "value1");
        assert_eq!(env::var("KEY2").unwrap(), "value2");

        env::remove_var("KEY1");
        env::remove_var("KEY2");
    }

    #[test]
    fn test_parse_and_set_env_override() {
        let content = r#"
            KEY3="value1"
            "#;

        env::set_var("KEY3", "value3");

        parse_and_set_env(content, true);

        assert_eq!(env::var("KEY3").unwrap(), "value1");

        env::remove_var("KEY3");
    }

    #[test]
    fn test_parse_and_set_env_no_override() {
        let content = r#"
            KEY4="value1"
            "#;

        env::set_var("KEY4", "value3");

        parse_and_set_env(content, false);

        assert_eq!(env::var("KEY4").unwrap(), "value3");

        env::remove_var("KEY4");
    }
}
