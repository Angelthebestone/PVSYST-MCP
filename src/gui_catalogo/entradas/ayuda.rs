//! Main form menus Licencia, Ayuda and Idioma (command ids read from the open
//! popups of PVsyst 8.1.7; labels are Spanish descriptions of what each item
//! showed). The tutorials submenu holds the PDF guides.

use crate::gui_catalogo::dominio::{Efecto, Funcion};

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::menu(
        "licencia.detalles",
        "Licencia > Detalles de la licencia",
        74,
    ),
    Funcion::menu("ayuda.ayuda", "Ayuda > Ayuda", 76),
    Funcion::menu(
        "ayuda.notas_de_lanzamiento",
        "Ayuda > Notas de lanzamiento",
        77,
    ),
    Funcion::menu("ayuda.sitio_web", "Ayuda > Visite el sitio web", 79),
    Funcion::menu("ayuda.foro", "Ayuda > Visite el foro", 80),
    Funcion::menu("ayuda.tutoriales_video", "Ayuda > Tutoriales en vídeo", 81),
    Funcion::menu("ayuda.condiciones_de_uso", "Ayuda > Condiciones de uso", 93),
    Funcion::menu(
        "ayuda.buscar_actualizaciones",
        "Ayuda > Buscar actualizaciones",
        95,
    ),
    Funcion::menu(
        "ayuda.buscar_actualizaciones_al_inicio",
        "Ayuda > Buscar actualizaciones al inicio (casilla)",
        96,
    )
    .con_efecto(Efecto::Modifica),
    Funcion::menu("ayuda.acerca_de", "Ayuda > Acerca de PVsyst", 98),
    Funcion::menu(
        "tutoriales_pdf.conectado_a_la_red",
        "Ayuda > Tutoriales en PDF > Conectado a la red",
        83,
    ),
    Funcion::menu(
        "tutoriales_pdf.independiente",
        "Ayuda > Tutoriales en PDF > Independiente",
        84,
    ),
    Funcion::menu(
        "tutoriales_pdf.bombeo",
        "Ayuda > Tutoriales en PDF > Bombeo",
        85,
    ),
    Funcion::menu(
        "tutoriales_pdf.escenas_3d",
        "Ayuda > Tutoriales en PDF > Exportación de escenas 3D a PVsyst",
        86,
    ),
    Funcion::menu(
        "tutoriales_pdf.terreno_topografia",
        "Ayuda > Tutoriales en PDF > Terreno & Topografía",
        87,
    ),
    Funcion::menu(
        "tutoriales_pdf.pseudo_subhoraria",
        "Ayuda > Tutoriales en PDF > Simulación pseudo-subhoraria (EN)",
        88,
    ),
    Funcion::menu(
        "tutoriales_pdf.base_de_componentes",
        "Ayuda > Tutoriales en PDF > Base de datos de componentes",
        89,
    ),
    Funcion::menu(
        "tutoriales_pdf.datos_meteorologicos",
        "Ayuda > Tutoriales en PDF > Gestión de datos meteorológicos",
        90,
    ),
    Funcion::menu(
        "tutoriales_pdf.paso_a_paso_independiente",
        "Ayuda > Tutoriales en PDF > Paso a paso Independiente",
        91,
    ),
    Funcion::menu(
        "tutoriales_pdf.paso_a_paso_bombeo",
        "Ayuda > Tutoriales en PDF > Paso a paso Bombeo",
        92,
    ),
    Funcion::menu("idioma.ingles", "Idioma > English", 130).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.frances", "Idioma > Français", 131).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.aleman", "Idioma > Deutsch", 132).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.italiano", "Idioma > Italiano", 133).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.espanol", "Idioma > Español", 134).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.portugues", "Idioma > Português", 135).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.turco", "Idioma > Türkçe", 136).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.coreano", "Idioma > 한글", 137).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.chino", "Idioma > 简体中文", 138).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.japones", "Idioma > 日本語", 139).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.polaco", "Idioma > Polski", 140).con_efecto(Efecto::Modifica),
    Funcion::menu("idioma.ruso", "Idioma > Русский", 141).con_efecto(Efecto::Modifica),
];
