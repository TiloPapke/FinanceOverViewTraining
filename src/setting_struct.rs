use std::path::PathBuf;

use ini::Ini;
use once_cell::sync::OnceCell;

#[derive(Copy, Clone)]
pub struct SettingStruct {
    pub web_server_ip_part1:u8,
    pub web_server_ip_part2:u8,
    pub web_server_ip_part3:u8,
    pub web_server_ip_part4:u8,
    pub web_server_port:u16 
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
            .set("GITWARNING2", "Only push to repository if this file does not contain any private information");
        conf.with_section(Some("WebServer"))
            .set("ip_part1", "127")
            .set("ip_part2", "0")
            .set("ip_part3", "0")
            .set("ip_part4", "1")
            .set("port", "3000");
        conf.write_to_file(&settingpath).unwrap();
    }

    pub fn load_from_file(settingpath:&PathBuf) -> Self
    {
        let conf:Ini = Ini::load_from_file(&settingpath).unwrap();
        let _web_server_ip_part1:u8 = conf.get_from_or(Some("WebServer"),"ip_part1","127").parse().unwrap();
        let _web_server_ip_part2:u8 = conf.get_from_or(Some("WebServer"),"ip_part2","0").parse().unwrap();
        let _web_server_ip_part3:u8 = conf.get_from_or(Some("WebServer"),"ip_part3","0").parse().unwrap();
        let _web_server_ip_part4:u8 = conf.get_from_or(Some("WebServer"),"ip_part4","1").parse().unwrap();
        let _web_server_port:u16 = conf.get_from_or(Some("WebServer"),"port","3000").parse().unwrap();

        return SettingStruct { 
            web_server_ip_part1: _web_server_ip_part1,
            web_server_ip_part2: _web_server_ip_part2,
            web_server_ip_part3: _web_server_ip_part3,
            web_server_ip_part4: _web_server_ip_part4,
            web_server_port: _web_server_port 
        };
    }
}