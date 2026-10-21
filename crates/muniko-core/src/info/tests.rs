use std::net::IpAddr;
use std::sync::Arc;

use super::*;
use crate::constants::info::ROUTE_INFO;
use crate::device::DeviceInfo;
use crate::identity::Identity;
use crate::lifecycle::Lifecycle;
use crate::net::{pinned_client_config, probe, PeerClient};
use crate::server::{HttpServer, RouteProvider};
use crate::util::test_support::shared_test_identity;

fn device(id: &str) -> DeviceInfo {
    DeviceInfo { id: id.into(), name: "Laptop".into(), os: "test".into(), addresses: Vec::new(), port: 0, protocol_version: 1 }
}

async fn server_claiming(identity: &Arc<Identity>, claimed_id: &str) -> (HttpServer, LocalDevice) {
    let local = LocalDevice::new(device(claimed_id));
    let providers: Vec<Arc<dyn RouteProvider>> = vec![Arc::new(InfoRoutes::new(local.clone()))];
    let mut server = HttpServer::new(identity.clone(), providers);
    server.start().await.unwrap();
    local.set_port(server.port().unwrap());
    (server, local)
}

fn loopback() -> IpAddr {
    "127.0.0.1".parse().unwrap()
}

#[tokio::test]
async fn info_route_returns_local_device_to_a_client_with_a_certificate() {
    let (server_identity, client_identity) = (shared_test_identity(), shared_test_identity());
    let (mut server, local) = server_claiming(&server_identity, server_identity.device_id()).await;
    let peer = DeviceInfo { addresses: vec![loopback()], ..local.snapshot().unwrap() };

    let client = PeerClient::new(&client_identity, &peer).unwrap();
    let response = client.send(|http, base| async move { Ok(http.get(format!("{base}{ROUTE_INFO}"))) }).await.unwrap();
    assert!(response.status().is_success());
    assert_eq!(response.json::<DeviceInfo>().await.unwrap(), local.snapshot().unwrap());
    server.stop().await.unwrap();
}

#[tokio::test]
async fn info_route_rejects_a_client_without_a_certificate() {
    let identity = shared_test_identity();
    let (mut server, _) = server_claiming(&identity, identity.device_id()).await;
    let config = pinned_client_config(identity.device_id(), None).unwrap();
    let client = reqwest::Client::builder().tls_backend_preconfigured(config).build().unwrap();
    let url = format!("https://127.0.0.1:{}{ROUTE_INFO}", server.port().unwrap());
    assert_eq!(client.get(url).send().await.unwrap().status(), reqwest::StatusCode::UNAUTHORIZED);
    server.stop().await.unwrap();
}

#[tokio::test]
async fn probe_returns_the_device_behind_the_address() {
    let (server_identity, client_identity) = (shared_test_identity(), shared_test_identity());
    let (mut server, _) = server_claiming(&server_identity, server_identity.device_id()).await;
    let port = server.port().unwrap();

    let found = probe(&client_identity, loopback(), port).await.unwrap();
    assert_eq!(found.id, server_identity.device_id());
    assert_eq!((found.addresses, found.port, found.name), (vec![loopback()], port, "Laptop".to_string()));
    server.stop().await.unwrap();
}

#[tokio::test]
async fn probe_rejects_an_id_that_differs_from_the_certificate() {
    let (server_identity, client_identity) = (shared_test_identity(), shared_test_identity());
    let (mut server, _) = server_claiming(&server_identity, &"f".repeat(64)).await;
    assert!(probe(&client_identity, loopback(), server.port().unwrap()).await.is_err());
    server.stop().await.unwrap();
}
