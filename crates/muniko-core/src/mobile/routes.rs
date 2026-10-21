//! Mounts the phone page and API on the shared server.

use axum::extract::DefaultBodyLimit;
use axum::routing::{get, post};
use axum::Router;

use super::service::MobileService;
use super::{board_routes, events_route, inbox_routes, session_routes, spa, text_routes, upload};
use crate::constants::mobile::*;
use crate::constants::server::MAX_SMALL_JSON_BYTES;
use crate::server::RouteProvider;

pub struct MobileRoutes {
    service: MobileService,
}

impl MobileRoutes {
    pub fn new(service: MobileService) -> Self {
        Self { service }
    }
}

impl RouteProvider for MobileRoutes {
    fn routes(&self) -> Router {
        let text = get(text_routes::latest).post(text_routes::receive).layer(DefaultBodyLimit::max(MAX_TEXT_BODY_BYTES));
        let redeem = post(session_routes::redeem).layer(DefaultBodyLimit::max(MAX_SMALL_JSON_BYTES));
        let session = get(session_routes::current).merge(redeem).delete(session_routes::logout);
        Router::new()
            .route(ROUTE_MOBILE, get(spa::trailing_slash))
            .route(ROUTE_MOBILE_DIR, get(spa::root))
            .route(ROUTE_MOBILE_ASSET, get(spa::asset))
            .route(ROUTE_SESSION, session)
            .route(ROUTE_UPLOAD, post(upload::upload))
            .route(ROUTE_INBOX, get(inbox_routes::list))
            .route(ROUTE_INBOX_ITEM, get(inbox_routes::download))
            .route(ROUTE_TEXT, text)
            .route(ROUTE_BOARD, get(board_routes::list))
            .route(ROUTE_BOARD_TEXT, post(board_routes::post_text).layer(DefaultBodyLimit::max(MAX_TEXT_BODY_BYTES)))
            .route(ROUTE_BOARD_FILES, post(board_routes::post_file))
            .route(ROUTE_BOARD_FILE, get(board_routes::download))
            .route(ROUTE_EVENTS, get(events_route::events))
            .with_state(self.service.clone())
    }
}
