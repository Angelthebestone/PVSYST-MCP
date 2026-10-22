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
    /// The playbook needs the MCP's own calculations, not PVsyst's route.
    pub(crate) requiere_aproximaciones: bool,
}

pub(super) const INTRO: &str =
    "Recetas listas para usar. Cada una indica cuándo usarla y la secuencia de tools.";
pub(super) const CONSEJO: &str =
    "Si dudas, llama a 'entorno' y luego a 'disenar_sistema'. Todo lo demás es opcional.";

static PROYECTO_Y_VARIANTE: [ArgumentoReceta; 2] = [
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
];

/// Every recipe, in the order shown to clients.
pub(crate) static RECETAS: &[Receta] = &[
    Receta {
        id: "disenar_desde_coordenadas",
        requiere_aproximaciones: false,
        nombre: "Disenar sistema desde coordenadas (recomendada)",
        cuando: "El usuario da coordenadas y una potencia (p.ej. '10 kWp en 7.07,-73.84').",
        tools: &["disenar_sistema"],
        ejemplo: Some(
            r#"{"latitud":7.07,"longitud":-73.84,"kwp":10,"n_mod_serie":10,"n_strings":3}"#,
        ),
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
        plantilla: "Diseña un sistema fotovoltaico de {kwp} kWp en latitud {latitud}, longitud {longitud} con la tool disenar_sistema. Pide o propón los módulos en serie y el número de strings y pásalos como n_mod_serie y n_strings (el dimensionado automático es una aproximación del MCP y solo existe con PVSYST_MCP_APROXIMACIONES=1). Después resume energía anual, producción específica y PR del informe de PVsyst, y señala los avisos de PVsyst.",
    },
    Receta {
        id: "estudio_paso_a_paso",
        requiere_aproximaciones: false,
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
        requiere_aproximaciones: false,
        nombre: "Leer resultados",
        cuando: "Ya hay una simulación hecha.",
        tools: &["resultados kpis", "resultados csv", "resultados perdidas"],
        ejemplo: None,
        nota: None,
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Lee los resultados de {proyecto} variante {variante} con resultados kpis, csv y perdidas, y explica las tres pérdidas mayores.",
    },
    Receta {
        id: "optimizar_orientacion",
        requiere_aproximaciones: false,
        nombre: "Optimizar inclinacion/azimut",
        cuando: "Se busca el ángulo óptimo.",
        tools: &["simular ejecutar", "resultados kpis"],
        ejemplo: None,
        nota: Some("Cada fila de 'filas' (tilt/azim) es una simulación de PVsyst en modo lote (-bpf) y cuenta como una ejecución."),
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: "Nombre del proyecto",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Busca la mejor inclinación y azimut para {proyecto} con PVsyst: propone un conjunto de orientaciones, dime cuántas ejecuciones gastará, lánzalas con simular accion:ejecutar y 'filas' (tilt/azim por fila) y compara los casos con resultados kpis.",
    },
    Receta {
        id: "operar_gui",
        requiere_aproximaciones: false,
        nombre: "Operar la GUI (funciones solo-GUI)",
        cuando: "Editor 3D, gráficos I/V, tablas, asistentes, diálogos.",
        tools: &[
            "gui_actuar accion:lanzar",
            "gui_observar que:ventanas",
            "gui_observar que:controles",
            "gui_actuar accion:click",
            "gui_observar que:capturar",
            "gui_actuar accion:cerrar",
            "gui_actuar accion:funcion",
            "gui_actuar accion:clic_texto",
            "gui_observar que:formulario",
            "gui_observar que:esquema",
            "gui_observar que:vista",
            "gui_actuar accion:rellenar",
        ],
        ejemplo: None,
        nota: Some("Para formularios con esquema (que:esquema) rellena con accion:rellenar entidad+valores y lee con que:vista: claves estables, sin OCR ni coordenadas. accion:'clic_fisico' mueve el cursor real; accion:'clic_texto' y accion:'menu' (menu+item) no. Con un dialogo modal abierto no lances comandos de menu; tras un fallo de PVsyst usa accion:'reiniciar'."),
        argumentos: &[],
        plantilla: "Opera la GUI de PVsyst con gui_actuar y gui_observar: lanza, enumera ventanas y controles, escribe en los campos, actúa y captura el resultado.",
    },
    Receta {
        id: "explorar_archivo",
        requiere_aproximaciones: false,
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
        requiere_aproximaciones: false,
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
        requiere_aproximaciones: false,
        nombre: "Convertir a seguidor de un eje",
        cuando: "Se quiere pasar una variante de plano fijo a seguidor de un eje.",
        tools: &["proyecto seguidor", "proyecto doctor", "simular ejecutar"],
        ejemplo: None,
        nota: Some("La variante debe tener una sola orientación y no usar escena 3D ni bifacial."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Convierte {proyecto} variante {variante} a seguidor de un eje con proyecto accion:seguidor, ejecuta proyecto doctor y simula para comparar con la variante fija.",
    },
    Receta {
        id: "autoconsumo_con_baterias",
        requiere_aproximaciones: false,
        nombre: "Autoconsumo con baterías",
        cuando: "Se quiere añadir almacenamiento para autoconsumo o recorte de picos.",
        tools: &[
            "proyecto limite_inyeccion",
            "proyecto almacenamiento",
            "proyecto doctor",
        ],
        ejemplo: None,
        nota: Some("Para recorte de picos define antes el límite de inyección con proyecto limite_inyeccion."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Configura {proyecto} variante {variante} para autoconsumo con baterías: define el límite de inyección si vas a recortar picos, aplica proyecto accion:almacenamiento y revisa con proyecto doctor.",
    },
    Receta {
        id: "comparar_medido",
        requiere_aproximaciones: true,
        nombre: "Comparar medido vs simulado",
        cuando: "Hay una serie mensual medida y un informe PDF de PVsyst.",
        tools: &["resultados medido_vs_simulado"],
        ejemplo: None,
        nota: Some("Aproximación del MCP (requiere PVSYST_MCP_APROXIMACIONES=1); el equivalente de PVsyst es su herramienta de datos medidos en la GUI. La serie medida puede ir como 12 valores kWh o como CSV 'mes;energia_kwh'."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Compara la energía mensual medida de {proyecto} variante {variante} con la simulada usando resultados que:medido_vs_simulado; pásame la serie medida en kWh o un CSV y explica los meses fuera del umbral.",
    },
];
