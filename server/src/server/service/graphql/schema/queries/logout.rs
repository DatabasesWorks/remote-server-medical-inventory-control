use async_graphql::*;

use crate::server::service::graphql::schema::types::InternalError;
use crate::server::service::graphql::{ContextExt, RequestUserData};
use service::token::TokenService;

use super::{set_refresh_token_cookie, ErrorWrapper};

pub struct Logout {
    pub user_id: String,
}

#[Object]
impl Logout {
    /// User id of the logged out user
    pub async fn user_id(&self) -> &str {
        &self.user_id
    }
}

pub struct MissingAuthToken;
#[Object]
impl MissingAuthToken {
    pub async fn description(&self) -> &'static str {
        "Auth token was not provided"
    }
}

pub struct ExpiredSignature;
#[Object]
impl ExpiredSignature {
    pub async fn description(&self) -> &'static str {
        "Provided token is expired"
    }
}

pub struct InvalidToken;
#[Object]
impl InvalidToken {
    pub async fn description(&self) -> &'static str {
        "Provided token is invalid"
    }
}

pub struct TokenInvalided;
#[Object]
impl TokenInvalided {
    pub async fn description(&self) -> &'static str {
        "Token has been invalidated by the server"
    }
}

pub struct NotAnApiToken;
#[Object]
impl NotAnApiToken {
    pub async fn description(&self) -> &'static str {
        "Not an api token"
    }
}

#[derive(Interface)]
#[graphql(field(name = "description", type = "&str"))]
pub enum LogoutErrorInterface {
    MissingAuthToken(MissingAuthToken),
    ExpiredSignature(ExpiredSignature),
    InvalidToken(InvalidToken),
    TokenInvalided(TokenInvalided),
    NotAnApiToken(NotAnApiToken),
    InternalError(InternalError),
}

pub type LogoutError = ErrorWrapper<LogoutErrorInterface>;

#[derive(Union)]
pub enum LogoutResponse {
    Error(LogoutError),
    Response(Logout),
}

pub fn logout(ctx: &Context<'_>) -> LogoutResponse {
    let auth_data = ctx.get_auth_data();
    let mut service = TokenService::new(
        &auth_data.token_bucket,
        auth_data.auth_token_secret.as_bytes(),
    );

    // invalid the refresh token cookie first (just in case an error happens before we do so)
    set_refresh_token_cookie(ctx, "logged out", 0, auth_data.debug_no_ssl);

    let auth_token = match ctx
        .data_opt::<RequestUserData>()
        .and_then(|d| d.auth_token.to_owned())
    {
        Some(data) => data,
        None => {
            return LogoutResponse::Error(ErrorWrapper {
                error: LogoutErrorInterface::MissingAuthToken(MissingAuthToken),
            })
        }
    };

    // verify that the provided token is valid
    let claims = match service.verify_token(&auth_token) {
        Ok(claims) => claims,
        Err(err) => {
            let e = match err {
                service::token::JWTValidationError::ExpiredSignature => todo!(),
                service::token::JWTValidationError::NotAnApiToken => {
                    LogoutErrorInterface::NotAnApiToken(NotAnApiToken)
                }
                service::token::JWTValidationError::InvalidToken(_) => {
                    LogoutErrorInterface::InvalidToken(InvalidToken)
                }
                service::token::JWTValidationError::TokenInvalided => {
                    LogoutErrorInterface::TokenInvalided(TokenInvalided)
                }
                service::token::JWTValidationError::ConcurrencyLockError(_) => {
                    LogoutErrorInterface::InternalError(InternalError("Lock error".to_string()))
                }
            };
            return LogoutResponse::Error(ErrorWrapper { error: e });
        }
    };

    // invalided all tokens of the user on the server
    let user_id = claims.sub;
    match service.logout(&user_id) {
        Ok(_) => {}
        Err(e) => match e {
            service::token::JWTLogoutError::ConcurrencyLockError(_) => {
                return LogoutResponse::Error(ErrorWrapper {
                    error: LogoutErrorInterface::InternalError(InternalError(
                        "Lock error".to_string(),
                    )),
                });
            }
        },
    };

    LogoutResponse::Response(Logout { user_id })
}
