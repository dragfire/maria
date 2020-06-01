use std::io::{BufRead, Read};

// Constant declarations
pub const TAB: char = '\t';
pub const NEW_LINE: char = '\n';
pub const SPACE: char = ' ';

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

    /// Label count, used in control statements
    pub lcount: usize,
}

impl<R: BufRead> Cradle<R> {
    pub fn new(input: R) -> Self {
        let mut cradle = Cradle {
            look: '2',
            input,
            lcount: 0,
        };
        cradle.look = cradle.get_char();
        cradle.other();
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

    /// Skip over leading White Space
    pub fn skip_white(&mut self) {
        while self.is_white() {
            self.look = self.get_char();
        }
    }

    /// Returns true if Lookahead character is TAB or SPACE
    pub fn is_white(&mut self) -> bool {
        [TAB, SPACE].iter().any(|w| *w == self.look)
    }

    /// Recognize and Translate an "Other"
    pub fn other(&mut self) {
        let name = self.get_name();
        self.emitln(&name.to_string());
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
    pub fn get_num(&mut self) -> String {
        if !self.look.is_ascii_digit() {
            expected("Integer");
        }

        let mut value = String::new();
        while self.look.is_ascii_digit() {
            value.push(self.look);
            self.look = self.get_char();
        }

        self.skip_white();

        value
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
        self.emitln("<expr>");
    }

    /// Parse and Translate an Assignment statement
    pub fn assignment(&mut self) {
        let name = self.get_name();
        self.match_char('=');
        self.expression();
        self.emitln(&format!("LEA {}(PC),A0", name));
        self.emitln("MOVE D0,(A0)");
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
    /// <factor> ::= <number> | (<expression>)
    ///
    /// This supports parenthesis, like (2+3)/(6*2)
    ///
    /// We can support variables also, i.e b * b + 4 * a * c:
    /// <factor> ::= <number> | (<expression>) | <variable>
    pub fn factor(&mut self) {
        if self.look == '(' {
            self.match_char('(');
            self.expression();
            self.match_char(')');
        } else if self.look.is_alphabetic() {
            self.ident();
        } else {
            let num = self.get_num();
            self.emitln(&format!("MOVE #{},D0", num));
        }
    }

    /// Deal with variable and function calls
    pub fn ident(&mut self) {
        let name = self.get_name();
        if self.look == '(' {
            self.match_char('(');
            self.match_char(')');
            self.emitln(&format!("BSR {}", name));
        } else {
            self.emitln(&format!("MOVE {}(PC),D0", name));
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

    /// Parse and Translate a Program
    pub fn do_program(&mut self) {
        self.block("");
        if self.look != 'e' {
            expected("End");
        }
        self.emitln("END");
    }

    /// Recognize and Translate a Statement Block
    pub fn block(&mut self, label: &str) {
        while !['e', 'l', 'u'].iter().any(|c| *c == self.look) {
            match self.look {
                'i' => self.do_if(&label),
                'w' => self.do_while(),
                'p' => self.do_loop(),
                'r' => self.do_repeat(),
                'f' => self.do_for(),
                'd' => self.do_do(),
                'b' => self.do_break(label),
                _ => self.other(),
            }
        }
    }

    /// Parse and Translate a Boolean Condition
    pub fn condition(&mut self) {
        self.emitln("<condition>");
    }

    /// Parse and Translate a BREAK
    pub fn do_break(&mut self, label: &str) {
        self.match_char('b');
        if label != "" {
            self.emitln(&format!("BRA {}", label));
        } else {
            panic!("No loop to break from");
        }
    }

    /// Recognize and Translate an IF Construct
    pub fn do_if(&mut self, label: &str) {
        self.match_char('i');
        let label1 = self.new_label();
        let mut label2 = label1.to_string();
        self.condition();
        self.emitln(&format!("BEQ {}", &label1));
        self.block(label);
        if self.look == 'l' {
            self.match_char('l');
            label2 = self.new_label();
            self.emitln(&format!("BRA {}", label2));
            self.post_label(&label1);
            self.block(label);
        }
        self.match_char('e');
        self.post_label(&label2);
    }

    /// Recognize and Translate a WHILE Statement
    pub fn do_while(&mut self) {
        self.match_char('w');
        let l1 = self.new_label();
        let l2 = self.new_label();
        self.post_label(&l1);
        self.condition();
        self.emitln(&format!("BEQ {}", l2));
        self.block(&l2);
        self.match_char('e');
        self.emitln(&format!("BRA {}", l1));
        self.post_label(&l2);
    }

    /// Parse and Translate a LOOP Statement
    pub fn do_loop(&mut self) {
        self.match_char('p');
        let l1 = self.new_label();
        let l2 = self.new_label();
        self.post_label(&l1);
        self.block(&l2);
        self.match_char('e');
        self.emitln(&format!("BRA {}", &l1));
        self.post_label(&l2);
    }

    /// Parse and Translate a REPEAT Statement
    pub fn do_repeat(&mut self) {
        self.match_char('r');
        let l1 = self.new_label();
        let l2 = self.new_label();
        self.post_label(&l1);
        self.block(&l2);
        self.match_char('u');
        self.condition();
        self.emitln(&format!("BEQ {}", l1));
        self.post_label(&l2);
    }

    /// Parse and Translate a FOR statement
    pub fn do_for(&mut self) {
        self.match_char('f');
        let l1 = self.new_label();
        let l2 = self.new_label();
        let name = self.get_name();
        self.match_char('=');
        self.expression();
        self.emitln("SUBQ #1,D0");
        self.emitln(&format!("LEA {}(PC),A0", name));
        self.emitln("MOVE DO,(A0)");
        self.expression();
        self.emitln("MOVE DO,-(SP)");
        self.post_label(&l1);
        self.emitln(&format!("LEA {}(PC),A0", name));
        self.emitln("MOVE (A0),D0");
        self.emitln("MOVE #1,D0");
        self.emitln("MOVE DO,(A0)");
        self.emitln("CMP (SP),(A0)");
        self.emitln(&format!("BGT {}", l2));
        self.block(&l2);
        self.match_char('e');
        self.emitln(&format!("BRA {}", l1));
        self.post_label(&l2);
        self.emitln("ADDQ #2,SP");
    }

    /// Parse and Translate a DO Statement
    pub fn do_do(&mut self) {
        self.match_char('d');
        let l1 = self.new_label();
        let l2 = self.new_label();
        self.expression();
        self.emitln("SUBQ #1,D0");
        self.post_label(&l1);
        self.emitln("MOVE D0,-(SP)");
        self.block(&l2);
        self.emitln("MOVE (SP)+,D0");
        self.emitln(&format!("DBRA D0,{}", l1));
        self.emitln("SUBQ #2,SP");
        self.post_label(&l2);
        self.emitln("ADDQ #2,SP");
    }

    /// Generate a Unique Label
    pub fn new_label(&mut self) -> String {
        let label = format!("L{}", &usize::to_string(&self.lcount));
        self.lcount += 1;
        label
    }

    /// Post a label to Output
    pub fn post_label(&mut self, label: &str) {
        println!("{}:", label);
    }
}

pub fn expected(x: &str) {
    panic!("{} Expected", x);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_control_constructs() {
        let inp = b"afi=xikbeece\n";
        let mut c = Cradle::new(&inp[..]);
        c.do_program();
    }
}
