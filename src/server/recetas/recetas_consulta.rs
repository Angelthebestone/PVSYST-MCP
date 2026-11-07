//! Inspection and operation recipes: results, files, environment, GUI and comparison.

use super::datos::{ArgumentoReceta, Receta, Requerido, PROYECTO_Y_VARIANTE};

pub(super) static GRUPO: [Receta; 5] = [
    Receta {
        id: "leer_resultados",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Leer resultados",
            en: "Read results",
        },
        cuando: crate::idioma::Texto {
            es: "Ya hay una simulación hecha.",
            en: "A simulation already exists.",
        },
        tools: &["resultados kpis", "resultados perdidas", "resultados mensual"],
        ejemplo: None,
        nota: None,
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: crate::idioma::Texto {
            es: "Lee los resultados de {proyecto} variante {variante} con resultados kpis, perdidas y mensual, y explica las tres pérdidas mayores.",
            en: "Read the results of {proyecto} variant {variante} with resultados kpis, perdidas and mensual, and explain the three largest losses.",
        },
    },
    Receta {
        id: "operar_gui",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Operar la GUI (funciones solo-GUI)",
            en: "Operate the GUI (GUI-only features)",
        },
        cuando: crate::idioma::Texto {
            es: "Editor 3D, gráficos I/V, tablas, asistentes, diálogos.",
            en: "3D editor, I/V curves, tables, wizards, dialogs.",
        },
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
        nota: Some(crate::idioma::Texto {
            es: "Para formularios con esquema (que:esquema) rellena con accion:rellenar entidad+valores y lee con que:vista: claves estables, sin OCR ni coordenadas. accion:'escena' (escena+objetos) con simular:true valida un pedido de escena 3D sin tocar PVsyst y sin simular la crea en el editor 3D abierto (vacío) y compara el .SHD que guardó PVsyst con lo pedido; accion:'escena_leer' (escena) exporta y lee la escena abierta; accion:'escena_importar' (archivo+escena) importa una malla 3DS/DAE/PVC/PVC2/PVZ2 con el importador de PVsyst; accion:'menu' admite subitem para las entradas de un submenú; accion:'clic_fisico' mueve el cursor real; accion:'fila' (ventana=TVirtualStringTree o TListBox, indice) selecciona esa fila con un clic físico y falla si la banda resaltada no es la pedida o si el clic abrió el elemento; accion:'clic_texto' y accion:'menu' (menu+item) no. Toda respuesta (tambien las de error) trae vista{estado,resumen,siguiente,error}: sigue vista.siguiente tal cual. Con un dialogo modal abierto no lances comandos de menu; tras un fallo de PVsyst usa accion:'reiniciar'.",
            en: "For forms with a schema (que:esquema) fill with accion:rellenar entidad+valores and read with que:vista: stable keys, no OCR or coordinates. accion:'escena' (escena+objetos) with simular:true validates a 3D scene request without touching PVsyst, and with simular it builds it in the open (empty) 3D editor and compares the .SHD PVsyst saved with what was requested; accion:'escena_leer' (escena) exports and reads the open scene; accion:'escena_importar' (archivo+escena) imports a 3DS/DAE/PVC/PVC2/PVZ2 mesh with PVsyst's importer; accion:'menu' accepts subitem for the entries of a submenu; accion:'clic_fisico' moves the real cursor; accion:'fila' (ventana=TVirtualStringTree or TListBox, indice) selects that row with a physical click and fails if the highlighted band is not the requested one or if the click opened the item; accion:'clic_texto' and accion:'menu' (menu+item) do not. Every response (error ones too) carries vista{estado,resumen,siguiente,error}: follow vista.siguiente as-is. With a modal dialog open do not launch menu commands; after a PVsyst crash use accion:'reiniciar'.",
        }),
        argumentos: &[],
        plantilla: crate::idioma::Texto {
            es: "Opera la GUI de PVsyst: lanza con gui_actuar accion:lanzar, abre la función con accion:funcion (resuelve sus prerrequisitos), rellena con accion:rellenar y lee el resultado con gui_observar que:vista. Al terminar, cierra las ventanas con accion:cerrar.",
            en: "Operate the PVsyst GUI: launch with gui_actuar accion:lanzar, open the feature with accion:funcion (it resolves its prerequisites), fill with accion:rellenar and read the outcome with gui_observar que:vista. When done, close the windows with accion:cerrar.",
        },
    },
    Receta {
        id: "explorar_archivo",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Explorar/editar cualquier archivo",
            en: "Explore/edit any file",
        },
        cuando: crate::idioma::Texto {
            es: "Ajustes finos no cubiertos por tools de dominio.",
            en: "Fine adjustments not covered by the domain tools.",
        },
        tools: &["leer", "listar que:objetos", "editar campos", "listar que:archivos"],
        ejemplo: None,
        nota: None,
        argumentos: &[ArgumentoReceta {
            nombre: "archivo",
            descripcion: crate::idioma::Texto {
            es: "Ruta o nombre del archivo",
            en: "File path or name",
        },
            requerido: Requerido::Si,
            defecto: "",
        }],
        plantilla: crate::idioma::Texto {
            es: "Lee {archivo} con leer, lista sus objetos con listar que:objetos (si se trunca, repite con detalle:completo) y propón los cambios con editar antes de aplicarlos.",
            en: "Read {archivo} with leer, list its objects with listar que:objetos (if truncated, repeat with detalle:completo) and propose the changes with editar before applying them.",
        },
    },
    Receta {
        id: "diagnostico_entorno",
        requiere_aproximaciones: false,
        nombre: crate::idioma::Texto {
            es: "Diagnostico del entorno",
            en: "Environment diagnostics",
        },
        cuando: crate::idioma::Texto {
            es: "Antes de empezar o si algo falla.",
            en: "Before starting or if something fails.",
        },
        tools: &["entorno diagnostico", "entorno licencia"],
        ejemplo: None,
        nota: None,
        argumentos: &[],
        plantilla: crate::idioma::Texto {
            es: "Ejecuta entorno accion:diagnostico y explica cualquier problema de instalación, licencia o rutas.",
            en: "Run entorno accion:diagnostico and explain any installation, licence or path problems.",
        },
    },
    Receta {
        id: "comparar_medido",
        requiere_aproximaciones: true,
        nombre: crate::idioma::Texto {
            es: "Comparar medido vs simulado",
            en: "Compare measured vs simulated",
        },
        cuando: crate::idioma::Texto {
            es: "Hay una serie mensual medida y un informe PDF de PVsyst.",
            en: "There is a measured monthly series and a PVsyst PDF report.",
        },
        tools: &["resultados medido_vs_simulado"],
        ejemplo: None,
        nota: Some(crate::idioma::Texto {
            es: "Aproximación del MCP (requiere PVSYST_MCP_APROXIMACIONES=1); el equivalente de PVsyst es su herramienta de datos medidos en la GUI. La serie medida puede ir como 12 valores kWh o como CSV 'mes;energia_kwh'.",
            en: "MCP approximation (requires PVSYST_MCP_APROXIMACIONES=1); the PVsyst equivalent is its measured-data tool in the GUI. The measured series can be given as 12 kWh values or as a CSV 'mes;energia_kwh'.",
        }),
        argumentos: &PROYECTO_Y_VARIANTE,
        plantilla: crate::idioma::Texto {
            es: "Compara la energía mensual medida de {proyecto} variante {variante} con la simulada usando resultados que:medido_vs_simulado; pásame la serie medida en kWh o un CSV y explica los meses fuera del umbral.",
            en: "Compare the measured monthly energy of {proyecto} variant {variante} with the simulated one using resultados que:medido_vs_simulado; pass me the measured series in kWh or a CSV and explain the months outside the threshold.",
        },
    },
];
