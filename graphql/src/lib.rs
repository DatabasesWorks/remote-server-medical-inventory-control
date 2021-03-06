pub mod loader;
pub mod schema;
pub mod standard_graphql_error;

use actix_web::cookie::Cookie;
use actix_web::HttpRequest;
use actix_web::{guard::fn_guard, web::Data, HttpResponse, Result};
use async_graphql::http::{playground_source, GraphQLPlaygroundConfig};
use async_graphql::{Context, EmptySubscription, SchemaBuilder};
use async_graphql_actix_web::{Request, Response};
use repository::StorageConnectionManager;
use reqwest::header::COOKIE;
use service::auth_data::AuthData;
use service::service_provider::ServiceProvider;

use self::{
    loader::LoaderRegistry,
    schema::{Mutations, Queries, Schema},
};

// Sugar that helps make things neater and avoid errors that would only crop up at runtime.
pub trait ContextExt {
    fn get_connection_manager(&self) -> &StorageConnectionManager;
    fn get_loader<T: anymap::any::Any + Send + Sync>(&self) -> &T;
    fn service_provider(&self) -> &ServiceProvider;
    fn get_auth_data(&self) -> &AuthData;
    fn get_auth_token(&self) -> Option<String>;
}

impl<'a> ContextExt for Context<'a> {
    fn get_connection_manager(&self) -> &StorageConnectionManager {
        &self.data_unchecked::<Data<StorageConnectionManager>>()
    }

    fn get_loader<T: anymap::any::Any + Send + Sync>(&self) -> &T {
        self.data_unchecked::<Data<LoaderRegistry>>().get::<T>()
    }

    fn service_provider(&self) -> &ServiceProvider {
        self.data_unchecked::<Data<ServiceProvider>>()
    }

    fn get_auth_data(&self) -> &AuthData {
        self.data_unchecked::<Data<AuthData>>()
    }

    fn get_auth_token(&self) -> Option<String> {
        self.data_opt::<RequestUserData>()
            .and_then(|d| d.auth_token.to_owned())
    }
}

type Builder = SchemaBuilder<Queries, Mutations, EmptySubscription>;

pub fn build_schema() -> Builder {
    Schema::build(Queries, Mutations, EmptySubscription)
}

pub fn config(
    connection_manager: Data<StorageConnectionManager>,
    loader_registry: Data<LoaderRegistry>,
    service_provider: Data<ServiceProvider>,
    auth_data: Data<AuthData>,
) -> impl FnOnce(&mut actix_web::web::ServiceConfig) {
    |cfg| {
        let schema = build_schema()
            .data(connection_manager)
            .data(loader_registry)
            .data(service_provider)
            .data(auth_data)
            .finish();
        cfg.service(
            actix_web::web::scope("/graphql")
                .data(schema)
                .route("", actix_web::web::post().to(graphql))
                // It???s nicest to have the playground on the same URL, but if it???s a GET request and
                // there???s a `query` parameter, we want it to be treated as a GraphQL query. The
                // simplest way of doing this is to just require no query string for playground access.
                .route(
                    "",
                    actix_web::web::get()
                        .guard(fn_guard(|head| head.uri.query().is_none()))
                        .to(playground),
                )
                .route("", actix_web::web::get().to(graphql)),
        );
    }
}

pub struct RequestUserData {
    auth_token: Option<String>,
    refresh_token: Option<String>,
}

fn auth_data_from_request(http_req: &HttpRequest) -> RequestUserData {
    let headers = http_req.headers();
    // retrieve auth token
    let auth_token = headers.get("Authorization").and_then(|header_value| {
        header_value.to_str().ok().and_then(|header| {
            if header.starts_with("Bearer ") {
                return Some(header["Bearer ".len()..header.len()].to_string());
            }
            None
        })
    });

    // retrieve refresh token
    let refresh_token = headers.get(COOKIE).and_then(|header_value| {
        header_value
            .to_str()
            .ok()
            .and_then(|header| Cookie::parse(header).ok())
            .map(|cookie| cookie.value().to_owned())
    });

    RequestUserData {
        auth_token,
        refresh_token,
    }
}

async fn graphql(schema: Data<Schema>, http_req: HttpRequest, req: Request) -> Response {
    let user_data = auth_data_from_request(&http_req);
    let query = req.into_inner().data(user_data);
    schema.execute(query).await.into()
}

async fn playground() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(playground_source(
            GraphQLPlaygroundConfig::new("/graphql")
                // allow to set cookies
                .with_setting("request.credentials", "same-origin"),
        )))
}
