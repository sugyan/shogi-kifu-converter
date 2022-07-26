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
    LM,
    LD,
    CU,
    CD,
    RU,
    RM,
    RD,
    H, // 打
}

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct JsonKifFormat {
    pub header: HashMap<String, String>,
    pub initial: Option<Initial>,
    pub moves: Vec<MoveFormat>,
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

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct MoveFormat {
    #[serde(rename = "move")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub move_: Option<MoveMoveFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comments: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time: Option<Time>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub special: Option<String>,
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

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaceFormat {
    pub x: u8,
    pub y: u8,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
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
    use jsonschema::JSONSchema;
    use std::ffi::OsStr;
    use std::fs::{DirEntry, File};
    use std::io::{BufReader, Read, Result};
    use std::path::Path;

    fn load_schema() -> JSONSchema {
        let file = File::open("data/specification/json-kifu-format.schema.json")
            .expect("failed to open file");
        JSONSchema::compile(
            &serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file))
                .expect("failed to parse JSON"),
        )
        .expect("failed to compile schema")
    }

    fn visit_dirs(dir: &Path, cb: &dyn Fn(&DirEntry)) -> Result<()> {
        if dir.is_dir() {
            for entry in dir.read_dir()? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    visit_dirs(&path, cb)?;
                } else {
                    cb(&entry);
                }
            }
        }
        Ok(())
    }

    #[test]
    fn deserialize() {
        visit_dirs(Path::new("data/tests"), &|entry: &DirEntry| {
            let path = entry.path();
            if path.extension() == Some(OsStr::new("json")) {
                let file = File::open(&path).expect("failed to open file");
                assert!(
                    serde_json::from_reader::<_, JsonKifFormat>(BufReader::new(file)).is_ok(),
                    "failed to deserialize: {}",
                    path.display()
                );
            }
        })
        .expect("failed to visit dirs");
    }

    #[test]
    fn validate_default() {
        let schema = load_schema();

        let value = serde_json::to_value(&JsonKifFormat::default()).expect("failed to serialize");
        let result = schema.validate(&value);
        if let Err(errors) = result {
            for err in errors {
                panic!("{:?}", err);
            }
        }
    }

    #[test]
    fn validate_from_csa_files() {
        let schema = load_schema();

        for entry in Path::new("data/tests/csa")
            .read_dir()
            .expect("failed to read dir")
        {
            let entry = entry.expect("failed to read entry");
            let path = entry.path();
            if path.extension() == Some(OsStr::new("csa")) {
                let mut file = File::open(&path).expect("failed to open file");
                let mut buf = String::new();
                file.read_to_string(&mut buf).expect("failed to read file");
                let record = csa::parse_csa(&buf).expect("failed to parse csa");
                let value = serde_json::to_value(&JsonKifFormat::from(record))
                    .expect("failed to serialize");
                let result = schema.validate(&value);
                if let Err(errors) = result {
                    for err in errors {
                        panic!("error on {}: {:?}", path.display(), err);
                    }
                }
            }
        }
    }
}
