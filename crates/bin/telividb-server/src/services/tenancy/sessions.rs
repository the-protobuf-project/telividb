//! The `Sessions` service.

use super::convert::{born, session};
use super::{TenancySvc, already_exists, not_found, now_millis, parse};
use crate::error::storage_status;
use telividb_buffers::protobuf::tenancy::v1::sessions_server::Sessions;
use telividb_buffers::protobuf::tenancy::v1::{
    CreateSessionRequest, DeleteSessionRequest, GetSessionRequest, ListSessionsRequest,
    ListSessionsResponse, Session,
};
use telividb_core::Session as DomainSession;
use tonic::{Request, Response, Status};

#[tonic::async_trait]
impl Sessions for TenancySvc {
    async fn create_session(
        &self,
        request: Request<CreateSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let req = request.into_inner();
        let parent = parse(&req.parent)?;

        // `session_id` is optional in the proto, so the server names one when
        // the caller does not. A session is a working period rather than a
        // thing a person names up front, and refusing for want of a name would
        // make the common case the awkward one.
        let id = match req.session_id.is_empty() {
            true => format!("s-{}", now_millis()),
            false => req.session_id.clone(),
        };
        let name = parse(&format!("{}/sessions/{}", parent.as_str(), id))?;

        let payload = req.session.unwrap_or_default();
        let space = match payload.space.is_empty() {
            true => None,
            false => {
                let space = parse(&payload.space)?;
                // Checked before the write: a session pointing at a space that
                // does not exist is a dangling reference, and the graph would
                // carry it forward.
                self.store
                    .space(&space)
                    .map_err(|e| storage_status(&e))?
                    .ok_or_else(|| not_found(&space))?;
                Some(space)
            }
        };

        let value = DomainSession {
            name: name.clone(),
            display_name: payload.display_name,
            space,
            lifecycle: born(now_millis()),
        };

        match self
            .store
            .create_session(&value)
            .map_err(|e| storage_status(&e))?
        {
            true => Ok(Response::new(session(&value))),
            false => Err(already_exists(&name)),
        }
    }

    async fn get_session(
        &self,
        request: Request<GetSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self.store.session(&name).map_err(|e| storage_status(&e))? {
            Some(found) => Ok(Response::new(session(&found))),
            None => Err(not_found(&name)),
        }
    }

    async fn list_sessions(
        &self,
        request: Request<ListSessionsRequest>,
    ) -> Result<Response<ListSessionsResponse>, Status> {
        let parent = parse(&request.into_inner().parent)?;
        let found = self.store.sessions(false).map_err(|e| storage_status(&e))?;

        let prefix = format!("{}/sessions/", parent.as_str());
        Ok(Response::new(ListSessionsResponse {
            sessions: found
                .iter()
                .filter(|s| s.name.as_str().starts_with(&prefix))
                .map(session)
                .collect(),
            next_page_token: String::new(),
        }))
    }

    async fn delete_session(
        &self,
        request: Request<DeleteSessionRequest>,
    ) -> Result<Response<Session>, Status> {
        let name = parse(&request.into_inner().name)?;
        match self
            .store
            .delete_session(&name, now_millis())
            .map_err(|e| storage_status(&e))?
        {
            Some(deleted) => Ok(Response::new(session(&deleted))),
            None => Err(not_found(&name)),
        }
    }
}
