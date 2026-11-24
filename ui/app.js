const { invoke, convertFileSrc } = window.__TAURI__.core;
const $ = (id) => document.getElementById(id);

const st = {
  items: [],
  dir: '',
  vista: 'regiones',          // 'regiones' | 'categorias'
  sel: null,                  // { tipo: 'region'|'cat', a, b }
  estado: 'todos',
  farmear: false,
  ng: false,
  q: '',
  actual: null,               // id en la ficha
  version: {},                // id -> marca para refrescar la imagen tras cambiarla
};

const norm = (s) => s.normalize('NFD').replace(/\p{M}/gu, '').toLowerCase();

// Fecha local (toISOString usaría UTC y podría dar el día anterior/siguiente).
function hoy() {
  const d = new Date();
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;
}

function srcImagen(it) {
  if (!it.imagen) return null;
  return convertFileSrc(`${st.dir}/${it.imagen}`) + (st.version[it.id] ? `?v=${st.version[it.id]}` : '');
}

function error(msg) {
  $('error').hidden = !msg;
  $('error').textContent = msg || '';
}

async function cargar() {
  try {
    st.items = await invoke('listar_items');
    error(null);
  } catch (e) {
    st.items = [];
    error(String(e));
  }
  if (!st.sel && st.items.length) st.sel = { tipo: 'region', a: st.items[0].zona, b: st.items[0].region };
  pintar();
}

// ---------- Lateral ----------

function agrupar(clave1, clave2) {
  const g = new Map();
  for (const it of st.items) {
    const a = clave1(it), b = clave2(it);
    if (!g.has(a)) g.set(a, new Map());
    const m = g.get(a);
    if (!m.has(b)) m.set(b, [0, 0]);
    const c = m.get(b);
    c[1]++;
    if (it.conseguido) c[0]++;
  }
  return g;
}

function pintarArbol() {
  const regiones = st.vista === 'regiones';
  const g = regiones
    ? agrupar((i) => i.zona, (i) => i.region)
    : agrupar((i) => i.categoria, (i) => i.subcategoria ?? '—');
  const tipo = regiones ? 'region' : 'cat';
  const nav = $('arbol');
  nav.replaceChildren();
  for (const [a, hijos] of g) {
    let tc = 0, tt = 0;
    for (const [c, t] of hijos.values()) { tc += c; tt += t; }
    const det = document.createElement('details');
    det.open = st.sel?.tipo === tipo && st.sel.a === a;
    const sum = document.createElement('summary');
    sum.append(a + ' ', Object.assign(document.createElement('span'), { className: 'cuenta', textContent: `${tc}/${tt}` }));
    det.append(sum);
    for (const [b, [c, t]] of hijos) {
      const btn = document.createElement('button');
      btn.className = 'nodo';
      btn.append(
        Object.assign(document.createElement('span'), { textContent: b }),
        Object.assign(document.createElement('span'), { className: 'cuenta' + (c === t ? ' completa' : ''), textContent: `${c}/${t}` }),
      );
      if (st.sel?.tipo === tipo && st.sel.a === a && st.sel.b === b) btn.setAttribute('aria-current', 'true');
      btn.onclick = () => { st.sel = { tipo, a, b }; st.q = ''; $('buscar').value = ''; pintar(); };
      det.append(btn);
    }
    nav.append(det);
  }
}

// ---------- Lista ----------

function visibles() {
  let v;
  if (st.q) {
    const q = norm(st.q);
    v = st.items.filter((i) => norm(i.nombre).includes(q));
  } else if (st.sel?.tipo === 'region') {
    v = st.items.filter((i) => i.zona === st.sel.a && i.region === st.sel.b);
  } else if (st.sel?.tipo === 'cat') {
    v = st.items.filter((i) => i.categoria === st.sel.a && (i.subcategoria ?? '—') === st.sel.b);
  } else v = [];
  if (st.estado === 'pendientes') v = v.filter((i) => !i.conseguido);
  if (st.estado === 'conseguidos') v = v.filter((i) => i.conseguido);
  if (st.farmear) v = v.filter((i) => i.farmear);
  if (st.ng) v = v.filter((i) => i.ng_plus);
  return v;
}

function pintarLista() {
  const v = visibles();
  $('titulo').textContent = st.q ? `Búsqueda: “${st.q}” (${v.length})` : st.sel ? `${st.sel.b} (${v.length})` : '';
  const ul = $('lista');
  ul.replaceChildren();
  for (const it of v) {
    const li = document.createElement('li');
    li.className = (it.conseguido ? 'conseguido ' : '') + (it.id === st.actual ? 'activo' : '');

    const cb = document.createElement('input');
    cb.type = 'checkbox';
    cb.checked = it.conseguido;
    cb.setAttribute('aria-label', `Conseguido: ${it.nombre}`);
    cb.onchange = () => marcar(it.id, cb.checked);

    const btn = document.createElement('button');
    btn.className = 'abrir';
    const src = srcImagen(it);
    const img = src ? Object.assign(document.createElement('img'), { src, alt: '', loading: 'lazy' })
                    : Object.assign(document.createElement('span'), { className: 'hueco' });
    const nombre = Object.assign(document.createElement('span'), { className: 'nombre', textContent: it.nombre });
    btn.append(img, nombre);
    // En vista por región se ve la categoría; en búsqueda o categoría, la región.
    const grupo = st.sel?.tipo === 'region' && !st.q ? (it.subcategoria ?? it.categoria) : it.region;
    btn.append(Object.assign(document.createElement('span'), { className: 'grupo', textContent: grupo }));
    if (it.farmear) btn.append(Object.assign(document.createElement('span'), { className: 'etiqueta farmear', textContent: 'Farmear' }));
    if (it.ng_plus) btn.append(Object.assign(document.createElement('span'), { className: 'etiqueta ng', textContent: 'NG+' }));
    btn.onclick = () => { st.actual = it.id; pintarLista(); pintarFicha(); };

    li.append(cb, btn);
    ul.append(li);
  }
}

// ---------- Ficha ----------

const actual = () => st.items.find((i) => i.id === st.actual);

function pintarFicha() {
  const it = actual();
  $('ficha').hidden = !it;
  if (!it) return;
  const src = srcImagen(it);
  $('f-img').hidden = !src;
  if (src) $('f-img').src = src;
  $('f-img').alt = it.nombre;
  $('f-sinfoto').hidden = !!src;
  $('f-nombre').value = it.nombre;
  $('f-lugar').textContent = `${it.region} · ${it.categoria}${it.subcategoria ? ' › ' + it.subcategoria : ''}`;
  $('f-conseguido').checked = it.conseguido;
  $('f-fecha').value = it.conseguido_en ?? '';
  $('f-fecha').disabled = !it.conseguido;
  $('f-m-farmear').checked = it.farmear;
  $('f-m-ng').checked = it.ng_plus;
  $('f-desc').value = it.descripcion;
  $('f-ingles').hidden = it.descripcion_origen !== 'wiki_en';
}

function pintar() {
  pintarArbol();
  pintarLista();
  pintarFicha();
}

// ---------- Cambios ----------

async function actualizar(id, cambios) {
  try {
    const nuevo = await invoke('actualizar_item', { id, cambios });
    st.items[st.items.findIndex((i) => i.id === id)] = nuevo;
    error(null);
  } catch (e) {
    error(String(e));
  }
  pintar();
}

const marcar = (id, si) => actualizar(id, si ? { conseguido: true, conseguido_en: hoy() } : { conseguido: false });

$('f-conseguido').onchange = (e) => marcar(st.actual, e.target.checked);
$('f-fecha').onchange = (e) => actualizar(st.actual, { conseguido_en: e.target.value || null });
$('f-m-farmear').onchange = (e) => actualizar(st.actual, { farmear: e.target.checked });
$('f-m-ng').onchange = (e) => actualizar(st.actual, { ng_plus: e.target.checked });
$('f-nombre').onchange = (e) => { if (e.target.value.trim()) actualizar(st.actual, { nombre: e.target.value.trim() }); };
$('f-desc').onchange = (e) => { if (e.target.value !== actual()?.descripcion) actualizar(st.actual, { descripcion: e.target.value }); };

$('f-archivo').onchange = async (e) => {
  const f = e.target.files[0];
  e.target.value = '';
  if (!f) return;
  const ext = f.name.split('.').pop().toLowerCase();
  try {
    const bytes = Array.from(new Uint8Array(await f.arrayBuffer()));
    const nuevo = await invoke('guardar_imagen', { id: st.actual, ext, bytes });
    st.items[st.items.findIndex((i) => i.id === nuevo.id)] = nuevo;
    st.version[nuevo.id] = Date.now();
    error(null);
  } catch (err) {
    error(String(err));
  }
  pintar();
};

// ---------- Filtros y navegación ----------

for (const b of document.querySelectorAll('[data-estado]')) {
  b.onclick = () => {
    st.estado = b.dataset.estado;
    document.querySelectorAll('[data-estado]').forEach((x) => x.setAttribute('aria-pressed', x === b));
    pintarLista();
  };
}
$('f-farmear').onchange = (e) => { st.farmear = e.target.checked; pintarLista(); };
$('f-ng').onchange = (e) => { st.ng = e.target.checked; pintarLista(); };
$('buscar').oninput = (e) => { st.q = e.target.value.trim(); pintarLista(); };

function pestana(vista) {
  st.vista = vista;
  $('tab-regiones').setAttribute('aria-selected', vista === 'regiones');
  $('tab-categorias').setAttribute('aria-selected', vista === 'categorias');
  pintarArbol();
}
$('tab-regiones').onclick = () => pestana('regiones');
$('tab-categorias').onclick = () => pestana('categorias');

// Claude puede editar items.json mientras la app está abierta: se relee al volver a la ventana.
window.addEventListener('focus', cargar);

(async () => {
  st.dir = await invoke('ruta_datos');
  await cargar();
})();
