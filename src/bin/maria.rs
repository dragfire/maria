use maria::{self, Cradle};

fn main() {
    let stdio = std::io::stdin();
    let input = stdio.lock();
    let mut c = Cradle::new(input);
    c.expression();
}
