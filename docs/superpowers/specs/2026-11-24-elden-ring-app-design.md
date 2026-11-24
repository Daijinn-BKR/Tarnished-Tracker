# App de colección de Elden Ring — Diseño

## Intención

El usuario lleva su progreso de colección de Elden Ring en un vault de Obsidian
(`~/Proyectos/elden-ring-wiki/Elden ring/`). Quiere una app de escritorio propia,
en Rust, que **reemplace** a Obsidian para esto, con control total del aspecto
y la posibilidad de añadir a cada objeto **foto y descripción**.

## Estado actual (el vault)

- `Regiones/Tierras intermedias/*.md` (17) y `Regiones/Reino de las sombras/*.md` (11):
  cada nota es una región con objetos como casillas, agrupados por categoría:
  ```
  ## [[Armas]]
  ###### [[Puños]]
  - [ ] <font color="#ffff00">Puño de golem</font>
  ```
  - `##` = categoría (Armas, Armaduras, Talismanes, Hechizos, Cenizas de guerra, Espíritus de ceniza…).
  - `######` = subcategoría (Dagas, Cabeza, Encantamientos…). Puede no haberla.
  - `[x]` = conseguido. 1826 objetos, 1754 marcados.
  - `#ffff00` (280) = **se tiene que farmear**. `#8064a2` (32) = **se consigue en NG+**.
  - 23 nombres se repiten en varias regiones: son entradas distintas (una por región).
- Las notas de categoría (`Armas/Dagas.md`, etc.) están vacías: solo destinos de enlace. Se ignoran.
- `Guias/1000%.md`: tablas con el total esperado por categoría y pasos de misión numerados por región.

## Objetivos

1. Importar una vez todo el vault a los datos de la app sin perder nada (región,
   categoría, subcategoría, conseguido, marcas Farmear/NG+).
2. Ver objetos por región y por categoría, marcar/desmarcar como conseguido.
3. Filtrar: por conseguido/pendiente, por marca (Farmear, NG+), búsqueda por nombre.
4. Ficha de objeto con foto y descripción.
5. Rellenar fotos y descripciones por tres vías:
   - descarga automática desde una wiki pública (foto siempre; descripción en inglés si no hay otra);
   - edición manual en la app (texto e imagen);
   - el usuario manda capturas del juego a Claude en el chat, que las transcribe y
     las guarda en los datos (no hay OCR en la app).
6. Progreso: conseguidos/total por región y por categoría.
7. Que se vea bien: estética cuidada, con aire al juego.

## No objetivos

- Sincronizar con Obsidian. Tras la importación, la app es la única fuente de verdad.
- OCR dentro de la app.
- Varias partidas/perfiles, cuentas, nube.
- Estadísticas de objetos (daño, escalado…) — solo foto y descripción.
- `Guias/1000%.md` (totales esperados y pasos de misión): no se importa por ahora (decisión del usuario).

## Decisiones del usuario sobre los datos

- Las líneas "Set de X" bajo cada pieza de armadura se importan tal cual: una entrada por
  línea, con su nombre actual. El usuario las renombrará cuando las encuentre en el juego.

## Anti-objetivos

- Perder o corromper el progreso marcado. Guardar debe ser atómico
  (escribir a temporal y renombrar).
- Que la descarga de la wiki pise una descripción en español o editada a mano.

## Enfoques considerados

- **egui (todo Rust)**: lo más simple, pero aspecto funcional, poco "bonito".
- **Tauri (Rust + HTML/CSS/JS)** — **elegido por el usuario** "para que se vea bien":
  control visual total; a cambio la interfaz no es Rust.
- **Iced/Slint (todo Rust, más pulido)**: más código y menos ecosistema.

## Diseño

### Estructura

```
elden-ring-app/
  src-tauri/          # Rust: modelo, carga/guardado, comandos Tauri
  ui/                 # HTML/CSS/JS sin framework ni bundler
  data/items.json     # todos los objetos
  data/imagenes/      # fotos, nombradas por id del objeto
```
Binarios auxiliares (ejecutados una vez desde terminal, no botones):
- `importar <ruta-vault>`: genera `data/items.json` desde el vault.
- `descargar-wiki`: rellena foto (y descripción en inglés si está vacía) de cada objeto.

### Modelo (items.json)

```json
{
  "id": "base-de-rauh/puno-de-golem",
  "nombre": "Puño de golem",
  "zona": "Reino de las sombras",
  "region": "Base de Rauh",
  "categoria": "Armas",
  "subcategoria": "Puños",
  "conseguido": false,
  "farmear": true,
  "ng_plus": false,
  "descripcion": "",
  "descripcion_origen": "ninguna",   // ninguna | wiki_en | wiki_es | juego_es | manual
  "imagen": "imagenes/base-de-rauh__puno-de-golem.png"
}
```
`id` = slug de región + slug de nombre (único aunque el nombre se repita en otra región).
El orden de los objetos en el archivo conserva el orden del vault.

### Rust (Tauri)

Comandos mínimos: `listar_items`, `actualizar_item(id, cambios)`,
`guardar_imagen(id, bytes)`. Carga `items.json` al arrancar y lo reescribe
atómicamente en cada cambio.

### Interfaz

- Barra lateral: zonas → regiones, y vista alternativa por categoría; cada una con su progreso.
- Lista central: objetos con casilla, nombre, iconos Farmear/NG+, miniatura.
- Filtros arriba: pendiente/conseguido/todos, Farmear, NG+, búsqueda.
- Ficha: foto grande, descripción (editable), marcas, y botón para cambiar la foto.
- Estética oscura con dorados tipo Elden Ring. Sin usar recursos con copyright del juego
  más allá de las fotos de objetos que el usuario guarda para uso personal.

### Descarga de la wiki

Fuente preferida: wiki de Fandom en español (`eldenring.fandom.com/es`, API MediaWiki),
porque los nombres del vault están en español; si no encuentra el objeto, probar la
wiki en inglés. Nunca sobrescribe una descripción con origen `juego_es` o `manual`,
ni una imagen ya presente. Lo que no encuentre se lista al final para que el usuario
lo corrija a mano. Respetar límites de la API (peticiones espaciadas).

## Decisiones tras la revisión (llamadas de Claude, corregibles)

- **Datos reales del vault** (verificado): 1827 líneas de casilla, una vacía (`- [x] ` en Base de
  Rauh) y un `- ` suelto en Rio Siofra → se ignoran. Se acepta `[x]`, `[X]` y `-  [x]`.
  El test de importación afirma los números reales que salgan del vault.
- **Id único**: slug región + slug categoría + slug subcategoría + slug nombre; si aún choca, sufijo `-2`.
- **Categorías**: además de las listadas, `Libros de recetas`, `Lagrimas de cristal`, `Ítems` se importan igual.
- **Estructura irregular**: encabezados indentados se aceptan; `[[Carpeta/Nombre]]` → `Nombre`;
  subcategoría sin categoría (Vista sombria) y enlaces sueltos/invertidos (Nokstella) se
  corrigen con reglas explícitas en el importador, cada una con su test.
- **Nombres de subcategoría** se normalizan (`Hojas Dobles`→`Hojas dobles`, `Mayal`→`Mayales`).
  Los nombres de objetos no se tocan.
- **"Puños"** existe en Armas y en Armaduras: la vista por categoría agrupa por (categoría, subcategoría).
- **Dónde viven los datos**: `data/` dentro de la carpeta del proyecto (uso personal, se ejecuta desde ahí).
- **Imágenes**: se guarda la extensión original (webp/jpg/png).
- **Conseguido con fecha** (decisión del usuario): campo `conseguido_en` (fecha `YYYY-MM-DD` o `null`)
  junto a `conseguido`. Al marcar se pone la fecha de hoy; al desmarcar vuelve a `null`; editable en la ficha.
  Los objetos ya marcados en el vault se importan con `conseguido: true` y `conseguido_en: null` (fecha desconocida).
- **Ficha editable**: nombre, conseguido y su fecha, Farmear, NG+, descripción, foto.
  Región/categoría no editables. El `id` no cambia al renombrar.
- **Ediciones de Claude con la app abierta**: la app recarga `items.json` al recuperar el foco
  si su fecha de modificación cambió, antes de aplicar cualquier cambio propio.

## Lo que queda al constructor

- Estructura interna del Rust y del JS, nombres de archivos de la UI.
- Detalles visuales concretos dentro de la línea estética indicada.
- Cómo hacer coincidir nombres con la wiki (normalización, búsqueda).
