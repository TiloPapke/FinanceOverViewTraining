use askama::Template;
use async_session::{
    chrono::{DateTime, Utc},
    SessionStore,
};
use axum::{
    extract::Form,
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse, Redirect, Response},
};
use log::{debug, trace, warn};
use mongodb::bson::Uuid;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};

use crate::{
    accounting_config_logic::FinanceAccountingConfigHandle,
    database_handler_mongodb::{DbConnectionSetting, DbHandlerMongoDB, EmailVerificationStatus},
    frontend_functions::get_general_userdata_fromdatabase,
    password_handle::{
        check_email_status_by_name, create_credentials, validate_credentials, UserCredentials,
    },
    session_data_handle::{SessionData, SessionDataResult},
    setting_struct::SettingStruct,
    user_handling::validate_user_email,
};

#[derive(Template)]
#[template(path = "WelcomePage.html")]
pub struct MainPageTemplate {
    pub web_running_port: u16,
    pub additional_info: String,
    pub called_times: i32,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct LoginFormInput {
    username: String,
    password: Secret<String>,
}

#[derive(Template)]
#[template(path = "UserHome.html")]
pub struct UserHomeTemplate {
    username: String,
    session_expire_timestamp: String,
    logged_in: bool,
    logout_reason: String,
    information_show: bool,
    information_text: String,
    user_vorname: String,
    user_nachname: String,
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
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    headers,
                    format!("Failed to render template. Error: {}", err),
                )
                    .into_response()
            }
        }
    }
}

pub async fn accept_login_form(
    session_data: SessionDataResult,
    input: Form<LoginFormInput>,
) -> impl IntoResponse {
    let credentials = UserCredentials {
        username: input.username.clone(),
        password: input.password.clone(),
    };
    let local_settings: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password),
        instance: String::from(local_settings.backend_database_instance),
    };

    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap();
    let _result = session.insert("user_name", &credentials.username);

    let a_store: async_mongodb_session::MongodbSessionStore = session_data.session_store;

    match validate_credentials(&db_connection, &credentials).await {
        Ok(user_id) => {
            let local_settings: SettingStruct = SettingStruct::global().clone();
            let db_connection = DbConnectionSetting {
                url: String::from(local_settings.backend_database_url),
                user: String::from(local_settings.backend_database_user),
                password: String::from(local_settings.backend_database_password),
                instance: String::from(local_settings.backend_database_instance),
            };
            let mail_check_result =
                check_email_status_by_name(&db_connection, &credentials.username).await;
            if mail_check_result.is_err() {
                debug!(target: "app::FinanceOverView","error with email: {}",&mail_check_result.unwrap_err());
                Redirect::to("/invalid").into_response()
            } else {
                match mail_check_result.unwrap() {
                    EmailVerificationStatus::NotVerified => {
                        debug!(target: "app::FinanceOverView","email not verified");
                        Redirect::to("/registration_incomplete").into_response()
                    }
                    _ => {
                        let _result = session.insert("logged_in", true);
                        let _result2 = session.insert("user_account_id", user_id);
                        let _cookie3 = a_store.store_session(session).await;

                        debug!(target: "app::FinanceOverView","user_id is {}",user_id);
                        Redirect::to("/user_home").into_response()
                    }
                }
            }
        }
        Err(_) => {
            debug!(target: "app::FinanceOverView","no valid user name");
            Redirect::to("/invalid").into_response()
        }
    }
}

pub async fn user_home_handler(session_data: SessionDataResult) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    if !is_logged_in {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let template = UserHomeTemplate {
            logout_reason: "not logged in".to_string(),
            username: "".to_string(),
            session_expire_timestamp,
            logged_in: false,
            information_show: false,
            information_text: "".to_string(),
            user_vorname: "".to_string(),
            user_nachname: "".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return (headers, HtmlTemplate(template));
    }

    let username: String = session.get("user_name").unwrap();

    if session.is_expired() {
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );
        let template = UserHomeTemplate {
            logout_reason: "Session expired".to_string(),
            username: "".to_string(),
            session_expire_timestamp,
            logged_in: false,
            information_show: false,
            information_text: "".to_string(),
            user_vorname: "".to_string(),
            user_nachname: "".to_string(),
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        (headers, HtmlTemplate(template))
    } else {
        session.expire_in(std::time::Duration::from_secs(60 * 1));
        let session_expire_timestamp = format!(
            "{} UTC",
            (session
                .expiry()
                .unwrap_or(&DateTime::<Utc>::MIN_UTC)
                .naive_local()
                .format("%Y-%m-%d %H:%M:%S"))
        );

        let local_settings: SettingStruct = SettingStruct::global().clone();
        let db_connection = DbConnectionSetting {
            url: String::from(local_settings.backend_database_url),
            user: String::from(local_settings.backend_database_user),
            password: String::from(local_settings.backend_database_password),
            instance: String::from(local_settings.backend_database_instance),
        };
        let user_data_get_result_async =
            get_general_userdata_fromdatabase(&db_connection, &username);

        let user_data_result = user_data_get_result_async.await;

        if user_data_result.is_err() {
            let template = UserHomeTemplate {
                logout_reason: "error calling database".to_string(),
                username: username.to_string(),
                session_expire_timestamp,
                logged_in: false,
                information_show: false,
                information_text: "".to_string(),
                user_vorname: "".to_string(),
                user_nachname: "".to_string(),
            };
            headers.insert(
                axum::http::header::REFRESH,
                axum::http::HeaderValue::from_str("5; url = /").unwrap(),
            );
            (headers, HtmlTemplate(template))
        } else {
            let user_data = user_data_result.unwrap();

            let template = UserHomeTemplate {
                username: username.to_string(),
                session_expire_timestamp,
                logged_in: true,
                logout_reason: "".to_string(),
                information_show: false,
                information_text: "".to_string(),
                user_vorname: user_data.first_name,
                user_nachname: user_data.last_name,
            };

            let _new_cookie = session_data.session_store.store_session(session).await;

            (headers, HtmlTemplate(template))
        }
    }
}

pub async fn do_logout_handler(session_data: SessionDataResult) -> impl IntoResponse {
    let session_data = SessionData::from_session_data_result(session_data);

    let mut session = session_data.session_option.unwrap();

    let session_expire_timestamp = format!(
        "{} UTC",
        (session
            .expiry()
            .unwrap_or(&DateTime::<Utc>::MIN_UTC)
            .naive_local()
            .format("%Y-%m-%d %H:%M:%S"))
    );

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);
    let _result = session.remove("user_account_id");

    let _destroy_result = session_data.session_store.destroy_session(session).await;

    let template = UserHomeTemplate {
        username: "".to_string(),
        session_expire_timestamp,
        logged_in: false,
        logout_reason: if is_logged_in {
            "You logout yourself".to_string()
        } else {
            "Not loged in".to_string()
        },
        information_show: false,
        information_text: "".to_string(),
        user_vorname: "".to_string(),
        user_nachname: "".to_string(),
    };

    HtmlTemplate(template);
    Redirect::to("/").into_response()
}

#[derive(Template)]
#[template(path = "RegistrationIncomplete.html")]
pub struct RegistrationIncompleteTemplate {
    username: String,
    registration_failure: String,
}

pub async fn registration_incomplete_handler() -> impl IntoResponse {
    let username = "chosen login";
    let reason_info = "missing email validation";

    let st: RegistrationIncompleteTemplate = RegistrationIncompleteTemplate {
        username: format!("{}", username),
        registration_failure: reason_info.to_string(),
    };
    HtmlTemplate(st)
}

#[derive(Template)]
#[template(path = "InvalidUser.html")]
pub struct InvalidTemplate {
    username: String,
}

pub async fn invalid_handler() -> impl IntoResponse {
    let username = "chosen login";

    let st = InvalidTemplate {
        username: format!("{} is invalid", username),
    };
    HtmlTemplate(st)
}

#[derive(Template)]
#[template(path = "CreateLogin.html")]
pub struct CreateLoginTemplate {
    user_name: String,
    create_result: String,
}

pub async fn create_login_handler(form: Form<LoginFormInput>) -> impl IntoResponse {
    debug!(target: "app::FinanceOverView","create_login data user {} with {:?}",&form.username,form.password);

    let mut clt_template = CreateLoginTemplate {
        user_name: "not_set".to_string(),
        create_result: "no Result given".to_string(),
    };

    let new_user_credentials = UserCredentials {
        username: form.username.to_string(),
        password: form.password.clone(),
    };

    let local_settings: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password),
        instance: String::from(local_settings.backend_database_instance),
    };

    let create_result = create_credentials(&db_connection, &new_user_credentials).await;
    if create_result.is_err() {
        clt_template.user_name = new_user_credentials.username.to_string();
        clt_template.create_result = create_result.unwrap_err().to_string();
    } else {
        clt_template.user_name = new_user_credentials.username.to_string();
        clt_template.create_result = format!("your user id is {}", create_result.unwrap());
    }
    HtmlTemplate(clt_template)
}

#[derive(Template)]
#[template(path = "RegisterUser.html")]
pub struct RegisterUserTemplate {
    //just a dummy template, currently no addtional data is shown
}

pub async fn register_user_handler() -> impl IntoResponse {
    let st = RegisterUserTemplate {};
    HtmlTemplate(st)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct ValidateUserEmailInput {
    user_name: String,
    token: Secret<String>,
}

#[derive(Template)]
#[template(path = "EmailValidationResult.html")]
pub struct EmailValidationResultTemplate {
    do_html_redirect: String,
    validation_main_result: String,
    validation_detail_result: String,
}

pub async fn validate_user_email_handler(form: Form<ValidateUserEmailInput>) -> impl IntoResponse {
    debug!(target: "app::FinanceOverView","validateUserEmail");

    let mut st: EmailValidationResultTemplate = EmailValidationResultTemplate {
        do_html_redirect: "false".to_string(),
        validation_detail_result: "something went wrong".to_string(),
        validation_main_result: "Error during validaiton".to_string(),
    };

    let local_settings: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(local_settings.backend_database_url),
        user: String::from(local_settings.backend_database_user),
        password: String::from(local_settings.backend_database_password),
        instance: String::from(local_settings.backend_database_instance),
    };
    let check_result = validate_user_email(&db_connection, &form.user_name, &form.token).await;

    if check_result.is_err() {
        st.validation_detail_result = check_result.unwrap_err();
    } else {
        match check_result.unwrap() {
            EmailVerificationStatus::NotGiven => {
                st.validation_detail_result = "Validation not startet".to_string();
            }
            EmailVerificationStatus::NotVerified => {
                st.validation_detail_result = "not verified".to_string();
                return HtmlTemplate(st);
            }
            EmailVerificationStatus::Verified => {
                st.validation_detail_result = format!("email for {} validated", form.user_name);
            }
        }
    }

    HtmlTemplate(st)
}

#[derive(Template)]
#[template(path = "PasswordResetTokenRequest.html")]
pub struct RequestPasswortResetTokenTemplate {
    //just a dummy template, currently no addtional data is shown
}

pub async fn display_paswword_reset_token_request_page() -> impl IntoResponse {
    let st: RequestPasswortResetTokenTemplate = RequestPasswortResetTokenTemplate {};
    HtmlTemplate(st)
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)]
pub struct PasswordResetWithTokenDisplayRequest {
    user_name: String,
    token: Secret<String>,
}

#[derive(Template)]
#[template(path = "PasswordResetWithToken.html")]
pub struct PasswordResetRequestTemplate {
    pub user_name: String,
    pub reset_token: String,
}

pub async fn display_paswword_reset_with_token_page(
    form: Form<PasswordResetWithTokenDisplayRequest>,
) -> impl IntoResponse {
    debug!(target: "app::FinanceOverView","display password reset");
    let st: PasswordResetRequestTemplate = PasswordResetRequestTemplate {
        user_name: form.user_name.clone(),
        reset_token: form.token.expose_secret().clone(),
    };
    HtmlTemplate(st)
}

#[derive(Template)]
#[template(path = "AccountingConfig/AccountingConfig_main.html")]
pub struct AccountingMainConfigTemplate {
    username: String,
    account_types: Vec<AccountTypeTemplate>,
    accounts: Vec<AccountTemplate>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountTypeTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AccountTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub type_title: String,
}

pub async fn display_accounting_config_main_page(
    session_data: SessionDataResult,
) -> impl IntoResponse {
    debug!(target: "app::FinanceOverView","display accounting main config page");

    let session_data = SessionData::from_session_data_result(session_data);
    let mut session = session_data.session_option.unwrap().clone();

    let is_logged_in: bool = session.get("logged_in").unwrap_or(false);

    let mut headers = HeaderMap::new();

    let empty_account_type_list: Vec<AccountTypeTemplate> = Vec::with_capacity(0);
    let empty_account_list: Vec<AccountTemplate> = Vec::with_capacity(0);
    let mut return_account_type_list: Vec<AccountTypeTemplate> = Vec::new();
    let mut return_account_list: Vec<AccountTemplate> = Vec::new();

    if !is_logged_in {
        let return_value: AccountingMainConfigTemplate = AccountingMainConfigTemplate {
            username: "not logged in".to_string(),
            account_types: empty_account_type_list,
            accounts: empty_account_list,
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return HtmlTemplate(return_value);
    }

    if session.is_expired() {
        let return_value: AccountingMainConfigTemplate = AccountingMainConfigTemplate {
            username: "Session expired".to_string(),
            account_types: empty_account_type_list,
            accounts: empty_account_list,
        };
        headers.insert(
            axum::http::header::REFRESH,
            axum::http::HeaderValue::from_str("5; url = /").unwrap(),
        );
        return HtmlTemplate(return_value);
    }

    let username: String = session.get("user_name").unwrap();
    let user_id: Uuid = session.get("user_account_id").unwrap();

    let local_setting: SettingStruct = SettingStruct::global().clone();
    let db_connection = DbConnectionSetting {
        url: String::from(&local_setting.backend_database_url),
        user: String::from(local_setting.backend_database_user),
        password: String::from(local_setting.backend_database_password),
        instance: String::from(&local_setting.backend_database_instance),
    };
    let db_handler = DbHandlerMongoDB::new(&db_connection);

    {
        let accounting_config_handle =
            FinanceAccountingConfigHandle::new(&db_connection, &user_id, &db_handler);

        {
            let account_types_result: Result<Vec<crate::datatypes::FinanceAccountType>, String> =
                accounting_config_handle.finance_account_type_list();

            if account_types_result.is_err() {
                warn!(target: "app::FinanceOverView","error in display_accounting_config_main_page for user {}: {}",username,account_types_result.unwrap_err());
                let return_value: AccountingMainConfigTemplate = AccountingMainConfigTemplate {
                    username: "problems while getting account types list".to_string(),
                    account_types: empty_account_type_list,
                    accounts: empty_account_list,
                };
                return HtmlTemplate(return_value);
            }

            //let available_account_types = account_types_result.unwrap();
            for some_type in account_types_result.as_ref().unwrap() {
                return_account_type_list.push(AccountTypeTemplate {
                    id: some_type.id.to_string(),
                    name: some_type.title.clone(),
                    description: some_type.description.clone(),
                });
            }

            let accounts_result: Result<Vec<crate::datatypes::FinanceAccount>, String> =
                accounting_config_handle.finance_account_list(None);

            if accounts_result.is_err() {
                warn!(target: "app::FinanceOverView","error in display_accounting_config_main_page for user {}: {}",username,accounts_result.unwrap_err());
                let return_value: AccountingMainConfigTemplate = AccountingMainConfigTemplate {
                    username: "problems while getting account list".to_string(),
                    account_types: empty_account_type_list,
                    accounts: empty_account_list,
                };
                return HtmlTemplate(return_value);
            }

            let available_account_types = &account_types_result.unwrap();
            for some_account in accounts_result.unwrap() {
                let type_position_result = available_account_types
                    .iter()
                    .position(|elem| elem.id.eq(&some_account.finance_account_type_id));
                let type_title = match type_position_result {
                    Some(position) => &available_account_types[position].title,
                    _ => "no title found",
                };
                return_account_list.push(AccountTemplate {
                    id: some_account.id.to_string(),
                    name: some_account.title,
                    description: some_account.description,
                    type_title: type_title.into(),
                });
            }
        }
    }

    let return_value: AccountingMainConfigTemplate = AccountingMainConfigTemplate {
        username: username,
        account_types: return_account_type_list,
        accounts: return_account_list,
    };

    session.expire_in(std::time::Duration::from_secs(60 * 10));
    let _new_cookie = session_data.session_store.store_session(session).await;

    trace!(target: "app::FinanceOverView","Loaded finance accounting types for user id {}", user_id);

    HtmlTemplate(return_value)
}
