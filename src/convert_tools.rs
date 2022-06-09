use mongodb::bson::Document;
use uuid;

pub struct ConvertTools{

}

impl ConvertTools{
    pub fn get_uuid_from_document(a_doc:&Document, field_name:&str)-> Result<uuid::Uuid,String>
    {
        let bson_option =a_doc.get(field_name);   
        if bson_option.is_none()
        {return Err(format!("field {} is empty",field_name));}
        let bson_var =  bson_option.unwrap();
        let uuid_bson_parse_result:Result<mongodb::bson::Uuid,mongodb::bson::de::Error> = mongodb::bson::from_bson(bson_var.clone());
        if uuid_bson_parse_result.is_err()
        {return Err(uuid_bson_parse_result.unwrap_err().to_string())}
        
        let uuid_var =uuid_bson_parse_result.unwrap().to_uuid_0_8();

        return Ok(uuid_var);
    }
}