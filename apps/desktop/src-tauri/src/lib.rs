mod contract;

use contract::DesktopBootstrap;

#[tauri::command]
fn desktop_bootstrap() -> DesktopBootstrap {
    DesktopBootstrap::current()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![desktop_bootstrap])
        .run(tauri::generate_context!())
        .expect("failed to run QuireForge");
}
