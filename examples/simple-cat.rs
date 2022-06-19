use std::io::{self, Read, Write};

const BUFFER_SIZE: usize = 64 * 1024;

fn main() -> io::Result<()>{
    let stdin = &mut io::stdin();
    let stdout = &mut io::stdout();
    let mut buffer= vec![0; BUFFER_SIZE];

    loop {
        let len_read = stdin.read(&mut buffer)?;
        if len_read == 0 {
            return Ok(());
        }
        stdout.write_all(&buffer[..len_read])?;
    }
}