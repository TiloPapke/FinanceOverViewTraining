use askama::Template;

use axum::{response::{Html, Response, IntoResponse, Redirect}, http::{StatusCode}, extract::Form};
use log::debug;
use secrecy::Secret;
use serde::Deserialize;

use crate::password_handle::{validate_credentials, Credentials};

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
    username: String
}

pub struct HtmlTemplate<T>(pub T);

impl<T> IntoResponse for HtmlTemplate<T>
where
    T: Template,
{
    fn into_response(self) -> Response {
        match self.0.render() {
            Ok(html) => Html(html).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to render template. Error: {}", err),
            )
                .into_response(),
        }
    }
}

pub async fn accept_login_form(Form (input): Form<LoginFormInput>)  -> Redirect   {
    let credentials = Credentials {
        username: input.username,
        password: input.password,
    };

    match validate_credentials(credentials).await {
        Ok(user_id) => {
            debug!("user_id is {}",user_id);
            Redirect::to("/user_home")
        }
        Err(_) => {
            Redirect::to("/invalid")
        }
    }


}

pub async fn user_home_handler( form: Form<LoginFormInput>)  -> impl IntoResponse {

    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    let user_id = validate_credentials(credentials).await;

 /*  let template = UserHomeTemplate { 
        username: "Fake".to_string()
     };
    HtmlTemplate(template);
    */
    format!(
        "Welcome to the protected area :)\nHere's your info:\n{:?}",
        user_id
    )
}

#[derive(Template)]
#[template(path = "InvalidUser.html")]
pub struct InvalidTemplate{
    username: String 
}

pub async fn invalid_handler()  -> impl IntoResponse {
    let st = InvalidTemplate{username:"invalid_check".to_string()};
    HtmlTemplate(st)
}

#[derive(Template)]
#[template(path = "CreateLogin.html")]
pub struct CreateLoginTemplate{
    username: String 
}

pub async fn create_login_handler()  -> impl IntoResponse {
    let st = CreateLoginTemplate{username:"not_set".to_string()};
    HtmlTemplate(st)
}
