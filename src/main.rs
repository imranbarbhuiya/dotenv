use dotenv::parse_content;
use std::fs;

fn main() -> std::io::Result<()> {
    let file_contents = fs::read_to_string(".env")?;

    println!("{}\n", &file_contents);

    let store = parse_content(&file_contents);

    for (key, value) in store {
        println!("{} == {}\n", key, value);
    }

    Ok(())
}
