use crate::model::{Cambios, Item};
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::SystemTime;

const EXTENSIONES: [&str; 4] = ["png", "jpg", "jpeg", "webp"];

pub struct Store {
    pub dir: PathBuf,
    pub items: Vec<Item>,
    mtime: Option<SystemTime>,
}

fn ruta(dir: &Path) -> PathBuf {
    dir.join("items.json")
}

fn leer(dir: &Path) -> io::Result<(Vec<Item>, Option<SystemTime>)> {
    let p = ruta(dir);
    let texto = fs::read_to_string(&p)?;
    let items = serde_json::from_str(&texto).map_err(|e| Error::new(ErrorKind::InvalidData, format!("{}: {e}", p.display())))?;
    Ok((items, fs::metadata(&p)?.modified().ok()))
}

/// Escritura atómica: temporal + rename, para no dejar nunca un items.json a medias.
pub fn escribir_items(dir: &Path, items: &[Item]) -> io::Result<()> {
    let tmp = dir.join("items.json.tmp");
    fs::write(&tmp, serde_json::to_vec_pretty(items)?)?;
    fs::rename(tmp, ruta(dir))
}

impl Store {
    pub fn abrir(dir: &Path) -> io::Result<Store> {
        let (items, mtime) = leer(dir)?;
        Ok(Store { dir: dir.to_path_buf(), items, mtime })
    }

    pub fn guardar(&mut self) -> io::Result<()> {
        escribir_items(&self.dir, &self.items)?;
        self.mtime = fs::metadata(ruta(&self.dir))?.modified().ok();
        Ok(())
    }

    /// Recarga si alguien (p.ej. Claude) cambió items.json desde la última lectura/escritura.
    pub fn recargar_si_cambio(&mut self) -> io::Result<bool> {
        let actual = fs::metadata(ruta(&self.dir))?.modified().ok();
        if actual == self.mtime {
            return Ok(false);
        }
        let (items, mtime) = leer(&self.dir)?;
        self.items = items;
        self.mtime = mtime;
        Ok(true)
    }

    fn buscar(&mut self, id: &str) -> io::Result<&mut Item> {
        self.items.iter_mut().find(|i| i.id == id)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("no existe el objeto {id}")))
    }

    pub fn actualizar(&mut self, id: &str, c: Cambios) -> io::Result<Item> {
        self.recargar_si_cambio()?;
        let it = self.buscar(id)?;
        it.aplicar(c);
        let copia = it.clone();
        self.guardar()?;
        Ok(copia)
    }

    pub fn guardar_imagen(&mut self, id: &str, ext: &str, bytes: &[u8]) -> io::Result<Item> {
        let ext = ext.to_ascii_lowercase();
        if !EXTENSIONES.contains(&ext.as_str()) {
            return Err(Error::new(ErrorKind::InvalidInput, format!("extensión no permitida: {ext}")));
        }
        self.recargar_si_cambio()?;
        let dir = self.dir.clone();
        let it = self.buscar(id)?;
        let rel = format!("imagenes/{}.{ext}", id.replace('/', "__"));
        fs::create_dir_all(dir.join("imagenes"))?;
        fs::write(dir.join(&rel), bytes)?;
        if let Some(vieja) = it.imagen.replace(rel.clone()) {
            if vieja != rel { let _ = fs::remove_file(dir.join(vieja)); }
        }
        let copia = it.clone();
        self.guardar()?;
        Ok(copia)
    }
}
