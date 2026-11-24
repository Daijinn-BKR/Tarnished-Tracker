# App de colección de Elden Ring — Plan de implementación

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** App de escritorio Tauri que reemplaza el vault de Obsidian como tracker de colección de Elden Ring, con foto, descripción y fecha de obtención por objeto.

**Architecture:** Workspace Cargo con dos crates: `core` (modelo, almacén JSON, importador del vault, descargador de wiki; dos binarios de línea de comandos) y `src-tauri` (la app, comandos finos sobre `core`). La UI es HTML/CSS/JS plano en `ui/`, sin framework ni bundler. Datos en `data/items.json` y fotos en `data/imagenes/`.

**Tech Stack:** Rust 1.100, Tauri 2 (`tauri-cli` ^2, feature `protocol-asset`), serde/serde_json, ureq (solo en el descargador), HTML/CSS/JS.

**Spec:** `docs/superpowers/specs/2026-11-24-elden-ring-app-design.md`

## Global Constraints

- **Commits sin ninguna atribución a Claude**: nada de `Co-Authored-By` ni "Generated with". El autor es la identidad git del usuario.
- El vault `~/Proyectos/elden-ring-wiki/Elden ring/` es de **solo lectura**: nada lo modifica.
- Guardado de `items.json` **atómico**: escribir `items.json.tmp` y `rename`.
- La descarga de wiki nunca sobrescribe una descripción con origen `juego_es` o `manual`, ni una imagen ya presente.
- Marcas: `#ffff00` → `farmear`, `#8064a2` → `ng_plus`.
- Fechas `conseguido_en`: `"YYYY-MM-DD"` o `null`. Desmarcar ⇒ `null`.
- Estética oscura con dorados; textos de la UI en español.
- Sin dependencias nuevas fuera de: tauri, tauri-build, serde, serde_json, ureq.

## Review Focus

1. **Volver a ejecutar `importar` sobre un `items.json` existente** borraría fotos, descripciones y fechas → debe negarse salvo `--forzar`. (Test en Task 3.)
2. **`items.json` ausente o corrupto al arrancar la app** → mostrar error y no guardar nunca una lista vacía encima. (Test en Task 1.)
3. **Claude edita `items.json` con la app abierta** → antes de aplicar cualquier cambio, la app recarga si cambió la fecha de modificación. (Test en Task 1.)
4. **Renombrar un objeto** → su `id` y su imagen no cambian. (Test en Task 1.)
5. **Repetir `descargar-wiki`** → no pisa descripciones `manual`/`juego_es` ni imágenes existentes; reanuda sin repetir lo ya hecho. (Test en Task 6.)

---

### Task 0: Requisitos del sistema (el usuario los ejecuta)

- [ ] **Step 1:** `sudo pacman -S --needed webkit2gtk-4.1 base-devel` (Tauri 2 en Linux).
- [ ] **Step 2:** `cargo install tauri-cli --version '^2' --locked`
- [ ] **Step 3:** Verificar: `cargo tauri --version` imprime `tauri-cli 2.x`; `pkg-config --modversion webkit2gtk-4.1` imprime una versión.

Tasks 1–3 no dependen de esto; Task 4 sí.

---

### Task 1: Workspace, modelo y almacén

**Files:**
- Create: `Cargo.toml` (workspace: `core`, `src-tauri`), `.gitignore` (`target/`, `data/`)
- Create: `core/Cargo.toml` (lib `er_core`), `core/src/lib.rs`, `core/src/model.rs`, `core/src/store.rs`
- Test: `core/tests/store.rs`

`data/` va en `.gitignore`: son datos personales y fotos, no código.

**Interfaces:**
- Produces:
  ```rust
  // model.rs
  #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
  pub struct Item {
      pub id: String, pub nombre: String, pub zona: String, pub region: String,
      pub categoria: String, pub subcategoria: Option<String>,
      pub conseguido: bool, pub conseguido_en: Option<String>,
      pub farmear: bool, pub ng_plus: bool,
      pub descripcion: String, pub descripcion_origen: Origen,
      pub imagen: Option<String>,          // ruta relativa a data/, p.ej. "imagenes/<id-con-__>.webp"
  }
  #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
  #[serde(rename_all = "snake_case")]
  pub enum Origen { Ninguna, WikiEn, WikiEs, JuegoEs, Manual }
  #[derive(Deserialize, Default)]
  pub struct Cambios {
      pub nombre: Option<String>, pub conseguido: Option<bool>,
      pub conseguido_en: Option<Option<String>>,
      pub farmear: Option<bool>, pub ng_plus: Option<bool>,
      pub descripcion: Option<String>,
  }
  // store.rs
  pub struct Store { pub dir: PathBuf, pub items: Vec<Item>, /* mtime privado */ }
  impl Store {
      pub fn abrir(dir: &Path) -> io::Result<Store>;          // Err si items.json falta o no parsea
      pub fn guardar(&mut self) -> io::Result<()>;             // atómico; actualiza mtime
      pub fn recargar_si_cambio(&mut self) -> io::Result<bool>;
      pub fn actualizar(&mut self, id: &str, c: Cambios) -> io::Result<Item>; // recarga→aplica→guarda
      pub fn guardar_imagen(&mut self, id: &str, ext: &str, bytes: &[u8]) -> io::Result<Item>;
  }
  pub fn escribir_items(dir: &Path, items: &[Item]) -> io::Result<()>; // atómico, usado por importar
  ```
  `WikiEs` se añade respecto al spec: si la wiki en español da descripción, no es `wiki_en`.

- [ ] **Step 1: Tests que fallan** en `core/tests/store.rs` (usar un dir temporal bajo `std::env::temp_dir()` con nombre único):
  - `abrir_falla_sin_archivo`: `Store::abrir(vacio).is_err()`.
  - `abrir_falla_con_json_corrupto`: escribe `"{no"` → `is_err()`, y el archivo sigue conteniendo `"{no"`.
  - `marcar_y_desmarcar`: `actualizar(id, conseguido=Some(true), conseguido_en=Some(Some("2026-11-24")))` → `conseguido_en == Some("2026-11-24")`; luego `conseguido=Some(false)` → `conseguido_en == None`.
  - `editar_descripcion_marca_manual`: `descripcion=Some("x")` → `descripcion_origen == Origen::Manual`.
  - `renombrar_conserva_id_e_imagen`: tras `nombre=Some("Nuevo")`, `id` e `imagen` iguales.
  - `recarga_cambios_externos`: abrir; otro proceso (simulado: `escribir_items` + ajustar mtime a +2 s con `File::set_modified`) cambia la descripción de B; `actualizar(A, farmear=Some(true))` → en disco A.farmear y B con la descripción externa.
  - `guardar_imagen_escribe_y_enlaza`: `guardar_imagen(id,"webp",b"x")` → existe `dir/imagenes/<id con '/'→'__'>.webp`, `item.imagen` apunta ahí; ext `"exe"` → `Err`.
  - `id_inexistente`: `actualizar("nope", ..)` → `Err(NotFound)`.
- [ ] **Step 2:** `cargo test -p er_core` → FAIL (no compila).
- [ ] **Step 3:** Implementar. Extensiones permitidas: `png`, `jpg`, `jpeg`, `webp`.
- [ ] **Step 4:** `cargo test -p er_core` → PASS.
- [ ] **Step 5:** `git init && git add -A && git commit -m "feat: modelo y almacén de items"`

---

### Task 2: Parser de notas de región

**Files:**
- Create: `core/src/import.rs`, `core/src/slug.rs`
- Test: `core/tests/import.rs`

**Interfaces:**
- Produces:
  ```rust
  pub fn slug(s: &str) -> String;  // minúsculas, áéíóúüñ→aeiouun, no-alfanumérico→'-', sin '-' repetidos ni en extremos
  pub fn parsear_region(texto: &str, zona: &str, region: &str) -> Vec<Item>;
  ```
- Reglas del parser (decididas en el spec, revisión):
  - Encabezados: se recortan espacios iniciales. `## X` → categoría (resetea subcategoría); `###### X` → subcategoría. `X` sin `[[ ]]`, y `[[Carpeta/Nombre]]` → `Nombre`.
  - Línea que es solo `[[X]]` (Nokstella): se trata como subcategoría.
  - En Nokstella, `## [[Hechicerías]]` seguido de `[[Hechizos]]`: si un `##` es un nombre de subcategoría conocida y la línea siguiente es una categoría conocida, se intercambian. Categorías conocidas: `Armas, Armaduras, Talismanes, Hechizos, Cenizas de guerra, Espíritus de ceniza, Libros de recetas, Lagrimas de cristal, Ítems`.
  - `######` sin `##` previo (Vista sombria): categoría inferida de la subcategoría con una tabla fija construida con las subcategorías que aparecen bajo cada categoría en el resto del vault (el implementador la genera al leer el vault y la escribe como `const`).
  - Casillas: regex-libre: línea que tras recortar empieza por `-`, luego espacios, `[x]`/`[X]`/`[ ]`. Nombre = resto sin etiquetas `<...>`, recortado; si vacío → se ignora. `- ` sin casilla → se ignora.
  - `color="#ffff00"` → `farmear`; `color="#8064a2"` → `ng_plus`.
  - Normalizar subcategoría: `Hojas Dobles`→`Hojas dobles`, `Mayal`→`Mayales`.
  - `id = slug(region)/slug(categoria)/slug(sub o "-")/slug(nombre)`; si repetido dentro de la región, sufijo `-2`, `-3`… por orden de aparición.
  - Resto de campos: `conseguido_en: None`, `descripcion: ""`, `Origen::Ninguna`, `imagen: None`.

- [ ] **Step 1: Tests que fallan** (cada uno con un texto corto en línea):
  - `slug_acentos`: `slug("Puño de golem") == "puno-de-golem"`, `slug("  Set de X!! ") == "set-de-x"`.
  - `basico`: el fragmento de Base de Rauh con `- [ ] <font color="#ffff00">Puño de golem</font>` → nombre `Puño de golem`, categoría `Armas`, sub `Puños`, `farmear`, `!conseguido`.
  - `ng_plus_y_mayusculas`: `-  [X] <font color="#8064a2">A</font>` → `conseguido`, `ng_plus`.
  - `ignora_vacias`: `- [x] ` y `- ` no producen items.
  - `encabezado_indentado_y_carpeta`: ` ###### [[Armas/Espadones curvos]]` → sub `Espadones curvos`.
  - `sub_sin_categoria`: texto que empieza en `###### [[Dagas]]` → categoría `Armas`.
  - `nokstella_invertido`: `## [[Hechicerías]]\n[[Hechizos]]\n- [x] A` → cat `Hechizos`, sub `Hechicerías`.
  - `ids_unicos`: dos `- [x] Lluvia de fuego` bajo la misma sub → ids `…/lluvia-de-fuego` y `…/lluvia-de-fuego-2`.
  - `puños_en_dos_categorias`: `Puños` bajo `Armas` y bajo `Armaduras` → categorías distintas.
- [ ] **Step 2:** `cargo test -p er_core --test import` → FAIL.
- [ ] **Step 3:** Implementar.
- [ ] **Step 4:** `cargo test -p er_core --test import` → PASS.
- [ ] **Step 5:** `git commit -am "feat: parser de notas de región"` (añadir archivos nuevos antes).

---

### Task 3: Binario `importar`

**Files:**
- Create: `core/src/bin/importar.rs`
- Test: `core/tests/importar_vault.rs`

**Interfaces:**
- Consumes: `parsear_region`, `escribir_items`.
- Produces: `pub fn importar_vault(vault: &Path) -> io::Result<Vec<Item>>` en `import.rs` (recorre `Regiones/<zona>/*.md` en orden alfabético; zona = nombre de carpeta, región = nombre de archivo sin `.md`). CLI: `cargo run -p er_core --bin importar -- <vault> <data-dir> [--forzar]`.

- [ ] **Step 1:** Obtener los números reales para el test:
  `grep -rhE '^\s*-\s+\[[xX ]\]\s*\S' --include='*.md' "<vault>/Regiones" | wc -l` (total) y lo mismo con `\[[xX]\]` (conseguidos), y con `ffff00` / `8064a2`. Anotar los cuatro.
- [ ] **Step 2: Tests que fallan:**
  - `vault_real` (marcado `#[ignore]` si el vault no existe en esa ruta): `importar_vault(~/Proyectos/elden-ring-wiki/Elden ring)` → longitud = total, conseguidos, farmear y ng_plus = los números del Step 1; todos los `id` únicos; ninguna `categoria` vacía.
  - `no_pisa_sin_forzar`: con `items.json` existente, ejecutar el binario sin `--forzar` (via `std::process::Command` sobre `env!("CARGO_BIN_EXE_importar")`) → código de salida ≠ 0 y archivo intacto.
- [ ] **Step 3:** `cargo test -p er_core --test importar_vault` → FAIL.
- [ ] **Step 4:** Implementar. Si los totales no cuadran, revisar el parser (Task 2), no el test.
- [ ] **Step 5:** PASS; luego ejecutar de verdad: `cargo run -p er_core --bin importar -- "$HOME/Proyectos/elden-ring-wiki/Elden ring" data` → imprime `N objetos importados`.
- [ ] **Step 6:** `git add -A && git commit -m "feat: importar vault de Obsidian"`

---

### Task 4: App Tauri y comandos

**Files:**
- Create: `src-tauri/` con `cargo tauri init` (frontendDist `../ui`, sin dev server), `src-tauri/src/main.rs`
- Modify: `src-tauri/tauri.conf.json`: `app.security.assetProtocol.enable = true`, título de ventana `Elden Ring — Colección`
- Create: `ui/index.html` mínimo que llama a `listar_items` y muestra el número.

**Interfaces:**
- Consumes: `Store`, `Cambios`, `Item` de `er_core`.
- Produces (comandos invocables desde JS con `window.__TAURI__.core.invoke`; `withGlobalTauri: true`):
  - `listar_items() -> Result<Vec<Item>, String>` (llama antes a `recargar_si_cambio`)
  - `actualizar_item(id: String, cambios: Cambios) -> Result<Item, String>`
  - `guardar_imagen(id: String, ext: String, bytes: Vec<u8>) -> Result<Item, String>`
  - `dir_datos() -> String` (ruta absoluta de `data/`, para `convertFileSrc`)
- Data dir: `concat!(env!("CARGO_MANIFEST_DIR"), "/../data")` canonicalizado. Al arrancar, `app.asset_protocol_scope().allow_directory(dir, true)`. Estado: `Mutex<Option<Store>>`; si `Store::abrir` falla, los comandos devuelven `Err` con el mensaje y la UI lo muestra (no se crea archivo nuevo).

- [ ] **Step 1:** Scaffolding y comandos.
- [ ] **Step 2:** `cargo build -p src-tauri` (o el nombre de paquete generado) compila sin warnings.
- [ ] **Step 3:** `cargo tauri dev` → la ventana muestra el total importado en Task 3. Con `data/items.json` renombrado → la ventana muestra el error y no aparece un `items.json` nuevo.
- [ ] **Step 4:** `git add -A && git commit -m "feat: esqueleto de la app Tauri"`

---

### Task 5: Interfaz

**Files:**
- Create/Modify: `ui/index.html`, `ui/style.css`, `ui/app.js`

**Interfaces:**
- Consumes: los comandos de Task 4.

Pantalla (spec, sección Interfaz):
```
┌ Zonas/Regiones │ Categorías ┐┌ [Pendientes|Conseguidos|Todos] [Farmear] [NG+] [buscar…] ┐
│ Tierras intermedias  812/840 ││ ☑ [img] Hacha de letra herrérica                  │┌ Ficha ─────────┐
│   Caelid             61/63   ││ ☐ [img] Puño de golem        ⚒Farmear            ││ foto grande     │
│ Reino de las sombras ...     ││ ...                                              ││ nombre (edit)   │
└─────────────────────────────┘└──────────────────────────────────────────────────┘│ ☑ fecha (edit)  │
                                                                                    │ Farmear  NG+    │
                                                                                    │ descripción     │
                                                                                    │ [Cambiar foto]  │
                                                                                    └─────────────────┘
```
- Barra lateral con pestaña Regiones (zona → región) y Categorías (categoría → subcategoría, agrupando por el par); cada fila con `conseguidos/total`.
- Filtros combinables; búsqueda insensible a mayúsculas y acentos (`normalize('NFD')`).
- Al marcar: `conseguido_en` = fecha local de hoy `YYYY-MM-DD` (no `toISOString`, que usa UTC).
- Editar descripción: se guarda al salir del campo. Si `descripcion_origen` es `wiki_en`, mostrar una etiqueta "inglés".
- Cambiar foto: `<input type=file accept="image/png,image/jpeg,image/webp">` → bytes → `guardar_imagen`.
- Imágenes con `convertFileSrc(dir + '/' + item.imagen)`; sin imagen → marcador neutro.
- Al recuperar el foco de la ventana (`window` `focus`): volver a pedir `listar_items` y repintar.
- Estética: fondo casi negro, texto hueso, acentos dorados (`#c9a85a` aprox.), serif para títulos; contraste AA; foco visible con teclado; casillas son `<input type=checkbox>` reales con `<label>`.
- Lista de ~1800 filas: pintar solo la región/categoría seleccionada (no todas a la vez) e imágenes con `loading="lazy"`.

- [ ] **Step 1:** Implementar.
- [ ] **Step 2: Verificación manual en `cargo tauri dev`** (anotar resultado de cada una):
  - marcar Puño de golem → aparece fecha de hoy; reiniciar la app → sigue marcado con la fecha.
  - desmarcar → fecha vacía.
  - filtro Farmear en Base de Rauh muestra Puño de golem; NG+ muestra solo objetos con `#8064a2`.
  - buscar `puno` encuentra `Puño de golem`.
  - editar `items.json` a mano (cambiar una descripción) con la app abierta, volver a la ventana → se ve el cambio.
  - cambiar foto de un objeto → se ve en lista y ficha; el archivo está en `data/imagenes/`.
  - navegación completa solo con teclado.
- [ ] **Step 3:** `git add -A && git commit -m "feat: interfaz de colección"`

---

### Task 6: Binario `descargar-wiki`

**Files:**
- Create: `core/src/wiki.rs`, `core/src/bin/descargar_wiki.rs`, `core/tests/fixtures/wiki_*.json`
- Test: `core/tests/wiki.rs`
- Modify: `core/Cargo.toml` (añadir `ureq` con feature `json`)

**Interfaces:**
- Consumes: `Store`, `Item`, `Origen`.
- Produces:
  ```rust
  pub struct Resultado { pub imagen_url: Option<String>, pub descripcion: Option<String>, pub origen: Origen }
  pub fn interpretar_respuesta(json: &serde_json::Value, origen: Origen) -> Resultado; // puro, testeable
  pub fn aplicar(item: &mut Item, r: &Resultado) -> bool;  // reglas de no-sobrescritura; true si cambió algo
  ```
  CLI: `cargo run -p er_core --bin descargar-wiki -- <data-dir>`. Solo con la app cerrada (lo imprime al empezar).

- [ ] **Step 1: Sondeo (sin código de producto).** Con `curl`, para 3 objetos (`Puño de golem`, `Daga`, `Medallón de ámbar cerúleo +3`), probar `https://eldenring.fandom.com/es/api.php?action=query&format=json&redirects=1&prop=pageimages|extracts&piprop=original&exintro=1&explaintext=1&titles=<nombre>`; si `extracts` no está disponible o la wiki ES no tiene la página, probar `prop=pageimages|revisions&rvprop=content&rvslots=main` y la wiki EN `eldenring.fandom.com/api.php` (nombre EN: tomar el enlace interlingüe `prop=langlinks&lllang=en` desde la página ES). Guardar las respuestas reales en `core/tests/fixtures/`. Si ninguna fuente da descripción utilizable, informar al usuario antes de seguir (solo foto).
- [ ] **Step 2: Tests que fallan** sobre las fixtures:
  - `interpreta_imagen_y_texto`: fixture ES de Puño de golem → `imagen_url` es `Some(https://…)`.
  - `pagina_inexistente`: fixture con `"missing"` → todo `None`.
  - `no_pisa_manual_ni_juego`: item con `Origen::Manual` y `Origen::JuegoEs` → `aplicar` no cambia la descripción.
  - `no_pisa_imagen`: item con `imagen: Some(..)` → no cambia `imagen`.
  - `rellena_vacio`: item `Origen::Ninguna` sin imagen → toma descripción y origen del resultado; devuelve `true`.
- [ ] **Step 3:** FAIL → implementar → PASS.
- [ ] **Step 4:** Bucle del binario: para cada item sin imagen o con descripción vacía: consultar (1 petición/s, `User-Agent` genérico `elden-ring-coleccion/0.1`), descargar imagen con su extensión original vía `Store::guardar_imagen`, aplicar, y `guardar` cada 25 items y al final (así se reanuda si se corta). Al terminar imprime cuántos rellenó y la lista de no encontrados en `data/no_encontrados.txt`.
- [ ] **Step 5:** Ejecutarlo sobre una copia de `data/` limitada a una región (variable `ER_SOLO_REGION="Base de Rauh"`) y revisar a ojo 5 resultados en la app. Luego avisar al usuario antes de la ejecución completa (~1800 peticiones, ~30–60 min).
- [ ] **Step 6:** `git add -A && git commit -m "feat: descargar fotos y descripciones de la wiki"`

---

## Flujo de capturas (sin código)

Cuando el usuario mande capturas, Claude transcribe la descripción y la escribe en `data/items.json` con `descripcion_origen: "juego_es"` (con la app abierta o cerrada; la app recarga al recuperar el foco, Task 1/5).
