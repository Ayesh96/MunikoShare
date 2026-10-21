/// A feature module that plugs its routes into the shared server.
pub trait RouteProvider: Send + Sync {
    fn routes(&self) -> axum::Router;
}
