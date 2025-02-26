use serde::{Serialize, Deserialize};
use tokio_pg_mapper_derive::PostgresMapper;
use chrono::NaiveDateTime;

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "ServiceSections")]
pub struct ServiceSection {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "Services")]
pub struct Service {
    pub id: i32,
    pub section_id: i32,
    pub name: String,
    pub description: String,
    pub created_at: NaiveDateTime,
}
