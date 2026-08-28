// @generated
/// Generated client implementations.
pub mod graph_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    #[derive(Debug, Clone)]
    pub struct GraphClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl GraphClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> GraphClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::Body>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + std::marker::Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + std::marker::Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> GraphClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                    http::Request<tonic::body::Body>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                    >,
                >,
            <T as tonic::codegen::Service<http::Request<tonic::body::Body>>>::Error:
                Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            GraphClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        pub async fn create_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/CreateEdge");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "CreateEdge"));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single edge.
        */
        pub async fn get_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::GetEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/GetEdge");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "GetEdge"));
            self.inner.unary(req, path, codec).await
        }
        /** Lists edges.
        */
        pub async fn list_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::ListEdgesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListEdgesResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/ListEdges");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "ListEdges"));
            self.inner.unary(req, path, codec).await
        }
        /** Creates several edges in one request.
        */
        pub async fn batch_create_edges(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchCreateEdgesRequest>,
        ) -> std::result::Result<tonic::Response<super::BatchCreateEdgesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/BatchCreateEdges");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.graph.v1.Graph",
                "BatchCreateEdges",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Soft-deletes an edge.

         Returns the tombstoned edge. Deleting from the graph is reversible until
         expiry, which matters more here than elsewhere: an edge is often the only
         record that two things were ever related.
        */
        pub async fn delete_edge(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/DeleteEdge");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "DeleteEdge"));
            self.inner.unary(req, path, codec).await
        }
        /** Traverses the graph outward from a set of seeds.

         Bounded best-first expansion, not a search over all paths. Each hop depends
         on the last, so this runs on the host rather than a device, and the bounds
         — hop count, result ceiling, per-hop decay — are what keep it from becoming
         the most expensive thing in a query.
        */
        pub async fn traverse_graph(
            &mut self,
            request: impl tonic::IntoRequest<super::TraverseGraphRequest>,
        ) -> std::result::Result<tonic::Response<super::TraverseGraphResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/TraverseGraph");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "TraverseGraph"));
            self.inner.unary(req, path, codec).await
        }
        /** Creates an edge type, declaring the default weight its edges take.
        */
        pub async fn create_edge_type(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/CreateEdgeType");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "CreateEdgeType"));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single edge type.
        */
        pub async fn get_edge_type(
            &mut self,
            request: impl tonic::IntoRequest<super::GetEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/GetEdgeType");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "GetEdgeType"));
            self.inner.unary(req, path, codec).await
        }
        /** Lists edge types.
        */
        pub async fn list_edge_types(
            &mut self,
            request: impl tonic::IntoRequest<super::ListEdgeTypesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListEdgeTypesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/ListEdgeTypes");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "ListEdgeTypes"));
            self.inner.unary(req, path, codec).await
        }
        /** Soft-deletes an edge type.
        */
        pub async fn delete_edge_type(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.graph.v1.Graph/DeleteEdgeType");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.graph.v1.Graph", "DeleteEdgeType"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod graph_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with GraphServer.
    #[async_trait]
    pub trait Graph: std::marker::Send + std::marker::Sync + 'static {
        async fn create_edge(
            &self,
            request: tonic::Request<super::CreateEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status>;
        /** Retrieves a single edge.
        */
        async fn get_edge(
            &self,
            request: tonic::Request<super::GetEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status>;
        /** Lists edges.
        */
        async fn list_edges(
            &self,
            request: tonic::Request<super::ListEdgesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListEdgesResponse>, tonic::Status>;
        /** Creates several edges in one request.
        */
        async fn batch_create_edges(
            &self,
            request: tonic::Request<super::BatchCreateEdgesRequest>,
        ) -> std::result::Result<tonic::Response<super::BatchCreateEdgesResponse>, tonic::Status>;
        /** Soft-deletes an edge.

         Returns the tombstoned edge. Deleting from the graph is reversible until
         expiry, which matters more here than elsewhere: an edge is often the only
         record that two things were ever related.
        */
        async fn delete_edge(
            &self,
            request: tonic::Request<super::DeleteEdgeRequest>,
        ) -> std::result::Result<tonic::Response<super::Edge>, tonic::Status>;
        /** Traverses the graph outward from a set of seeds.

         Bounded best-first expansion, not a search over all paths. Each hop depends
         on the last, so this runs on the host rather than a device, and the bounds
         — hop count, result ceiling, per-hop decay — are what keep it from becoming
         the most expensive thing in a query.
        */
        async fn traverse_graph(
            &self,
            request: tonic::Request<super::TraverseGraphRequest>,
        ) -> std::result::Result<tonic::Response<super::TraverseGraphResponse>, tonic::Status>;
        /** Creates an edge type, declaring the default weight its edges take.
        */
        async fn create_edge_type(
            &self,
            request: tonic::Request<super::CreateEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status>;
        /** Retrieves a single edge type.
        */
        async fn get_edge_type(
            &self,
            request: tonic::Request<super::GetEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status>;
        /** Lists edge types.
        */
        async fn list_edge_types(
            &self,
            request: tonic::Request<super::ListEdgeTypesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListEdgeTypesResponse>, tonic::Status>;
        /** Soft-deletes an edge type.
        */
        async fn delete_edge_type(
            &self,
            request: tonic::Request<super::DeleteEdgeTypeRequest>,
        ) -> std::result::Result<tonic::Response<super::EdgeType>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct GraphServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> GraphServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for GraphServer<T>
    where
        T: Graph,
        B: Body + std::marker::Send + 'static,
        B::Error: Into<StdError> + std::marker::Send + 'static,
    {
        type Response = http::Response<tonic::body::Body>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            match req.uri().path() {
                "/telividb.graph.v1.Graph/CreateEdge" => {
                    #[allow(non_camel_case_types)]
                    struct CreateEdgeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::CreateEdgeRequest> for CreateEdgeSvc<T> {
                        type Response = super::Edge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateEdgeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::create_edge(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateEdgeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/GetEdge" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::GetEdgeRequest> for GetEdgeSvc<T> {
                        type Response = super::Edge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetEdgeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move { <T as Graph>::get_edge(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetEdgeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/ListEdges" => {
                    #[allow(non_camel_case_types)]
                    struct ListEdgesSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::ListEdgesRequest> for ListEdgesSvc<T> {
                        type Response = super::ListEdgesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListEdgesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::list_edges(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListEdgesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/BatchCreateEdges" => {
                    #[allow(non_camel_case_types)]
                    struct BatchCreateEdgesSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::BatchCreateEdgesRequest>
                        for BatchCreateEdgesSvc<T>
                    {
                        type Response = super::BatchCreateEdgesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchCreateEdgesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Graph>::batch_create_edges(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = BatchCreateEdgesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/DeleteEdge" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteEdgeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::DeleteEdgeRequest> for DeleteEdgeSvc<T> {
                        type Response = super::Edge;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteEdgeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::delete_edge(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteEdgeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/TraverseGraph" => {
                    #[allow(non_camel_case_types)]
                    struct TraverseGraphSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::TraverseGraphRequest> for TraverseGraphSvc<T> {
                        type Response = super::TraverseGraphResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::TraverseGraphRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::traverse_graph(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = TraverseGraphSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/CreateEdgeType" => {
                    #[allow(non_camel_case_types)]
                    struct CreateEdgeTypeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::CreateEdgeTypeRequest> for CreateEdgeTypeSvc<T> {
                        type Response = super::EdgeType;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateEdgeTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Graph>::create_edge_type(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateEdgeTypeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/GetEdgeType" => {
                    #[allow(non_camel_case_types)]
                    struct GetEdgeTypeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::GetEdgeTypeRequest> for GetEdgeTypeSvc<T> {
                        type Response = super::EdgeType;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetEdgeTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::get_edge_type(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetEdgeTypeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/ListEdgeTypes" => {
                    #[allow(non_camel_case_types)]
                    struct ListEdgeTypesSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::ListEdgeTypesRequest> for ListEdgeTypesSvc<T> {
                        type Response = super::ListEdgeTypesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListEdgeTypesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Graph>::list_edge_types(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListEdgeTypesSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/telividb.graph.v1.Graph/DeleteEdgeType" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteEdgeTypeSvc<T: Graph>(pub Arc<T>);
                    impl<T: Graph> tonic::server::UnaryService<super::DeleteEdgeTypeRequest> for DeleteEdgeTypeSvc<T> {
                        type Response = super::EdgeType;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteEdgeTypeRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Graph>::delete_edge_type(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteEdgeTypeSvc(inner);
                        let codec = tonic_prost::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    let mut response = http::Response::new(tonic::body::Body::default());
                    let headers = response.headers_mut();
                    headers.insert(
                        tonic::Status::GRPC_STATUS,
                        (tonic::Code::Unimplemented as i32).into(),
                    );
                    headers.insert(
                        http::header::CONTENT_TYPE,
                        tonic::metadata::GRPC_CONTENT_TYPE,
                    );
                    Ok(response)
                }),
            }
        }
    }
    impl<T> Clone for GraphServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    /// Generated gRPC service name
    pub const SERVICE_NAME: &str = "telividb.graph.v1.Graph";
    impl<T> tonic::server::NamedService for GraphServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
