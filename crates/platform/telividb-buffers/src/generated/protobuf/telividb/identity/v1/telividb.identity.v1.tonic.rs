// @generated
/// Generated client implementations.
pub mod identity_client {
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
    pub struct IdentityClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl IdentityClient<tonic::transport::Channel> {
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
    impl<T> IdentityClient<T>
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
        ) -> IdentityClient<InterceptedService<T, F>>
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
            IdentityClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_user(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/CreateUser");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "CreateUser",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/GetUser");
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("telividb.identity.v1.Identity", "GetUser"));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_users(
            &mut self,
            request: impl tonic::IntoRequest<super::ListUsersRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUsersResponse>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/ListUsers");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "ListUsers",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/DeleteUser");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "DeleteUser",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn undelete_user(
            &mut self,
            request: impl tonic::IntoRequest<super::UndeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/UndeleteUser");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "UndeleteUser",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_user_group(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/CreateUserGroup",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "CreateUserGroup",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_user_group(
            &mut self,
            request: impl tonic::IntoRequest<super::GetUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path =
                http::uri::PathAndQuery::from_static("/telividb.identity.v1.Identity/GetUserGroup");
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "GetUserGroup",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_user_groups(
            &mut self,
            request: impl tonic::IntoRequest<super::ListUserGroupsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUserGroupsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/ListUserGroups",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "ListUserGroups",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_user_group(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/DeleteUserGroup",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "DeleteUserGroup",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn create_role_binding(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/CreateRoleBinding",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "CreateRoleBinding",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn get_role_binding(
            &mut self,
            request: impl tonic::IntoRequest<super::GetRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/GetRoleBinding",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "GetRoleBinding",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn list_role_bindings(
            &mut self,
            request: impl tonic::IntoRequest<super::ListRoleBindingsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListRoleBindingsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/ListRoleBindings",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "ListRoleBindings",
            ));
            self.inner.unary(req, path, codec).await
        }
        pub async fn delete_role_binding(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.identity.v1.Identity/DeleteRoleBinding",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.identity.v1.Identity",
                "DeleteRoleBinding",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod identity_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with IdentityServer.
    #[async_trait]
    pub trait Identity: std::marker::Send + std::marker::Sync + 'static {
        async fn create_user(
            &self,
            request: tonic::Request<super::CreateUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status>;
        async fn get_user(
            &self,
            request: tonic::Request<super::GetUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status>;
        async fn list_users(
            &self,
            request: tonic::Request<super::ListUsersRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUsersResponse>, tonic::Status>;
        async fn delete_user(
            &self,
            request: tonic::Request<super::DeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status>;
        async fn undelete_user(
            &self,
            request: tonic::Request<super::UndeleteUserRequest>,
        ) -> std::result::Result<tonic::Response<super::User>, tonic::Status>;
        async fn create_user_group(
            &self,
            request: tonic::Request<super::CreateUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status>;
        async fn get_user_group(
            &self,
            request: tonic::Request<super::GetUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status>;
        async fn list_user_groups(
            &self,
            request: tonic::Request<super::ListUserGroupsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListUserGroupsResponse>, tonic::Status>;
        async fn delete_user_group(
            &self,
            request: tonic::Request<super::DeleteUserGroupRequest>,
        ) -> std::result::Result<tonic::Response<super::UserGroup>, tonic::Status>;
        async fn create_role_binding(
            &self,
            request: tonic::Request<super::CreateRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status>;
        async fn get_role_binding(
            &self,
            request: tonic::Request<super::GetRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status>;
        async fn list_role_bindings(
            &self,
            request: tonic::Request<super::ListRoleBindingsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListRoleBindingsResponse>, tonic::Status>;
        async fn delete_role_binding(
            &self,
            request: tonic::Request<super::DeleteRoleBindingRequest>,
        ) -> std::result::Result<tonic::Response<super::RoleBinding>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct IdentityServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> IdentityServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for IdentityServer<T>
    where
        T: Identity,
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
                "/telividb.identity.v1.Identity/CreateUser" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::CreateUserRequest> for CreateUserSvc<T> {
                        type Response = super::User;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Identity>::create_user(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = CreateUserSvc(inner);
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
                "/telividb.identity.v1.Identity/GetUser" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::GetUserRequest> for GetUserSvc<T> {
                        type Response = super::User;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Identity>::get_user(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = GetUserSvc(inner);
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
                "/telividb.identity.v1.Identity/ListUsers" => {
                    #[allow(non_camel_case_types)]
                    struct ListUsersSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::ListUsersRequest> for ListUsersSvc<T> {
                        type Response = super::ListUsersResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListUsersRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Identity>::list_users(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = ListUsersSvc(inner);
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
                "/telividb.identity.v1.Identity/DeleteUser" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::DeleteUserRequest> for DeleteUserSvc<T> {
                        type Response = super::User;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut =
                                async move { <T as Identity>::delete_user(&inner, request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let method = DeleteUserSvc(inner);
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
                "/telividb.identity.v1.Identity/UndeleteUser" => {
                    #[allow(non_camel_case_types)]
                    struct UndeleteUserSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::UndeleteUserRequest> for UndeleteUserSvc<T> {
                        type Response = super::User;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UndeleteUserRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::undelete_user(&inner, request).await
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
                        let method = UndeleteUserSvc(inner);
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
                "/telividb.identity.v1.Identity/CreateUserGroup" => {
                    #[allow(non_camel_case_types)]
                    struct CreateUserGroupSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::CreateUserGroupRequest>
                        for CreateUserGroupSvc<T>
                    {
                        type Response = super::UserGroup;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateUserGroupRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::create_user_group(&inner, request).await
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
                        let method = CreateUserGroupSvc(inner);
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
                "/telividb.identity.v1.Identity/GetUserGroup" => {
                    #[allow(non_camel_case_types)]
                    struct GetUserGroupSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::GetUserGroupRequest> for GetUserGroupSvc<T> {
                        type Response = super::UserGroup;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetUserGroupRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::get_user_group(&inner, request).await
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
                        let method = GetUserGroupSvc(inner);
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
                "/telividb.identity.v1.Identity/ListUserGroups" => {
                    #[allow(non_camel_case_types)]
                    struct ListUserGroupsSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::ListUserGroupsRequest>
                        for ListUserGroupsSvc<T>
                    {
                        type Response = super::ListUserGroupsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListUserGroupsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::list_user_groups(&inner, request).await
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
                        let method = ListUserGroupsSvc(inner);
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
                "/telividb.identity.v1.Identity/DeleteUserGroup" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteUserGroupSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::DeleteUserGroupRequest>
                        for DeleteUserGroupSvc<T>
                    {
                        type Response = super::UserGroup;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteUserGroupRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::delete_user_group(&inner, request).await
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
                        let method = DeleteUserGroupSvc(inner);
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
                "/telividb.identity.v1.Identity/CreateRoleBinding" => {
                    #[allow(non_camel_case_types)]
                    struct CreateRoleBindingSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::CreateRoleBindingRequest>
                        for CreateRoleBindingSvc<T>
                    {
                        type Response = super::RoleBinding;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateRoleBindingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::create_role_binding(&inner, request).await
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
                        let method = CreateRoleBindingSvc(inner);
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
                "/telividb.identity.v1.Identity/GetRoleBinding" => {
                    #[allow(non_camel_case_types)]
                    struct GetRoleBindingSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::GetRoleBindingRequest>
                        for GetRoleBindingSvc<T>
                    {
                        type Response = super::RoleBinding;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetRoleBindingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::get_role_binding(&inner, request).await
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
                        let method = GetRoleBindingSvc(inner);
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
                "/telividb.identity.v1.Identity/ListRoleBindings" => {
                    #[allow(non_camel_case_types)]
                    struct ListRoleBindingsSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::ListRoleBindingsRequest>
                        for ListRoleBindingsSvc<T>
                    {
                        type Response = super::ListRoleBindingsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListRoleBindingsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::list_role_bindings(&inner, request).await
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
                        let method = ListRoleBindingsSvc(inner);
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
                "/telividb.identity.v1.Identity/DeleteRoleBinding" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteRoleBindingSvc<T: Identity>(pub Arc<T>);
                    impl<T: Identity> tonic::server::UnaryService<super::DeleteRoleBindingRequest>
                        for DeleteRoleBindingSvc<T>
                    {
                        type Response = super::RoleBinding;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteRoleBindingRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Identity>::delete_role_binding(&inner, request).await
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
                        let method = DeleteRoleBindingSvc(inner);
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
    impl<T> Clone for IdentityServer<T> {
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
    pub const SERVICE_NAME: &str = "telividb.identity.v1.Identity";
    impl<T> tonic::server::NamedService for IdentityServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
