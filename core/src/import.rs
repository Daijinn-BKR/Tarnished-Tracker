use crate::model::{Item, Origen};
use crate::slug::slug;
use std::collections::HashMap;

const CATEGORIAS: [&str; 9] = [
    "Armas", "Armaduras", "Talismanes", "Hechizos", "Cenizas de guerra",
    "Espíritus de ceniza", "Libros de recetas", "Lagrimas de cristal", "Ítems",
];

/// `[[Carpeta/Nombre]]` → `Nombre`; texto sin enlace se devuelve recortado.
fn enlace(s: &str) -> String {
    let s = s.trim().trim_start_matches("[[").trim_end_matches("]]");
    s.rsplit('/').next().unwrap_or(s).trim().to_string()
}

fn normalizar_sub(s: String) -> String {
    match s.as_str() {
        "Hojas Dobles" => "Hojas dobles".into(),
        "Mayal" => "Mayales".into(),
        _ => s,
    }
}

/// Para subcategorías sin `##` encima (Vista sombria).
fn categoria_de(sub: &str) -> &'static str {
    match sub {
        "Cabeza" | "Pecho" | "Piernas" | "Brazos" => "Armaduras",
        "Encantamientos" | "Hechicerías" => "Hechizos",
        s if s.starts_with("Cenizas") => "Cenizas de guerra",
        _ => "Armas",
    }
}

/// Devuelve (conseguido, resto) si la línea es una casilla `- [x] ...`.
fn casilla(l: &str) -> Option<(bool, &str)> {
    let r = l.strip_prefix('-')?.trim_start();
    let marca = r.get(..3)?;
    let conseguido = match marca {
        "[x]" | "[X]" => true,
        "[ ]" => false,
        _ => return None,
    };
    Some((conseguido, &r[3..]))
}

fn sin_etiquetas(s: &str) -> String {
    let mut out = String::new();
    let mut dentro = false;
    for c in s.chars() {
        match c {
            '<' => dentro = true,
            '>' => dentro = false,
            c if !dentro => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}

pub fn parsear_region(texto: &str, zona: &str, region: &str) -> Vec<Item> {
    let mut cat = String::new();
    let mut sub: Option<String> = None;
    let mut vistos: HashMap<String, usize> = HashMap::new();
    let mut items = Vec::new();

    for linea in texto.lines() {
        let l = linea.trim();
        if let Some(r) = l.strip_prefix("###### ") {
            let s = normalizar_sub(enlace(r));
            if cat.is_empty() {
                cat = categoria_de(&s).into();
            }
            sub = Some(s);
        } else if let Some(r) = l.strip_prefix("## ") {
            cat = enlace(r);
            sub = None;
        } else if l.starts_with("[[") && l.ends_with("]]") {
            let y = enlace(l);
            // Nokstella: `## [[Hechicerías]]` + `[[Hechizos]]` tienen los niveles invertidos.
            if CATEGORIAS.contains(&y.as_str()) && !CATEGORIAS.contains(&cat.as_str()) {
                sub = Some(normalizar_sub(std::mem::replace(&mut cat, y)));
            } else {
                sub = Some(normalizar_sub(y));
            }
        } else if let Some((conseguido, resto)) = casilla(l) {
            let nombre = sin_etiquetas(resto);
            if nombre.is_empty() {
                continue;
            }
            let base = format!(
                "{}/{}/{}/{}",
                slug(region), slug(&cat), sub.as_deref().map(slug).unwrap_or("-".into()), slug(&nombre)
            );
            let n = vistos.entry(base.clone()).or_insert(0);
            *n += 1;
            let id = if *n == 1 { base } else { format!("{base}-{n}") };
            items.push(Item {
                id,
                nombre,
                zona: zona.into(),
                region: region.into(),
                categoria: cat.clone(),
                subcategoria: sub.clone(),
                conseguido,
                conseguido_en: None,
                farmear: resto.contains("#ffff00"),
                ng_plus: resto.contains("#8064a2"),
                descripcion: String::new(),
                descripcion_origen: Origen::Ninguna,
                imagen: None,
            });
        }
    }
    items
}

/// Recorre `Regiones/<zona>/<region>.md` en orden alfabético (zona, luego región).
pub fn importar_vault(vault: &std::path::Path) -> std::io::Result<Vec<Item>> {
    let mut items = Vec::new();
    let mut zonas: Vec<_> = std::fs::read_dir(vault.join("Regiones"))?
        .collect::<Result<Vec<_>, _>>()?
        .into_iter().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    zonas.sort();
    for z in zonas {
        let zona = z.file_name().unwrap().to_string_lossy().to_string();
        let mut notas: Vec<_> = std::fs::read_dir(&z)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter().map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|e| e == "md")).collect();
        notas.sort();
        for n in notas {
            let region = n.file_stem().unwrap().to_string_lossy().to_string();
            items.extend(parsear_region(&std::fs::read_to_string(&n)?, &zona, &region));
        }
    }
    Ok(items)
}
