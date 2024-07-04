use futures::{executor, StreamExt};
use mongodb::bson::Document;

pub struct MdbConvertTools{

}

impl MdbConvertTools{
    pub fn get_vector_from_cursor(cursor: mongodb::Cursor<Document>) -> Vec<Document>
    {
        let convert_result= executor::block_on(MdbConvertTools::get_vector_from_cursor_async(cursor));
        if convert_result.is_ok()
            {return convert_result.unwrap();}

        return Vec::new();
        
    }

    pub async fn get_vector_from_cursor_async(mut cursor: mongodb::Cursor<Document>) -> Result<Vec<Document>, Box<dyn std::error::Error>>
    {
        let mut docs= Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(document) => {
                        docs.push(document);
                                }
                Err(e) => return Err(Box::new(e)),
            }
        }
        return Ok(docs)
        
    }

    pub fn get_binary_from_bson_uuid(input:&mongodb::bson::Uuid)->mongodb::bson::Binary{
        let result_value = mongodb::bson::Binary::from_uuid(input.clone());
        return result_value
    }

}