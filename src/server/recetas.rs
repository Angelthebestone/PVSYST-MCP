//! Ready-to-use recipes, exposed both as the `pvsyst://recetas` resource and as
//! MCP prompts. The prompt template is filled from the caller's arguments.

use crate::args;
use crate::error::{Error, Result};
use serde_json::{json, Value};

/// Whether a prompt argument must be supplied by the client.
pub(crate) enum Requerido {
    Si,
    No,
}

/// One argument of a prompt template.
pub(crate) struct ArgumentoReceta {
    pub(crate) nombre: &'static str,
    pub(crate) descripcion: &'static str,
    pub(crate) requerido: Requerido,
    /// Value substituted when the argument is optional and absent.
    pub(crate) defecto: &'static str,
}

/// A playbook with the tool sequence and the prompt template derived from it.
pub(crate) struct Receta {
    pub(crate) id: &'static str,
    pub(crate) nombre: &'static str,
    pub(crate) cuando: &'static str,
    pub(crate) tools: &'static [&'static str],
    /// JSON text of a sample call, parsed by [`como_recurso`].
    pub(crate) ejemplo: Option<&'static str>,
    pub(crate) nota: Option<&'static str>,
    pub(crate) argumentos: &'static [ArgumentoReceta],
    pub(crate) plantilla: &'static str,
}

const INTRO: &str =
    "Recetas listas para usar. Cada una indica cuándo usarla y la secuencia de tools.";
const CONSEJO: &str =
    "Si dudas, llama a 'entorno' y luego a 'disenar_sistema'. Todo lo demás es opcional.";

/// Every recipe, in the order shown to clients.
pub(crate) static RECETAS: &[Receta] = &[
    Receta {
        id: "disenar_desde_coordenadas",
        nombre: "Disenar sistema desde coordenadas (recomendada)",
        cuando: "El usuario da coordenadas y una potencia (p.ej. '10 kWp en 7.07,-73.84').",
        tools: &["disenar_sistema"],
        ejemplo: Some(r#"{"latitud":7.07,"longitud":-73.84,"kwp":10}"#),
        nota: None,
        argumentos: &[
            ArgumentoReceta {
                nombre: "latitud",
                descripcion: "Latitud del sitio en grados",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "longitud",
                descripcion: "Longitud del sitio en grados",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "kwp",
                descripcion: "Potencia nominal en kWp",
                requerido: Requerido::Si,
                defecto: "",
            },
        ],
        plantilla: "Diseña un sistema fotovoltaico de {kwp} kWp en latitud {latitud}, longitud {longitud} con la tool disenar_sistema. Después resume energía anual, producción específica y PR, y señala los avisos.",
    },
    Receta {
        id: "estudio_paso_a_paso",
        nombre: "Estudio completo paso a paso",
        cuando: "Se quiere control fino de cada fase.",
        tools: &[
            "sitio crear",
            "proyecto crear",
            "editar campos",
            "proyecto dimensionar",
            "simular",
            "resultados kpis",
        ],
        ejemplo: None,
        nota: None,
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: "Nombre del proyecto",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Haz un estudio paso a paso del proyecto {proyecto}: sitio crear, proyecto crear, editar campos, proyecto dimensionar, simular y resultados kpis. Confírmame cada fase antes de simular.",
    },
    Receta {
        id: "leer_resultados",
        nombre: "Leer resultados",
        cuando: "Ya hay una simulación hecha.",
        tools: &["resultados kpis", "resultados csv", "resultados perdidas"],
        ejemplo: None,
        nota: None,
        argumentos: &[
            ArgumentoReceta {
                nombre: "proyecto",
                descripcion: "Nombre del proyecto",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "variante",
                descripcion: "Variante del proyecto",
                requerido: Requerido::No,
                defecto: "VC0",
            },
        ],
        plantilla: "Lee los resultados de {proyecto} variante {variante} con resultados kpis, csv y perdidas, y explica las tres pérdidas mayores.",
    },
    Receta {
        id: "optimizar_orientacion",
        nombre: "Optimizar inclinacion/azimut",
        cuando: "Se busca el ángulo óptimo.",
        tools: &["simular barrido", "simular ejecutar", "resultados kpis"],
        ejemplo: None,
        nota: Some("Barrido: simular con 'filas' crea el CSV de batch y lanza con batch_params_file."),
        argumentos: &[
            ArgumentoReceta {
                nombre: "proyecto",
                descripcion: "Nombre del proyecto",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "latitud",
                descripcion: "Latitud del sitio en grados",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "longitud",
                descripcion: "Longitud del sitio en grados",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "kwp",
                descripcion: "Potencia nominal en kWp",
                requerido: Requerido::Si,
                defecto: "",
            },
        ],
        plantilla: "Busca la mejor inclinación y azimut para {proyecto} ({kwp} kWp en {latitud}, {longitud}) con simular barrido y compara los mejores casos con resultados kpis.",
    },
    Receta {
        id: "operar_gui",
        nombre: "Operar la GUI (funciones solo-GUI)",
        cuando: "Editor 3D, gráficos I/V, tablas, asistentes, diálogos.",
        tools: &[
            "gui_actuar accion:lanzar",
            "gui_observar que:ventanas",
            "gui_observar que:controles",
            "gui_actuar accion:click",
            "gui_observar que:capturar",
            "gui_actuar accion:cerrar",
        ],
        ejemplo: None,
        nota: Some("gui_actuar con accion:'clic_fisico' mueve el cursor real."),
        argumentos: &[],
        plantilla: "Opera la GUI de PVsyst con gui_actuar y gui_observar (modo experto): lanza, enumera ventanas y controles, actúa y captura el resultado.",
    },
    Receta {
        id: "explorar_archivo",
        nombre: "Explorar/editar cualquier archivo",
        cuando: "Ajustes finos no cubiertos por tools de dominio.",
        tools: &["leer", "listar que:objetos", "editar campos", "listar que:archivos"],
        ejemplo: None,
        nota: None,
        argumentos: &[ArgumentoReceta {
            nombre: "archivo",
            descripcion: "Ruta o nombre del archivo",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Lee {archivo} con leer, lista sus objetos con listar que:objetos y propón los cambios con editar antes de aplicarlos.",
    },
    Receta {
        id: "diagnostico_entorno",
        nombre: "Diagnostico del entorno",
        cuando: "Antes de empezar o si algo falla.",
        tools: &["entorno diagnostico", "entorno licencia"],
        ejemplo: None,
        nota: None,
        argumentos: &[],
        plantilla: "Ejecuta entorno accion:diagnostico y explica cualquier problema de instalación, licencia o rutas.",
    },
];

/// Resource payload for `pvsyst://recetas`; same shape as before prompts existed.
pub(crate) fn como_recurso() -> Value {
    let recetas: Vec<Value> = RECETAS.iter().map(recurso_de).collect();
    json!({
        "intro": INTRO,
        "recetas": recetas,
        "consejo": CONSEJO,
    })
}

fn recurso_de(receta: &Receta) -> Value {
    let mut valor = json!({
        "nombre": receta.nombre,
        "cuando": receta.cuando,
        "tools": receta.tools,
    });
    if let Some(texto) = receta.ejemplo {
        if let Ok(ejemplo) = serde_json::from_str::<Value>(texto) {
            valor["ejemplo"] = ejemplo;
        }
    }
    if let Some(nota) = receta.nota {
        valor["nota"] = json!(nota);
    }
    valor
}

/// Prompt catalogue for `prompts/list`.
pub(crate) fn listar_prompts() -> Value {
    let prompts: Vec<Value> = RECETAS.iter().map(prompt_de).collect();
    json!({ "prompts": prompts })
}

fn prompt_de(receta: &Receta) -> Value {
    let argumentos: Vec<Value> = receta
        .argumentos
        .iter()
        .map(|a| {
            json!({
                "name": a.nombre,
                "description": a.descripcion,
                "required": matches!(a.requerido, Requerido::Si),
            })
        })
        .collect();
    json!({
        "name": receta.id,
        "title": receta.nombre,
        "description": receta.cuando,
        "arguments": argumentos,
    })
}

/// One filled prompt for `prompts/get`.
pub(crate) fn obtener_prompt(params: &Value) -> Result<Value> {
    let nombre = params
        .get("name")
        .and_then(Value::as_str)
        .filter(|n| !n.is_empty())
        .ok_or_else(|| args::missing("name", "id de la receta"))?;
    let receta = RECETAS
        .iter()
        .find(|r| r.id == nombre)
        .ok_or_else(|| Error::NotFound(format!("Prompt desconocido: {nombre}")))?;
    let argumentos = params.get("arguments").unwrap_or(&Value::Null);
    let texto = rellenar(receta, argumentos)?;
    Ok(json!({
        "description": receta.cuando,
        "messages": [{
            "role": "user",
            "content": { "type": "text", "text": texto },
        }],
    }))
}

fn rellenar(receta: &Receta, recibidos: &Value) -> Result<String> {
    receta
        .argumentos
        .iter()
        .try_fold(receta.plantilla.to_string(), |texto, argumento| {
            let valor = valor_de(argumento, recibidos.get(argumento.nombre))?;
            Ok(texto.replace(&format!("{{{}}}", argumento.nombre), &valor))
        })
}

fn valor_de(argumento: &ArgumentoReceta, recibido: Option<&Value>) -> Result<String> {
    if let Some(texto) = recibido.and_then(texto_de) {
        return Ok(texto);
    }
    match argumento.requerido {
        Requerido::Si => Err(args::missing(argumento.nombre, argumento.descripcion)),
        Requerido::No => Ok(argumento.defecto.to_string()),
    }
}

fn texto_de(valor: &Value) -> Option<String> {
    match valor {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/unit/server/recetas.rs"]
mod tests;
