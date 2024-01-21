use serde::Deserialize;


#[derive(Deserialize, Debug)]
pub struct GenerallUserData{
    pub first_name:String,
    pub last_name:String,
    pub reset_secret:String,
}