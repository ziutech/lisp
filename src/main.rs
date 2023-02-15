use std::error::Error;
use std::io::{BufRead, Write};

use std::result::Result;

fn read(str: &str) -> &str {
    str
}
fn eval(str: &str) -> &str {
    str
}
fn print(str: &str) -> &str {
    str
}
fn rep(str: &str) -> &str {
    print(eval(read(str)))
}

fn _loop<R: BufRead, W: Write>(bufin: &mut R, bufout: &mut W) -> Result<(), Box<dyn Error>> {
    loop {
        // TODO: line editing
        // TODO: repl history
        write!(bufout, "user> ")?;
        bufout.flush()?;
        let mut input = String::new();
        if bufin.read_line(&mut input)? == 0 {
            break;
        }
        let output = rep(&input);
        writeln!(bufout, "{}", output)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    // TODO: remove lock on stdin (?)
    let mut bufin = std::io::stdin().lock();
    let mut bufout = std::io::stdout();
    _loop(&mut bufin, &mut bufout)?;
    Ok(())
}
