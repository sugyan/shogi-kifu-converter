use shogi_kifu_converter::converter::ToKif;
use shogi_kifu_converter::error::ConvertError;
use shogi_kifu_converter::parser::parse_csa_file;
use std::env;

fn main() -> Result<(), ConvertError> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <CSA file>", argv[0]);
        std::process::exit(1);
    }
    match parse_csa_file(&argv[1]) {
        Ok(jkf) => print!("{}", jkf.to_kif_owned()),
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
    Ok(())
}
