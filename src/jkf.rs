use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Default)]
pub struct JsonKifFormat {
    header: HashMap<String, String>,
    moves: Vec<MoveFormat>,
}

#[derive(Serialize, Default)]
pub struct MoveFormat {}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonschema::JSONSchema;
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn validate_jkf() {
        let file = File::open("data/specification/json-kifu-format.schema.json")
            .expect("Failed to open file");
        let schema = JSONSchema::compile(
            &serde_json::from_reader::<_, serde_json::Value>(BufReader::new(file))
                .expect("Failed to parse JSON"),
        )
        .expect("Failed to compile schema");

        let value = serde_json::to_value(&JsonKifFormat::default()).expect("Failed to serialize");
        let result = schema.validate(&value);
        if let Err(errors) = result {
            for err in errors {
                panic!("{:?}", err);
            }
        }
    }
}
