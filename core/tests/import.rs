use er_core::import::parsear_region;
use er_core::slug::slug;

fn p(t: &str) -> Vec<er_core::model::Item> {
    parsear_region(t, "Reino de las sombras", "Base de Rauh")
}

#[test]
fn slug_acentos() {
    assert_eq!(slug("Puño de golem"), "puno-de-golem");
    assert_eq!(slug("  Set de X!! "), "set-de-x");
}

#[test]
fn basico() {
    let v = p("## [[Armas]]\n###### [[Puños]]\n- [ ] <font color=\"#ffff00\">Puño de golem</font>\n");
    assert_eq!(v.len(), 1);
    let i = &v[0];
    assert_eq!(i.nombre, "Puño de golem");
    assert_eq!(i.categoria, "Armas");
    assert_eq!(i.subcategoria.as_deref(), Some("Puños"));
    assert!(i.farmear && !i.ng_plus && !i.conseguido);
    assert_eq!(i.id, "base-de-rauh/armas/punos/puno-de-golem");
    assert_eq!(i.zona, "Reino de las sombras");
    assert_eq!(i.region, "Base de Rauh");
}

#[test]
fn ng_plus_y_mayusculas() {
    let v = p("## [[Talismanes]]\n-  [X] <font color=\"#8064a2\">A</font>\n");
    assert!(v[0].conseguido && v[0].ng_plus);
    assert_eq!(v[0].subcategoria, None);
    assert_eq!(v[0].id, "base-de-rauh/talismanes/-/a");
}

#[test]
fn ignora_vacias() {
    assert!(p("## [[Armas]]\n- [x] \n- \n").is_empty());
}

#[test]
fn encabezado_indentado_y_carpeta() {
    let v = p("## [[Armas]]\n ###### [[Armas/Espadones curvos]]\n- [x] A\n");
    assert_eq!(v[0].subcategoria.as_deref(), Some("Espadones curvos"));
}

#[test]
fn sub_sin_categoria() {
    let v = p("###### [[Dagas]]\n- [x] A\n");
    assert_eq!(v[0].categoria, "Armas");
}

#[test]
fn nokstella_invertido() {
    let v = p("## [[Hechicerías]]\n[[Hechizos]]\n- [x] A\n## [[Cenizas de guerra]]\n[[Cenizas mágicas]]\n- [x] B\n");
    assert_eq!(v[0].categoria, "Hechizos");
    assert_eq!(v[0].subcategoria.as_deref(), Some("Hechicerías"));
    assert_eq!(v[1].categoria, "Cenizas de guerra");
    assert_eq!(v[1].subcategoria.as_deref(), Some("Cenizas mágicas"));
}

#[test]
fn ids_unicos() {
    let v = p("## [[Hechizos]]\n###### [[Encantamientos]]\n- [x] Lluvia de fuego\n- [x] Lluvia de fuego\n");
    assert_eq!(v[0].id, "base-de-rauh/hechizos/encantamientos/lluvia-de-fuego");
    assert_eq!(v[1].id, "base-de-rauh/hechizos/encantamientos/lluvia-de-fuego-2");
}

#[test]
fn punos_en_dos_categorias() {
    let v = p("## [[Armas]]\n###### [[Puños]]\n- [x] A\n## [[Armaduras]]\n###### [[Puños]]\n- [x] B\n");
    assert_eq!(v[0].categoria, "Armas");
    assert_eq!(v[1].categoria, "Armaduras");
}

#[test]
fn normaliza_subcategorias() {
    let v = p("## [[Armas]]\n###### [[Hojas Dobles]]\n- [x] A\n###### [[Mayal]]\n- [x] B\n");
    assert_eq!(v[0].subcategoria.as_deref(), Some("Hojas dobles"));
    assert_eq!(v[1].subcategoria.as_deref(), Some("Mayales"));
}
