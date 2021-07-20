# Front Matter

```
Title           : Transport headers passthrough
Category        : Feature
Author(s)       : Wojciech Polak
Team            : CommonDataLayer
Created         : 09-07-2021
CDL feature ID  : CFLF-0001F-00
```

## Glossary
* `gRPC` - Remote procedure call protocol - [website][gRPC]
* `AMQP` - Advanced Message Queueing Protocol - [website][AMQP]
* `Kafka` - Distributed event streaming platform - [website][Kafka]
* `ingestion`/`ingest` - push new data into CDL system

## Introduction
When the user ingests data into CDL, he might want to pass extra metadata linked to the request, for example, breadcrumb.
This metadata should not be stripped away (which is the current state) but passed through the ingestion part of the CDL into the Command Service notification.

Thanks to that, the system reacting to those notifications can still operate on metadata.
## Proposed solution
To understand how to implement this feature, one needs to consider all communication methods used to ingest.

Currently, CDL supports:
* Kafka
* AMQP
* gRPC

### Kafka
Kafka v0.11.0.0 adds support for [custom headers][Kafka-Headers].

#### in Rust
Rust implementation, rdKafka supports it as well; it is already used in the CDL for passing Open Telemetry span ID deeper into the system.

To use it, the implementator of this RFC might want to use `OwnedHeaders`, add there all headers, and pass it into FutureRecord like this:

```rust
let mut headers = OwnedHeaders::new();
...
FutureRecord::to(...)
    .payload(...)
    .key(...)
    .headers(headers);
```

Then headers can be retrieved by using `BorrowedMessage` and `BorrowedHeaders`. Unfortunately headers are not stored as a map.
Instead, one has to take `.count()` and then retrieve all headers manually. 

It might be wise to reuse `KafkaConsumerHeaders` from `tracing_utils`.

### AMQP
AMQP supports message headers in field table format: [spec][AMQP].

#### in Rust
Rust implementation, lapin, supports headers by providing [`with_headers`][Lapin-Set] and [`headers`][Lapin-Get] methods in `AMQPProperties`.

The implementation of this feature should be quite similar to one described in Kafka section, although fortunately, lapin uses `BTreeMap` as a backend for `FieldTable`.

### gRPC
gRPC is based on HTTP2; therefore, it is possible to add custom headers.
Based on [gRPC protocol][gRPC-Protocol], it would be wise to use `Custom-Metadata` for that purpose.
It is stored in a separate header, so there is no chance for collision between the header set by the user and what gRPC uses internally.

#### in Rust
Rust implementation, tonic, supports `Custom-Metadata`. There is [`MetadataMap`][Tonic-Metadata], which can be used to store the header. One could combine it with the tonics `Interceptor`. Interceptor uses delegate function to inject headers into each request sent by the client.

However, that means the client has to be created with an attached interceptor, as it seems that there is no simple way to change the interceptor after the client has been made. It might be a problem whenever there is a need to reuse the same client (and the connection behind it) between different requests.

The other option for how `MetadataMap` can be used is simply by providing `Request` - `tonic` takes as an argument, not the message itself but `IntoRequest<Message>` where `Message` always implements this trait. `Request<Message>` also implements it; therefore, the client can accept already (manually) injected requests with proper headers. The downside of this approach is that one has to maintain all places where the client should pass headers manually.

## Header conflicts

Based on the fact that CDL is using headers for Open Telemetry support, and in the future, it might be possible to use more headers for other purposes.

Therefore implementation should either:
* allow for header shadowing: new header has higher priority, and consequently, it overrides the old one,
* keep only the first header: old header has higher priority, and therefore application ignores new insertions by always keeping the original value,
* return an error when such a situation occurs: header with the name `XXX` is forbidden in the CDL and can be used only internally by CDL

As an outcome of this RFC implementation, one must provide documentation enlisting all headers (including one used for Open Telemetry) used internally in CDL to inform the user what may generate such a conflict.

## Configurability

Header passthrough, as well as conflict resolution, should be configurable.

Preferably conflict resolution strategy should be configurable on a global (configuration of the service) level or/and in the local schema or object level. This topic could be researched in the future.

Header passthrough should be available under the feature flag, which could be by default disabled to prevent any breaking changes; however, whenever CDL reaches a new MAJOR version, it might be wise to reconsider enabling it by default.

## Alternatives

One posibility could be to read headers from communication method, but include it in payload as a field of CDL notification. Then Header conflict would be resolved automatically. If the client needs headers be present as actual headers, one might need some kind of Shim transforming notification and resolving header conflicts there.

## Further considerations

### Impact on other teams

This feature does not provide any breaking change in nearby future.

### Scalability

No impact.

### Testing

This feature MUST undergo thorough testing:
* When feature flag is disabled,
* When feature flag is enabled:
   * Green path,
   * Collision with another header,
   * When header contains binary data,
* Performance tests

# Notes after meeting 19-07-2021
* This RFC is treated as Spike and R&D and team does not expect further implementation in near future.
* New issue and another RFC should be created for describing passing metadata field

[gRPC]: https://grpc.io/
[gRPC-Protocol]: https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-HTTP2.md
[Kafka]: https://kafka.apache.org/
[AMQP]: https://www.rabbitmq.com/amqp-0-9-1-reference.html
[Kafka-Headers]: https://cwiki.apache.org/confluence/display/KAFKA/KIP-82+-+Add+Record+Headers
[Lapin-Set]: https://docs.rs/lapin/1.7.1/lapin/protocol/basic/struct.AMQPProperties.html#method.with_headers
[Lapin-Get]: https://docs.rs/lapin/1.7.1/lapin/protocol/basic/struct.AMQPProperties.html#method.headers
[Tonic-Metadata]: https://docs.rs/tonic/0.5.0/tonic/metadata/struct.MetadataMap.html
