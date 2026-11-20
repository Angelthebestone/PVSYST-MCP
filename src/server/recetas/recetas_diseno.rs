//! Design and build recipes: from coordinates to a sized, simulated system.

use super::datos::{ArgumentoReceta, Receta, Requerido, PROYECTO_Y_VARIANTE};

pub(super) static GRUPO: [Receta; 5] = [
    Receta {
        id: "disenar_desde_coordenadas",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Disenar sistema desde coordenadas (recomendada)",
            en: "Design system from coordinates (recommended)",
        },
        cuando: crate::idioma::Texto {
            es: "El usuario da coordenadas y una potencia (p.ej. '10 kWp en 7.07,-73.84').",
            en: "The user provides coordinates and a power (e.g. '10 kWp at 7.07,-73.84').",
        },
        tools: &["disenar_sistema"],
        ejemplo: Some(
            r#"{"latitud":7.07,"longitud":-73.84,"kwp":10}"#,
        ),
        nota: Some(crate::idioma::Texto {
            es: "modulo e inversor son opcionales: sin ellos disenar_sistema usa los de la plantilla. Para elegir otros, lista componentes con listar que:componentes (tipo PAN u OND) y pásalos como modulo/inversor.",
            en: "modulo and inversor are optional: without them disenar_sistema uses the template's. To pick others, list components with listar que:componentes (type PAN or OND) and pass them as modulo/inversor.",
        }),
        argumentos: &[
            ArgumentoReceta {
                nombre: "latitud",
                descripcion: crate::idioma::Texto {
            es: "Latitud del sitio en grados",
            en: "Site latitude in degrees",
        },
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "longitud",
                descripcion: crate::idioma::Texto {
            es: "Longitud del sitio en grados",
            en: "Site longitude in degrees",
        },
                requerido: Requerido::Si,
                defecto: "",
            },
            ArgumentoReceta {
                nombre: "kwp",
                descripcion: crate::idioma::Texto {
            es: "Potencia nominal en kWp",
            en: "Nominal power in kWp",
        },
                requerido: Requerido::Si,
                defecto: "",
            },
        ],
        plantilla: crate::idioma::Texto {
            es: "Diseña un sistema fotovoltaico de {kwp} kWp en latitud {latitud}, longitud {longitud} con disenar_sistema (modulo, inversor, n_mod_serie y n_strings son opcionales). Después resume energía anual, producción específica y PR del informe de PVsyst, y señala los avisos de PVsyst.",
            en: "Design a photovoltaic system of {kwp} kWp at latitude {latitud}, longitude {longitud} with disenar_sistema (modulo, inversor, n_mod_serie and n_strings are optional). Then summarize the annual energy, specific production and PR from the PVsyst report, and point out the PVsyst warnings.",
        },
    },
    Receta {
        id: "estudio_paso_a_paso",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Estudio completo paso a paso",
            en: "Full step-by-step study",
        },
        cuando: crate::idioma::Texto {
            es: "Se quiere control fino de cada fase.",
            en: "Fine-grained control of every phase is wanted.",
        },
        tools: &[
            "sitio crear",
            "proyecto crear",
            "leer",
            "proyecto dimensionar",
            "simular ejecutar",
            "resultados kpis",
        ],
        ejemplo: None,
        nota: Some(crate::idioma::Texto {
            es: "sitio crear, sitio convertir_meteo, sitio meteo_pvgis y simular gastan ejecuciones TRIAL de PVsyst (meteo_pvgis no gasta si las coordenadas ya están en caché). La plantilla debe existir en la carpeta Projects del workspace.",
            en: "sitio crear, sitio convertir_meteo, sitio meteo_pvgis and simular consume PVsyst TRIAL executions (meteo_pvgis does not if the coordinates are already cached). The template must exist in the workspace's Projects folder.",
        }),
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: crate::idioma::Texto {
            es: "Nombre del proyecto",
            en: "Project name",
        },
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: crate::idioma::Texto {
            es: "Haz un estudio paso a paso del proyecto {proyecto}: 1) sitio crear con las coordenadas; 2) proyecto crear con plantilla y nombre_nuevo {proyecto}; 3) comprueba con leer que el proyecto usa ese sitio; 4) proyecto dimensionar con n_mod_serie y n_strings; 5) simular ejecutar; 6) resultados kpis. Pasa en cada llamada los argumentos obligatorios que indica su esquema.",
            en: "Do a step-by-step study of project {proyecto}: 1) sitio crear with the coordinates; 2) proyecto crear with plantilla and nombre_nuevo {proyecto}; 3) check with leer that the project uses that site; 4) proyecto dimensionar with n_mod_serie and n_strings; 5) simular ejecutar; 6) resultados kpis. Pass the required arguments stated by each tool's schema in every call.",
        },
    },
    Receta {
        id: "optimizar_orientacion",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Optimizar inclinacion/azimut",
            en: "Optimize tilt/azimuth",
        },
        cuando: crate::idioma::Texto {
            es: "Se busca el ángulo óptimo.",
            en: "The optimal angle is sought.",
        },
        tools: &["simular ejecutar", "resultados kpis"],
        ejemplo: None,
        nota: Some(crate::idioma::Texto {
            es: "Cada fila de 'filas' (tilt/azim) es una simulación de PVsyst en modo lote (-bpf) y cuenta como una ejecución.",
            en: "Each row of 'filas' (tilt/azim) is one PVsyst simulation in batch mode (-bpf) and counts as one execution.",
        }),
        argumentos: &[ArgumentoReceta {
            nombre: "proyecto",
            descripcion: crate::idioma::Texto {
            es: "Nombre del proyecto",
            en: "Project name",
        },
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: crate::idioma::Texto {
            es: "Busca la mejor inclinación y azimut para {proyecto} con PVsyst: propone las orientaciones, indica cuántas ejecuciones gastará (una por fila de 'filas') y pide confirmación antes de lanzarlas con simular accion:ejecutar; compara los casos con resultados kpis.",
            en: "Find the best tilt and azimuth for {proyecto} with PVsyst: propose the orientations, state how many executions it will spend (one per row of 'filas') and ask for confirmation before launching them with simular accion:ejecutar; compare the cases with resultados kpis.",
        },
    },
    Receta {
        id: "convertir_a_seguidor",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Convertir a seguidor de un eje",
            en: "Convert to single-axis tracker",
        },
        cuando: crate::idioma::Texto {
            es: "Se quiere pasar una variante de plano fijo a seguidor de un eje.",
            en: "A fixed-plane variant is to become a single-axis tracker.",
        },
        tools: &["proyecto crear_variante", "proyecto seguidor", "proyecto doctor", "simular ejecutar"],
        ejemplo: None,
        nota: Some(crate::idioma::Texto {
            es: "La variante debe tener una sola orientación y no usar escena 3D ni bifacial.",
            en: "The variant must have a single orientation and must not use a 3D scene or bifacial.",
        }),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: crate::idioma::Texto {
            es: "Convierte {proyecto} variante {variante} a seguidor de un eje: crea primero una copia con proyecto accion:crear_variante (variante_nueva) para conservar la variante fija, convierte la copia con proyecto accion:seguidor, ejecuta proyecto doctor y simula ambas para comparar.",
            en: "Convert {proyecto} variant {variante} to a single-axis tracker: first create a copy with proyecto accion:crear_variante (variante_nueva) to keep the fixed variant, convert the copy with proyecto accion:seguidor, run proyecto doctor and simulate both to compare.",
        },
    },
    Receta {
        id: "autoconsumo_con_baterias",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Autoconsumo con baterías",
            en: "Self-consumption with batteries",
        },
        cuando: crate::idioma::Texto {
            es: "Se quiere añadir almacenamiento para autoconsumo o recorte de picos.",
            en: "Storage is to be added for self-consumption or peak shaving.",
        },
        tools: &[
            "proyecto limite_inyeccion",
            "proyecto almacenamiento",
            "proyecto doctor",
        ],
        ejemplo: None,
        nota: Some(crate::idioma::Texto {
            es: "Para recorte de picos define antes el límite de inyección con proyecto limite_inyeccion.",
            en: "For peak shaving, define the injection limit first with proyecto limite_inyeccion.",
        }),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: crate::idioma::Texto {
            es: "Configura {proyecto} variante {variante} para autoconsumo con baterías: define el límite de inyección si vas a recortar picos, aplica proyecto accion:almacenamiento y revisa con proyecto doctor.",
            en: "Set up {proyecto} variant {variante} for self-consumption with batteries: define the injection limit if you will shave peaks, apply proyecto accion:almacenamiento and review with proyecto doctor.",
        },
    },
];
