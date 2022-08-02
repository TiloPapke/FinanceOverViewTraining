use askama::Template;
use async_session::SessionStore;
use axum::{response::{Html, Response, IntoResponse, Redirect}, http::{StatusCode, HeaderMap}, extract::Form};
use log::debug;
use secrecy::Secret;
use serde::Deserialize;

use crate::{password_handle::{validate_credentials, UserCredentials, create_credentials}, session_data_handle::{SessionDataResult, SessionData}};

#[derive(Template)]
#[template(path = "WelcomePage.html")]
pub struct MainPageTemplate {
   pub web_running_port: u16,
   pub additional_info:String,
   pub called_times:i32
}


#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LoginFormInput {
    username: String,
    password: Secret<String>,
}

#[derive(Template)]
#[template(path = "UserHome.html")]
pub struct UserHomeTemplate{
    username: String,
    session_expire_timestamp: String,
    logged_in:bool,
    logout_reason:String
}
pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => {
                let mut headers = HeaderMap::new();
                headers.remove(axum::http::header::SET_COOKIE);
                (   StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    format!("Failed to render template. Error: {}", err),
                )
                .into_response()
            },
        }
    }
}

pub async fn accept_login_form(session_data: SessionDataResult, Form (input): Form<LoginFormInput>)  -> impl IntoResponse    {
    
    let credentials = UserCredentials {
        username: input.username.clone(),
        password: input.password.clone(),
    };


    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap();
    let _result = session.insert("user_name", &credentials.username);

    let a_store= session_data.session_store;

    let _cookie3 = a_store.store_session(session).await;

    match validate_credentials(&credentials).await {
        Ok(user_id) => {
            debug!(target: "app::FinanceOverView","user_id is {}",user_id);
            Redirect::to("/user_home").into_response()
        }
        Err(_) => {
            debug!(target: "app::FinanceOverView","no valid user name");
            Redirect::to("/invalid").into_response()
        }
    }


}

pub async fn user_home_handler(session_data: SessionDataResult)  -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let session = session_data.session_option.unwrap();
    
    let username:String = session.get("user_name").unwrap();

    let session_expire_timestamp = format!("{} seconds", session.expires_in().unwrap_or(std::time::Duration::new(1,0)).as_secs());

    if session.is_expired()
    {
        let template = UserHomeTemplate{
            logout_reason: "Session expired".to_string(),
            username :"".to_string(),
            session_expire_timestamp,
            logged_in: false,
        };
        HtmlTemplate(template)
    }
    else
    {
        let template = UserHomeTemplate { 
            username: username.to_string(),
            session_expire_timestamp,
            logged_in: true,
            logout_reason: "".to_string()
        };
        HtmlTemplate(template)
    }
}

pub async fn do_logout_handler(session_data: SessionDataResult)  -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap(); 

    let session_expire_timestamp = format!("{} seconds", session.expires_in().unwrap_or(std::time::Duration::new(1,0)).as_secs());

    session.destroy();

    let template = UserHomeTemplate { 
        username: "".to_string(),
        session_expire_timestamp,
        logged_in: false,
        logout_reason: "You logout yourself".to_string()
    };
    HtmlTemplate(template)
}

#[derive(Template)]
#[template(path = "InvalidUser.html")]
pub struct InvalidTemplate{
    username: String 
}

pub async fn invalid_handler()  -> impl IntoResponse  {

    let username="chosen login";

    let st = InvalidTemplate{username:format!("{} is invalid", username)};
        HtmlTemplate(st)

}

#[derive(Template)]
#[template(path = "CreateLogin.html")]
pub struct CreateLoginTemplate{
    user_name: String,
    create_result:String 
}

pub async fn create_login_handler(form: Form<LoginFormInput>)  -> impl IntoResponse {
    debug!(target: "app::FinanceOverView","create_login data user {} with {:?}",&form.username,form.password);

    let mut clt_template = CreateLoginTemplate{user_name:"not_set".to_string(), create_result:"no Result given".to_string()};

    let new_user_credentials = UserCredentials{
        username: form.username.to_string(), 
        password: form.password.clone()
    };


    let create_result=create_credentials(&new_user_credentials).await;
    if create_result.is_err()
    {
        clt_template.user_name=new_user_credentials.username.to_string();
        clt_template.create_result=create_result.unwrap_err().to_string();
    }
    else
    {
        clt_template.user_name=new_user_credentials.username.to_string();
        clt_template.create_result=format!("your user id is {}", create_result.unwrap());
    }


    HtmlTemplate(clt_template)
}
