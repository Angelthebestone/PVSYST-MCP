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
    "Ready-to-use recipes. Each one states when to use it and the tool sequence.";
pub(super) const CONSEJO: &str =
    "Start with entorno accion:diagnostico if something fails. Before each call, check the tool's schema for the required arguments of its action. Everything else is optional.";

static PROYECTO_Y_VARIANTE: [ArgumentoReceta; 2] = [
    ArgumentoReceta {
        nombre: "proyecto",
        descripcion: "Project name",
        requerido: Requerido::Si,
        defecto: "",
    },
    ArgumentoReceta {
        nombre: "variante",
        descripcion: "Project variant",
        requerido: Requerido::No,
        defecto: "VC0",
    },
];

/// Every recipe, in the order shown to clients.
pub(crate) static RECETAS: &[Receta] = &[
    Receta {
        id: "disenar_desde_coordenadas",
        requiere_aproximaciones: false,
        nombre: "Design system from coordinates (recommended)",
        cuando: "The user provides coordinates and a power (e.g. '10 kWp at 7.07,-73.84').",
        tools: &["disenar_sistema"],
        ejemplo: Some(
            r#"{"latitud":7.07,"longitud":-73.84,"kwp":10}"#,
        ),
        nota: Some("modulo and inversor are optional: without them disenar_sistema uses the template's. To pick others, list components with listar que:componentes (type PAN or OND) and pass them as modulo/inversor."),
        argumentos: &[
            ArgumentoReceta {
                nombre: "latitud",
                descripcion: "Site latitude in degrees",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "longitud",
                descripcion: "Site longitude in degrees",
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "kwp",
                descripcion: "Nominal power in kWp",
                requerido: Requerido::Si,
                defecto: "",
            },
        ],
        plantilla: "Design a photovoltaic system of {kwp} kWp at latitude {latitud}, longitude {longitud} with disenar_sistema (modulo, inversor, n_mod_serie and n_strings are optional). Then summarize the annual energy, specific production and PR from the PVsyst report, and point out the PVsyst warnings.",
    },
    Receta {
        id: "estudio_paso_a_paso",
        requiere_aproximaciones: false,
        nombre: "Full step-by-step study",
        cuando: "Fine-grained control of every phase is wanted.",
        tools: &[
            "sitio crear",
            "proyecto crear",
            "leer",
            "proyecto dimensionar",
            "simular ejecutar",
            "resultados kpis",
        ],
        ejemplo: None,
        nota: Some("sitio crear, sitio convertir_meteo, sitio meteo_pvgis and simular consume PVsyst TRIAL executions (meteo_pvgis does not if the coordinates are already cached). The template must exist in the workspace's Projects folder."),
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: "Project name",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Do a step-by-step study of project {proyecto}: 1) sitio crear with the coordinates; 2) proyecto crear with plantilla and nombre_nuevo {proyecto}; 3) check with leer that the project uses that site; 4) proyecto dimensionar with n_mod_serie and n_strings; 5) simular ejecutar; 6) resultados kpis. Pass the required arguments stated by each tool's schema in every call.",
    },
    Receta {
        id: "leer_resultados",
        requiere_aproximaciones: false,
        nombre: "Read results",
        cuando: "A simulation already exists.",
        tools: &["resultados kpis", "resultados perdidas", "resultados mensual"],
        ejemplo: None,
        nota: None,
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Read the results of {proyecto} variant {variante} with resultados kpis, perdidas and mensual, and explain the three largest losses.",
    },
    Receta {
        id: "optimizar_orientacion",
        requiere_aproximaciones: false,
        nombre: "Optimize tilt/azimuth",
        cuando: "The optimal angle is sought.",
        tools: &["simular ejecutar", "resultados kpis"],
        ejemplo: None,
        nota: Some("Each row of 'filas' (tilt/azim) is one PVsyst simulation in batch mode (-bpf) and counts as one execution."),
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: "Project name",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Find the best tilt and azimuth for {proyecto} with PVsyst: propose the orientations, state how many executions it will spend (one per row of 'filas') and ask for confirmation before launching them with simular accion:ejecutar; compare the cases with resultados kpis.",
    },
    Receta {
        id: "operar_gui",
        requiere_aproximaciones: false,
        nombre: "Operate the GUI (GUI-only features)",
        cuando: "3D editor, I/V curves, tables, wizards, dialogs.",
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
        nota: Some("For forms with a schema (que:esquema) fill with accion:rellenar entidad+valores and read with que:vista: stable keys, no OCR or coordinates. accion:'escena' (escena+objetos) with simular:true validates a 3D scene request without touching PVsyst, and with simular it builds it in the open (empty) 3D editor and compares the .SHD PVsyst saved with what was requested; accion:'escena_leer' (escena) exports and reads the open scene; accion:'escena_importar' (archivo+escena) imports a 3DS/DAE/PVC/PVC2/PVZ2 mesh with PVsyst's importer; accion:'menu' accepts subitem for the entries of a submenu; accion:'clic_fisico' moves the real cursor; accion:'fila' (ventana=TVirtualStringTree or TListBox, indice) selects that row with a physical click and fails if the highlighted band is not the requested one or if the click opened the item; accion:'clic_texto' and accion:'menu' (menu+item) do not. Every response (error ones too) carries vista{estado,resumen,siguiente,error}: follow vista.siguiente as-is. With a modal dialog open do not launch menu commands; after a PVsyst crash use accion:'reiniciar'."),
        argumentos: &[],
        plantilla: "Operate the PVsyst GUI: launch with gui_actuar accion:lanzar, open the feature with accion:funcion (it resolves its prerequisites), fill with accion:rellenar and read the outcome with gui_observar que:vista. When done, close the windows with accion:cerrar.",
    },
    Receta {
        id: "explorar_archivo",
        requiere_aproximaciones: false,
        nombre: "Explore/edit any file",
        cuando: "Fine adjustments not covered by the domain tools.",
        tools: &["leer", "listar que:objetos", "editar campos", "listar que:archivos"],
        ejemplo: None,
        nota: None,
        argumentos: &[ArgumentoReceta {
            nombre: "archivo",
            descripcion: "File path or name",
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: "Read {archivo} with leer, list its objects with listar que:objetos (if truncated, repeat with detalle:completo) and propose the changes with editar before applying them.",
    },
    Receta {
        id: "diagnostico_entorno",
        requiere_aproximaciones: false,
        nombre: "Environment diagnostics",
        cuando: "Before starting or if something fails.",
        tools: &["entorno diagnostico", "entorno licencia"],
        ejemplo: None,
        nota: None,
        argumentos: &[],
        plantilla: "Run entorno accion:diagnostico and explain any installation, licence or path problems.",
    },
    Receta {
        id: "convertir_a_seguidor",
        requiere_aproximaciones: false,
        nombre: "Convert to single-axis tracker",
        cuando: "A fixed-plane variant is to become a single-axis tracker.",
        tools: &["proyecto crear_variante", "proyecto seguidor", "proyecto doctor", "simular ejecutar"],
        ejemplo: None,
        nota: Some("The variant must have a single orientation and must not use a 3D scene or bifacial."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Convert {proyecto} variant {variante} to a single-axis tracker: first create a copy with proyecto accion:crear_variante (variante_nueva) to keep the fixed variant, convert the copy with proyecto accion:seguidor, run proyecto doctor and simulate both to compare.",
    },
    Receta {
        id: "autoconsumo_con_baterias",
        requiere_aproximaciones: false,
        nombre: "Self-consumption with batteries",
        cuando: "Storage is to be added for self-consumption or peak shaving.",
        tools: &[
            "proyecto limite_inyeccion",
            "proyecto almacenamiento",
            "proyecto doctor",
        ],
        ejemplo: None,
        nota: Some("For peak shaving, define the injection limit first with proyecto limite_inyeccion."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Set up {proyecto} variant {variante} for self-consumption with batteries: define the injection limit if you will shave peaks, apply proyecto accion:almacenamiento and review with proyecto doctor.",
    },
    Receta {
        id: "comparar_medido",
        requiere_aproximaciones: true,
        nombre: "Compare measured vs simulated",
        cuando: "There is a measured monthly series and a PVsyst PDF report.",
        tools: &["resultados medido_vs_simulado"],
        ejemplo: None,
        nota: Some("MCP approximation (requires PVSYST_MCP_APROXIMACIONES=1); the PVsyst equivalent is its measured-data tool in the GUI. The measured series can be given as 12 kWh values or as a CSV 'mes;energia_kwh'."),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: "Compare the measured monthly energy of {proyecto} variant {variante} with the simulated one using resultados que:medido_vs_simulado; pass me the measured series in kWh or a CSV and explain the months outside the threshold.",
    },
];
