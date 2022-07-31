use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    White = 1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    FU = 0,
    KY = 1,
    KE = 2,
    GI = 3,
    KI = 4,
    KA = 5,
    HI = 6,
    OU = 7,
    TO = 8,
    NY = 9,
    NK = 10,
    NG = 11,
    UM = 12,
    RY = 13,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    #[serde(rename = "HIRATE")]
    PresetHirate, // 平手
    #[serde(rename = "KY")]
    PresetKY, // 香落ち
    #[serde(rename = "KY_R")]
    PresetKYR, // 右香落ち
    #[serde(rename = "KA")]
    PresetKA, // 角落ち
    #[serde(rename = "HI")]
    PresetHI, // 飛車落ち
    #[serde(rename = "HIKY")]
    PresetHIKY, // 飛香落ち
    #[serde(rename = "2")]
    Preset2, // 二枚落ち
    #[serde(rename = "3")]
    Preset3, // 三枚落ち
    #[serde(rename = "4")]
    Preset4, // 四枚落ち
    #[serde(rename = "5")]
    Preset5, // 五枚落ち
    #[serde(rename = "5_L")]
    Preset5L, // 左五枚落ち
    #[serde(rename = "6")]
    Preset6, // 六枚落ち
    #[serde(rename = "7_L")]
    Preset7L, // 左七枚落ち
    #[serde(rename = "7_R")]
    Preset7R, // 右七枚落ち
    #[serde(rename = "8")]
    Preset8, // 八枚落ち
    #[serde(rename = "10")]
    Preset10, // 十枚落ち
    #[serde(rename = "OTHER")]
    PresetOther, // その他
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relative {
    L, // 左
    C, // 直
    R, // 右
    U, // 上
    M, // 寄
    D, // 引
    LU,
    LD,
    RU,
    RD,
    H, // 打
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum MoveSpecial {
    #[serde(rename = "TORYO")]
    SpecialToryo, // 投了
    #[serde(rename = "CHUDAN")]
    SpecialChudan, // 中断
    #[serde(rename = "SENNICHITE")]
    SpecialSennichite, // 千日手
    #[serde(rename = "TIME_UP")]
    SpecialTimeUp, // 手番側が時間切れで負け、切れ負け
    #[serde(rename = "ILLEGAL_MOVE")]
    SpecialIllegalMove, // 反則負け
    #[serde(rename = "+ILLEGAL_ACTION")]
    SpecialIllegalActionBlack, // 先手(下手)の反則行為により、後手(上手)の勝ち
    #[serde(rename = "-ILLEGAL_ACTION")]
    SpecialIllegalActionWhite, // 後手(上手)の反則行為により、先手(下手)の勝ち
    #[serde(rename = "JISHOGI")]
    SpecialJishogi, // 持将棋
    #[serde(rename = "KACHI")]
    SpecialKachi, // (入玉で)勝ちの宣言
    #[serde(rename = "HIKIWAKE")]
    SpecialHikiwake, // (入玉で)引き分けの宣言
    #[serde(rename = "MATTA")]
    SpecialMatta, // 待った
    #[serde(rename = "TSUMI")]
    SpecialTsumi, // 詰み
    #[serde(rename = "FUZUMI")]
    SpecialFuzumi, // 不詰
    #[serde(rename = "ERROR")]
    SpecialError, // エラー
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct JsonKifuFormat {
    pub header: HashMap<String, String>,
    pub initial: Option<Initial>,
    pub moves: Vec<MoveFormat>,
}

impl Default for JsonKifuFormat {
    fn default() -> Self {
        JsonKifuFormat {
            header: HashMap::new(),
            initial: None,
            moves: vec![MoveFormat::default()],
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Initial {
    pub preset: Preset,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<StateFormat>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct StateFormat {
    pub color: Color,
    pub board: [[Piece; 9]; 9],
    pub hands: [Hand; 2],
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[allow(non_snake_case)]
pub struct Hand {
    pub FU: u8,
    pub KY: u8,
    pub KE: u8,
    pub GI: u8,
    pub KI: u8,
    pub KA: u8,
    pub HI: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Piece {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<Kind>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct MoveFormat {
    #[serde(rename = "move")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_: Option<MoveMoveFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Time>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special: Option<MoveSpecial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forks: Option<Vec<Vec<MoveFormat>>>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MoveMoveFormat {
    pub color: Color,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<PlaceFormat>,
    pub to: PlaceFormat,
    pub piece: Kind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub same: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promote: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capture: Option<Kind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relative: Option<Relative>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlaceFormat {
    pub x: u8,
    pub y: u8,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Time {
    pub now: TimeFormat,
    pub total: TimeFormat,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TimeFormat {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub h: Option<u8>,
    pub m: u8,
    pub s: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{parse_csa_file, parse_kif_file};
    use jsonschema::JSONSchema;
    use std::ffi::OsStr;
    use std::fs::{DirEntry, File};
    use std::io::{BufReader, Result};
    use std::path::Path;

    fn load_schema() -> Result<JSONSchema> {
        let file = File::open("data/specification/json-kifu-format.schema.json")?;
        Ok(JSONSchema::compile(
            &serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file))
                .expect("failed to parse JSON"),
        )
        .expect("failed to compile schema"))
    }

    fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry) -> Result<()>) -> Result<()> {
        if dir.is_dir() {
            for entry in dir.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry)?;
                }
            }
        }
        Ok(())
    }

    #[test]
    fn deserialize() -> Result<()> {
        visit_dirs(Path::new("data/tests"), &|entry: &DirEntry| -> Result<()> {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("json")) {
                let file = File::open(&path)?;
                let reader = BufReader::new(file);
                assert!(
                    serde_json::from_reader::<_, JsonKifuFormat>(reader).is_ok(),
                    "failed to deserialize: {}",
                    path.display()
                );
            }
            Ok(())
        })
    }

    #[test]
    fn validate_default() -> Result<()> {
        let schema = load_schema()?;

        let value = serde_json::to_value(&JsonKifuFormat::default()).expect("failed to serialize");
        let result = schema.validate(&value);
        if let Err(errors) = result {
            for err in errors {
                panic!("{:?}", err);
            }
        }
        Ok(())
    }

    #[test]
    fn validate_from_files() -> Result<()> {
        let schema = load_schema()?;

        visit_dirs(Path::new("data/tests"), &|entry: &DirEntry| -> Result<()> {
            let path = entry.path();
            let jkf = match path.extension().and_then(|s| s.to_str()) {
                Some("csa") => parse_csa_file(&path).expect("failed to parse csa file"),
                Some("kif") => parse_kif_file(&path).expect("failed to parse kif file"),
                _ => return Ok(()),
            };
            let value = serde_json::to_value(&jkf).expect("failed to serialize");
            let result = schema.validate(&value);
            if let Err(errors) = result {
                for err in errors {
                    panic!("error on {}: {:?}", path.display(), err);
                }
            }
            Ok(())
        })
    }
}
