use er_core::import::importar_vault;
use std::collections::HashSet;
use std::path::PathBuf;
use std::process::Command;

fn vault() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap()).join("Proyectos/elden-ring-wiki/Elden ring")
}

// Números sacados del vault con grep (2026-11-24), independientes del parser.
#[test]
fn vault_real() {
    if !vault().exists() {
        eprintln!("vault no encontrado, se omite");
        return;
    }
    let v = importar_vault(&vault()).unwrap();
    assert_eq!(v.len(), 1826);
    assert_eq!(v.iter().filter(|i| i.conseguido).count(), 1759);
    assert_eq!(v.iter().filter(|i| i.farmear).count(), 280);
    assert_eq!(v.iter().filter(|i| i.ng_plus).count(), 32);
    let ids: HashSet<_> = v.iter().map(|i| &i.id).collect();
    assert_eq!(ids.len(), v.len());
    assert!(v.iter().all(|i| !i.categoria.is_empty()));
    assert_eq!(v.iter().map(|i| &i.region).collect::<HashSet<_>>().len(), 28);
}

#[test]
fn no_pisa_sin_forzar() {
    let d = std::env::temp_dir().join(format!("er_importar_{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    std::fs::write(d.join("items.json"), "[\"mio\"]").unwrap();
    let st = Command::new(env!("CARGO_BIN_EXE_importar")).arg(vault()).arg(&d).status().unwrap();
    assert!(!st.success());
    assert_eq!(std::fs::read_to_string(d.join("items.json")).unwrap(), "[\"mio\"]");
}
