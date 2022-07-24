use askama::Template;
use axum::{response::{Html, Response, IntoResponse, Redirect}, http::{StatusCode}, extract::Form};
use log::debug;
use secrecy::Secret;
use serde::Deserialize;

use crate::{password_handle::{validate_credentials, UserCredentials, create_credentials}};

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

pub async fn accept_login_form( Form (input): Form<LoginFormInput>)  -> impl IntoResponse    {
    
    let credentials = UserCredentials {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    //session.insert("user_name", &credentials.username);

    match validate_credentials(&credentials).await {
        Ok(user_id) => {
            debug!(target: "app::FinanceOverView","user_id is {}",user_id);
            Redirect::to("/user_home").into_response()
            //user_home_handler(Form(input))
        }
        Err(_) => {
            debug!(target: "app::FinanceOverView","no valid user name");
            Redirect::to("/invalid").into_response()
            //invalid_handler(Form(input))
            //user_home_handler(Form(input)).await
        }
    }


}

pub async fn user_home_handler()  -> impl IntoResponse {
//let username:String = session.get("username").unwrap();
let username="SOME VALID";
/*
    let credentials = UserCredentials {
        username: form.0.username,
        password: form.0.password,
    };
*/
    //let _user_id = validate_credentials(&credentials).await;

   let template = UserHomeTemplate { 
        username: username.to_string() //credentials.username
     };
    HtmlTemplate(template)
    /*
    format!(
        "Welcome to the protected area :)\nHere's your info:\n{:?}",
        user_id
    )
    */
}

#[derive(Template)]
#[template(path = "InvalidUser.html")]
pub struct InvalidTemplate{
    username: String 
}

pub async fn invalid_handler()  -> impl IntoResponse  {
    //let username:String = session.get("username").unwrap();

let username="INVALID";

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
