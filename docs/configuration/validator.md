```toml
communication_method = "kafka"
cache_capacity = 1024
async_task_limit = 32
send_to = ""

[kafka]
brokers = ""
group_id = ""
ingest_topic = ""

[amqp]
exchange_url = ""
tag = ""
ingest_queue = ""

[amqp.consume_options]
no_local = false
no_act = false
exclusive = false
nowait = false

[grpc]
address = ""

[monitoring]
metrics_port = 0
status_port = 0
otel_service_name = ""

[services]
schema_registry_url = ""

[log]
rust_log = ""
```