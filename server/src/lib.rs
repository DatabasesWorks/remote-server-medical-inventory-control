use self::{
    actor_registry::ActorRegistry,
    middleware::{compress as compress_middleware, logger as logger_middleware},
    settings::Settings,
    sync::{SyncConnection, SyncReceiverActor, SyncSenderActor, Synchroniser},
};
use openssl::ssl::{SslAcceptor, SslAcceptorBuilder, SslFiletype, SslMethod};

use graphql::{
    config as graphql_config,
    loader::{get_loaders, LoaderMap, LoaderRegistry},
};
use log::{error, warn};
use repository::get_storage_connection_manager;
use service::{auth_data::AuthData, service_provider::ServiceProvider, token_bucket::TokenBucket};

use actix_cors::Cors;
use actix_web::{web::Data, App, HttpServer};
use std::{
    net::TcpListener,
    sync::{Arc, Mutex, RwLock},
    time::Duration,
};

pub mod actor_registry;
pub mod configuration;
pub mod environment;
pub mod middleware;
pub mod settings;
pub mod sync;
pub mod test_utils;

pub async fn start_server(settings: Settings) -> std::io::Result<()> {
    let auth_data = Data::new(AuthData {
        auth_token_secret: settings.auth.token_secret.to_owned(),
        token_bucket: RwLock::new(TokenBucket::new()),
        debug_no_ssl: false,
        // TODO: disable once frontend supports auth!
        debug_no_access_control: true,
    });
    let connection_manager = get_storage_connection_manager(&settings.database);
    let loaders: LoaderMap = get_loaders(&connection_manager).await;
    let service_provider = ServiceProvider::new(connection_manager.clone());
    let (mut sync_sender, mut sync_receiver): (SyncSenderActor, SyncReceiverActor) =
        sync::get_sync_actors();

    let actor_registry = ActorRegistry {
        sync_sender: Arc::new(Mutex::new(sync_sender.clone())),
    };

    let connection_manager_data_app = Data::new(connection_manager);
    let connection_manager_data_sync = connection_manager_data_app.clone();
    let loader_registry_data = Data::new(LoaderRegistry { loaders });
    let service_provider_data = Data::new(service_provider);
    let actor_registry_data = Data::new(actor_registry);

    let mut http_server = HttpServer::new(move || {
        App::new()
            .app_data(connection_manager_data_app.clone())
            .app_data(actor_registry_data.clone())
            .wrap(logger_middleware())
            .wrap(Cors::permissive())
            .wrap(compress_middleware())
            .configure(graphql_config(
                connection_manager_data_app.clone(),
                loader_registry_data.clone(),
                service_provider_data.clone(),
                auth_data.clone(),
            ))
    });
    match load_certs() {
        Ok(ssl_builder) => {
            http_server = http_server.bind_openssl(
                format!("{}:{}", settings.server.host, settings.server.port),
                ssl_builder,
            )?;
        }
        Err(err) => {
            error!("Failed to load certificates: {}", err);
            warn!("Run in HTTP mode");

            let listener = TcpListener::bind(settings.server.address())
                .expect("Failed to bind server to address");
            http_server = http_server.listen(listener)?;
        }
    }
    let running_sever = http_server.run();
    let connection = match SyncConnection::new(&settings.sync) {
        Ok(connection) => connection,
        Err(err) => {
            let err_msg = format!("Failed to initialize SyncConnection: {}", err);
            error!("{}", err_msg);
            panic!("{}", err_msg);
        }
    };
    let mut synchroniser = Synchroniser { connection };
    // http_server is the only one that should quit; a proper shutdown signal can cause this,
    // and so we want an orderly exit. This achieves it nicely.
    tokio::select! {
        result = running_sever => result,
        () = async {
          sync_sender.schedule_send(Duration::from_secs(settings.sync.interval)).await;
        } => unreachable!("Sync receiver unexpectedly died!?"),
        () = async {
            sync_receiver.listen(&mut synchroniser, &connection_manager_data_sync).await;
        } => unreachable!("Sync scheduler unexpectedly died!?"),
    }
}

fn load_certs() -> Result<SslAcceptorBuilder, anyhow::Error> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder.set_private_key_file("certs/key.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("certs/cert.pem")?;
    Ok(builder)
}
