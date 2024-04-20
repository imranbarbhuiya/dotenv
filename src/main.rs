use dotenv::parse_content;
use std::{env, fs};

fn main() -> std::io::Result<()> {
    let file_path = env::args().nth(1).unwrap_or(".env".to_string());
    let file_contents = fs::read_to_string(file_path).unwrap_or_else(|err| {
        panic!("Problem parsing arguments: {err}");
    });

    let values = parse_content(&file_contents);

    for (key, value) in values {
        println!("{} == {}\n", key, value);
    }

    Ok(())
}
