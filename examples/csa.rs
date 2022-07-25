use serde_json::Result;
use shogi_kifu_converter::parser::parse_csa;
use std::env;

fn main() -> Result<()> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <CSA file>", argv[0]);
        std::process::exit(1);
    }
    match parse_csa(&argv[1]) {
        Ok(jkf) => println!("{}", serde_json::to_string_pretty(&jkf)?),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
    Ok(())
}