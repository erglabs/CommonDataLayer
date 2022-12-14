use std::collections::HashMap;

use cdl_dto::materialization::FieldType;
use rpc::common::types::LogicOperator;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ObjectIdPair;

#[derive(Debug, PartialEq, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldDefinitionSource {
    Simple {
        object: ObjectIdPair,
        field_name: String,
        field_type: FieldType,
    },
    Computed {
        computation: ComputationSource,
        field_type: FieldType,
    },
    SubObject {
        fields: HashMap<String, FieldDefinitionSource>,
    },
}

#[derive(Debug, PartialEq, Deserialize, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComputationSource {
    RawValue {
        value: Value,
    },
    FieldValue {
        object: ObjectIdPair,
        field_path: String,
    },
    Equals {
        lhs: Box<ComputationSource>,
        rhs: Box<ComputationSource>,
    },
}

#[derive(Debug, PartialEq)]
pub struct RowSource {
    pub objects: HashMap<ObjectIdPair, Value>,
    pub root_object: ObjectIdPair,
    pub fields: HashMap<String, FieldDefinitionSource>,
    pub filters: Option<FilterSource>,
    pub relation_order: Vec<ObjectIdPair>,
}

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterSource {
    Equals {
        lhs: FilterValueSource,
        rhs: FilterValueSource,
    },
    Complex {
        operator: LogicOperator,
        operands: Vec<FilterSource>,
    },
}

#[derive(Debug, PartialEq, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum FilterValueSource {
    SchemaField {
        object: ObjectIdPair,
        field_path: String,
    },
    ViewPath {
        field_path: String,
    },
    RawValue {
        value: Value,
    },
    Computed {
        computation: ComputationSource,
    },
}
