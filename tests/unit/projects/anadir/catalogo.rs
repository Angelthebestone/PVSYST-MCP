use super::*;

fn demo(nombre: &str) -> Vec<u8> {
    std::fs::read(crate::config::proyectos_demo_ro().join(nombre)).expect("PVsyst demo data")
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_have_parent_in_demo_for_every_element() {
    for elemento in ELEMENTOS {
        let fuente = demo(elemento.demo);
        let spans = edit::spans_of(&fuente);
        for tipo in elemento.tipos {
            let si = edit::find_span(&spans, tipo)
                .unwrap_or_else(|| panic!("{tipo} ausente en {}", elemento.demo));
            assert!(
                padre_de(&spans, si).is_some(),
                "{tipo} sin padre en {}",
                elemento.demo
            );
        }
    }
}

#[test]
fn should_list_valid_elements_when_unknown_element_requested() {
    let mut paths = Paths::resolve();
    paths.workspace = std::env::temp_dir();
    let args = json!({"proyecto": "x", "elemento": "nada"});
    let err = anadir(&paths, &args).unwrap_err();
    assert!(err.to_string().contains("baterias"));
}

#[test]
#[ignore = "needs the PVsyst 8.1.7 demo data: run with --ignored"]
fn should_not_contain_meteonorm_in_any_catalogue_element() {
    for elemento in ELEMENTOS {
        let fuente = demo(elemento.demo);
        for tipo in elemento.tipos {
            let bloque = insert::extraer_objeto(&fuente, tipo).expect("bloque");
            let bloque = if elemento.nombre == "horizonte" {
                neutralizar_horizonte(&bloque).expect("neutralizado")
            } else {
                bloque
            };
            let texto = String::from_utf8_lossy(&bloque).to_ascii_lowercase();
            assert!(
                !texto.contains("meteonorm"),
                "{tipo} de {} contiene Meteonorm",
                elemento.demo
            );
        }
    }
}
