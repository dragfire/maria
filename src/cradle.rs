use std::io::{BufRead, Read};

// Constant declarations
pub const TAB: char = '\t';

#[derive(Debug, PartialEq, Eq)]
enum AddOps {
    ADD,
    SUB,
    INVALID,
}

impl From<char> for AddOps {
    fn from(c: char) -> Self {
        match c {
            '+' => AddOps::ADD,
            '-' => AddOps::SUB,
            _ => AddOps::INVALID,
        }
    }
}

/// Cradle contains the lookahead character
/// and Input that implements Read trait
/// This way it helps testing with dependency injection
pub struct Cradle<R> {
    /// Lookahead character
    pub look: char,

    /// Input reader
    pub input: R,
}

impl<R: BufRead> Cradle<R> {
    pub fn new(input: R) -> Self {
        let mut cradle = Cradle { look: '2', input };
        cradle.look = cradle.get_char();
        cradle
    }

    /// Get character
    pub fn get_char(&mut self) -> char {
        // TODO: don't use unwrap
        self.input
            .by_ref()
            .bytes()
            .next()
            .map(|b| b.ok().unwrap() as char)
            .unwrap()
    }

    /// Match a specific input character with Lookahead character
    ///
    /// If it does not match, it will panic
    pub fn match_char(&mut self, x: char) {
        if self.look != x {
            expected(&x.to_string());
        }
        self.look = self.get_char();
    }

    /// Get an Identifier
    pub fn get_name(&mut self) -> char {
        if !self.look.is_alphabetic() {
            expected("Name");
        }

        let look_upcase = self.look.to_ascii_uppercase();
        self.look = self.get_char();

        look_upcase
    }

    /// Get a Number
    pub fn get_num(&mut self) -> char {
        if !self.look.is_ascii_digit() {
            expected("Integer");
        }

        let look = self.look;
        self.look = self.get_char();

        look
    }

    /// Output a string with Tab
    pub fn emit(&mut self, s: &str) {
        print!("{}", TAB.to_string() + s);
    }

    /// Output a string with Tab and CRLF
    pub fn emitln(&mut self, s: &str) {
        self.emit(s);
        println!();
    }

    /// Recognize and Translate Add
    pub fn add(&mut self) {
        self.match_char('+');
        self.term();
        self.emitln("ADD D1,D0");
    }

    /// Recognize and Translate Add
    pub fn subtract(&mut self) {
        self.match_char('-');
        self.term();
        self.emitln("SUB D1,D0");
        self.emitln("NEG D0");
    }

    /// Parse and Translate a Math Expression
    ///
    ///         1+2
    /// or      4-3
    /// or, in general, <term> +/- <term>
    ///
    /// <expression> ::= <term> [<addop> <term>]*
    ///
    pub fn expression(&mut self) {
        self.term();
        let ops = vec![AddOps::ADD, AddOps::SUB];

        while ops.iter().any(|val| *val == AddOps::from(self.look)) {
            self.emitln("MOVE D0,D1");
            match AddOps::from(self.look) {
                AddOps::ADD => {
                    self.add();
                }
                AddOps::SUB => {
                    self.subtract();
                }
                AddOps::INVALID => {
                    expected("Addop");
                }
            }
        }
    }

    /// Represent <term>
    pub fn term(&mut self) {
        let num = self.get_num();
        self.emitln(&format!("MOVE #{},D0", num));
    }
}

fn expected(x: &str) {
    panic!("{} Expected", x);
}

#[cfg(test)]
mod tests {
    use super::*;

    // TODO: assert with generated machine code
    #[test]
    fn test_valid_expression() {
        let input = b"2+2 ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }

    #[test]
    fn test_invalid_expression() {
        let input = b"2aa ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }

    #[test]
    fn test_single_expression() {
        let input = b"1 ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }
}
