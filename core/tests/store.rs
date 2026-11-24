use er_core::model::{Cambios, Item, Origen};
use er_core::store::{escribir_items, Store};
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

fn dir_temp(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("er_store_{}_{}", nombre, std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn item(id: &str) -> Item {
    Item {
        id: id.into(), nombre: id.into(), zona: "Z".into(), region: "R".into(),
        categoria: "Armas".into(), subcategoria: None,
        conseguido: false, conseguido_en: None, farmear: false, ng_plus: false,
        descripcion: String::new(), descripcion_origen: Origen::Ninguna, imagen: None,
    }
}

fn con_items(nombre: &str) -> (PathBuf, Store) {
    let d = dir_temp(nombre);
    escribir_items(&d, &[item("r/a"), item("r/b")]).unwrap();
    let s = Store::abrir(&d).unwrap();
    (d, s)
}

#[test]
fn abrir_falla_sin_archivo() {
    assert!(Store::abrir(&dir_temp("vacio")).is_err());
}

#[test]
fn abrir_falla_con_json_corrupto() {
    let d = dir_temp("corrupto");
    fs::write(d.join("items.json"), "{no").unwrap();
    assert!(Store::abrir(&d).is_err());
    assert_eq!(fs::read_to_string(d.join("items.json")).unwrap(), "{no");
}

#[test]
fn marcar_y_desmarcar() {
    let (_d, mut s) = con_items("marcar");
    let it = s.actualizar("r/a", Cambios { conseguido: Some(true), conseguido_en: Some(Some("2026-11-24".into())), ..Default::default() }).unwrap();
    assert!(it.conseguido);
    assert_eq!(it.conseguido_en.as_deref(), Some("2026-11-24"));
    let it = s.actualizar("r/a", Cambios { conseguido: Some(false), ..Default::default() }).unwrap();
    assert!(!it.conseguido);
    assert_eq!(it.conseguido_en, None);
}

#[test]
fn editar_descripcion_marca_manual() {
    let (_d, mut s) = con_items("desc");
    let it = s.actualizar("r/a", Cambios { descripcion: Some("x".into()), ..Default::default() }).unwrap();
    assert_eq!(it.descripcion, "x");
    assert_eq!(it.descripcion_origen, Origen::Manual);
}

#[test]
fn renombrar_conserva_id_e_imagen() {
    let (_d, mut s) = con_items("renombrar");
    let antes = s.guardar_imagen("r/a", "png", b"x").unwrap();
    let it = s.actualizar("r/a", Cambios { nombre: Some("Nuevo".into()), ..Default::default() }).unwrap();
    assert_eq!(it.nombre, "Nuevo");
    assert_eq!(it.id, "r/a");
    assert_eq!(it.imagen, antes.imagen);
}

#[test]
fn recarga_cambios_externos() {
    let (d, mut s) = con_items("recarga");
    let mut externo = vec![item("r/a"), item("r/b")];
    externo[1].descripcion = "de Claude".into();
    escribir_items(&d, &externo).unwrap();
    let f = fs::File::options().write(true).open(d.join("items.json")).unwrap();
    f.set_modified(SystemTime::now() + Duration::from_secs(2)).unwrap();
    s.actualizar("r/a", Cambios { farmear: Some(true), ..Default::default() }).unwrap();
    let disco: Vec<Item> = serde_json::from_str(&fs::read_to_string(d.join("items.json")).unwrap()).unwrap();
    assert!(disco[0].farmear);
    assert_eq!(disco[1].descripcion, "de Claude");
}

#[test]
fn guardar_imagen_escribe_y_enlaza() {
    let (d, mut s) = con_items("imagen");
    let it = s.guardar_imagen("r/a", "webp", b"x").unwrap();
    assert_eq!(it.imagen.as_deref(), Some("imagenes/r__a.webp"));
    assert_eq!(fs::read(d.join("imagenes/r__a.webp")).unwrap(), b"x");
    assert!(s.guardar_imagen("r/a", "exe", b"x").is_err());
}

#[test]
fn id_inexistente() {
    let (_d, mut s) = con_items("nope");
    let e = s.actualizar("nope", Cambios::default()).unwrap_err();
    assert_eq!(e.kind(), std::io::ErrorKind::NotFound);
}
