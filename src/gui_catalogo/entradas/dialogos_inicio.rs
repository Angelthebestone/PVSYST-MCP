//! Second-level dialogs reached from the start page: the preliminary design
//! windows, the "Nuevo" button of each component database and the user-info
//! editor of Preferencias. Observed live on PVsyst 8.1.7; entries that opened
//! nothing say why in `nota`.

use crate::gui_catalogo::dominio::Funcion;

pub(super) const FUNCIONES: &[Funcion] = &[
    Funcion::boton(
        "diseno_preliminar.conectado_a_la_red.horizonte",
        "TDimensForm",
        "Horizonte",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema de red")
    .abre("TDimensSituationForm")
    .requiere("diseno_preliminar.conectado_a_la_red"),
    Funcion::boton(
        "diseno_preliminar.conectado_a_la_red.sistema",
        "TDimensForm",
        "Sistema",
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
    Funcion::boton(
        "diseno_preliminar.independiente.horizonte",
        "TDimensForm",
        "Horizonte",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema independiente")
    .requiere("diseno_preliminar.independiente")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton(
        "diseno_preliminar.independiente.sistema",
        "TDimensForm",
        "Sistema",
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
    Funcion::boton(
        "diseno_preliminar.bombeo.horizonte",
        "TDimensForm",
        "Horizonte",
    )
    .en_ventana_titulada("Pre-dimensionamiento sistema bombeo")
    .requiere("diseno_preliminar.bombeo")
    .nota("no abrió ventana sin sitio geográfico definido (botón deshabilitado)"),
    Funcion::boton("diseno_preliminar.bombeo.sistema", "TDimensForm", "Sistema")
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
    Funcion::boton(
        "bases_de_datos.modulos_pv.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un módulo FV")
    .abre("TPVModuleForm")
    .requiere("bases_de_datos.modulos_pv"),
    Funcion::boton(
        "bases_de_datos.inversores_red.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un inversor de red")
    .abre("TGridInverterForm")
    .requiere("bases_de_datos.inversores_red"),
    Funcion::boton(
        "bases_de_datos.baterias.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo una batería")
    .abre("TBatteryForm")
    .requiere("bases_de_datos.baterias"),
    Funcion::boton(
        "bases_de_datos.controladores_independientes.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un contolador de carga/descarga")
    .abre("TSAControllerForm")
    .requiere("bases_de_datos.controladores_independientes"),
    Funcion::boton("bases_de_datos.bombas.nuevo", "TChoixObjetPVForm", "Nuevo")
        .en_ventana_titulada("Eligiendo una bomba")
        .abre("TPumpForm")
        .requiere("bases_de_datos.bombas"),
    Funcion::boton(
        "bases_de_datos.controladores_bombeo.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un controlador para sistemas de bombeo")
    .abre("TPumpControllerForm")
    .requiere("bases_de_datos.controladores_bombeo"),
    Funcion::boton(
        "bases_de_datos.generadores.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un generador de respaldo")
    .abre("TGeneratorForm")
    .requiere("bases_de_datos.generadores"),
    Funcion::boton(
        "bases_de_datos.fabricantes_minoristas.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
    )
    .en_ventana_titulada("Eligiendo un fabricante o minorista")
    .abre("TContactForm")
    .requiere("bases_de_datos.fabricantes_minoristas"),
    Funcion::boton(
        "bases_de_datos.sitios_geograficos.nuevo",
        "TChoixObjetPVForm",
        "Nuevo",
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
