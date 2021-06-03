//! src/main.rs

use std::{env};

use actix_cors::Cors;
use actix_web::{http::header, middleware, web, App, Error, HttpResponse, HttpServer};
use juniper::{graphql_object, EmptySubscription, GraphQLObject, RootNode};
use juniper_actix::{graphql_handler};
use sqlx::PgPool;

#[derive(Clone, GraphQLObject)]
// A requisition.
pub struct Requisition {
    id: String,
    from_id: String,
    to_id: String,
    requisition_lines: Vec<RequisitionLine>,
}

#[derive(Clone, GraphQLObject)]
// A requisition line.
pub struct RequisitionLine {
    id: String,
    requisition_id: String,
    item_name: String,
    item_quantity: f64,
}

#[derive(Clone)]
pub struct Database {
    connection_pool: PgPool,
}

struct RequisitionRow { 
    id: String,
    from_id: String,
    to_id: String
}

struct RequisitionLineRow {
    id: String,
    requisition_id: String,
    item_name: String,
    item_quantity: f32
}

async fn select_requisition(connection_pool: &PgPool, id: String) -> Result<RequisitionRow, sqlx::Error> {
    let requisition_row = sqlx::query_as!(RequisitionRow,
        r#"
        SELECT id, from_id, to_id
        FROM requisition
        WHERE id = $1
        "#, id
    )
    .fetch_one(connection_pool)
    .await?;

    Ok(requisition_row)
}

#[allow(dead_code)]
async fn select_requisition_line(connection_pool: &PgPool, id: String) -> Result<RequisitionLineRow, sqlx::Error> {
    let requisition_line = sqlx::query_as!(RequisitionLineRow,
        r#"
        SELECT id, requisition_id, item_name, item_quantity
        FROM requisition_line
        WHERE id = $1
        "#, id
    )
    .fetch_one(connection_pool)
    .await?;

    Ok(requisition_line)
}

async fn select_requisition_lines(connection_pool: &PgPool, requisition_id: String) -> Result<Vec<RequisitionLineRow>, sqlx::Error> {
    let requisition_lines = sqlx::query_as!(RequisitionLineRow,
        r#"
        SELECT id, requisition_id, item_name, item_quantity
        FROM requisition_line 
        WHERE requisition_id = $1
        "#, requisition_id
    )
    .fetch_all(connection_pool)
    .await?;

    Ok(requisition_lines)
}

async fn insert_requisition(connection_pool: &PgPool, requisition: &Requisition) -> Result<(), sqlx::Error> {
    match requisition {
        Requisition { id, from_id, to_id, .. } => { sqlx::query!(
            r#"
            INSERT INTO requisition (id, from_id, to_id)
            VALUES ($1, $2, $3)
            "#,
            id,
            from_id,
            to_id
        )
        .execute(connection_pool)
        .await?;
        }
    }

    Ok(())
}

async fn insert_requisition_line(connection_pool: &PgPool, requisition_line: &RequisitionLine) -> Result<(), sqlx::Error> {
    match requisition_line {
        RequisitionLine { id, requisition_id, item_name, item_quantity } => {
            sqlx::query!(
                r#"
                INSERT INTO requisition_line (id, requisition_id, item_name, item_quantity)
                VALUES ($1, $2, $3, $4)
                "#,
                id,
                requisition_id,
                item_name,
                *item_quantity as f32,
            )
            .execute(connection_pool)
            .await?;
        }
    }

    Ok(())
}

async fn insert_mock_data(connection_pool: &PgPool) -> Result<(), sqlx::Error> {
    let requisition_a = Requisition {
        id: "requisition_a".to_string(),
        from_id: "store_a".to_string(),
        to_id: "store_b".to_string(),
        requisition_lines: vec![],
    };

    let requisition_b = Requisition {
        id:  "requisition_b".to_string(),
        from_id: "store_a".to_string(),
        to_id: "store_c".to_string(),
        requisition_lines: vec![],
    };

    let requisition_c = Requisition {
        id: "requisition_c".to_string(),
        from_id: "store_b".to_string(),
        to_id: "store_c".to_string(),
        requisition_lines: vec![],
    };

    let requisition_line_a = RequisitionLine {
        id: "requisition_line_a".to_string(),
        requisition_id: "requisition_a".to_string(),
        item_name: "item_a".to_string(),
        item_quantity: 1.0,
    };

    let requisition_line_b = RequisitionLine {
        id: "requisition_line_b".to_string(),
        requisition_id: "requisition_a".to_string(),
        item_name: "item_b".to_string(),
        item_quantity: 2.0,
    };

    let requisition_line_c = RequisitionLine {
        id: "requisition_line_c".to_string(),
        requisition_id: "requisition_b".to_string(),
        item_name: "item_a".to_string(), 
        item_quantity: 3.0,
    };

    let requisition_line_d = RequisitionLine {
        id: "requisition_line_d".to_string(),
        requisition_id: "requisition_b".to_string(), 
        item_name: "item_b".to_string(),
        item_quantity: 4.0,
    };

    let requisition_line_e = RequisitionLine {
        id: "requisition_line_e".to_string(),
        requisition_id: "requisition_c".to_string(),
        item_name: "item_a".to_string(),
        item_quantity: 5.0,
    };
    
    insert_requisition(connection_pool, &requisition_a).await?;
    insert_requisition(connection_pool, &requisition_b).await?;
    insert_requisition(connection_pool, &requisition_c).await?;

    insert_requisition_line(connection_pool, &requisition_line_a).await?;
    insert_requisition_line(connection_pool, &requisition_line_b).await?;
    insert_requisition_line(connection_pool, &requisition_line_c).await?;
    insert_requisition_line(connection_pool, &requisition_line_d).await?;
    insert_requisition_line(connection_pool, &requisition_line_e).await?;

    Ok(())
}

impl Database {
    pub async fn new(connection_pool: PgPool) -> Database {
        Database { connection_pool }
    }

    pub async fn insert_requisition(&self, requisition: Requisition) -> Result<Requisition, sqlx::Error> {
        insert_requisition(&self.connection_pool, &requisition).await?;
        Ok(requisition)
    }

    pub async fn get_requisition(&self, id: String) -> Result<Requisition, sqlx::Error> {
        let requisition_row = select_requisition(&self.connection_pool, id.to_string()).await?;
        let requisition_line_rows = select_requisition_lines(&self.connection_pool, id.to_string()).await?;

        let requisition = Requisition {
            id: requisition_row.id,
            from_id: requisition_row.from_id,
            to_id: requisition_row.to_id,
            requisition_lines: requisition_line_rows.into_iter().map(|line| RequisitionLine {
                id: line.id,
                requisition_id: line.requisition_id,
                item_name: line.item_name,
                item_quantity: line.item_quantity as f64
            }).collect()
        };

        Ok(requisition)
    }
}

impl juniper::Context for Database {}

struct Query;
#[graphql_object(context = Database)]
impl Query {
    fn apiVersion() -> String {
        "1.0".to_string()
    }

    #[graphql(arguments(id(description = "id of the requisition")))]
    async fn requisition(database: &Database, id: String) -> Requisition {
        database.get_requisition(id).await.unwrap()
    }
}

struct Mutations;
#[graphql_object(context = Database)]
impl Mutations {
    async fn insert_requisition(database: &Database, id: String, from_id: String, to_id: String) -> Requisition {
        let requisition = Requisition {
            id: id,
            from_id: from_id,
            to_id: to_id,
            requisition_lines: vec![]
        };

        database.insert_requisition(requisition).await.unwrap()
    }
}

type Schema = RootNode<'static, Query, Mutations, EmptySubscription<Database>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        Mutations,
        EmptySubscription::<Database>::new(),
    )
}

async fn graphql_route(
    req: actix_web::HttpRequest,
    payload: actix_web::web::Payload,
    schema: web::Data<Schema>,
) -> Result<HttpResponse, Error> {
    let connection_pool =
        PgPool::connect("postgres://postgres:password@localhost/omsupply-database")
            .await
            .expect("Failed to connect to database");
    let context = Database::new(connection_pool).await;
    graphql_handler(&schema, &context, req, payload).await
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info");
    env_logger::init();

    let connection_pool =
    PgPool::connect("postgres://postgres:password@localhost/omsupply-database")
        .await
        .expect("Failed to connect to database");

    insert_mock_data(&connection_pool)
        .await
        .expect("Failed to insert mock data");

    let server = HttpServer::new(move || {
        App::new()
            .data(schema())
            .wrap(middleware::Compress::default())
            .wrap(middleware::Logger::default())
            .wrap(
                Cors::default()
                    .allowed_origin("http://127.0.0.1:8080")
                    .allowed_methods(vec!["POST", "GET"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .supports_credentials()
                    .max_age(3600),
            )
            .service(
                web::resource("/graphql")
                    .route(web::post().to(graphql_route))
                    .route(web::get().to(graphql_route)),
            )
    });
    server.bind("127.0.0.1:8080").unwrap().run().await
}
