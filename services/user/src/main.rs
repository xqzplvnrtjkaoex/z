use madome_proto::user::user_service_server::UserServiceServer;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    madome_common::tracing::init_tracing("user");

    let addr = madome_common::env::required_env("USER_LISTEN_ADDR")
        .parse()
        .expect("invalid USER_LISTEN_ADDR");

    let database_url = madome_common::env::required_env("USER_DATABASE_URL");

    tracing::info!("connecting to database");
    let db = Database::connect(&database_url).await?;

    tracing::info!("running migrations");
    user::migration::Migrator::up(&db, None).await?;

    let user_repo = user::adapter::postgres::user_repository::PostgresUserRepository::new(db);
    let ctx = user::adapter::context::UserContext::new(user_repo);
    let handler = user::app::handler::UserHandler::new(ctx);

    tracing::info!(%addr, "user service starting");

    Server::builder()
        .add_service(UserServiceServer::new(handler))
        .serve(addr)
        .await?;

    Ok(())
}
