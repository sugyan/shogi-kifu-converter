use shogi_core::ToUsi;
use shogi_kifu_converter::error::ConvertError;
use shogi_kifu_converter::parser::parse_jkf_file;
use std::env;

fn main() -> Result<(), ConvertError> {
    let argv = env::args().collect::<Vec<_>>();
    if argv.len() != 2 {
        eprintln!("Usage: {} <JKF file>", argv[0]);
        std::process::exit(1);
    }
    match parse_jkf_file(&argv[1]) {
        Ok(jkf) => {
            println!("{}", jkf.to_usi_owned());
        }
        Err(err) => {
            eprintln!("{err}");
            std::process::exit(1);
        }
    }
    Ok(())
}
