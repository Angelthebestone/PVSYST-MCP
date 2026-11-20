//! Second-level dialogs reached from the start page: the preliminary design
//! windows, the "Nuevo" button of each component database and the user-info
//! editor of Preferencias. Observed live on PVsyst 8.1.7; entries that opened
//! nothing say why in `nota`.

use crate::adapters::gui::catalogo::dominio::Funcion;
use crate::adapters::gui::rotulos::dialogos::NUEVO;
use crate::adapters::gui::rotulos::pvsyst::{HORIZONTE, SISTEMA};

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton_en_idiomas(
        "diseno_preliminar.conectado_a_la_red.horizonte",
        "TDimensForm",
        HORIZONTE,
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema de red")
    .abre("TDimensSituationForm")
    .requiere("diseno_preliminar.conectado_a_la_red"),
    Funcion::boton_en_idiomas(
        "diseno_preliminar.conectado_a_la_red.sistema",
        "TDimensForm",
        SISTEMA,
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema de red")
    .requiere("diseno_preliminar.conectado_a_la_red")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.conectado_a_la_red.resultados",
        "TDimensForm",
        "Resultados",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema de red")
    .requiere("diseno_preliminar.conectado_a_la_red")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.conectado_a_la_red.sitio_y_datos_meteo",
        "TDimensForm",
        "Sitio y datos meteo",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema de red")
    .abre("TDimensSituationForm")
    .requiere("diseno_preliminar.conectado_a_la_red"),
    Funcion::boton(
        "diseno_preliminar.independiente.necesidades_del_usuario",
        "TDimensForm",
        "Necesidades del usuario",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .requiere("diseno_preliminar.independiente")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton_en_idiomas(
        "diseno_preliminar.independiente.horizonte",
        "TDimensForm",
        HORIZONTE,
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .requiere("diseno_preliminar.independiente")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton_en_idiomas(
        "diseno_preliminar.independiente.sistema",
        "TDimensForm",
        SISTEMA,
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .requiere("diseno_preliminar.independiente")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.independiente.resultados",
        "TDimensForm",
        "Resultados",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .requiere("diseno_preliminar.independiente")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.independiente.sitio_y_datos_meteo",
        "TDimensForm",
        "Sitio y datos meteo",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .abre("TDimensSituationForm")
    .requiere("diseno_preliminar.independiente"),
    Funcion::boton_en_idiomas(
        "diseno_preliminar.bombeo.horizonte",
        "TDimensForm",
        HORIZONTE,
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema bombeo")
    .requiere("diseno_preliminar.bombeo")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton_en_idiomas("diseno_preliminar.bombeo.sistema", "TDimensForm", SISTEMA)
        .en_ventana_titulada("Pre-dimensionamiento sistema bombeo")
        .requiere("diseno_preliminar.bombeo")
        .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.bombeo.resultados",
        "TDimensForm",
        "Resultados",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema bombeo")
    .requiere("diseno_preliminar.bombeo")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.bombeo.sitio_y_datos_meteo",
        "TDimensForm",
        "Sitio y datos meteo",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema bombeo")
    .abre("TDimensSituationForm")
    .requiere("diseno_preliminar.bombeo"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.modulos_pv.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un módulo FV")
    .abre("TPVModuleForm")
    .requiere("bases_de_datos.modulos_pv"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.inversores_red.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un inversor de red")
    .abre("TGridInverterForm")
    .requiere("bases_de_datos.inversores_red"),
    Funcion::boton_en_idiomas("bases_de_datos.baterias.nuevo", "TChoixObjetPVForm", NUEVO)
        .en_ventana_titulada("Eligiendo una batería")
        .abre("TBatteryForm")
        .requiere("bases_de_datos.baterias"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.controladores_independientes.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un contolador de carga/descarga")
    .abre("TSAControllerForm")
    .requiere("bases_de_datos.controladores_independientes"),
    Funcion::boton_en_idiomas("bases_de_datos.bombas.nuevo", "TChoixObjetPVForm", NUEVO)
        .en_ventana_titulada("Eligiendo una bomba")
        .abre("TPumpForm")
        .requiere("bases_de_datos.bombas"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.controladores_bombeo.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un controlador para sistemas de bombeo")
    .abre("TPumpControllerForm")
    .requiere("bases_de_datos.controladores_bombeo"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.generadores.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un generador de respaldo")
    .abre("TGeneratorForm")
    .requiere("bases_de_datos.generadores"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.fabricantes_minoristas.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un fabricante o minorista")
    .abre("TContactForm")
    .requiere("bases_de_datos.fabricantes_minoristas"),
    Funcion::boton_en_idiomas(
        "bases_de_datos.sitios_geograficos.nuevo",
        "TChoixObjetPVForm",
        NUEVO,
    )
    .en_ventana_titulada("Eligiendo un sitio geográfico")
    .abre("TGeoSiteForm")
    .requiere("bases_de_datos.sitios_geograficos"),
    Funcion::boton(
        "configuraciones.preferencias.editar_info_usuario",
        "TPreferencesForm",
        "Editar info usuario",
    )
    .abre("TContactForm")
    .requiere("configuraciones.preferencias"),
];
