use std::path::PathBuf;

use ini::Ini;
use once_cell::sync::OnceCell;

#[derive(Clone)]
pub struct SettingStruct {
    pub web_server_ip_part1:u8,
    pub web_server_ip_part2:u8,
    pub web_server_ip_part3:u8,
    pub web_server_ip_part4:u8,
    pub web_server_port_http:u16,
    pub web_server_port_https:u16,
    pub backend_database_url:String,
    pub backend_database_user:String,
    pub backend_database_password:String,
    pub backend_database_instance:String
}

pub static GLOBAL_SETTING: OnceCell<SettingStruct> = OnceCell::new();

impl SettingStruct{
    pub fn global() -> &'static SettingStruct {
        GLOBAL_SETTING.get().expect("GLOBAL_SETTING is not initialized")
    }

    pub fn create_dummy_setting(settingpath:&PathBuf)
    {
        let mut conf:Ini = Ini::new();
        conf.with_section(Some("[WARNING]"))
            .set("GITWARNING", "This is a default config file, when entering own value be sure to add this file to ignore")
            .set("GITWARNING2", "Only push to repository if this file does not contain any private information")
            .set("Renaming", "Use this file as tempalte to create your own ServerSettings.ini file");
        conf.with_section(Some("WebServer"))
            .set("ip_part1", "127")
            .set("ip_part2", "0")
            .set("ip_part3", "0")
            .set("ip_part4", "1")
            .set("port_http", "3000")
            .set("port_https", "3300");
        conf.with_section(Some("BackendDatabase"))
            .set("DB_URL", "mongodb://localhost:27017")
            .set("DB_User", "Administrator")
            .set("DB_Password", "password")
            .set("DB_Instance", "StructureName");
        conf.write_to_file(&settingpath).unwrap();
    }

    pub fn load_from_file(settingpath:&PathBuf) -> Self
    {
        let conf:Ini = Ini::load_from_file(&settingpath).unwrap();
        let _web_server_ip_part1:u8 = conf.get_from_or(Some("WebServer"),"ip_part1","127").parse().unwrap();
        let _web_server_ip_part2:u8 = conf.get_from_or(Some("WebServer"),"ip_part2","0").parse().unwrap();
        let _web_server_ip_part3:u8 = conf.get_from_or(Some("WebServer"),"ip_part3","0").parse().unwrap();
        let _web_server_ip_part4:u8 = conf.get_from_or(Some("WebServer"),"ip_part4","1").parse().unwrap();
        let _web_server_port_http:u16 = conf.get_from_or(Some("WebServer"),"port_http","3000").parse().unwrap();
        let _web_server_port_https:u16 = conf.get_from_or(Some("WebServer"),"port_https","3300").parse().unwrap();
        let _db_url:String = conf.get_from_or(Some("BackendDatabase"),"DB_URL","").to_string();
        let _db_user:String = conf.get_from_or(Some("BackendDatabase"),"DB_User","").to_string();
        let _db_password:String = conf.get_from_or(Some("BackendDatabase"),"DB_Password","").to_string();
        let _db_instance:String = conf.get_from_or(Some("BackendDatabase"),"DB_Instance","").to_string();

        return SettingStruct { 
            web_server_ip_part1: _web_server_ip_part1,
            web_server_ip_part2: _web_server_ip_part2,
            web_server_ip_part3: _web_server_ip_part3,
            web_server_ip_part4: _web_server_ip_part4,
            web_server_port_http: _web_server_port_http,
            web_server_port_https: _web_server_port_https,
            backend_database_url: _db_url,
            backend_database_user:_db_user,
            backend_database_password:_db_password,
            backend_database_instance:_db_instance
        };
    }
}