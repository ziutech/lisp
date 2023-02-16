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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::File;
    use std::io::{BufReader, Cursor, Read};

    use crate::rep;
    use std::iter::zip;
    fn parse_test(str: &String) -> (String, String) {
        let mut input = String::new();
        let mut output = String::new();
        for line in str.lines() {
            match line.get(0..=0) {
                Some(";") => match line.get(1..=1) {
                    Some(";") => continue,
                    Some(">") => continue,
                    Some("=") => {
                        output.push_str(line.strip_prefix(";=>").unwrap());
                        output.push_str("\n");
                    }
                    Some(_) => continue,
                    None => continue,
                },
                Some(_) => {
                    input.push_str(line);
                    input.push_str("\n");
                }
                None => continue,
            }
        }
        (input, output)
    }

    #[test]
    fn from_file() {
        let dir = env::var("CARGO_MANIFEST_DIR").unwrap() + "/tests/step0_repl.mal";
        let mut file = File::open(dir).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();
        let (input_str, expected_output_str) = parse_test(&contents);
        for (i, exp) in zip(input_str.lines(), expected_output_str.lines()) {
            assert_eq!(exp, rep(i));
        }
    }
}
