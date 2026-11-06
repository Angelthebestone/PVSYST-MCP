//! Behavior of the language selection and the bilingual wording.

use super::*;

#[test]
fn valor_es_solo_con_es() {
    assert_eq!(Idioma::de_valor(Some("es")), Idioma::Es);
    assert_eq!(Idioma::de_valor(Some(" ES ")), Idioma::Es);
    assert_eq!(Idioma::de_valor(Some("Es")), Idioma::Es);
}

#[test]
fn valor_ingles_por_defecto() {
    assert_eq!(Idioma::de_valor(Some("en")), Idioma::En);
    assert_eq!(Idioma::de_valor(Some("fr")), Idioma::En);
    assert_eq!(Idioma::de_valor(Some("")), Idioma::En);
    assert_eq!(Idioma::de_valor(None), Idioma::En);
}

#[test]
fn texto_segun_idioma() {
    const SALUDO: Texto = Texto {
        es: "hola",
        en: "hello",
    };
    assert_eq!(SALUDO.en_idioma(Idioma::Es), "hola");
    assert_eq!(SALUDO.en_idioma(Idioma::En), "hello");
}

#[test]
fn texto_con_sustituye_claves() {
    const MENSAJE: Texto = Texto {
        es: "Falta {campo} en {lugar}",
        en: "Missing {campo} in {lugar}",
    };
    let texto = MENSAJE.texto_con(Idioma::Es, &[("campo", "archivo"), ("lugar", "proyecto")]);
    assert_eq!(texto, "Falta archivo en proyecto");
    let texto = MENSAJE.texto_con(Idioma::En, &[("campo", "file"), ("lugar", "project")]);
    assert_eq!(texto, "Missing file in project");
}

#[test]
fn texto_con_deja_claves_sin_valor() {
    const MENSAJE: Texto = Texto {
        es: "Falta {campo}",
        en: "Missing {campo}",
    };
    assert_eq!(MENSAJE.texto_con(Idioma::En, &[]), "Missing {campo}");
}
