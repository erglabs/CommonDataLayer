```toml
communication_method = "kafka"
input_port = 0
insert_destination = ""

[kafka]
brokers = ""
group_id = ""

[amqp]
exchange_url = ""
tag = ""

[services]
schema_registry_url = ""
edge_registry_url = ""
on_demand_materializer_url = ""
query_router_url = ""

[notification_consumer]
source = ""

[log]
rust_log = "info,api=debug"
```
