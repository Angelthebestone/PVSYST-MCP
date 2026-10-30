//! Project window (`TProjectForm`): the project's description, its weather file and the active
//! variant, PVsyst 8.1.8. Anchors are the control corners in the form's 96-DPI layout, measured live.

use crate::vista_agente::dominio::{Ancla, CampoEsquema, Entidad};

const fn en(x: i32, y: i32) -> Ancla {
    Ancla { x, y }
}

pub(crate) static PROYECTO: Entidad = Entidad {
    clave: "proyecto.ventana",
    formulario: "TProjectForm",
    titulo: "Proyecto",
    campos: &[
        CampoEsquema::texto("descripcion", "Descripción del proyecto", en(201, 116)),
        CampoEsquema::lista("archivo_meteo", "Datos meteorológicos", en(201, 185)),
        CampoEsquema::lista("variante_activa", "Variante de simulación", en(166, 381)),
    ],
};
