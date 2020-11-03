# Common Data Layer

A collection of services that make up the Common Data Layer, implemented in the [Rust][rust] language.


## What is the Common Data Layer?

The Common Data Layer (CDL) is a data storage service. It is designed with performance, versatility,
scalability, and ease-of-modification as key tenets of its design, among others.

There is a [confluence page][confluence wiki] describing the motivations of this project.


## How does it work?

Data intake is all performed over Message Queue, and via the Data Router (message Query (MQ) is an abstract entity.
currently supported MQs are [kafka][kafka] and [RabbitMQ][rmq]) CDL listens over a single topic queue
for messages keyed on strings, each providing a schema ID. The schema ID is used to load the appropriate
topic (stored per-schema in the schema registry), which is used to route the message along to the correct repository.

_Note: the crate for the [schema registry][schema registry] has more information on schemas and views._

For each repository, a command service is listening to its specific MQ topic for incoming messages. Each message is
stored according to the repository's format. Though most of our command service implementations use append-only
storage with each value under a key being assigned a version, it is not required by user-implemented command
services.

Each repository also has a query service listening for gRPC requests for data. These query services are used
for both direct queries of data from the repositories and for the object builder to run. As repositories are
meant to be easily introduced to an already running CDL, but the Kafka topic per repository can't be used to make
a gRPC request, each schema also stores the dynamic address of the query service it belongs to.


## Structure

This project is structured as a collection of Rust crates:

Crate Name              | Purpose
------------------------|--------
data-router             | Route incoming data from and through MQ for consumption by the specific Command Service
schema-registry         | Manage user-defined schemas that define the format of incoming values and their respective topics
query-service           | Wrap each individual repository for retrieval of data
command-service         | Intake data from a MQ and storege, in specific repository
leader-elector          | Elect master nodes in replicated services (_only for the Schema Repository, currently_)
document-storage        | Store document data keyed on UUIDs powered by [sled][sled]
blob-store              | Store binary data keyed on UUIDs from user powered by [sled][sled]
db-shrinker-storage     | A service to remove older data from storage
query-router            | Route incoming requests to query service given a schema and object id
utils                   | A collection of utilities used throughout the Common Data Layer
cdl-cli                 | Provide a command-line interface for managing schemas in the schema registry and storing and retrieving data

In addition to the above crates, there are some other useful directories:

Directory       | Purpose
----------------|--------
helm            | helm charts for remote deployment
benchmarking    | scripts and scaffolding data for benchmarking
examples/deploy | sample deployment guide for docker


## Getting Started

See the [Getting Started Wiki][Getting Started Wiki] to see how to install and use this service.


[rust]: https://www.rust-lang.org
[sled]: https://github.com/spacejam/sled
[kafka]: https://kafka.apache.org/
[rmq]: https://www.rabbitmq.com/
[schema registry]: ./schema-registry/
[confluence wiki]: https://pgga-es.atlassian.net/wiki/spaces/EPI/pages/602938021/Common+Data+Layer+concept
[Getting Started Wiki]: https://github.com/pgga-es/epiphany-cdl-rust/wiki/Getting-Started
