use std::io::{BufRead, Read};

// Constant declarations
pub const TAB: char = '\t';

#[derive(Debug, PartialEq, Eq)]
enum Ops {
    ADD,
    SUB,
    MUL,
    DIV,
    INVALID,
}

impl From<char> for Ops {
    fn from(c: char) -> Self {
        match c {
            '+' => Ops::ADD,
            '-' => Ops::SUB,
            '*' => Ops::MUL,
            '/' => Ops::DIV,
            _ => Ops::INVALID,
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

    /// Check if lookahead character is Mulop: * or /
    pub fn is_mulop(&mut self) -> bool {
        let ops = vec![Ops::DIV, Ops::MUL];
        ops.iter().any(|op| *op == Ops::from(self.look))
    }

    /// Check if lookahead character is Addop: + or -
    pub fn is_addop(&mut self) -> bool {
        let ops = vec![Ops::ADD, Ops::SUB];
        ops.iter().any(|val| *val == Ops::from(self.look))
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
        if self.is_addop() {
            self.emitln("CLR D0");
        } else {
            self.term();
        }
        while self.is_addop() {
            self.emitln("MOVE D0,-(SP)");
            match Ops::from(self.look) {
                Ops::ADD => {
                    self.add();
                }
                Ops::SUB => {
                    self.subtract();
                }
                _ => {
                    expected("Addop");
                }
            }
        }
    }

    /// Represent <term>
    ///
    /// <mulop> -> *, /
    ///
    /// <term> ::= <factor> [<mulop> <factor>]*
    pub fn term(&mut self) {
        self.factor();
        while self.is_mulop() {
            self.emitln("MOVE D0,-(SP)");
            match Ops::from(self.look) {
                Ops::MUL => {
                    self.multiply();
                }
                Ops::DIV => {
                    self.divide();
                }
                _ => {
                    expected("Mulop");
                }
            }
        }
    }

    /// Represent <factor>
    ///
    /// <factor> ::= (<expression>)
    ///
    /// This supports parentheesis, like (2+3)/(6*2)
    pub fn factor(&mut self) {
        if self.look == '(' {
            self.match_char('(');
            self.expression();
            self.match_char(')');
        } else {
            let num = self.get_num();
            self.emitln(&format!("MOVE #{},D0", num));
        }
    }

    /// Recognize and Translate Multiply
    pub fn multiply(&mut self) {
        self.match_char('*');
        self.factor();
        self.emitln("MULS (SP)+,D0");
    }

    /// Recognize and Translate Divide
    pub fn divide(&mut self) {
        self.match_char('/');
        self.factor();
        self.emitln("MOVE (SP)+,D1");
        self.emitln("DIVS D1,D0");
    }

    /// Recognize and Translate Add
    pub fn add(&mut self) {
        self.match_char('+');
        self.term();
        self.emitln("ADD (SP)+,D0");
    }

    /// Recognize and Translate Subtract
    pub fn subtract(&mut self) {
        self.match_char('-');
        self.term();
        self.emitln("SUB (SP)+,D0");
        self.emitln("NEG D0");
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
        let input = b"2+3-4 ";
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

    #[test]
    fn test_with_mulops() {
        let input = b"2+3*5-6/3 ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }

    #[test]
    fn test_with_paren() {
        let input = b"(((2+3)*5)-6)/3 ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }

    #[test]
    fn test_unary_minus() {
        let input = b"-1 ";
        let mut c = Cradle::new(&input[..]);
        c.expression();
    }
}
