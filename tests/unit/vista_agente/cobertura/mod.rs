use super::basica::{campos_con_clave, clave_de};
use crate::vista_agente::campos_vivos::CampoVivo;
use crate::vista_agente::dominio::Ancla;
use std::collections::HashSet;

fn vivo(clase: &str, x: i32, y: i32) -> CampoVivo {
    CampoVivo {
        hwnd: 1,
        clase: clase.into(),
        texto: String::new(),
        habilitado: true,
        posicion: Ancla { x, y },
    }
}

#[test]
fn should_keep_keys_unique_when_two_controls_share_a_spot() {
    let claves: HashSet<String> = campos_con_clave(vec![vivo("TEdit", 5, 5), vivo("TEdit", 5, 5)])
        .into_iter()
        .map(|(clave, _)| clave)
        .collect();
    assert_eq!(claves.len(), 2);
    assert!(claves.contains(&clave_de(&vivo("TEdit", 5, 5))));
}
