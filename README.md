# Tarnished Tracker

App de escritorio para llevar el progreso de colección de Elden Ring: qué objetos
tienes de cada región, cuáles te faltan y cuáles hay que farmear o dejar para NG+.

Hecha en Rust con [Tauri 2](https://tauri.app); la interfaz es HTML/CSS/JS sin framework.

## Qué hace

- Lista los objetos por **región** y por **categoría** (armas, armaduras, talismanes, hechizos…).
- Marca cada objeto como conseguido (con fecha), **Farmear** o **NG+**.
- Filtra por pendientes / conseguidos / todos y busca por nombre.
- Ficha de cada objeto con **foto** y **descripción** editables.
- Importa el progreso inicial desde un vault de Obsidian.

## Requisitos

- [Rust](https://rustup.rs) estable
- Tauri CLI: `cargo install tauri-cli --version "^2"`
- Las dependencias de sistema de Tauri para tu SO:
  <https://tauri.app/start/prerequisites/>

## Uso

```bash
# 1. Importar el vault una sola vez (crea data/items.json)
cargo run -p er_core --bin importar -- <ruta-al-vault> data

# 2. Abrir la app
cargo tauri dev
```

El importador no sobrescribe un `data/items.json` existente; para reimportar
(y perder fotos, descripciones y fechas) añade `--forzar`.

Los datos viven en `data/` (`items.json` + `imagenes/`) y no se suben al repo.

### Formato del vault

Una nota por región en `Regiones/<mapa>/<región>.md`, con los objetos como casillas:

```markdown
## [[Armas]]
###### [[Puños]]
- [x] Puño de golem
- [ ] <font color="#ffff00">Objeto a farmear</font>
- [ ] <font color="#8064a2">Objeto de NG+</font>
```

`##` es la categoría, `######` la subcategoría (opcional), `[x]` = conseguido.

## Estructura

| Carpeta      | Contenido                                               |
|--------------|---------------------------------------------------------|
| `core/`      | Modelo, importador del vault y almacén en JSON (`er_core`) |
| `src-tauri/` | App Tauri y comandos que usa la interfaz                |
| `ui/`        | Interfaz (HTML/CSS/JS)                                  |
| `docs/`      | Diseño y plan                                           |

Tests: `cargo test --workspace`

## Licencia

[MIT](LICENSE).

Proyecto de fans sin relación con FromSoftware ni Bandai Namco.
*Elden Ring* es una marca de sus respectivos propietarios.
