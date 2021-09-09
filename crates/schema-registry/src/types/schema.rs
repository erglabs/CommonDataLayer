use ::types::schemas::SchemaDefinition;
use rpc::schema_registry::types::SchemaType;
use serde::{Deserialize, Serialize};
use sqlx::types::Json;
use uuid::Uuid;

use crate::types::view::FullView;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Schema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Json<SchemaDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewSchema {
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Json<SchemaDefinition>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchemaUpdate {
    pub name: Option<String>,
    pub insert_destination: Option<String>,
    pub query_address: Option<String>,
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
    pub definition: Option<Json<SchemaDefinition>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FullSchema {
    pub id: Uuid,
    pub name: String,
    pub insert_destination: String,
    pub query_address: String,
    #[serde(rename = "type")]
    pub schema_type: SchemaType,
    pub definition: Json<SchemaDefinition>,
    pub views: Vec<FullView>,
}
