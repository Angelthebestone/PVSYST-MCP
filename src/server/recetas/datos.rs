//! Static recipe catalogue: the playbooks and the prompt templates derived from them.

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
    /// JSON text of a sample call, parsed by `como_recurso`.
    pub(crate) ejemplo: Option<&'static str>,
    pub(crate) nota: Option<&'static str>,
    pub(crate) argumentos: &'static [ArgumentoReceta],
    pub(crate) plantilla: &'static str,
}

pub(super) const INTRO: &str =
    "Recetas listas para usar. Cada una indica cuándo usarla y la secuencia de tools.";
pub(super) const CONSEJO: &str =
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
    Receta {
        id: "convertir_a_seguidor",
        nombre: "Convertir a seguidor de un eje",
        cuando: "Se quiere pasar una variante de plano fijo a seguidor de un eje.",
        tools: &["proyecto seguidor", "proyecto doctor", "simular ejecutar"],
        ejemplo: None,
        nota: Some("La variante debe tener una sola orientación y no usar escena 3D ni bifacial."),
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
        plantilla: "Convierte {proyecto} variante {variante} a seguidor de un eje con proyecto accion:seguidor, ejecuta proyecto doctor y simula para comparar con la variante fija.",
    },
    Receta {
        id: "autoconsumo_con_baterias",
        nombre: "Autoconsumo con baterías",
        cuando: "Se quiere añadir almacenamiento para autoconsumo o recorte de picos.",
        tools: &[
            "proyecto limite_inyeccion",
            "proyecto almacenamiento",
            "proyecto doctor",
        ],
        ejemplo: None,
        nota: Some("Para recorte de picos define antes el límite de inyección con proyecto limite_inyeccion."),
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
        plantilla: "Configura {proyecto} variante {variante} para autoconsumo con baterías: define el límite de inyección si vas a recortar picos, aplica proyecto accion:almacenamiento y revisa con proyecto doctor.",
    },
    Receta {
        id: "comparar_medido",
        nombre: "Comparar medido vs simulado",
        cuando: "Hay una serie mensual medida y un informe PDF de PVsyst.",
        tools: &["resultados medido_vs_simulado"],
        ejemplo: None,
        nota: Some("La serie medida puede ir como 12 valores kWh o como CSV 'mes;energia_kwh'."),
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
        plantilla: "Compara la energía mensual medida de {proyecto} variante {variante} con la simulada usando resultados que:medido_vs_simulado; pásame la serie medida en kWh o un CSV y explica los meses fuera del umbral.",
    },
];
