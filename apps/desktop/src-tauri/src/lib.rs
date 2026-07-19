mod codex;
mod contract;

use codex::{types::CodexRuntimeSnapshot, CodexRuntimeService};
use contract::DesktopBootstrap;

#[tauri::command]
fn desktop_bootstrap() -> DesktopBootstrap {
    DesktopBootstrap::current()
}

#[tauri::command]
async fn codex_runtime_probe(
    service: tauri::State<'_, CodexRuntimeService>,
) -> Result<CodexRuntimeSnapshot, ()> {
    Ok(service.snapshot().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(CodexRuntimeService::default())
        .invoke_handler(tauri::generate_handler![
            desktop_bootstrap,
            codex_runtime_probe
        ])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}
