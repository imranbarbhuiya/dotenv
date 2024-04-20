pub struct ConfigIterator<'a> {
    content: &'a str,
}

impl<'a> ConfigIterator<'a> {
    pub fn new(input: &'a str) -> ConfigIterator<'a> {
        ConfigIterator {
            content: input.trim(),
        }
    }
}

impl<'a> Iterator for ConfigIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        while !self.content.is_empty() {
            if self.content.starts_with('\n') || self.content.starts_with('#') {
                if let Some(newline) = self.content.find('\n') {
                    self.content = &self.content[newline + 1..];
                    continue;
                }
            }

            let equal = self.content.find('=');
            let equal = match equal {
                Some(index) => index,
                None => break,
            };

            let mut key = &self.content[..equal];
            self.content = &self.content[equal + 1..];
            key = key.trim();

            if key.is_empty() {
                break;
            }

            if key.starts_with("export ") {
                key = &key["export ".len()..];
            }

            if self.content.is_empty() {
                return Some((key.to_string(), "".to_string()));
            }

            let mut value;

            if let Some(first_char) = self.content.chars().next() {
                if first_char == '"' || first_char == '\'' || first_char == '`' {
                    if let Some(found) = self.content[1..].find(first_char) {
                        value = &self.content[1..found + 1];
                        self.content = &self.content[found + 2..];
                        return Some((key.to_string(), value.replace("\\n", "\n").to_string()));
                    }
                }
            }

            if let Some(newline) = self.content.find('\n') {
                value = &self.content[..newline];
                if let Some(hash) = value.find('#') {
                    value = &value[..hash];
                }
                self.content = &self.content[newline + 1..];
            } else {
                value = self.content;
                self.content = "";
            }

            value = value.trim();
            return Some((key.to_string(), value.to_string()));
        }
        None
    }
}

pub fn parse_content(input: &str) -> ConfigIterator {
    ConfigIterator::new(input)
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
        let mut iter = parse_content(input);

        assert_eq!(
            iter.next(),
            Some(("API_URL".to_string(), "https://api.example.com".to_string()))
        );
        assert_eq!(
            iter.next(),
            Some(("USERNAME".to_string(), "admin".to_string()))
        );
        assert_eq!(
            iter.next(),
            Some(("PASSWORD".to_string(), "secret".to_string()))
        );
        assert_eq!(iter.next(), Some(("PORT".to_string(), "8000".to_string())));
        assert_eq!(
            iter.next(),
            Some((
                "MULTILINE".to_string(),
                "This is a multi-line string\nthat spans two lines".to_string()
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                "MULTILINE_2".to_string(),
                "This is a \nmultiline string that spans two lines".to_string()
            ))
        );
        assert_eq!(
            iter.next(),
            Some((
                "MULTILINE_3".to_string(),
                "This is a \nmultiline string that spans three lines\nand contains a blank line\n"
                    .to_string()
            ))
        );
        assert_eq!(
            iter.next(),
            Some(("SINGLEQUOTE".to_string(), "something".to_string()))
        );
        assert_eq!(iter.next(), Some(("BACKTIC".to_string(), "ok".to_string())));
        assert_eq!(
            iter.next(),
            Some(("VALUEWITHQUOTE".to_string(), "\"something".to_string()))
        );
        assert_eq!(
            iter.next(),
            Some(("KEYWITHEXPORT".to_string(), "hi".to_string()))
        );
        assert_eq!(iter.next(), Some(("EMPTY".to_string(), "".to_string())));
        assert_eq!(iter.next(), None);
    }
}
