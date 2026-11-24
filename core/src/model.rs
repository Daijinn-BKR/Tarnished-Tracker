use serde::{Deserialize, Deserializer, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Item {
    pub id: String,
    pub nombre: String,
    pub zona: String,
    pub region: String,
    pub categoria: String,
    pub subcategoria: Option<String>,
    pub conseguido: bool,
    pub conseguido_en: Option<String>,
    pub farmear: bool,
    pub ng_plus: bool,
    pub descripcion: String,
    pub descripcion_origen: Origen,
    /// Ruta relativa a data/, p.ej. "imagenes/r__a.webp".
    pub imagen: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Origen {
    Ninguna,
    WikiEn,
    WikiEs,
    JuegoEs,
    Manual,
}

#[derive(Deserialize, Default, Debug)]
pub struct Cambios {
    pub nombre: Option<String>,
    pub conseguido: Option<bool>,
    /// Ausente = no tocar; `null` = borrar la fecha.
    #[serde(default, deserialize_with = "presente")]
    pub conseguido_en: Option<Option<String>>,
    pub farmear: Option<bool>,
    pub ng_plus: Option<bool>,
    pub descripcion: Option<String>,
}

fn presente<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Option<String>>, D::Error> {
    Option::<String>::deserialize(d).map(Some)
}

impl Item {
    pub fn aplicar(&mut self, c: Cambios) {
        if let Some(v) = c.nombre { self.nombre = v; }
        if let Some(v) = c.conseguido_en { self.conseguido_en = v; }
        if let Some(v) = c.conseguido {
            self.conseguido = v;
            if !v { self.conseguido_en = None; }
        }
        if let Some(v) = c.farmear { self.farmear = v; }
        if let Some(v) = c.ng_plus { self.ng_plus = v; }
        if let Some(v) = c.descripcion {
            self.descripcion = v;
            self.descripcion_origen = Origen::Manual;
        }
    }
}
