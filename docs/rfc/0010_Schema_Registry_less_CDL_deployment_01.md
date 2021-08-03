# Front Matter

```
Title           : Schema-Registry-less CDL deployment
Author(s)       : Wojciech Polak
Team            : CommonDataLayer
Reviewer        : CommonDataLayer
Created         : 2021-06-24
Last updated    : 2021-06-24
Version         : 1.0.0
CDL feature ID  : CDLF-00016-00
```


## Glossary

### Terminology

* CDL - Common Data Layer
* DR - Data Router, a CDL component responsible for routing ingested messages.
* SR - Schema Registry, a CDL component responsible for keeping information about the type of object conveyed inside the message.
* QR - Query Router, a CDL component responsible for routing queries to the proper repository.
* CS - Command Service, a CDL component, ingestion part of the repository.
* QS - Query Service, a CDL component, query part of the repository.
* CIM - CDL Ingestion Message
* User - user of the CDL.
* Message - message sent to CDL for processing.
* Breaking Change - behavior change, or API of the CDL, may result in system breakage on release update.
* Routing information - information necessary for DR and QR to successfully deliver the provided message and querying the repository's data.

## Formats

v1.0 input message format up to this point:
```
{
    object_id: UUID,
    schema_id: UUID,
    data: { any valid JSON }
}
```

v1.0 input message format to accommodate schema-registry-less routing:
```
{
    options: {
        repository: String?, // Optional field
    }, // If empty, can be ommited for backwards compability
    schema_id: UUID,
    object_id: UUID,
    data: { any valid JSON }
}
```

Static configuration format:
```
[repositories]
repository_name = { insert_destination: String, query_address: String, schema_type: SchemaType }
```

SchemaType
```
"documentstorage" | "timeseries"
```

## Introduction

### Background

It was requested from the CDL stack to route messages in an environment where SR is not deployed.

Currently, CDL is using SR to decide which insert destination and query address is used during data and query routing.
This information is tightly tied to the Schema (and schema id).
Each Schema stored in the PostgreSQL table contains these fields:

| id   | name   | schema type | insert destination | query address |
|------|--------|-------------|--------------------|---------------|
| UUID | String | SchemaType  | String             | String        |

Also worth noting is that the user can use one CS to store more than one Schema. Therefore one Schema is not equivalent to one service stack.

The second purpose (which in many cases can be omitted) is that each Schema is linked to its definition, which might be used for validation purposes.

After implementing this feature, CDL will insert and query data from proper repositories without using SR. It is intended for limited environments and therefore has a limited number of supported features.


### Assumptions

* CDL should allow to omit Schema Registry deployment and provide an alternative way of routing data.
* Considering CIM format, any new fields or configuration must not be a breaking change.

### Limitations
* In this alternative, state routing would become a static table, which requires restarting the services after each change.
* Data validation would be unavailable as we would not keep track of schema definitions.
* Materialization would be unavailable, as Views also are handled by SR and are coupled with Schemas.

## Proposed solution

### Static routing
Like reverse proxies allow the user to set up static routing, CDL should allow it via configuration files.
Schema definitions would be separated from routing information.
This routing would take precedence over querying SR (in case both of them are available).

Each routing service (DR and QR) would load the same static configuration file, and then whenever the field
`repository` would be present in the new CIM input message. It would use static routing information to send the request further.

* Data Router would use the `insert_destination` field.
* Query Router would use `query_address` and `schema_type` fields.

`schema_id`, while is not associated with any Schema definition, is still used as an element of the object identification pair:
(`schema_id`, `object_id`) and required by both QR and DR. As mentioned in the Background section, one repository stack might be used to store more than one Schema. Also, CDL does not guarantee the uniqueness of `object_id` outside of its Schema.

Because there is no SR responsible for registering Schema and assigning it `schema_id`, it means that the user must prepare their predefined UUIDs.

### Possible extension
In the future, whenever Configuration Service would be introduced, one might think about moving this Static routing into a dynamic environment.

### Major concerns

The static routing allows new kinds of errors and bugs in the CDL system. The CDL user must consider the possibility of service configuration sync issues because there is no single source of truth or consensus algorithm.

It is crucial to use the same configuration for both ingesting and querying data.

## Further Considerations

### Impact on other teams

This feature does not provide any breaking change. Therefore all other teams can use CDL with Schema Registry without problems.

### Scalability

No impact. Because static routing is Read-Only, one can scale all services without any extra cost or configuration.

### Testing

This feature MUST undergo thorough testing (in the best scenario, automatic E2E tests). Test cases MUST include:

* Failure scenarios:
    * Message is corrupted
    * Message contains repository which is not defined in static routing
    * Corrupted configuration
* Green path test
* Performance tests
