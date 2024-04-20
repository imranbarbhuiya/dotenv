use std::collections::HashMap;

pub fn parse_content(input: &str) -> HashMap<String, String> {
    let mut store = HashMap::new();
    let lines = input.replace('\r', "");
    let mut content = lines.trim();

    while !content.is_empty() {
        if content.starts_with('\n') || content.starts_with('#') {
            if let Some(newline) = content.find('\n') {
                content = &content[newline + 1..];
                continue;
            }
        }

        let equal = content.find('=');
        let equal = match equal {
            Some(index) => index,
            None => break,
        };

        let mut key = &content[..equal];
        content = &content[equal + 1..];
        key = key.trim();

        if key.is_empty() {
            break;
        }

        if key.starts_with("export ") {
            key = &key["export ".len()..];
        }

        if content.is_empty() {
            store.insert(key.to_string(), "".to_string());
            break;
        }

        let mut value;

        if let Some(first_char) = content.chars().next() {
            if first_char == '"' || first_char == '\'' || first_char == '`' {
                if let Some(found) = content[1..].find(first_char) {
                    value = &content[1..found + 1];
                    store.insert(key.to_string(), value.replace("\\n", "\n"));
                    // content = &content[content.find('\n').unwrap_or(content.len())..];
                    content = &content[found + 2..];
                    continue;
                }
            }
        }

        if let Some(newline) = content.find('\n') {
            value = &content[..newline];
            if let Some(hash) = value.find('#') {
                value = &value[..hash];
            }
            content = &content[newline + 1..];
        } else {
            value = content;
            content = "";
        }

        value = value.trim();
        store.insert(key.to_string(), value.to_string());
    }

    store
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_content() {
        let input = r#"
 # Sample .env file
API_URL="https://api.example.com"
USERNAME="admin"
PASSWORD="secret"
PORT=8000
MULTILINE="This is a multi-line string\nthat spans two lines"
MULTILINE_2="This is a 
multiline string that spans two lines"
MULTILINE_3="This is a 
multiline string that spans three lines
and contains a blank line\n"
SINGLEQUOTE='something'
BACKTIC=`ok`
VALUEWITHQUOTE="something
export KEYWITHEXPORT = hi
EMPTY=
=EMPTY
        "#;

        let store = parse_content(input);

        assert_eq!(store["API_URL"], "https://api.example.com");
        assert_eq!(store["USERNAME"], "admin");
        assert_eq!(store["PASSWORD"], "secret");
        assert_eq!(store["PORT"], "8000");
        assert_eq!(
            store["MULTILINE"],
            "This is a multi-line string\nthat spans two lines"
        );
        assert_eq!(
            store["MULTILINE_2"],
            "This is a \nmultiline string that spans two lines"
        );
        assert_eq!(
            store["MULTILINE_3"],
            "This is a \nmultiline string that spans three lines\nand contains a blank line\n"
        );
        assert_eq!(store["SINGLEQUOTE"], "something");
        assert_eq!(store["BACKTIC"], "ok");
        assert_eq!(store["VALUEWITHQUOTE"], "\"something");
        assert_eq!(store["KEYWITHEXPORT"], "hi");
        assert_eq!(store["EMPTY"], "");
        assert_eq!(store.len() as i32, 12)
    }
}
