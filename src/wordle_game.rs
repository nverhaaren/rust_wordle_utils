use std::env;
use wordle_utils::{Clue, checkers};

use std::io;
use colored::Colorize;
use itertools::Itertools;

fn print_clue(guess: &str, clues: &[Clue]) {
    let mut buffer = [0; 4];
    for (gch, clue) in guess.chars().zip_eq(clues) {
        match clue {
            Clue::Absent => print!("{}", gch),
            Clue::Present => print!("{}", gch.encode_utf8(&mut buffer).on_yellow()),
            Clue::Exact => print!("{}", gch.encode_utf8(&mut buffer).on_green()),
        }
    }
    println!()
}

fn main() -> io::Result<()>{
    let args: Vec<String> = env::args().collect();
    assert_eq!(args.len(), 2, "Usage: {} WORD", args[0]);
    let word = &args[1];
    // Probably a good place to review 'logically' immutable types with interior mutability.
    let mut checker = checkers::Solution::new(word.clone());

    let stdin = &mut io::stdin();
    let mut buffer = String::with_capacity(word.len() + 1);
    loop {
        let len_read = stdin.read_line(&mut buffer)?;
        if len_read == 0 {
            return Ok(());
        }
        let trimmed = buffer.strip_suffix('\n').unwrap_or(&buffer);
        if trimmed.len() != word.len() {
            println!("Incorrect guess length, expected {}", word.len());
        } else {
            print_clue(trimmed, &checker.check(trimmed).collect::<Vec<_>>());
        }
        buffer.clear();
    }
}
