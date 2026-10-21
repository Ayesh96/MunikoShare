use tempfile::TempDir;

use super::*;

async fn node_in(root: &TempDir) -> MunikoNode {
    let config = NodeConfig {
        device_name: "node".to_string(),
        data_dir: root.path().join("data"),
        download_dir: root.path().join("downloads"),
    };
    MunikoNode::new(config).await.unwrap()
}

#[tokio::test]
async fn failed_start_releases_every_started_service() {
    let root = TempDir::new().unwrap();
    let mut node = node_in(&root).await;
    node.fail_start = true;
    assert!(node.start().await.is_err());
    assert_eq!(node.state(), LifecycleState::Stopped);
    assert!(node.server.port().is_none());
    assert!(node.discovery.is_none());
    assert_eq!(node.recorder.state(), LifecycleState::Stopped);
    assert_eq!(node.watcher.state(), LifecycleState::Stopped);
    assert_eq!(node.board_runtime.state(), LifecycleState::Stopped);
    node.stop().await.unwrap();
}

#[tokio::test]
async fn stop_before_start_is_safe() {
    let root = TempDir::new().unwrap();
    let mut node = node_in(&root).await;
    node.stop().await.unwrap();
    assert_eq!(node.state(), LifecycleState::Stopped);
}
