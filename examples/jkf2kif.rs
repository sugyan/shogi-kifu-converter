use shogi_kifu_converter::converter::ToKif;
use shogi_kifu_converter::error::ParseError;
use shogi_kifu_converter::parser::parse_jkf_file;
use std::env;

fn main() -> Result<(), ParseError> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <JKF file>", argv[0]);
        std::process::exit(1);
    }
    let jkf = parse_jkf_file(&argv[1])?;
    print!("{}", jkf.to_kif_owned());
    Ok(())
}
