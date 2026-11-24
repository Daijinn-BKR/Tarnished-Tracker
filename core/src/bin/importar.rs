use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let forzar = args.iter().any(|a| a == "--forzar");
    let rutas: Vec<&String> = args.iter().filter(|a| *a != "--forzar").collect();
    let [vault, data] = rutas[..] else {
        eprintln!("uso: importar <ruta-vault> <dir-data> [--forzar]");
        return ExitCode::from(2);
    };
    let data = PathBuf::from(data);
    // Reimportar borraría fotos, descripciones y fechas.
    if data.join("items.json").exists() && !forzar {
        eprintln!("{} ya existe; no se sobrescribe (usa --forzar si de verdad quieres perder los datos de la app)", data.join("items.json").display());
        return ExitCode::FAILURE;
    }
    let res = er_core::import::importar_vault(vault.as_ref())
        .and_then(|items| {
            std::fs::create_dir_all(&data)?;
            er_core::store::escribir_items(&data, &items)?;
            Ok(items.len())
        });
    match res {
        Ok(n) => { println!("{n} objetos importados"); ExitCode::SUCCESS }
        Err(e) => { eprintln!("error: {e}"); ExitCode::FAILURE }
    }
}
