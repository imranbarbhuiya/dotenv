use std::collections::HashMap;

enum Token {
    Key(String),
    Value(String),
}

struct Parser {
    tokens: Vec<Token>,
    map: HashMap<String, String>,
    key: String,
    value: String,
    in_value: bool,
    escape_next: bool,
}

impl Parser {
    fn new() -> Self {
        Parser {
            tokens: Vec::new(),
            map: HashMap::new(),
            key: String::new(),
            value: String::new(),
            in_value: false,
            escape_next: false,
        }
    }

    fn parse(&mut self, file_contents: &str) {
        for line in file_contents.lines() {
            for c in line.chars() {
                if self.escape_next {
                    self.escape_next = false;
                    self.value.push(c);
                    continue;
                }

                match c {
                    '#' => {
                        if self.in_value {
                            self.tokens.push(Token::Value(self.value.clone()));
                            self.value.clear();
                            self.in_value = false;
                        }
                        break;
                    }
                    '=' => {
                        if self.in_value {
                            self.tokens.push(Token::Value(self.value.clone()));
                        } else {
                            self.tokens.push(Token::Key(self.key.clone()));
                            self.key.clear();
                        }
                    }
                    '\n' => {
                        // if not in value, add a key with empty value
                        // if in value, check if value starts with `"/'` and didnt ended with same then parse for multiline
                        // else set value and add to tokens and prepare for next key

                        if self.in_value {
                            self.tokens.push(Token::Value(self.value.clone()));
                            self.value.clear();
                            self.in_value = false;
                        } else {
                            self.tokens.push(Token::Key(self.key.clone()));
                            self.key.clear();
                        }
                    }
                    '"' => {
                        if self.in_value {
                            self.value.push(c);
                            self.in_value = false;
                        } else {
                            self.tokens.push(Token::Quotation);
                            self.in_value = true;
                        }
                    }
                    '\\' => {
                        self.escape_next = true;
                    }
                    '$' => {
                        self.tokens.push(Token::Dollar);
                    }
                    '{' => {
                        self.tokens.push(Token::OpenBrace);
                    }
                    '}' => {
                        self.tokens.push(Token::CloseBrace);
                    }
                    _ => {
                        if self.in_value {
                            self.value.push(c);
                        } else {
                            self.tokens.push(Token::Key(c.to_string()));
                        }
                    }
                }
            }
        }
    }

    fn substitute_variables(&mut self) {
        for token in &self.tokens {
            match token {
                Token::Key(k) => {
                    if self.in_value {
                        self.map.insert(self.key.clone(), self.value.clone());
                        self.value.clear();
                    }
                    self.key = k.clone();
                    self.in_value = false;
                }
                Token::Equals => {
                    self.in_value = true;
                }
                Token::Value(v) => {
                    if self.in_value {
                        self.value = v.clone();
                    }
                }
                Token::Newline => {
                    if self.in_value {
                        self.map.insert(self.key.clone(), self.value.clone());
                        self.key.clear();
                        self.value.clear();
                        self.in_value = false;
                    }
                }
                Token::Comment(_) => {} // Ignore comments
                _ => {}
            }
        }
    }

    pub fn print(&self) {
        for (key, value) in &self.map {
            println!("{} == {}", key, value);
        }
    }
}
