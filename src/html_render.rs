use askama::Template;
use axum::{response::{Html, Response, IntoResponse, Redirect}, http::StatusCode, extract::Form};
use secrecy::Secret;
use serde::Deserialize;

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

pub async fn accept_login_form(Form(input): Form<LoginFormInput>)  -> Redirect   {
   // Redirect::to(&format!("/user_home&user={}", input.username));
   /*format!(
    "Welcome to the protected area :)\nHere's your info:\n{:?}",
    "Fake2"
)*/

    Redirect::to("/user_home")



}

pub async fn user_home_handler()  -> impl IntoResponse {
 /*  let template = UserHomeTemplate { 
        username: "Fake".to_string()
     };
    HtmlTemplate(template);
    */
    format!(
        "Welcome to the protected area :)\nHere's your info:\n{:?}",
        "Fake"
    )
}
