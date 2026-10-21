use tauri::State;

use crate::launch::LaunchQueue;

/// Returns and clears the paths waiting from launch arguments.
#[tauri::command]
pub fn take_launch_paths(queue: State<'_, LaunchQueue>) -> Vec<String> {
    queue.take()
}
