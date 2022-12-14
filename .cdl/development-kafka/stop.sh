#!/usr/bin/env bash
killall api || echo "api"
killall data-router || echo "data-router"
killall query-router || echo "query-router"
killall schema-registry || echo "schema-registry"
killall command-service || echo "command-service"
killall query-service || echo "query-service"
killall edge-registry || echo "edge-registry"
killall object-builder || echo "object-builder"
killall partial-update-engine || echo "partial-update-engine"
killall materializer-general || echo "materializer-general"
killall materializer-ondemand || echo "materializer-ondemand"
