use crate::config::ExampleConfig;
use crate::errors::MyError;
use crate::models::{Service, ServiceSection};
use confik::{Configuration as _, EnvSource};
use deadpool_postgres::{Client, Pool};
use tokio_pg_mapper::FromTokioPostgresRow;
use tokio_postgres::NoTls;

pub async fn establish_connection() -> Result<Pool, MyError> {
    let config = ExampleConfig::builder()
        .override_with(EnvSource::new())
        .try_build()
        .unwrap();
    Ok(config.pg.create_pool(None, NoTls).unwrap())
}

pub async fn create_service_section(
    client: &Client,
    title: &str,
    description: &str,
) -> Result<ServiceSection, MyError> {
    let stmt = include_str!("../sql/create_service_section.sql");
    let stmt = stmt.replace("$table_fields", &ServiceSection::sql_table_fields());
    let stmt = client.prepare(&stmt).await.map_err(MyError::PGError)?;

    let row = client
        .query_one(&stmt, &[&title, &description])
        .await
        .map_err(MyError::PGError)?;
    ServiceSection::from_row_ref(&row).map_err(MyError::PGMError)
}

pub async fn create_service(
    client: &Client,
    section_id: i32,
    name: &str,
    description: &str,
) -> Result<Service, MyError> {
    let stmt = include_str!("../sql/create_service.sql");
    let stmt = stmt.replace("$table_fields", &Service::sql_table_fields());
    let stmt = client.prepare(&stmt).await.map_err(MyError::PGError)?;

    let row = client
        .query_one(&stmt, &[&section_id, &name, &description])
        .await
        .map_err(MyError::PGError)?;
    Service::from_row_ref(&row).map_err(MyError::PGMError)
}

pub async fn get_services(pool: &Pool) -> Result<Vec<(ServiceSection, Vec<Service>)>, MyError> {
    let client: Client = pool.get().await.map_err(MyError::PoolError)?;
    let stmt_sections = include_str!("../sql/get_services.sql");
    let stmt_sections = client.prepare(&stmt_sections).await.map_err(MyError::PGError)?;

    let sections = client
        .query(&stmt_sections, &[])
        .await
        .map_err(MyError::PGError)?
        .iter()
        .map(|row| ServiceSection::from_row_ref(row).unwrap())
        .collect::<Vec<ServiceSection>>();

    let mut service_sections = Vec::new();

    for section in sections {
        let stmt_services = include_str!("../sql/get_services_by_section.sql");
        let stmt_services = client.prepare(&stmt_services).await.map_err(MyError::PGError)?;

        let services = client
            .query(&stmt_services, &[&section.id])
            .await
            .map_err(MyError::PGError)?
            .iter()
            .map(|row| Service::from_row_ref(row).unwrap())
            .collect::<Vec<Service>>();

        service_sections.push((section, services));
    }

    Ok(service_sections)
}

pub async fn get_database() -> Result<Pool, MyError> {
    let pool = establish_connection().await?;
    let client = pool.get().await.map_err(MyError::PoolError)?;
    client
        .batch_execute(include_str!("../sql/setup.sql"))
        .await
        .map_err(MyError::PGError)?;
    Ok(pool)
}
