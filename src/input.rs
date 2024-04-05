use std::io::{stdin, stdout, Write};

pub fn get_input(prompt: &str) -> String {
    print!("{}", prompt);
    stdout().flush().expect("");

    let mut input = String::new();
    stdin().read_line(&mut input).expect("Failed to read input");

    input
}