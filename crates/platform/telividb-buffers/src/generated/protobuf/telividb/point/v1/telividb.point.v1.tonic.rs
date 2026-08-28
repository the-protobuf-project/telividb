// @generated
/// Generated client implementations.
pub mod points_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    #[derive(Debug, Clone)]
    pub struct PointsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl PointsClient<tonic::transport::Channel> {
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
    impl<T> PointsClient<T>
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
        ) -> PointsClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::Body>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::Body>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::Body>,
            >>::Error: Into<StdError> + std::marker::Send + std::marker::Sync,
        {
            PointsClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_point(
            &mut self,
            request: impl tonic::IntoRequest<super::CreatePointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/CreatePoint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "CreatePoint"));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single point.
*/
        pub async fn get_point(
            &mut self,
            request: impl tonic::IntoRequest<super::GetPointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/GetPoint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "GetPoint"));
            self.inner.unary(req, path, codec).await
        }
        /** Lists the points of a collection.
*/
        pub async fn list_points(
            &mut self,
            request: impl tonic::IntoRequest<super::ListPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPointsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/ListPoints",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "ListPoints"));
            self.inner.unary(req, path, codec).await
        }
        /** Deletes a point.

 Returns the tombstoned point rather than an empty message. Deletion here
 is a soft delete by construction — sealed segments are immutable, so the
 point is tombstoned and excluded from queries at once while its bytes
 survive until compaction rewrites the segment. Returning the resource is
 the form AIP-135 defines for exactly that case, it matches what
 `BatchDeletePoints` already does, and it leaves room to add fields later
 that an empty return would have foreclosed.
*/
        pub async fn delete_point(
            &mut self,
            request: impl tonic::IntoRequest<super::DeletePointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status> {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/DeletePoint",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "DeletePoint"));
            self.inner.unary(req, path, codec).await
        }
        /** Creates several points in one request.
*/
        pub async fn batch_create_points(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchCreatePointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchCreatePointsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/BatchCreatePoints",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("telividb.point.v1.Points", "BatchCreatePoints"),
                );
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves several points in one request.
*/
        pub async fn batch_get_points(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchGetPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchGetPointsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/BatchGetPoints",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "BatchGetPoints"));
            self.inner.unary(req, path, codec).await
        }
        /** Deletes several points in one request.
*/
        pub async fn batch_delete_points(
            &mut self,
            request: impl tonic::IntoRequest<super::BatchDeletePointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchDeletePointsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/BatchDeletePoints",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("telividb.point.v1.Points", "BatchDeletePoints"),
                );
            self.inner.unary(req, path, codec).await
        }
        /** Searches a collection for the points nearest a query vector.

 A method on the points collection rather than a service of its own:
 searching is an operation over points, and modelling it as one keeps the
 resource hierarchy intact.
*/
        pub async fn search_points(
            &mut self,
            request: impl tonic::IntoRequest<super::SearchPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SearchPointsResponse>,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::unknown(
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.point.v1.Points/SearchPoints",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.point.v1.Points", "SearchPoints"));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod points_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value,
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with PointsServer.
    #[async_trait]
    pub trait Points: std::marker::Send + std::marker::Sync + 'static {
        async fn create_point(
            &self,
            request: tonic::Request<super::CreatePointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status>;
        /** Retrieves a single point.
*/
        async fn get_point(
            &self,
            request: tonic::Request<super::GetPointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status>;
        /** Lists the points of a collection.
*/
        async fn list_points(
            &self,
            request: tonic::Request<super::ListPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::ListPointsResponse>,
            tonic::Status,
        >;
        /** Deletes a point.

 Returns the tombstoned point rather than an empty message. Deletion here
 is a soft delete by construction — sealed segments are immutable, so the
 point is tombstoned and excluded from queries at once while its bytes
 survive until compaction rewrites the segment. Returning the resource is
 the form AIP-135 defines for exactly that case, it matches what
 `BatchDeletePoints` already does, and it leaves room to add fields later
 that an empty return would have foreclosed.
*/
        async fn delete_point(
            &self,
            request: tonic::Request<super::DeletePointRequest>,
        ) -> std::result::Result<tonic::Response<super::Point>, tonic::Status>;
        /** Creates several points in one request.
*/
        async fn batch_create_points(
            &self,
            request: tonic::Request<super::BatchCreatePointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchCreatePointsResponse>,
            tonic::Status,
        >;
        /** Retrieves several points in one request.
*/
        async fn batch_get_points(
            &self,
            request: tonic::Request<super::BatchGetPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchGetPointsResponse>,
            tonic::Status,
        >;
        /** Deletes several points in one request.
*/
        async fn batch_delete_points(
            &self,
            request: tonic::Request<super::BatchDeletePointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::BatchDeletePointsResponse>,
            tonic::Status,
        >;
        /** Searches a collection for the points nearest a query vector.

 A method on the points collection rather than a service of its own:
 searching is an operation over points, and modelling it as one keeps the
 resource hierarchy intact.
*/
        async fn search_points(
            &self,
            request: tonic::Request<super::SearchPointsRequest>,
        ) -> std::result::Result<
            tonic::Response<super::SearchPointsResponse>,
            tonic::Status,
        >;
    }
    #[derive(Debug)]
    pub struct PointsServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> PointsServer<T> {
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
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for PointsServer<T>
    where
        T: Points,
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
                "/telividb.point.v1.Points/CreatePoint" => {
                    #[allow(non_camel_case_types)]
                    struct CreatePointSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::CreatePointRequest>
                    for CreatePointSvc<T> {
                        type Response = super::Point;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreatePointRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::create_point(&inner, request).await
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
                        let method = CreatePointSvc(inner);
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
                "/telividb.point.v1.Points/GetPoint" => {
                    #[allow(non_camel_case_types)]
                    struct GetPointSvc<T: Points>(pub Arc<T>);
                    impl<T: Points> tonic::server::UnaryService<super::GetPointRequest>
                    for GetPointSvc<T> {
                        type Response = super::Point;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetPointRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::get_point(&inner, request).await
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
                        let method = GetPointSvc(inner);
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
                "/telividb.point.v1.Points/ListPoints" => {
                    #[allow(non_camel_case_types)]
                    struct ListPointsSvc<T: Points>(pub Arc<T>);
                    impl<T: Points> tonic::server::UnaryService<super::ListPointsRequest>
                    for ListPointsSvc<T> {
                        type Response = super::ListPointsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListPointsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::list_points(&inner, request).await
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
                        let method = ListPointsSvc(inner);
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
                "/telividb.point.v1.Points/DeletePoint" => {
                    #[allow(non_camel_case_types)]
                    struct DeletePointSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::DeletePointRequest>
                    for DeletePointSvc<T> {
                        type Response = super::Point;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeletePointRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::delete_point(&inner, request).await
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
                        let method = DeletePointSvc(inner);
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
                "/telividb.point.v1.Points/BatchCreatePoints" => {
                    #[allow(non_camel_case_types)]
                    struct BatchCreatePointsSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::BatchCreatePointsRequest>
                    for BatchCreatePointsSvc<T> {
                        type Response = super::BatchCreatePointsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchCreatePointsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::batch_create_points(&inner, request).await
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
                        let method = BatchCreatePointsSvc(inner);
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
                "/telividb.point.v1.Points/BatchGetPoints" => {
                    #[allow(non_camel_case_types)]
                    struct BatchGetPointsSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::BatchGetPointsRequest>
                    for BatchGetPointsSvc<T> {
                        type Response = super::BatchGetPointsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchGetPointsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::batch_get_points(&inner, request).await
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
                        let method = BatchGetPointsSvc(inner);
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
                "/telividb.point.v1.Points/BatchDeletePoints" => {
                    #[allow(non_camel_case_types)]
                    struct BatchDeletePointsSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::BatchDeletePointsRequest>
                    for BatchDeletePointsSvc<T> {
                        type Response = super::BatchDeletePointsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::BatchDeletePointsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::batch_delete_points(&inner, request).await
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
                        let method = BatchDeletePointsSvc(inner);
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
                "/telividb.point.v1.Points/SearchPoints" => {
                    #[allow(non_camel_case_types)]
                    struct SearchPointsSvc<T: Points>(pub Arc<T>);
                    impl<
                        T: Points,
                    > tonic::server::UnaryService<super::SearchPointsRequest>
                    for SearchPointsSvc<T> {
                        type Response = super::SearchPointsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SearchPointsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Points>::search_points(&inner, request).await
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
                        let method = SearchPointsSvc(inner);
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
                _ => {
                    Box::pin(async move {
                        let mut response = http::Response::new(
                            tonic::body::Body::default(),
                        );
                        let headers = response.headers_mut();
                        headers
                            .insert(
                                tonic::Status::GRPC_STATUS,
                                (tonic::Code::Unimplemented as i32).into(),
                            );
                        headers
                            .insert(
                                http::header::CONTENT_TYPE,
                                tonic::metadata::GRPC_CONTENT_TYPE,
                            );
                        Ok(response)
                    })
                }
            }
        }
    }
    impl<T> Clone for PointsServer<T> {
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
    pub const SERVICE_NAME: &str = "telividb.point.v1.Points";
    impl<T> tonic::server::NamedService for PointsServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
