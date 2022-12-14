# Common Data Layer Changelog

## 1.1.0

### Breaking Changes:
None! :D
Also we provided document of which api is stable and which is not.

### API Changes:
- SR - Schema's version field removed

### Fixes:
- Change uuid v1 to v4 in psql to avoid clashes (#727)
- human friendly message in API (#702)
- Add timeout handlers for RPC (#669)
- group by on unsorted elements in ER (#653)
- change fetch-many to fetch in schema_registry/db.rs (#654)
- materialization filtering (#650)

### Features:
- change ER resolve tree to new format (allow for two directional searches) (#683)
- materialization to elasticsearch (#704)
- config generator - provide grpc and rabbitmq development env for developers (#643)
- store schema_id, object_id information in postgres materializer (#674)
- Remove Versioning From Schema Definitions (#670)
- Error reporting for materialization (#663)
- pass (schema_id, object_id) pair to materializer general (#660)

### Documentation:
- performance testing process (#719)
- GraphQL (#725)
- Stability docs (#706)
- short description of resolve tree (#713)

### RFC's
- materialization types (#699)
- Custom Data Passthrough (#687)
- Configuration Service (#666)
- edge registry - resolving relation trees (#624)
- CI/CD setup R&D (#640)
- Simplify Schema Definitions (#631)
- Transport headers passthrough (#635)
- Documentation restructure and RFC process (#608)
- codegen (#607)

### Testing
- Add test for GetEdges (#733)
- materializer-general : elasticsearch support (#728)
- Add SchemaRegistry Relation tests (#698)
- SR Add tests for all endpoints (#665)
- tests Fetch in schema registry (#655)
- remove unused env and update readme file (#646)
- general cdl tests (new, revised framework) (#562)

## 1.0.0

### Breaking changes
- Data Router expects `version` field in CDL Ingestion Message in format "MAJOR_VERSION.MINOR_VERSION". In this release
  the only supported version is "1.0.0".
- Changed ENV variables

### New components
- CDL Web Admin (GUI) - For management purposes only
- Edge Registry (ER) - Manages relationships between schemas (schema to schema) and objects (object to object)
- Materializer - general (M-G) - Writes materialized data to the materialization-database
- Materializer - ondemand (M-O) - Returns materialized data to the user (Request-Response)
- Object Builder (OB) - Creates materialized data based on first stage repositories
- Partial Update Engine (PUE) - Notifies OB when materialized view needs to be processed

### Changes to existing components
- Data Router and Query Router supports static routing allowing Schema-Registry-absent deployment
- Query Service supports returning stream of data without allocating huge amounts of memory
- Schema Registry now uses Postgres as a backend
- Data Router supports message batching

### General changes
- Added gRPC as an alternative communication method between services
- CDL supports basic materialization with filtering
- CDL loads configuration from configuration files
- CDL supports Open Telemetry for traces

### Major bugfixes
- It is now possible to switch to multi/single threaded behaviour in Data Router

### Documentation
- Added MdBook on Github Pages
- Added Feature List and descriptions
- Added tutorial for basic materialization
- Added documentation for configuration files and for ENV variables
- Formalized commit messages and tags
- Added information about CDL versioning

### Geneal fixes
- Query-multiple RPC method returns an error for Timeseries schema type
- Metrics port defaulted to invalid one
- Consumer-based services will log an error and continue working on next message, no longer resulting in the crash loop

### RFCs
- Static routing
- CDL Ingestion API versioning
- CDL publishing deployment configurations
- Partial Materialization
- Edge Registry
- Alternative communication method to Kafka and RabbitMQ (gRPC)

### Known issues
- Relationships in materialization may generate wrong set of data

## v0.2.0

### Breaking changes
- Archived document-storage crate
- Archived blob-store crate

### Features
- Added store type of schema in Schema Registry
- Added RAW queries to postgres
- Added RAW to Query Router
- Added VictoriaMetrics Support
- Added check if Kafka/AMQP topic exists before inserting new schema in Schema Registry
- Added gRPC alternative ingestion method to Command Service
- Added input batching for TimeSeries
- Added GraphQL server - `api`
- Added (opt-in) message ordering
- Added input-output communication method selection from ENV vars and command-line arguments

### Fixes
- Added non-default namespace support
- Added string escape mechanism for schema query
- Fixed building docker images in some environments

### Documentation
- Updated documentation for basic services
- Added UML diagrams of our services

### Dependencies
- Bumped anyhow to 1.0.38
- Bumped thiserror to 1.0.23
- Bumped reqwest to 0.10.10
- Bumped jsonschema to 0.4.3
- Bumped rand to 0.8.2
- Bumped lapin to 1.6.6
- Bumped postgres to 0.19.0
- Bumped serde to 1.0.119
- Bumped serde_json to 1.0.61
- Bumped serde_yaml to 0.8.15
- Bumped futures to 0.3.12
- Bumped futures_util to 0.3.12
- Bumped envy to 0.4.2
- Bumped uuid to 0.8.2
- Bumped pbr to 1.0.4
- Bumped test-case to 1.1.0
- Bumped log to 0.4.13
- Bumped rust toolchain to 30-12-2020

### CI and infrastructure
- Improved performance of CI
- Added DepdendaBot
- Added cargo-deny to CI
- Added component tests jobs for PRs
- Added markdown link check

### Internal
- Refactored internal message format
- Moved gRPC definitions to `rpc` crate
- Moved crates to separate folder `crates`
- Splited helm component definitions
- Moved benchmarking outside crates

### Tests
- Added testcontainers for acceptance tests
- Added component tests for query-router
- Added component tests for query-service-ts
