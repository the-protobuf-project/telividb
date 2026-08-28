// @generated
/// Generated client implementations.
pub mod conversations_client {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::http::Uri;
    use tonic::codegen::*;
    /** Manages conversations and the messages within them.
    */
    #[derive(Debug, Clone)]
    pub struct ConversationsClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl ConversationsClient<tonic::transport::Channel> {
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
    impl<T> ConversationsClient<T>
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
        ) -> ConversationsClient<InterceptedService<T, F>>
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
            ConversationsClient::new(InterceptedService::new(inner, interceptor))
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
        pub async fn create_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/CreateConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "CreateConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single conversation.
        */
        pub async fn get_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::GetConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/GetConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "GetConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Lists conversations, filtered by space, session or project.
        */
        pub async fn list_conversations(
            &mut self,
            request: impl tonic::IntoRequest<super::ListConversationsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListConversationsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/ListConversations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "ListConversations",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Moves a conversation into another session, and so into another space.

         The resource name does not change, which is the entire reason session and
         space are fields rather than path segments: a move that renamed the
         conversation would invalidate every edge endpoint, archive entry and
         citation that refers to it.

         The move itself is a metadata transaction. No vector data is touched,
         because membership is never written into a segment — which is what makes
         this cheap rather than a rewrite.
        */
        pub async fn move_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::MoveConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::MoveConversationResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/MoveConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "MoveConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Excludes a conversation from ordinary recall without deleting it.

         Suppression is a third state, distinct from deletion and from a policy
         denial: the content is retained, its owner can still reach it deliberately,
         and traversal stops at it rather than ranking it low. Its existence stays
         visible to a caller assembling context — so an agent knows what not to
         raise — while the rendered answer must not surface it, because delivering
         the reminder is the harm suppression was asked for.

         A separate method rather than a field update so that setting it is its own
         audit entry. Only the owner may call it; a classifier may propose
         suppression and may never apply or lift it.
        */
        pub async fn suppress_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::SuppressConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/SuppressConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "SuppressConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Returns a suppressed conversation to ordinary recall.
        */
        pub async fn unsuppress_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::UnsuppressConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/UnsuppressConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "UnsuppressConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Soft-deletes a conversation and the messages beneath it.
        */
        pub async fn delete_conversation(
            &mut self,
            request: impl tonic::IntoRequest<super::DeleteConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/DeleteConversation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "DeleteConversation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Appends a message, branching where the request says to.

         Embedding happens server-side through the one inference server, so the
         model that produced a field's vectors is a property of the server rather
         than of whichever client wrote them. System and developer turns are stored
         without being encoded.
        */
        pub async fn create_message(
            &mut self,
            request: impl tonic::IntoRequest<super::CreateMessageRequest>,
        ) -> std::result::Result<tonic::Response<super::Message>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/CreateMessage",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "CreateMessage",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single message.
        */
        pub async fn get_message(
            &mut self,
            request: impl tonic::IntoRequest<super::GetMessageRequest>,
        ) -> std::result::Result<tonic::Response<super::Message>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/GetMessage",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "GetMessage",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Lists the messages of a conversation, every branch by default.
        */
        pub async fn list_messages(
            &mut self,
            request: impl tonic::IntoRequest<super::ListMessagesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListMessagesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/ListMessages",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "ListMessages",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single tool invocation.
        */
        pub async fn get_tool_invocation(
            &mut self,
            request: impl tonic::IntoRequest<super::GetToolInvocationRequest>,
        ) -> std::result::Result<tonic::Response<super::ToolInvocation>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/GetToolInvocation",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "GetToolInvocation",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Lists the tool invocations of a conversation.

         A collection of its own rather than a field on the message list. A call and
         its result are one resource here, and returning them inside the message
         stream would rebuild the two-message split this model exists to avoid.
        */
        pub async fn list_tool_invocations(
            &mut self,
            request: impl tonic::IntoRequest<super::ListToolInvocationsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListToolInvocationsResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/ListToolInvocations",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "ListToolInvocations",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Retrieves a single summary.
        */
        pub async fn get_summary(
            &mut self,
            request: impl tonic::IntoRequest<super::GetSummaryRequest>,
        ) -> std::result::Result<tonic::Response<super::Summary>, tonic::Status> {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/GetSummary",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "GetSummary",
            ));
            self.inner.unary(req, path, codec).await
        }
        /** Lists summaries, filtered by subject and level.

         The coarse tier of retrieval: a search runs against these first and
         expands into messages only where it needs to. Each carries the watermark
         it reached and whether its subject has changed since, so a stale summary
         is served honestly rather than passed off as current.
        */
        pub async fn list_summaries(
            &mut self,
            request: impl tonic::IntoRequest<super::ListSummariesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListSummariesResponse>, tonic::Status>
        {
            self.inner.ready().await.map_err(|e| {
                tonic::Status::unknown(format!("Service was not ready: {}", e.into()))
            })?;
            let codec = tonic_prost::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/telividb.conversation.v1.Conversations/ListSummaries",
            );
            let mut req = request.into_request();
            req.extensions_mut().insert(GrpcMethod::new(
                "telividb.conversation.v1.Conversations",
                "ListSummaries",
            ));
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
pub mod conversations_server {
    #![allow(
        unused_variables,
        dead_code,
        missing_docs,
        clippy::wildcard_imports,
        clippy::let_unit_value
    )]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with ConversationsServer.
    #[async_trait]
    pub trait Conversations: std::marker::Send + std::marker::Sync + 'static {
        async fn create_conversation(
            &self,
            request: tonic::Request<super::CreateConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status>;
        /** Retrieves a single conversation.
        */
        async fn get_conversation(
            &self,
            request: tonic::Request<super::GetConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status>;
        /** Lists conversations, filtered by space, session or project.
        */
        async fn list_conversations(
            &self,
            request: tonic::Request<super::ListConversationsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListConversationsResponse>, tonic::Status>;
        /** Moves a conversation into another session, and so into another space.

         The resource name does not change, which is the entire reason session and
         space are fields rather than path segments: a move that renamed the
         conversation would invalidate every edge endpoint, archive entry and
         citation that refers to it.

         The move itself is a metadata transaction. No vector data is touched,
         because membership is never written into a segment — which is what makes
         this cheap rather than a rewrite.
        */
        async fn move_conversation(
            &self,
            request: tonic::Request<super::MoveConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::MoveConversationResponse>, tonic::Status>;
        /** Excludes a conversation from ordinary recall without deleting it.

         Suppression is a third state, distinct from deletion and from a policy
         denial: the content is retained, its owner can still reach it deliberately,
         and traversal stops at it rather than ranking it low. Its existence stays
         visible to a caller assembling context — so an agent knows what not to
         raise — while the rendered answer must not surface it, because delivering
         the reminder is the harm suppression was asked for.

         A separate method rather than a field update so that setting it is its own
         audit entry. Only the owner may call it; a classifier may propose
         suppression and may never apply or lift it.
        */
        async fn suppress_conversation(
            &self,
            request: tonic::Request<super::SuppressConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status>;
        /** Returns a suppressed conversation to ordinary recall.
        */
        async fn unsuppress_conversation(
            &self,
            request: tonic::Request<super::UnsuppressConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status>;
        /** Soft-deletes a conversation and the messages beneath it.
        */
        async fn delete_conversation(
            &self,
            request: tonic::Request<super::DeleteConversationRequest>,
        ) -> std::result::Result<tonic::Response<super::Conversation>, tonic::Status>;
        /** Appends a message, branching where the request says to.

         Embedding happens server-side through the one inference server, so the
         model that produced a field's vectors is a property of the server rather
         than of whichever client wrote them. System and developer turns are stored
         without being encoded.
        */
        async fn create_message(
            &self,
            request: tonic::Request<super::CreateMessageRequest>,
        ) -> std::result::Result<tonic::Response<super::Message>, tonic::Status>;
        /** Retrieves a single message.
        */
        async fn get_message(
            &self,
            request: tonic::Request<super::GetMessageRequest>,
        ) -> std::result::Result<tonic::Response<super::Message>, tonic::Status>;
        /** Lists the messages of a conversation, every branch by default.
        */
        async fn list_messages(
            &self,
            request: tonic::Request<super::ListMessagesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListMessagesResponse>, tonic::Status>;
        /** Retrieves a single tool invocation.
        */
        async fn get_tool_invocation(
            &self,
            request: tonic::Request<super::GetToolInvocationRequest>,
        ) -> std::result::Result<tonic::Response<super::ToolInvocation>, tonic::Status>;
        /** Lists the tool invocations of a conversation.

         A collection of its own rather than a field on the message list. A call and
         its result are one resource here, and returning them inside the message
         stream would rebuild the two-message split this model exists to avoid.
        */
        async fn list_tool_invocations(
            &self,
            request: tonic::Request<super::ListToolInvocationsRequest>,
        ) -> std::result::Result<tonic::Response<super::ListToolInvocationsResponse>, tonic::Status>;
        /** Retrieves a single summary.
        */
        async fn get_summary(
            &self,
            request: tonic::Request<super::GetSummaryRequest>,
        ) -> std::result::Result<tonic::Response<super::Summary>, tonic::Status>;
        /** Lists summaries, filtered by subject and level.

         The coarse tier of retrieval: a search runs against these first and
         expands into messages only where it needs to. Each carries the watermark
         it reached and whether its subject has changed since, so a stale summary
         is served honestly rather than passed off as current.
        */
        async fn list_summaries(
            &self,
            request: tonic::Request<super::ListSummariesRequest>,
        ) -> std::result::Result<tonic::Response<super::ListSummariesResponse>, tonic::Status>;
    }
    /** Manages conversations and the messages within them.
    */
    #[derive(Debug)]
    pub struct ConversationsServer<T> {
        inner: Arc<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    impl<T> ConversationsServer<T> {
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
    impl<T, B> tonic::codegen::Service<http::Request<B>> for ConversationsServer<T>
    where
        T: Conversations,
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
                "/telividb.conversation.v1.Conversations/CreateConversation" => {
                    #[allow(non_camel_case_types)]
                    struct CreateConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::CreateConversationRequest>
                        for CreateConversationSvc<T>
                    {
                        type Response = super::Conversation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::create_conversation(&inner, request).await
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
                        let method = CreateConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/GetConversation" => {
                    #[allow(non_camel_case_types)]
                    struct GetConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::GetConversationRequest>
                        for GetConversationSvc<T>
                    {
                        type Response = super::Conversation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::get_conversation(&inner, request).await
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
                        let method = GetConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/ListConversations" => {
                    #[allow(non_camel_case_types)]
                    struct ListConversationsSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::ListConversationsRequest>
                        for ListConversationsSvc<T>
                    {
                        type Response = super::ListConversationsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListConversationsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::list_conversations(&inner, request).await
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
                        let method = ListConversationsSvc(inner);
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
                "/telividb.conversation.v1.Conversations/MoveConversation" => {
                    #[allow(non_camel_case_types)]
                    struct MoveConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::MoveConversationRequest>
                        for MoveConversationSvc<T>
                    {
                        type Response = super::MoveConversationResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::MoveConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::move_conversation(&inner, request).await
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
                        let method = MoveConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/SuppressConversation" => {
                    #[allow(non_camel_case_types)]
                    struct SuppressConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::SuppressConversationRequest>
                        for SuppressConversationSvc<T>
                    {
                        type Response = super::Conversation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::SuppressConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::suppress_conversation(&inner, request).await
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
                        let method = SuppressConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/UnsuppressConversation" => {
                    #[allow(non_camel_case_types)]
                    struct UnsuppressConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::UnsuppressConversationRequest>
                        for UnsuppressConversationSvc<T>
                    {
                        type Response = super::Conversation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::UnsuppressConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::unsuppress_conversation(&inner, request).await
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
                        let method = UnsuppressConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/DeleteConversation" => {
                    #[allow(non_camel_case_types)]
                    struct DeleteConversationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::DeleteConversationRequest>
                        for DeleteConversationSvc<T>
                    {
                        type Response = super::Conversation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::DeleteConversationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::delete_conversation(&inner, request).await
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
                        let method = DeleteConversationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/CreateMessage" => {
                    #[allow(non_camel_case_types)]
                    struct CreateMessageSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations> tonic::server::UnaryService<super::CreateMessageRequest>
                        for CreateMessageSvc<T>
                    {
                        type Response = super::Message;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::CreateMessageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::create_message(&inner, request).await
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
                        let method = CreateMessageSvc(inner);
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
                "/telividb.conversation.v1.Conversations/GetMessage" => {
                    #[allow(non_camel_case_types)]
                    struct GetMessageSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations> tonic::server::UnaryService<super::GetMessageRequest> for GetMessageSvc<T> {
                        type Response = super::Message;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetMessageRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::get_message(&inner, request).await
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
                        let method = GetMessageSvc(inner);
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
                "/telividb.conversation.v1.Conversations/ListMessages" => {
                    #[allow(non_camel_case_types)]
                    struct ListMessagesSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations> tonic::server::UnaryService<super::ListMessagesRequest>
                        for ListMessagesSvc<T>
                    {
                        type Response = super::ListMessagesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListMessagesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::list_messages(&inner, request).await
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
                        let method = ListMessagesSvc(inner);
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
                "/telividb.conversation.v1.Conversations/GetToolInvocation" => {
                    #[allow(non_camel_case_types)]
                    struct GetToolInvocationSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::GetToolInvocationRequest>
                        for GetToolInvocationSvc<T>
                    {
                        type Response = super::ToolInvocation;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetToolInvocationRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::get_tool_invocation(&inner, request).await
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
                        let method = GetToolInvocationSvc(inner);
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
                "/telividb.conversation.v1.Conversations/ListToolInvocations" => {
                    #[allow(non_camel_case_types)]
                    struct ListToolInvocationsSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations>
                        tonic::server::UnaryService<super::ListToolInvocationsRequest>
                        for ListToolInvocationsSvc<T>
                    {
                        type Response = super::ListToolInvocationsResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListToolInvocationsRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::list_tool_invocations(&inner, request).await
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
                        let method = ListToolInvocationsSvc(inner);
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
                "/telividb.conversation.v1.Conversations/GetSummary" => {
                    #[allow(non_camel_case_types)]
                    struct GetSummarySvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations> tonic::server::UnaryService<super::GetSummaryRequest> for GetSummarySvc<T> {
                        type Response = super::Summary;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GetSummaryRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::get_summary(&inner, request).await
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
                        let method = GetSummarySvc(inner);
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
                "/telividb.conversation.v1.Conversations/ListSummaries" => {
                    #[allow(non_camel_case_types)]
                    struct ListSummariesSvc<T: Conversations>(pub Arc<T>);
                    impl<T: Conversations> tonic::server::UnaryService<super::ListSummariesRequest>
                        for ListSummariesSvc<T>
                    {
                        type Response = super::ListSummariesResponse;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::ListSummariesRequest>,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Conversations>::list_summaries(&inner, request).await
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
                        let method = ListSummariesSvc(inner);
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
    impl<T> Clone for ConversationsServer<T> {
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
    pub const SERVICE_NAME: &str = "telividb.conversation.v1.Conversations";
    impl<T> tonic::server::NamedService for ConversationsServer<T> {
        const NAME: &'static str = SERVICE_NAME;
    }
}
