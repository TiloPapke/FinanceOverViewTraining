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

    //need to escape characters for url: https://docs.microfocus.com/OMi/10.62/Content/OMi/ExtGuide/ExtApps/URL_encoding.htm
    pub fn escape_htmltext(input: &String) -> String {
        let internal_input = input.clone();
        let mut vec_text: Vec<String> = Vec::new();
        for c in internal_input.chars() {
            match c {
                ' ' => vec_text.push("%20".to_string()),
                '<' => vec_text.push("%3C".to_string()),
                '>' => vec_text.push("%3E".to_string()),
                '#' => vec_text.push("%23".to_string()),
                '%' => vec_text.push("%25".to_string()),
                '+' => vec_text.push("%2B".to_string()),
                '{' => vec_text.push("%7B".to_string()),
                '}' => vec_text.push("%7D".to_string()),
                '|' => vec_text.push("%7C".to_string()),
                '\\' => vec_text.push("%5C".to_string()),
                '^' => vec_text.push("%5E".to_string()),
                '~' => vec_text.push("%7E".to_string()),
                '[' => vec_text.push("%5B".to_string()),
                ']' => vec_text.push("%5D".to_string()),
                '\'' => vec_text.push("%60".to_string()),
                ';' => vec_text.push("%3B".to_string()),
                '/' => vec_text.push("%2F".to_string()),
                '?' => vec_text.push("%3F".to_string()),
                ':' => vec_text.push("%3A".to_string()),
                '@' => vec_text.push("%40".to_string()),
                '=' => vec_text.push("%3D".to_string()),
                '&' => vec_text.push("%26".to_string()),
                '$' => vec_text.push("%24".to_string()),
                _ => {
                    vec_text.push(c.to_string());
                }
            }
        }
        let result = vec_text.concat();
        return result;
    }
}
