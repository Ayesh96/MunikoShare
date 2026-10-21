//! Server-Sent Events: tells the phone when to re-fetch.

use std::convert::Infallible;

use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use tokio::sync::broadcast::error::RecvError;

use super::auth::PhoneAuth;
use super::service::MobileService;
use crate::constants::mobile::{EVENT_DATA, SSE_KEEP_ALIVE};
use crate::messages::mobile as msg;

pub async fn events(State(service): State<MobileService>, PhoneAuth(session): PhoneAuth) -> Response {
    let Some(receiver) = service.subscribe(&session.id) else { return (axum::http::StatusCode::UNAUTHORIZED, msg::UNAUTHORIZED).into_response() };
    // The stream ends when the session's channel closes, i.e. right after `revoked`.
    let updates = stream::unfold(receiver, |mut receiver| async move {
        loop {
            match receiver.recv().await {
                Ok(event) => return Some((Ok::<Event, Infallible>(Event::default().event(event.name()).data(EVENT_DATA)), receiver)),
                Err(RecvError::Lagged(_)) => continue,
                Err(RecvError::Closed) => return None,
            }
        }
    });
    Sse::new(updates).keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE)).into_response()
}
