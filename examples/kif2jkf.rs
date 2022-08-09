use shogi_kifu_converter::error::ParseError;
use shogi_kifu_converter::parser::parse_kif_file;
use std::env;

fn main() -> Result<(), ParseError> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <KIF file>", argv[0]);
        std::process::exit(1);
    }
    let jkf = parse_kif_file(&argv[1])?;
    println!("{}", serde_json::to_string_pretty(&jkf)?);
    Ok(())
}
