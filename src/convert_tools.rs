use std::path::PathBuf;

use mongodb::bson::Document;
use uuid;

pub struct ConvertTools {}

impl ConvertTools {
    pub fn get_uuid_from_document(
        a_doc: &Document,
        field_name: &str,
    ) -> Result<uuid::Uuid, String> {
        let bson_option = a_doc.get(field_name);
        if bson_option.is_none() {
            return Err(format!("field {} is empty", field_name));
        }
        let bson_var = bson_option.unwrap();
        let uuid_bson_parse_result: Result<mongodb::bson::Uuid, mongodb::bson::de::Error> =
            mongodb::bson::from_bson(bson_var.clone());
        if uuid_bson_parse_result.is_err() {
            return Err(uuid_bson_parse_result.unwrap_err().to_string());
        }

        let uuid_var = uuid_bson_parse_result.unwrap().to_uuid_0_8();

        return Ok(uuid_var);
    }

    pub fn load_text_from_file(filepath: &PathBuf) -> Result<String, String> {
        let file = std::fs::File::open(filepath);
        if file.is_err() {
            Err(format!("Error accessing file: {}", file.unwrap_err()))
        } else {
            let mut file_content = String::new();
            let read_result = std::io::Read::read_to_string(&mut file.unwrap(), &mut file_content);
            if read_result.is_err() {
                return Err(format!("Error readin file: {}", read_result.unwrap_err()));
            }

            return Ok(file_content);
        }
    }
}
