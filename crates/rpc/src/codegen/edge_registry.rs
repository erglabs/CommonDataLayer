#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TreeQuery {
    #[prost(message, optional, tag = "1")]
    pub filters: ::core::option::Option<Filter>,
    #[prost(message, repeated, tag = "2")]
    pub relations: ::prost::alloc::vec::Vec<super::common::Relation>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Filter {
    #[prost(oneof = "filter::FilterKind", tags = "1, 2")]
    pub filter_kind: ::core::option::Option<filter::FilterKind>,
}
/// Nested message and enum types in `Filter`.
pub mod filter {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum FilterKind {
        #[prost(message, tag = "1")]
        Simple(super::SimpleFilter),
        #[prost(message, tag = "2")]
        Complex(super::ComplexFilter),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleFilterSide {
    #[prost(enumeration = "simple_filter_side::Side", required, tag = "1")]
    pub side: i32,
}
/// Nested message and enum types in `SimpleFilterSide`.
pub mod simple_filter_side {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
    #[repr(i32)]
    pub enum Side {
        InParentObjIds = 1,
        InChildObjIds = 2,
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SimpleFilter {
    #[prost(message, required, tag = "1")]
    pub side: SimpleFilterSide,
    #[prost(uint32, required, tag = "2")]
    pub relation: u32,
    #[prost(string, repeated, tag = "3")]
    pub ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ComplexFilter {
    #[prost(message, required, tag = "1")]
    pub operator: super::common::LogicOperator,
    #[prost(message, repeated, tag = "2")]
    pub operands: ::prost::alloc::vec::Vec<Filter>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationTreeResponse {
    #[prost(message, repeated, tag = "1")]
    pub rows: ::prost::alloc::vec::Vec<RelationTreeRow>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationTreeRow {
    #[prost(string, required, tag = "1")]
    pub base_object_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub relation_object_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AddSchemaRelation {
    #[prost(string, optional, tag = "1")]
    pub relation_id: ::core::option::Option<::prost::alloc::string::String>,
    #[prost(string, required, tag = "2")]
    pub parent_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub child_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaRelation {
    #[prost(string, required, tag = "1")]
    pub parent_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub child_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationId {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationQuery {
    #[prost(string, repeated, tag = "1")]
    pub relation_id: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ValidateRelationQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SchemaId {
    #[prost(string, required, tag = "1")]
    pub schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationDetails {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_schema_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "3")]
    pub child_schema_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationList {
    #[prost(message, repeated, tag = "1")]
    pub items: ::prost::alloc::vec::Vec<RelationDetails>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectRelations {
    #[prost(message, repeated, tag = "1")]
    pub relations: ::prost::alloc::vec::Vec<Edge>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Edge {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_object_id: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "3")]
    pub child_object_ids: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RelationIdQuery {
    #[prost(string, required, tag = "1")]
    pub relation_id: ::prost::alloc::string::String,
    #[prost(string, required, tag = "2")]
    pub parent_object_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectIdQuery {
    #[prost(string, required, tag = "1")]
    pub object_id: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Empty {}
#[doc = r" Generated client implementations."]
pub mod edge_registry_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct EdgeRegistryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl EdgeRegistryClient<tonic::transport::Channel> {
        #[doc = r" Attempt to create a new client by connecting to a given endpoint."]
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: std::convert::TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> EdgeRegistryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::ResponseBody: Body + Send + Sync + 'static,
        T::Error: Into<StdError>,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> EdgeRegistryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<http::Request<tonic::body::BoxBody>>>::Error:
                Into<StdError> + Send + Sync,
        {
            EdgeRegistryClient::new(InterceptedService::new(inner, interceptor))
        }
        #[doc = r" Compress requests with `gzip`."]
        #[doc = r""]
        #[doc = r" This requires the server to support it otherwise it might respond with an"]
        #[doc = r" error."]
        pub fn send_gzip(mut self) -> Self {
            self.inner = self.inner.send_gzip();
            self
        }
        #[doc = r" Enable decompressing responses with `gzip`."]
        pub fn accept_gzip(mut self) -> Self {
            self.inner = self.inner.accept_gzip();
            self
        }
        pub async fn add_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::AddSchemaRelation>,
        ) -> Result<tonic::Response<super::RelationId>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/AddRelation");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationQuery>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetRelation");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_by_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationId>,
        ) -> Result<tonic::Response<super::SchemaRelation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/GetSchemaByRelation",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_schema_relations(
            &mut self,
            request: impl tonic::IntoRequest<super::SchemaId>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/GetSchemaRelations",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn validate_relation(
            &mut self,
            request: impl tonic::IntoRequest<super::ValidateRelationQuery>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/edge_registry.EdgeRegistry/ValidateRelation",
            );
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn add_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::ObjectRelations>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/AddEdges");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::RelationIdQuery>,
        ) -> Result<tonic::Response<super::Edge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetEdge");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn get_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::ObjectIdQuery>,
        ) -> Result<tonic::Response<super::ObjectRelations>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/GetEdges");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn heartbeat(
            &mut self,
            request: impl tonic::IntoRequest<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/Heartbeat");
            self.inner.unary(request.into_request(), path, codec).await
        }
        pub async fn resolve_tree(
            &mut self,
            request: impl tonic::IntoRequest<super::TreeQuery>,
        ) -> Result<tonic::Response<super::RelationTreeResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::new(
                    tonic::Code::Unknown,
                    format!("Service was not ready: {}", e.into()),
                )
            })?;
            let codec = tonic::codec::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/edge_registry.EdgeRegistry/ResolveTree");
            self.inner.unary(request.into_request(), path, codec).await
        }
    }
}
#[doc = r" Generated server implementations."]
pub mod edge_registry_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with EdgeRegistryServer."]
    #[async_trait]
    pub trait EdgeRegistry: Send + Sync + 'static {
        async fn add_relation(
            &self,
            request: tonic::Request<super::AddSchemaRelation>,
        ) -> Result<tonic::Response<super::RelationId>, tonic::Status>;
        async fn get_relation(
            &self,
            request: tonic::Request<super::RelationQuery>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status>;
        async fn get_schema_by_relation(
            &self,
            request: tonic::Request<super::RelationId>,
        ) -> Result<tonic::Response<super::SchemaRelation>, tonic::Status>;
        async fn get_schema_relations(
            &self,
            request: tonic::Request<super::SchemaId>,
        ) -> Result<tonic::Response<super::RelationList>, tonic::Status>;
        async fn validate_relation(
            &self,
            request: tonic::Request<super::ValidateRelationQuery>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn add_edges(
            &self,
            request: tonic::Request<super::ObjectRelations>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn get_edge(
            &self,
            request: tonic::Request<super::RelationIdQuery>,
        ) -> Result<tonic::Response<super::Edge>, tonic::Status>;
        async fn get_edges(
            &self,
            request: tonic::Request<super::ObjectIdQuery>,
        ) -> Result<tonic::Response<super::ObjectRelations>, tonic::Status>;
        async fn heartbeat(
            &self,
            request: tonic::Request<super::Empty>,
        ) -> Result<tonic::Response<super::Empty>, tonic::Status>;
        async fn resolve_tree(
            &self,
            request: tonic::Request<super::TreeQuery>,
        ) -> Result<tonic::Response<super::RelationTreeResponse>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct EdgeRegistryServer<T: EdgeRegistry> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: EdgeRegistry> EdgeRegistryServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for EdgeRegistryServer<T>
    where
        T: EdgeRegistry,
        B: Body + Send + Sync + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/edge_registry.EdgeRegistry/AddRelation" => {
                    #[allow(non_camel_case_types)]
                    struct AddRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::AddSchemaRelation> for AddRelationSvc<T> {
                        type Response = super::RelationId;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::AddSchemaRelation>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddRelationSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/GetRelation" => {
                    #[allow(non_camel_case_types)]
                    struct GetRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationQuery> for GetRelationSvc<T> {
                        type Response = super::RelationList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetRelationSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/GetSchemaByRelation" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaByRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationId> for GetSchemaByRelationSvc<T> {
                        type Response = super::SchemaRelation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_by_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetSchemaByRelationSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/GetSchemaRelations" => {
                    #[allow(non_camel_case_types)]
                    struct GetSchemaRelationsSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::SchemaId> for GetSchemaRelationsSvc<T> {
                        type Response = super::RelationList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SchemaId>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_schema_relations(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetSchemaRelationsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/ValidateRelation" => {
                    #[allow(non_camel_case_types)]
                    struct ValidateRelationSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ValidateRelationQuery>
                        for ValidateRelationSvc<T>
                    {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ValidateRelationQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).validate_relation(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ValidateRelationSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/AddEdges" => {
                    #[allow(non_camel_case_types)]
                    struct AddEdgesSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ObjectRelations> for AddEdgesSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ObjectRelations>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).add_edges(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = AddEdgesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/GetEdge" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgeSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::RelationIdQuery> for GetEdgeSvc<T> {
                        type Response = super::Edge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::RelationIdQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_edge(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetEdgeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/GetEdges" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgesSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::ObjectIdQuery> for GetEdgesSvc<T> {
                        type Response = super::ObjectRelations;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ObjectIdQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).get_edges(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = GetEdgesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/Heartbeat" => {
                    #[allow(non_camel_case_types)]
                    struct HeartbeatSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::Empty> for HeartbeatSvc<T> {
                        type Response = super::Empty;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(&mut self, request: tonic::Request<super::Empty>) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).heartbeat(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = HeartbeatSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/edge_registry.EdgeRegistry/ResolveTree" => {
                    #[allow(non_camel_case_types)]
                    struct ResolveTreeSvc<T: EdgeRegistry>(pub Arc<T>);
                    impl<T: EdgeRegistry> tonic::server::UnaryService<super::TreeQuery> for ResolveTreeSvc<T> {
                        type Response = super::RelationTreeResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TreeQuery>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).resolve_tree(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ResolveTreeSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: EdgeRegistry> Clone for EdgeRegistryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: EdgeRegistry> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: EdgeRegistry> tonic::transport::NamedService for EdgeRegistryServer<T> {
        const NAME: &'static str = "edge_registry.EdgeRegistry";
    }
}
