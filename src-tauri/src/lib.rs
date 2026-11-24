use er_core::model::{Cambios, Item};
use er_core::store::Store;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{Manager, State};

// shortcut: data/ fijo junto al código fuente, válido para uso personal con `cargo tauri dev`; si se empaqueta, pasar a app_data_dir.
fn dir_datos() -> PathBuf {
    PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../data"))
}

/// Err guarda el motivo por el que no se pudo abrir items.json; nunca se crea uno vacío encima.
struct Estado(Mutex<Result<Store, String>>);

fn con_store<T>(e: &State<Estado>, f: impl FnOnce(&mut Store) -> std::io::Result<T>) -> Result<T, String> {
    let mut g = e.0.lock().map_err(|e| e.to_string())?;
    if g.is_err() {
        *g = Store::abrir(&dir_datos()).map_err(|e| e.to_string());
    }
    let s = g.as_mut().map_err(|m| format!("No se pudo abrir data/items.json: {m}"))?;
    f(s).map_err(|e| e.to_string())
}

#[tauri::command]
fn listar_items(e: State<Estado>) -> Result<Vec<Item>, String> {
    con_store(&e, |s| { s.recargar_si_cambio()?; Ok(s.items.clone()) })
}

#[tauri::command]
fn actualizar_item(e: State<Estado>, id: String, cambios: Cambios) -> Result<Item, String> {
    con_store(&e, |s| s.actualizar(&id, cambios))
}

#[tauri::command]
fn guardar_imagen(e: State<Estado>, id: String, ext: String, bytes: Vec<u8>) -> Result<Item, String> {
    con_store(&e, |s| s.guardar_imagen(&id, &ext, &bytes))
}

#[tauri::command(rename_all = "snake_case")]
fn ruta_datos() -> String {
    dir_datos().canonicalize().unwrap_or_else(|_| dir_datos()).display().to_string()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(Estado(Mutex::new(Store::abrir(&dir_datos()).map_err(|e| e.to_string()))))
        .setup(|app| {
            let _ = std::fs::create_dir_all(dir_datos().join("imagenes"));
            app.asset_protocol_scope().allow_directory(dir_datos().canonicalize()?, true)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![listar_items, actualizar_item, guardar_imagen, ruta_datos])
        .run(tauri::generate_context!())
        .expect("error al iniciar la app");
}
