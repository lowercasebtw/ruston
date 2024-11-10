use std::collections::HashMap;

#[derive(Debug)]
enum JsonObject {
    Object(HashMap<String, JsonObject>),
    Array(Vec<JsonObject>),
    String(String),
    Number(f32),
    Boolean(bool),
    Null,
}

struct JsonParser {
    source: String,
    cursor: usize,
}

impl JsonParser {
    fn new(input: String) -> Self {
        Self {
            source: input,
            cursor: 0,
        }
    }

    fn is_eof(&self) -> bool { self.cursor >= self.source.len() }

    fn current(&self) -> u8 {
        let bytes = self.source.as_bytes();
        *bytes.get(self.cursor).unwrap_or(&0)
    }

    fn peek(&self) -> u8 {
        let bytes = self.source.as_bytes();
        *bytes.get(self.cursor + 1).unwrap_or(&0)
    }

    fn try_consume(&mut self, it: &str) -> bool {
        let len = it.len();
        if self.cursor + len > self.source.len() {
            return false;
        }
        let slice = &self.source[self.cursor..self.cursor + len];
        let same = slice == it;
        if same {
            self.cursor += len;
        }
        same
    }

    fn try_consume_ch(&mut self, ch: u8) -> bool {
        let current = self.current();
        let same = ch == current;
        if same {
            self.cursor += 1;
        }
        same
    }

    fn trim_left(&mut self) {
        while !self.is_eof() && (self.current() == b' ' || self.current() == b'\t' || self.current() == b'\n') {
            self.cursor += 1;
        }
    }

    fn lex_string(&mut self) -> String {
        if !self.try_consume_ch(b'"') {
            panic!("Expected opening quote whilst parsing string");
        }
        let start = self.cursor;
        while !self.is_eof() && self.current() != b'"' {
            self.cursor += 1;
        }
        match !self.try_consume_ch(b'"') {
            true => panic!("Expected close quote whilst parsing string"),
            _ => String::from(&self.source[start..self.cursor - 1])
        }
    }

    fn parse_object(&mut self) -> JsonObject {
        if !self.try_consume_ch(b'{') {
            panic!("Expected open bracket whilst parsing object");
        }
        let mut children: HashMap<String, JsonObject> = HashMap::new();
        while !self.is_eof() && self.current() != b'}' {
            self.trim_left();
            let key = self.lex_string();
            let value = match !self.try_consume_ch(b':') {
                true => panic!("Expected colon after key whilst parsing object"),
                _ => self.parse()
            };
            children.insert(key, value);
            if !self.try_consume_ch(b',') {
                break;
            }
        }
        match !self.try_consume_ch(b'}') {
            true => panic!("Expected close bracket whilst parsing object"),
            _ => JsonObject::Object(children)
        }
    }

    fn parse_array(&mut self) -> JsonObject {
        if !self.try_consume_ch(b'[') {
            panic!("Expected open square bracket whilst parsing array");
        }
        let mut children: Vec<JsonObject> = Vec::new();
        while !self.is_eof() {
            children.push(self.parse());
            if self.current() == b']' {
                break;
            }
            if self.current() == b',' && self.peek() != b']' {
                self.cursor += 1;
                continue;
            }
            panic!("Unexpected end of input whilst parsing children in array");
        }
        match !self.try_consume_ch(b']') {
            true => panic!("Expected close square bracket whilst parsing array"),
            _ => JsonObject::Array(children)
        }
    }

    fn parse_string(&mut self) -> JsonObject {
        JsonObject::String(self.lex_string())
    }

    fn parse_boolean(&mut self) -> JsonObject {
        if self.try_consume("true") {
            return JsonObject::Boolean(true);
        } else if self.try_consume("false") {
            return JsonObject::Boolean(false);
        }
        panic!("Unexpected end of input whilst parsing boolean");
    }

    fn parse_number(&mut self) -> JsonObject {
        let mut is_negative = false;
        if self.try_consume_ch(b'-') {
            is_negative = true;
        } else if self.try_consume_ch(b'+') {
            is_negative = false;
        }

        let mut number = 0f32;
        while !self.is_eof() && self.current().is_ascii_digit() {
            number *= 10.0;
            number += (self.current() - b'0') as f32;
            self.cursor += 1;
        }

        JsonObject::Number(match is_negative {
            true => -number,
            _ => number
        })
    }

    fn parse_null(&mut self) -> JsonObject {
        self.try_consume("null");
        JsonObject::Null
    }

    fn parse(&mut self) -> JsonObject {
        if self.is_eof() {
            panic!("Unexpected end of JSON input");
        }
        self.trim_left();
        let current = self.current();
        match current {
            b'{' => self.parse_object(),
            b'[' => self.parse_array(),
            b'"' => self.parse_string(),
            b't' | b'f' => self.parse_boolean(),
            b'n' => self.parse_null(),
            b'-' | b'+' | b'0'..=b'9' => self.parse_number(),
            _ => panic!("Unexpected token '{current}', \"{current}\" is not valid JSON")
        }
    }
}

fn main() {
    let mut parser: JsonParser = JsonParser::new("[true, false, \"hello\", {}, -12]".to_string());
    let object: JsonObject = parser.parse();
    println!("{:?}", object);
}
