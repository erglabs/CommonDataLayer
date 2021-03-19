# Testing

This directory contains integration tests for *CDL* components.

Making sure they pass is vital to the longevity of our application.

## Prerequisites

### Python
Tests run on python `3`. They are tested on CI using the latest available python version.

### Pip
We require `pip3` command to be present in `$PATH`.

### Rust & Cargo

Rust is required to compile CDL. Users can also point tests to precompiled binaries via env variables (this is not supported in the shell script case).

| application | env |
|---|---|
| db-shrinker-postgres | DB_SHRINKER_POSTGRES_EXE |
| command-service | COMMAND_SERVICE_EXE |
| query-router | QUERY_ROUTER_EXE |
| schema-registry | SCHEMA_REGISTRY_EXE |
| query-service | QUERY_SERVICE_EXE |
| query-service-ts | QUERY_SERVICE_TS_EXE |

## Running

`./run_tests.sh` handles running all tests. It requires the user to be in the `tests` directory.

## Structure

### ./data
Test data, divided into directories per application.

### ./rpc
Directory generated by `grpc_tools.protoc` which contains python modules compiled from `rpc/proto/*.proto` files.

### application folders
eg. db_shrinker_postgres. Named after application, they test.