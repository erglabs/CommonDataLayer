use super::{scalar_type, schema_field_type_flag, schema_type};

rpc_enum! {
    SchemaType,
    schema_type::Type,
    schema_type,
    "schema type",
    "schema_type_enum",
    [
        DocumentStorage,
        Timeseries
    ]
}

rpc_enum! {
    ScalarType,
    scalar_type::Type,
    scalar_type,
    "scalar type",
    "scalar_type_enum",
    [
        Bool,
        String,
        Integer,
        Float,
        Any
    ]
}

rpc_enum! {
    SchemaFieldTypeFlag,
    schema_field_type_flag::Type,
    field_type,
    "schema field type",
    "schema_field_type_enum",
    [
        Scalar,
        Object,
        Array
    ]
}
