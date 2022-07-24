use argon2::password_hash::SaltString;
use argon2::{PasswordHash, PasswordHasher, Argon2, PasswordVerifier};
use anyhow::Error;
use secrecy::{Secret, ExposeSecret};

use crate::{setting_struct::SettingStruct, database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB}};


pub struct UserCredentials {
    // These two fields were not marked as `pub` before!
    pub username: String,
    pub password: Secret<String>,
}

pub struct UserCredentialsHashed {
    // These two fields were not marked as `pub` before!
    pub username: String,
    pub password_hash: Secret<String>,
}

#[derive(Debug)]
pub struct StoredCredentials {
    // These two fields were not marked as `pub` before!
    pub user_id: uuid::Uuid,
    pub password_hash: Secret<String>
}

pub async fn validate_credentials(
    credentials: &UserCredentials,

) -> Result<uuid::Uuid, Error> {
    let mut _user_id:Option<uuid::Uuid>=None;
    let mut _expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
        gZiV/M1gPc22ElAH/Jh1Hw$\
        CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string()
    );

    let get_result =  get_stored_credentials(&credentials.username).await;
    if get_result.is_err(){
       return Err(anyhow::anyhow!("Problem getting credentials"));
    }
    
    let stored_credentials = get_result.unwrap();
    let user_id = Some(stored_credentials.user_id);

    let expected_password_hash = stored_credentials.password_hash;

    

    let verify_result =  verify_password_hash(&expected_password_hash, &credentials.password);  
    if verify_result.is_err(){
       return Err(verify_result.unwrap_err());
    }

    if user_id.is_some()
    {
        return Ok(user_id.unwrap());
    }
    Err(anyhow::anyhow!("Unknown username."))
}

async fn get_stored_credentials(_user_id: &str) -> Result<StoredCredentials,Error>{
    let local_setting:SettingStruct = SettingStruct::global().clone();
    let _db_connection=DbConnectionSetting{
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password) ,
        instance: String::from(&local_setting.backend_database_instance)
    };

    let query_credentials = DbHandlerMongoDB::get_stored_credentials_by_name(&_db_connection, &_user_id.to_string()).await;

    if query_credentials.is_err()
    {return Err(anyhow::anyhow!(query_credentials.unwrap_err()));}

    let some_credential = query_credentials.unwrap();

    Ok(some_credential)
}

fn verify_password_hash(
    expected_password_hash: &Secret<String>,
    password_candidate: &Secret<String>,
) -> Result<(), Error> {
    let expected_password_hash_2 = PasswordHash::new(
        &expected_password_hash.expose_secret()
    );
    if expected_password_hash_2.is_err(){
       return Err(anyhow::anyhow!("Failed to parse hash in PHC string format."));
    }

    let check_result=Argon2::default()
        .verify_password(
            password_candidate.expose_secret().as_bytes(), 
            &expected_password_hash_2.unwrap()
        );
    if check_result.is_err(){
        return Err(anyhow::anyhow!("AUTH ERROR: {}",check_result.unwrap_err()));
    }

Ok(())
    

}

pub async fn create_credentials(
    credentials: &UserCredentials,

) -> Result<uuid::Uuid, Error> {

    let check_result = check_user_exsits(&credentials.username).await;
    if check_result.is_err()
    {
        return Err(check_result.unwrap_err());
    }

    let user_exsists = check_result.unwrap();
    if user_exsists
    {
        return Err(anyhow::anyhow!("User already exsists, can not recreate"));
    }

    let insert_result = insert_user(&credentials).await;
    if insert_result.is_err()
    {return Err(insert_result.unwrap_err());}

    return Ok(insert_result.unwrap())
}

async fn check_user_exsits(user_name: &str) -> Result<bool,Error>{
    let local_setting:SettingStruct = SettingStruct::global().clone();
    let db_connection=DbConnectionSetting{
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password) ,
        instance: String::from(&local_setting.backend_database_instance)
    };

    let check_result =  DbHandlerMongoDB::check_user_exsists_by_name(&db_connection,&user_name.to_string()).await;

    if check_result.is_err(){return Err(anyhow::anyhow!(check_result.unwrap_err()));}

    Ok(check_result.unwrap())
}

async fn insert_user(some_credentials:&UserCredentials) -> Result<uuid::Uuid,Error>
{    
    let local_setting:SettingStruct = SettingStruct::global().clone();
    let db_connection=DbConnectionSetting{
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password) ,
        instance: String::from(&local_setting.backend_database_instance)
    };

    let salt = SaltString::generate(&mut rand::thread_rng());
    let user_password_hashed = Argon2::default()
            .hash_password(some_credentials.password.expose_secret().as_bytes(), &salt)
            .unwrap()
            .to_string();

    let some_credentials_hashed = UserCredentialsHashed
    {
        username: some_credentials.username.clone(),
        password_hash:Secret::new(user_password_hashed)
    };

    let insert_result = DbHandlerMongoDB::insert_user(&db_connection, &some_credentials_hashed).await;

    if insert_result.is_err(){return Err(anyhow::anyhow!(insert_result.unwrap_err()));}

    Ok(insert_result.unwrap())
}