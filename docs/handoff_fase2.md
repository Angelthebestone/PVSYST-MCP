# Handoff: fase 2 (v0.2.0) de pvsyst-mcp

Fecha: 2026-10-20. Rama `main`. La fase 1 sigue publicada en `v0.1.1`.

## Estado: fase 2 implementada, verificada y publicada (2.1–2.13)

| Paso | Commit | Resumen |
|---|---|---|
| 2.1 | `e6b5bac` | `declaracion.rs` partido; validación en `server/tools/validacion.rs`. |
| 2.2 | `62384d7` | Títulos, anotaciones MCP y descripciones ES/EN en las 15 tools y en el `.mcpb`. |
| 2.3 | `40cf490` | `prompts/list` y `prompts/get` sobre las siete recetas; `pvsyst://recetas` intacto. |
| 2.4 | `0b87423` | Protocolo 2025-06-18, `structuredContent`, progreso, cancelación y bucle con un trabajador. |
| 2.5 | `7937751` | `MissingArg` / `NotFound` / `Io` / `Cli` en los sitios del plan. |
| 2.6 | `e64f60c` | Parámetros booleanos sustituidos por enums de dos variantes. |
| 2.7 | `191c9b3` | Un solo calendario gregoriano en `src/calendario/`. |
| 2.8 | `7135b10` | Fixture `.PAN` legacy Real48 sintético y test E2E no ignorado. |
| 2.9 | `c861b43` | Proptests de inserción/eliminación y restauración byte a byte. |
| 2.10 | `dd2254e` | Crate solo-Windows; `rust-version = "1.88"` verificado con GNU. |
| 2.11 | `5808a86` | `unsafe` 76 → 40; FFI en `gui/ffi/` y COM en `uia/com.rs`. |
| 2.12 | `432a5d8` | `leer_bytes` único, `ensure_template`, auditoría de docs y `tracing-subscriber` recortado. |
| 2.13 | `f900689`, tag `v0.2.0` | Release GitHub con zip, `.mcpb` y `.sha256`; `pvsyst-mcp@0.2.0` en npm (`latest`). |

Verificado antes de publicar: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`,
`cargo test` (303 unit + integración), `cargo +1.88-x86_64-pc-windows-gnu check --all-targets`,
`cargo audit`, `cargo deny check`, `npm test`, `npm audit --audit-level=high`, 40 bloques
`unsafe {` con `SAFETY`, fixture `.PAN` y proptests. El tarball publicado se instaló desde el
registro y respondió `serverInfo.version = "0.2.0"` y `protocolVersion = "2025-06-18"`.
Los tests `#[ignore]` que necesitan una instalación real de PVsyst, red o la GUI no se
ejecutaron en este entorno. La prueba manual de GUI y la de Claude Desktop/.mcpb quedan
pendientes de una sesión con PVsyst abierto.

Release: <https://github.com/Angelthebestone/PVSYST-MCP/releases/tag/v0.2.0>.
npm: <https://www.npmjs.com/package/pvsyst-mcp/v/0.2.0>.

## Estado histórico: fase 1 (v0.1.1) terminada y publicada

| Paso | Commit | Resumen |
|---|---|---|
| 1.1 | `5c23d9f` | `config::ruta_de_entorno` / `valor_de_entorno_util` (puro, testeado en `tests/unit/config.rs`); `config::local_app_data()` devuelve `Error::NotFound`; `mcpb/manifest.json` con rutas opcionales. |
| 1.2 | `5f8bf10` | `admin::importacion::error_de_publicacion` nombra el respaldo si no puede devolverlo. |
| 1.3 | `9e1c9e1` | `admin::archive`: `LimitesListado`, `listado_dentro_de_limites` (try_fold), `listar` en streaming con kill, tope de 4 GiB. |
| 1.4 | `3692abf` | `components::pan::DatosModulo` (`pan/datos_modulo.rs`), `mu_voc` en PAN de texto (`muVocSpec`/`muVoc`), doctor y `easy_mode/dimensionado.rs` lo usan; `kwp` numérico en `caso.rs`. |
| 1.5 | `14e94cd` | `entregable` escribe con `history::reemplazar` y devuelve `historial_id`. |
| 1.6 | `6b07530`, `2ad183b` | Auto-actualización npm (`npm/actualizacion/**`, launcher, CI con node 20, `package-lock.json`, READMEs, CHANGELOG). |
| 1.7 | `b4fe7d1`, tag `v0.1.1` | Release GitHub con zip, `.mcpb` y `.sha256`; `pvsyst-mcp@0.1.1` en npm (`latest`). |

Verificado: `cargo fmt/clippy/test` en verde (295 unit + integración), `cargo audit` y `cargo deny check` ok, `npm test` 22/22 (repetido 10 veces sin fallos), `objdump -p` sin DLLs de MinGW, `PVSYST_WORKSPACE=""` autodetecta `C:\Users\Lenovo\PVsyst8.1_Data`, `npx -y pvsyst-mcp@0.1.1` responde `serverInfo.version 0.1.1` y el actualizador registra `al_dia`, y una instalación real desde el registro (`enUso=0.0.9`) termina en `actualizada` con `versiones/0.1.0/.completa`.

No se hizo: publicar un 0.1.2 de prueba (el plan lo condiciona a tu aprobación). La ruta real de npm quedó probada instalando 0.1.0 con `enUso=0.0.9`.

## Defecto publicado en 0.1.1 (corregido en `main`, sin release)

- **Zips con nombres acentuados.** El listado en streaming de 1.3 usaba `BufReader::lines()`, que exige UTF-8. `tar.exe` imprime los nombres en la página de códigos de la consola (`Bogotá` sale como el byte 0xE1), así que la 0.1.1 rechaza al importar cualquier zip con un nombre acentuado. Lo reproduje con `tar -tf` y lo corregí en `9815188` (`admin::archive::lineas_de_listado` decodifica cada línea con pérdida y tiene tests con bytes reales y con un zip real). Hay que publicarlo en la próxima release: un 0.1.2 aparte si corre prisa, o dentro de la 0.2.0, con entrada en el CHANGELOG.
- `b3a9adb` añade un test con números reales para la conversión de μVoc: 49,5 V con −140 mV/°C a −10 °C dan 54,4 V. No cambia el comportamiento.
- Comprobado: el `.mcpb` de 0.1.1 contiene `manifest.json` y `server/pvsyst-mcp.exe` en la raíz, sin prefijos `..\`.
- **2.1–2.12 cerrados** en los commits de la tabla superior. El atajo de 2.1 se aplicó tal cual.

## Desviaciones del plan y aprendizajes (leer antes de seguir)

- **Fórmula de μVoc corregida.** El doctor multiplicaba `mu_voc_mv_c/1000` (V/K absoluto) como si fuera relativo. Ahora se usa `mu/1000/Voc` (1/K). Antes no se notaba porque la clave nunca se leía.
- **`npm view` sin red da `error`, no `al_dia`.** Se siguió la contingencia del plan («queda registrado») en lugar del paso 4.
- **Windows bloquea los archivos recién creados.** El antivirus retiene el `.exe` recién instalado y `rename` falla con EPERM durante unos segundos. Por eso `publicarVersion` reintenta (10 intentos, pausa lineal de 500 ms). Además el cerrojo se retira renombrándolo antes de borrarlo (evita «delete pending»), y `liberar()` nunca lanza.
- **`spawn` con `shell:true` y array de argumentos** está deprecado (DEP0190). `npm.js` pasa un solo string. En timeout mata el árbol con `taskkill /T /F` y stderr va a `ignore`.
- **El `mtime` de un archivo va hasta ~5 ms por delante de `Date.now()`** en esta máquina: no hagas tests de caducidad con márgenes de 1 ms.
- **npm publica de forma asíncrona.** `PUT 202`: la versión tarda 1-2 minutos en aparecer en `npm view`.
- **El shell de `bash` aquí no es PowerShell.** Es POSIX (grep/sed/cat). `Select-String` no existe.
- **Commits por paso.** Si se tocan varios pasos a la vez, para separarlos se recorta el archivo de tests, se commitea y se restaura (se hizo así en 1.2/1.3).
- **Empaquetado de releases.** Se hizo con un script Python ad-hoc que replica `.github/workflows/release.yml` en local: zip con exe + LICENSE, NOTICE, READMEs, CHANGELOG y `mcp.example.json`, más el `.mcpb` con `npx -y @anthropic-ai/mcpb@2 pack`, los `.sha256` y `notes.md` extraído del CHANGELOG. Los nombres siguen `pvsyst-mcp-vX.Y.Z-x86_64-pc-windows.{zip,mcpb}`. GitHub Actions sigue bloqueado por facturación, así que el push del tag no dispara el workflow; si se desbloquea, chocaría con `gh release create` manual.
- **Atajo para 2.1.** Ya se probó y se revirtió para cerrar limpio en la fase 1. Hay que mover las líneas de `validar` a `validar_tipos` de `src/server/tools/declaracion.rs` (153-282) a `src/server/tools/validacion.rs` como `impl Herramienta`, con `use super::declaracion::{con, nombre_tipo_recibido, Herramienta};` y `use serde_json::{json, Value};`, y añadir `pub(crate) mod validacion;` en `tools/mod.rs`.

## Cómo seguir

La fase 2 está cerrada y publicada. La fase 3 (v0.3.0) empieza en 3.1. Antes de cada commit: `cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test`. Se publica con el mismo procedimiento que 1.7, cambiando la versión.

> **Regla del usuario: ningún test dentro de `src/`.** Todo test (unitario, de integración o de propiedades) se escribe en archivos bajo `tests/`. En `src/` solo puede ir el enganche de una línea `#[cfg(test)] #[path = "../../tests/unit/…"] mod tests;` que ya usa el crate. Nunca bloques `#[cfg(test)] mod tests { … }` ni funciones `#[test]` en `src/`. Esta regla manda sobre cualquier paso del plan que diga otra cosa, incluido el «docstring con ejemplo que se ejecuta como doctest» de las convenciones: no añadas doctests nuevos en `src/` y cubre ese comportamiento con tests en `tests/`. Pendiente de confirmar con el usuario: los `*.test.js` de npm viven en `npm/actualizacion/**`, fuera de `src/`, así que se dejaron donde están.

## Convenciones obligatorias (aplican a todos los pasos)

- **Funcional.**
  - La lógica de dominio son funciones puras: reciben valores y devuelven valores nuevos (`&[u8] -> Result<Vec<u8>>`, `&Valor -> Resultado`).
  - Se usan `map`/`filter`/`fold`/`try_fold`/`scan`, `?` y los combinadores de `Result`/`Option`.
  - Prohibido `let mut` en código nuevo de dominio. Solo se admite como acumulador local dentro de un escáner de bytes cuando la versión con iteradores sea más compleja, con un comentario de una línea que diga por qué.
  - El IO (lectura, escritura con historial, procesos) vive solo en la capa de aplicación de cada feature (el handler de la tool o `aplicacion.rs`).
  - Los datos se modelan como value objects con constructor validador `fn nuevo(...) -> Result<Self>` y campos privados.
- **Estructura.** Cada feature nueva vive en su propia carpeta `src/<feature>/` con `mod.rs` (solo `mod` y re-exports), `dominio.rs` (puro, sin `crate::history`, `std::fs` ni `std::process`) y `aplicacion.rs` (orquesta IO usando el dominio). La presentación son las declaraciones de `src/server/tools/*.rs`. Prohibido crear carpetas `utils`, `helpers` o `common`.
- **Límites.**
  - Funciones: ≤50 líneas, ≤3 parámetros (si hacen falta más, un struct DTO), un solo nivel de indentación con guard clauses, complejidad cognitiva <15. `clippy.toml` ya lo hace cumplir.
  - Archivos: <300 líneas. Si un archivo que se toca pasa de 280, se parte **antes** de añadirle código.
  - Ningún parámetro `bool`: se usa un enum de dos variantes.
  - Sin números mágicos: todo literal con significado es una `const` con nombre.
- **Errores.** Siempre `crate::error::Error` con la variante adecuada: `MissingArg` mediante `args::missing`, `NotFound` cuando no existe un recurso, `Io` mediante `Error::io(contexto, e)` para fallos del sistema de archivos, e `Invalid` solo para valores con formato o rango incorrectos. Prohibidos `unwrap`, `expect` y `panic`.
- **Logs.** Solo `tracing` con campos estructurados (`tracing::info!(proyecto = %p, "…")`). Nada se escribe a stdout fuera del protocolo. En JavaScript, líneas JSON a stderr o a un archivo; nunca `console.log`.
- **Tests de comportamiento, Given-When-Then.** Los unitarios prueban solo funciones de dominio `pub(crate)` y se nombran `should_<resultado>_when_<condicion>`. Siguen el patrón actual: un archivo en `tests/unit/<ruta espejo>.rs` enganchado al final del módulo con `#[cfg(test)] #[path = "../../tests/unit/…"] mod tests;`. Los de integración son de extremo a extremo con `tests/common::Client` o con el `fake_pvsystcli`. Sin red ni `sleep` en los unitarios. Se cubren el caso feliz, el error y un caso límite.
  - El enganche `#[cfg(test)] #[path = "../../tests/unit/…"] mod tests;` es lo **único** relacionado con tests que se permite en `src/`; los cuerpos de los tests viven siempre en `tests/`. No cuenta como violación de la regla.
- **Docs.** Cada símbolo `pub` nuevo del crate lleva docstring, **sin doctests**: los ejemplos ejecutables van como tests en `tests/` (regla del usuario: ningún test en `src/`). Los comentarios explican el porqué.
- **Git.** Un commit Conventional Commits por paso (`feat:`, `fix:`, `refactor:`, `test:`, `chore:`), **sin trailers de coautoría**. Antes de cada commit tienen que pasar `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` y `cargo test`.


## Fase 2 — v0.2.0: protocolo MCP y deuda técnica

Orden: 2.1 → 2.2 → 2.3 → 2.4 son dependientes; 2.5 a 2.12 son independientes entre sí y van después de 2.1.

### 2.1 Partir `declaracion.rs` (295 líneas)
Mover `validar`, `validar_discriminador`, `validar_detalle`, `validar_requeridos_globales`, `validar_requeridos_por_accion`, `validar_tipos` y `preparar_args` a `src/server/tools/validacion.rs`, como `impl Herramienta` en ese archivo. Es un cambio sin efecto en el comportamiento: los tests existentes deben seguir en verde.

### 2.2 Anotaciones, títulos y descripciones bilingües
- En `declaracion.rs`, añadir:
  ```rust
  pub(crate) struct Anotaciones { pub(crate) titulo: &'static str, pub(crate) es_solo_lectura: bool, pub(crate) es_destructiva: bool, pub(crate) es_idempotente: bool, pub(crate) usa_red: bool }
  ```
  y el campo `pub(crate) anotaciones: Anotaciones` en `Herramienta`.
- `esquema()` emite `"title": titulo` y `"annotations": {"title", "readOnlyHint", "destructiveHint", "idempotentHint", "openWorldHint"}`.
- Valores en los 15 `const TOOL_*`:

| tool | titulo | RO | Destr | Idem | Red |
|---|---|---|---|---|---|
| entorno | Entorno / Environment | F | F | F | F |
| disenar_sistema | Diseñar sistema / Design system | F | T | F | F |
| listar | Listar / List | T | F | T | F |
| leer | Leer / Read | T | F | T | F |
| editar | Editar / Edit | F | T | F | F |
| escribir_archivo | Escribir archivo / Write file | F | T | T | F |
| proyecto | Proyecto / Project | F | T | F | F |
| componente | Componente / Component | F | T | F | F |
| sitio | Sitio / Site | F | T | F | T |
| simular | Simular / Simulate | F | F | F | F |
| resultados | Resultados / Results | F | F | T | T |
| workspace | Workspace | F | T | F | F |
| pvsyst_cli | PVsystCLI | F | T | F | F |
| gui_observar | Observar GUI / Observe GUI | T | F | T | F |
| gui_actuar | Actuar en GUI / Act on GUI | F | T | F | F |

- `descripcion` = `"<texto ES actual> / EN: <texto EN>"`. Textos en inglés:

| tool | EN |
|---|---|
| entorno | PVsyst environment: diagnostics, licence, logs and preferences. |
| disenar_sistema | End-to-end in one call: coordinates + kWp -> site -> project -> sizing -> simulation -> KPIs. |
| listar | Lists projects, components, modules, files or objects. |
| leer | Reads any PVsyst file: project, module, site, weather, component, object or raw bytes (detected from the file). |
| editar | Edits fields, inserts or removes PVObject objects (byte-preserving, with history). |
| escribir_archivo | Writes a file inside the workspace (with history). |
| proyecto | Project and variant lifecycle: create, delete, rename, size, diagnose, add objects. |
| componente | Component database: create, delete, import, export or create from a datasheet. |
| sitio | Sites and weather: create (coordinates or CSV), convert, solar geometry, PVGIS horizon and TMY. |
| simular | Simulation: run (optional rows/parameters), prepare input, status, pre-estimate and sweep. |
| resultados | Results: KPIs, CSV, losses, monthly, comparison, P50/P90, carbon, economics and deliverable. |
| workspace | Workspace: create, export, import, export project, change history and restore. |
| pvsyst_cli | Passthrough to any PVsystCLI command (expert mode). |
| gui_observar | Observes the PVsyst GUI: windows, controls, text, PNG capture, UIA tree (expert mode). |
| gui_actuar | Acts on the PVsyst GUI: click, keys, close, launch, UIA and waits (expert mode). |

- Copiar las mismas descripciones a `mcpb/manifest.json` → `tools[]`.
- Test de integración en `tests/protocol.rs`: `tools/list` devuelve `annotations.readOnlyHint == true` para `leer` y `destructiveHint == true` para `proyecto`, y toda descripción contiene `" / EN: "`.

### 2.3 Prompts desde las recetas
- Archivo nuevo `src/server/recetas.rs`:
  ```rust
  pub(crate) enum Requerido { Si, No }
  pub(crate) struct ArgumentoReceta { pub(crate) nombre: &'static str, pub(crate) descripcion: &'static str, pub(crate) requerido: Requerido, pub(crate) defecto: &'static str }
  pub(crate) struct Receta { pub(crate) id: &'static str, pub(crate) nombre: &'static str, pub(crate) cuando: &'static str, pub(crate) tools: &'static [&'static str], pub(crate) ejemplo: Option<&'static str>, pub(crate) nota: Option<&'static str>, pub(crate) argumentos: &'static [ArgumentoReceta], pub(crate) plantilla: &'static str }
  pub(crate) static RECETAS: &[Receta] = &[ … ];
  pub(crate) fn como_recurso() -> Value        // misma forma que hoy: {intro, recetas:[{nombre,cuando,tools,ejemplo?,nota?}], consejo}
  pub(crate) fn listar_prompts() -> Value      // {"prompts":[{name:id, title:nombre, description:cuando, arguments:[{name, description, required}]}]}
  pub(crate) fn obtener_prompt(params: &Value) -> Result<Value>  // {"description": cuando, "messages":[{"role":"user","content":{"type":"text","text": plantilla_rellena}}]}
  ```
- `ejemplo` es texto JSON que `como_recurso` convierte con `serde_json::from_str`.
- `obtener_prompt`:
  - Sin `name` → `args::missing("name", …)`.
  - Con un id desconocido → `Error::NotFound("Prompt desconocido: …")`.
  - Si falta un argumento `Requerido::Si` → `args::missing(arg, descripcion)`.
  - Cada `{arg}` se sustituye por el valor recibido (texto o `Value::to_string()` de un número) o, si es opcional y falta, por `defecto`. La sustitución es un `fold` sobre `argumentos`.
- `resources.rs::recetas()` se elimina y `read("pvsyst://recetas")` usa `recetas::como_recurso()`.
- `server/mod.rs`:
  - `prompts/list` → `listar_prompts()`.
  - Nuevo `prompts/get` → `obtener_prompt(params)`; un `Err` se convierte en JSON-RPC `-32602` con el mensaje.
  - `capabilities` añade `"prompts": {}`.
- Las siete recetas actuales con su `nombre`, `cuando`, `tools` y `ejemplo`/`nota` de hoy, más los ids, argumentos y plantillas siguientes (`*` = requerido):

| id | argumentos | plantilla |
|---|---|---|
| disenar_desde_coordenadas | latitud*, longitud*, kwp* | Diseña un sistema fotovoltaico de {kwp} kWp en latitud {latitud}, longitud {longitud} con la tool disenar_sistema. Después resume energía anual, producción específica y PR, y señala los avisos. |
| estudio_paso_a_paso | proyecto* | Haz un estudio paso a paso del proyecto {proyecto}: sitio crear, proyecto crear, editar campos, proyecto dimensionar, simular y resultados kpis. Confírmame cada fase antes de simular. |
| leer_resultados | proyecto*, variante (defecto VC0) | Lee los resultados de {proyecto} variante {variante} con resultados kpis, csv y perdidas, y explica las tres pérdidas mayores. |
| optimizar_orientacion | proyecto*, latitud*, longitud*, kwp* | Busca la mejor inclinación y azimut para {proyecto} ({kwp} kWp en {latitud}, {longitud}) con simular barrido y compara los mejores casos con resultados kpis. |
| operar_gui | — | Opera la GUI de PVsyst con gui_actuar y gui_observar (modo experto): lanza, enumera ventanas y controles, actúa y captura el resultado. |
| explorar_archivo | archivo* | Lee {archivo} con leer, lista sus objetos con listar que:objetos y propón los cambios con editar antes de aplicarlos. |
| diagnostico_entorno | — | Ejecuta entorno accion:diagnostico y explica cualquier problema de instalación, licencia o rutas. |

- Tests de integración en `tests/protocol.rs`:
  - `prompts/list` devuelve 7 prompts.
  - `prompts/get disenar_desde_coordenadas` con `{latitud:"7.07", longitud:"-73.84", kwp:"10"}` devuelve un texto que contiene `10 kWp` y `-73.84`.
  - Sin `kwp` devuelve el error `-32602`.
  - `resources/read pvsyst://recetas` sigue teniendo 7 recetas.

### 2.4 Progreso, cancelación y `structuredContent` (cambio de arquitectura del bucle)

**Versión del protocolo:**
- `server/mod.rs`: `PROTOCOL_VERSION = "2025-06-18"` y `SUPPORTED_PROTOCOL_VERSIONS = ["2024-11-05", "2025-03-26", "2025-06-18"]`.
- `Server` gana el campo `version: std::sync::OnceLock<&'static str>`, que `initialize_result` fija con la versión negociada; un segundo `initialize` no la cambia.
- Se actualiza `tests/protocol.rs:21`.

**`structuredContent`:**
- `resultado_de_herramienta(value, formato: FormatoResultado)` con `enum FormatoResultado { ConEstructura, SoloTexto }`. La variante es `ConEstructura` si la versión negociada es `"2025-06-18"`.
- Con estructura, y si `value` es un objeto, se añade `"structuredContent": value`. Se hace después de quitar `_imagen_png_base64`, y se mantiene el bloque de texto.
- Los errores con estructura añaden `"structuredContent": {"error": mensaje, "tipo": e.kind()}`.
- No se declara `outputSchema`, porque una misma tool devuelve formas distintas según la acción y el spec exige que el resultado cumpla el schema declarado.

**Seguimiento** (módulo nuevo `src/progreso/mod.rs`, neutral, sin dependencias del servidor):
```rust
pub(crate) struct SeguimientoTarea<'a> { cancelado: std::sync::Arc<std::sync::atomic::AtomicBool>, informar: Option<Box<dyn Fn(f64, &str) + Send + Sync + 'a>> }
impl<'a> SeguimientoTarea<'a> {
    pub(crate) fn inerte() -> SeguimientoTarea<'static>
    pub(crate) fn nuevo(cancelado: Arc<AtomicBool>, informar: Option<Box<dyn Fn(f64, &str) + Send + Sync + 'a>>) -> Self
    pub(crate) fn esta_cancelado(&self) -> bool       // Ordering::Relaxed
    pub(crate) fn informar_progreso(&self, porcentaje: f64, mensaje: &str)
}
```

**Ejecutor de las tools:**
- En `declaracion.rs`, `ejecutar` pasa a ser `ejecutar: Ejecutor`, con:
  ```rust
  pub(crate) enum Ejecutor { Simple(fn(&Paths, &Value) -> Result<Value>), Seguido(fn(&Paths, &Value, &SeguimientoTarea<'_>) -> Result<Value>) }
  ```
- Todos los `const TOOL_*` usan `Ejecutor::Simple(...)` salvo `TOOL_SIMULAR`, que usa `Ejecutor::Seguido(ejecutar_simular)`.
- `tools::call(paths, name, args)` se mantiene y delega en el nuevo `tools::call_seguido(paths, name, args, &SeguimientoTarea::inerte())`.

**Simulación:**
- `server/tools/simulacion.rs::ejecutar_simular(paths, args, seg)` pasa `seg` a `simulation::lanzar_simulacion_seguida(paths, args, seg)` (nueva) y a `preestimate::barrido(paths, args, seg)`.
- `lanzar_simulacion(paths, args)` queda como envoltorio con `inerte()` para sus otros llamadores (`easy_mode/simulacion.rs:196`).
- `ejecucion::ejecutar_cli` recibe `seg` y llama a `cli::run_seguido(paths, cli_args, timeout, seg)`. Este es nuevo; `cli::run` delega en él con `inerte()`.
- `ParametrosEjecucion` gana `seguimiento: &'a SeguimientoTarea<'a>`.
- `esperar_proceso(child, timeout, sink, seg)`, en cada vuelta de `INTERVALO_ESPERA_PROCESO`:
  1. Si `seg.esta_cancelado()`: `kill`, `wait` y `Err(Error::Cli("Simulación cancelada por el cliente".into()))`.
  2. Si no, lee el texto acumulado del sink de stdout con el mismo método que usa hoy `trabajo::estado_trabajo` para calcular `progreso`, aplica `trabajo::progreso(&texto)` y, si el porcentaje supera el último informado, llama a `seg.informar_progreso(p, "Simulando")`.
- Los trabajos en segundo plano siguen usando `inerte()`.

**Bucle** (`src/lib.rs::serve`), firma nueva:
```rust
pub fn serve<R: BufRead, W: Write + Send>(server: &Server, input: R, output: W) -> io::Result<()>
```
- Usa `std::thread::scope`:
  - La salida se envuelve en el nuevo `server::salida::Salida<W> { inner: Mutex<W> }`, con `fn escribir_linea(&self, linea: &str) -> io::Result<()>` que hace `writeln!` y `flush`, y convierte un mutex envenenado en `io::Error::other`.
  - Se lanza **un** hilo trabajador que consume `mpsc::Receiver<TrabajoHerramienta>` en orden. Las tools siguen serializadas, como hoy, para no introducir escrituras concurrentes sobre los mismos archivos.
  - El hilo principal lee stdin y clasifica cada línea con el nuevo `Server::clasificar(&self, linea: &str) -> Entrada`:
    ```rust
    pub(crate) enum Entrada { Herramienta { id: Option<Value>, params: Value }, Cancelacion { id_peticion: Value }, Inmediata(Option<String>) }
    ```
    - `tools/call` (con o sin id) → `Herramienta`.
    - `notifications/cancelled` → `Cancelacion` con `params.requestId`.
    - Todo lo demás → `Inmediata(self.handle_line(linea))`.
  - Las inmediatas se escriben al momento.
  - Para una `Herramienta`, se registra un `Arc<AtomicBool>` en el nuevo `server::cancelacion::Cancelaciones { activas: Mutex<HashMap<String, Arc<AtomicBool>>> }` con la clave `id.to_string()`, y se encola.
  - Para una `Cancelacion`, se marca la bandera si existe.
  - El trabajador:
    1. Construye `SeguimientoTarea::nuevo(bandera, informar)`. `informar` existe solo si `params._meta.progressToken` está presente (texto o número) y escribe `{"jsonrpc":"2.0","method":"notifications/progress","params":{"progressToken":tok,"progress":p,"total":100.0,"message":m}}`.
    2. Si la bandera ya está marcada antes de empezar, no ejecuta.
    3. Si no, llama a `server.ejecutar_herramienta(&params, &seg) -> Value`, que es el `call_tool` actual con el seguimiento.
    4. Escribe la respuesta solo si hay `id` y la bandera sigue sin marcar.
    5. Quita la entrada de `Cancelaciones`.
  - Al llegar a EOF se suelta el emisor, el trabajador termina lo que tenga encolado y `scope` retorna.
- `handle_line` sigue existiendo y es síncrono, con el comportamiento actual: lo usan los tests en proceso y las entradas inmediatas.
- `main.rs` pasa `io::BufReader::new(io::stdin())` e `io::stdout()`, porque `StdinLock` no es `Send`.
- El doctest existente de `serve` en `lib.rs` deja de compilar con la firma nueva: **elimínalo** (regla: sin tests en `src/`) y cubre el mismo caso con un test en `tests/protocol.rs` que use un `Cursor` de entrada y un `Vec<u8>` de salida.

**`fake_pvsystcli`:** si `FAKE_PVSYST_PROGRESO=1`, en `run-simulation` imprime `10%`, `50%` y `100%` (cada línea con `flush`), separadas por `FAKE_PVSYST_SLEEP_MS / 3` ms, antes de copiar los fixtures.

**Tests de integración** en `tests/cli_fake.rs`, con el patrón `Client::start_env` y `PVSYST_CLI=FAKE`:
- *Progreso*: con `FAKE_PVSYST_PROGRESO=1`, `FAKE_PVSYST_SLEEP_MS=600` y `simular` con `"_meta":{"progressToken":"t1"}`, se recibe al menos una `notifications/progress` con `progressToken == "t1"` antes de la respuesta del id.
- *Cancelación*: con `FAKE_PVSYST_SLEEP_MS=5000`:
  1. Se envía `simular` con id 7.
  2. Inmediatamente se envía `notifications/cancelled {requestId:7}` y luego `ping` con id 8.
  3. La siguiente línea recibida es la respuesta del id 8.
  4. Se envía `ping` con id 9 y la siguiente línea es el id 9, nunca el 7.
  5. Todo dura menos de 5 s.
- *Estructura*: tras `initialize` con `2025-06-18`, `tools/call listar` incluye `structuredContent`; con `2025-03-26` no lo incluye.

### 2.5 Variantes de error correctas
Cambiar exactamente estos sitios:
- **A `args::missing(clave, pista)`:**
  - `components/mod.rs:192` y `admin/componentes.rs:25` (`tipo`)
  - `projects/lifecycle/edicion.rs:31,35` (`objeto`, `campos`)
  - `projects/lifecycle/dimensionamiento.rs:82` (`n_mod_serie`)
  - `server/tools/validacion.rs` (antes `declaracion.rs:193,240,262`)
  - `simulation/engineering/supuestos.rs:30`
  - `sites/solar/mod.rs:146`
  - `server/tools/simulacion.rs:86` (`filas`, pista «o 'parametros'»)
- **A `Error::NotFound`:**
  - `projects/mod.rs:241`
  - `simulation/results/lookup.rs:214`
  - `projects/lifecycle/dimensionamiento.rs:50`
  - `components/ficha/modulo.rs:38`
  - `server/resources.rs:28` (el mapeo a `-32602` se conserva)
  - `server/tools/mod.rs:57`
  - `history/restauracion.rs:173`
  - `projects/anadir/catalogo.rs:88`
  - `projects/anadir/baterias.rs:9`
- **A `Error::io`:**
  - `easy_mode/proyecto.rs:29-31`
  - `history/restauracion.rs:42,83,197,274`
  - `workspace/rutas.rs:39,48`
- **A `Error::Cli`:** `simulation/gui.rs:46`.

Ajustar los tests que comprueban el texto exacto de esos mensajes.

### 2.6 Eliminar los parámetros booleanos
| función | cambio |
|---|---|
| `prefs::diagnostico::resumir_log(ruta, texto, solo_errores: bool)` | `filtro: FiltroLog`, con `enum FiltroLog { Todo, SoloErrores }` |
| `format::edit::field_line(indent, key, value, crlf: bool)` | `fin: FinDeLinea`, con `enum FinDeLinea { Lf, Crlf }` en `format/edit.rs`; `OpcionesBloque.crlf` pasa a `fin: FinDeLinea` |
| `easy_mode::correr_pasos(paths, params, reusable: bool)` y `easy_mode::sitio::ejecutar_paso_sitio(…, reutilizado: bool)` | `sitio: OrigenSitioDiseno`, con `enum OrigenSitioDiseno { Reutilizado, Nuevo }` en `easy_mode/pasos.rs` |
| `easy_mode::simulacion::actualizar_sitio_y_kpis(…, sim_ok: bool)` | `estado: EstadoSimulacion`, con `enum EstadoSimulacion { Correcta, Fallida }` |
| `adapters::cli::proceso::filtrar_cuerpo_stdout(stdout, success: bool)` | `estado: EstadoProceso`, con `enum EstadoProceso { Exito, Fallo }` |
| `projects::resumen_proyecto(ws, prj, include_variants: bool)` | `detalle: DetalleProyecto`, con `enum DetalleProyecto { ConVariantes, SinVariantes }` |
| `projects::lifecycle::dimensionamiento::validar_pnom(pnom, explicito: bool)` | `origen: OrigenPnom`, con `enum OrigenPnom { Argumento, Inferido }` |
| `easy_mode/diagnostico.rs:10`, closure `push(name, ok: bool, …)` | `estado: EstadoCheck`, con `enum EstadoCheck { Correcto, Fallido }` |
| `simulation/mod.rs:96,122`, variable local `open_gui` | usar `ModoGui` directamente |
| `tests/protocol.rs:34`, `tools_list_len(expert: bool)` | `enum Superficie { Normal, Experta }` local del test |

`ModoGui::de_bool` e `IncluirTexto::de_bool` se quedan: convierten el JSON en el borde.

### 2.7 Un solo calendario
- Módulo nuevo `src/calendario/mod.rs`, de dominio puro:
  - `pub(crate) struct FechaCivil { anio: i64, mes: u32, dia: u32 }` con getters.
  - `pub(crate) fn civil_desde_dias(dias: i64) -> FechaCivil`, con el algoritmo de Hinnant usando `div_euclid`.
  - `pub(crate) fn dias_desde_civil(f: FechaCivil) -> i64`.
  - `pub(crate) fn es_bisiesto(anio: i64) -> bool`.
  - `pub(crate) fn dias_del_mes(anio: i64, mes: u32) -> u32`.
- Sustituir:
  - `history/snapshot.rs:23 civil` (compone la fecha con `secs.div_euclid(86_400)` y el resto para h/m/s)
  - `simulation/batch/mod.rs:20 civil_from_days`
  - `sites/pvgis/normalizacion.rs:61,72`
  - `sites/solar/posicion.rs:18,23` (`dia_del_anio` usa el calendario)
- Borrar las copias.
- Tests: proptest de ida y vuelta `dias_desde_civil(civil_desde_dias(z)) == z` para `z ∈ -1_000_000..1_000_000`, `mes ∈ 1..=12` y `1 <= dia <= dias_del_mes`. Casos fijos: 0 → 1970-01-01, 11016 → 2000-02-29.

### 2.8 Fixture sintético de `.PAN` legacy
- `tools/pan_legacy_fixture.py` (con ruff y mypy --strict) añade `encode_real48(x: float) -> bytes`, que es la inversa exacta de `real48` en `tools/pan_decode.py`.
- Genera `tests/fixtures/workspace/ComposPV/PVModules/Synth_Legacy_300W.PAN` con la cabecera que exigen `parse_legacy_header` y `block1_start` en `src/components/pan/legacy.rs:127,170`. Hay que leer esas funciones para construir exactamente las líneas: primera línea `pvModule;…`, fabricante `SynthSolar`, el 3.er CR + 2 bytes antes del bloque, 25 slots y terminador `/`.
- Valores: slot 0 pnom 300.0, 8 isc 9.5, 10 voc 40.0, 12 impp 9.09, 13 vmpp 33.0; el resto 0.0.
- Se marca como binario en `.gitattributes` (`*.PAN binary` dentro de fixtures, si aún no lo cubre `-text`).
- Test Python: `decode_record(encode(...))` reproduce los valores.
- Test de integración nuevo, no ignorado, en `tests/mcp_smoke.rs`: `leer` sobre el fixture devuelve `format == "legacy-real48"`, `pnom ≈ 300` y `voc ≈ 40`. El test ignorado con el PAN real del demo se mantiene, porque prueba un archivo distinto.

### 2.9 Proptests pendientes (en `tests/unit/format/insert.rs` y `delete.rs`)
- Generador de documentos PVObject válidos: raíz con N hijos `pvX{i}`, cada uno con 0-5 campos `K=V` alfanuméricos y fin de línea CRLF.
- Propiedades:
  - `extraer_objeto(insertar_objeto(doc, raiz, bloque), "pvNuevo") == bloque` normalizado a la indentación.
  - `eliminar_objeto(insertar_objeto(doc, raiz, bloque), "pvNuevo") == doc`.
- Test de integración de historial en `tests/unit/history/tests.rs`: snapshot → escribir otros bytes → `restaurar` → los bytes son idénticos a los originales.

### 2.10 Crate solo para Windows
- En `src/lib.rs`: `#[cfg(not(windows))] compile_error!("pvsyst-mcp solo funciona en Windows: PVsyst 8.1.7 es una aplicación Windows.");`
- En `Cargo.toml`: `rust-version = "1.85"`. Antes, comprobar con `cargo +1.85 check`; si falla, poner la versión mínima que compile según `cargo msrv` o la que indique el error de `edition`/`let-else`.
- Los `#[cfg(windows)]` existentes se mantienen.

### 2.11 Menos bloques `unsafe`
- Regla: cada función extern o método COM que se llama desde ≥2 sitios recibe un único envoltorio seguro, con su `// SAFETY:`. Los de GUI van en `src/adapters/gui/ffi.rs` (si pasa de 280 líneas, se parte en `ffi/ventanas.rs`, `ffi/gdi.rs` y `ffi/entrada.rs`) y los de UIA en el nuevo `src/adapters/uia/com.rs`.
- Los llamadores quedan sin `unsafe`.
- Aceptación: `grep -c "unsafe {"` baja de 76 a ≤40, los tests y las pruebas manuales de GUI (abajo) siguen funcionando, y sin cambios de comportamiento.

### 2.12 Limpieza de duplicados y auditoría de textos
- Las tres copias de `leer_bytes` (`projects::anadir`, `projects::doctor`, `simulation::engineering::caso`) pasan a una sola `pub(crate) fn leer_bytes(ruta: &Path) -> Result<Vec<u8>>` en `src/workspace/lectura.rs`, con `Error::io`.
- `easy_mode::proyecto::ensure_template` usa `config::proyectos_demo_ro()`.
- Auditoría de documentación: para cada archivo de la lista del punto 2 de `docs/handoff.md`, ampliada a las rutas actuales (`src/admin/*`, `easy_mode/*`, `history/*`, `objects/*`, `prefs/*`, `workspace/*`, `adapters/**`, `components/**`, `format/{edit,insert,object}.rs`, `projects/**`, `server/{resources.rs,tools/*}`, `simulation/**`, `sites/**`), leer el `//!` y los docstrings y corregir el texto cortado o duplicado.
  - Añadir un `//!` a `simulation/engineering/{comparacion,caso,entregable}.rs`, `simulation/cache/key.rs`, `simulation/preparacion.rs` y `simulation/report_pdf/parse.rs`.
  - Corregir la errata «dorkspace» en `server/resources.rs:1`.
  - Los 5 mensajes reconstruidos (`caso.rs:117`, `incertidumbre.rs:54`, `carbono.rs:31`, `report_pdf/parse.rs:176`, `uia_vista.rs:205`) se dejan como están.
- En `Cargo.toml`: `tracing-subscriber = { version = "0.3.23", default-features = false, features = ["env-filter", "fmt", "std"] }`. Si no compila o cambia la salida de los logs, se revierte.

### 2.13 Publicar v0.2.0
Mismo procedimiento que 1.7, con la versión 0.2.0.

## Fase 3 — v0.3.0: funciones de dominio

Infraestructura compartida primero (3.1, 3.2). Después, cada feature es independiente.

### 3.1 Primitivas de formato nuevas (`src/format`, puro)

**`format/edit.rs`:**
```rust
pub(crate) struct EdicionObjeto { pub(crate) selector: String, pub(crate) cambios: Vec<(String, Option<String>)> }
pub(crate) fn editar_objetos(bytes: &[u8], ediciones: &[EdicionObjeto]) -> Result<(Vec<u8>, Vec<ChangeOutcome>)>
```
`editar_objetos` hace `try_fold` sobre `edit_fields`. `lifecycle::editar_variante` lo reutiliza.

**`format/insert.rs`:** `pub(crate) fn reemplazar_objeto(destino: &[u8], selector: &str, bloque: &[u8]) -> Result<Vec<u8>>`.
- Localiza el span con `find_span`; si no existe, `Error::NotFound`.
- Reindenta `bloque` a la indentación del span y conserva CRLF reutilizando `preparar_bloque`/`OpcionesBloque`.
- Sustituye las líneas `open..=close`.
- Valida el resultado con `object::validar_estructura`.

**Archivo nuevo `format/seccion.rs`**, para bloques `Clave=TTipo … End of TTipo` que no son `PVObject_` (`EconomicEval=TEcoEvaluation`, `WaterUse=TWaterUse`):
```rust
pub(crate) fn editar_campos_seccion(bytes: &[u8], clave: &str, cambios: &[(String, Option<String>)]) -> Result<(Vec<u8>, Vec<ChangeOutcome>)>
pub(crate) fn existe_seccion(bytes: &[u8], clave: &str) -> bool
pub(crate) fn campo_seccion(bytes: &[u8], clave: &str, campo: &str) -> Option<String>
```
- Apertura: la primera línea cuyo `trim()` empieza por `"{clave}=T"`. El tipo es el resto tras `=`.
- Cierre: la primera línea posterior con `trim() == "End of {tipo}"` y la misma indentación que la apertura.
- Campos directos: las líneas con indentación igual a la de apertura + 2.
- La edición reemplaza **todas** las líneas directas con esa clave, porque `DiscountRate` aparece dos veces. Si no hay ninguna, inserta una línea antes del cierre con `field_line`. `None` borra.
- Si no se encuentra la sección → `Error::NotFound`.

Tests unitarios con texto sintético: la clave duplicada se reemplaza en las dos líneas, se inserta la que falta, se respeta CRLF y una sección ausente da error.

### 3.2 Guardar variante y aplicar elementos del catálogo
- En `projects/lifecycle/variantes.rs`: `pub(crate) fn guardar_variante(v: &VarianteAbierta, nuevo: &[u8], motivo: &str) -> Result<Option<String>>`, que es `history::reemplazar(&v.workspace, &v.vpath, nuevo, motivo)`. Se re-exporta junto con `abrir_variante`.
- En `projects/anadir/catalogo.rs`, extraer de `anadir()` la función `pub(crate) fn aplicar_elemento(bytes: Vec<u8>, nombre: &str) -> Result<ElementoAplicado>`, que hace los pasos de pendientes → insertar → `ajustar_sistema` → `verificar_tipo`, con `pub(crate) struct ElementoAplicado { pub(crate) bytes: Vec<u8>, pub(crate) insertados: Vec<Value> }`. `anadir()` la llama y escribe.
- Helper de dominio en `src/projects/tipo_sistema.rs`:
  - `pub(crate) enum TipoSistema { Red, Aislado, Bombeo, RedDc }` con `fn de_variante(bytes: &[u8]) -> Result<TipoSistema>`, que lee `pvSystem.SystemType` mediante `object::parse` (`Grid`/`Battery`/`Pumping`/`DCGrid`); un valor desconocido → `Error::Format`.
  - `fn exigir(bytes, esperado) -> Result<()>`, que falla con `Error::Invalid("La variante es de tipo {real}; esta acción requiere {esperado}")`.

### 3.3 Seguidores de un eje (`src/seguidores/`)
**Dominio:**
```rust
pub(crate) struct ParametrosSeguidor { azimut_eje: f64, angulo_max: f64, pitch_m: f64, ancho_colector_m: f64, n_filas: u32, backtracking: Backtracking }
pub(crate) enum Backtracking { Activo, Inactivo }
pub(crate) fn nuevo(args) -> Result<ParametrosSeguidor>   // vía struct DTO de entrada
pub(crate) fn cambios_orientacion(p: &ParametrosSeguidor, no_orient: &str) -> Vec<(String, Option<String>)>
```
- Constantes por defecto (de `_DEMO_Tracking.VC0`): `AZIMUT_EJE_DEFECTO = 0.0`, `ANGULO_MAX_DEFECTO = 60.0`, `PITCH_DEFECTO_M = 8.0`, `ANCHO_COLECTOR_DEFECTO_M = 4.25`, `N_FILAS_DEFECTO = 62`, backtracking activo.
- Validación (`Error::Invalid`): `-180 ≤ azimut ≤ 180`, `0 < angulo_max ≤ 90`, `pitch > ancho > 0`, `n_filas ≥ 1`.
- `cambios_orientacion` produce: `NoOrient` (el valor original), `AxisAzim`, `MinPhi = -angulo`, `MaxPhi`, `UsesBacktracking` (`True`/`False`), `Pitch`, `CollectorWidth` (con 3 decimales), `NSheds`.

**Aplicación**, `convertir_a_seguidor(paths, args)`:
1. `abrir_variante`.
2. `TipoSistema::exigir(Red)`.
3. Exige una sola orientación: la línea `Orientations, list of=1` y exactamente un span `pvOrient`. Si hay más de una → `Error::Invalid("Solo se convierten variantes con una orientación")`.
4. Rechaza variantes con escena 3D (si `pvShading` contiene `ListeObjets`) con: «La variante tiene escena 3D de sombras; los seguidores 3D se crean desde plantilla: proyecto crear plantilla:_DEMO_Tracking variante:VC2 (o _DEMO_Utility VC7-VCD)».
5. Rechaza `pvBiFacialSheds` con «Quita el modelo bifacial de mesas antes de convertir (o usa la plantilla _DEMO_Utility.VC6)».
6. Lee el `NoOrient` original.
7. `reemplazar_objeto(bytes, "pvOrient", pvOrient de _DEMO_Tracking.VC0)`.
8. `reemplazar_objeto(.., "pvShading", pvShading de _DEMO_Tracking.VC0)`, que es una escena vacía.
9. `editar_objetos` con `[EdicionObjeto{ selector:"pvOrient", cambios }]`.
10. `guardar_variante(.., "proyecto seguidor")`.

La plantilla se lee desde `config::proyectos_demo_ro().join("_DEMO_Tracking.VC0")` con `extraer_objeto`.

**Respuesta:** `{proyecto, variante, archivo, tipo:"UnlimTracking", parametros, historial_id}`.

**Tool:** `TOOL_PROYECTO` añade la acción `"seguidor"`; `requeridos_por_accion: ("seguidor", &["proyecto"])`; propiedades `azimut_eje` (Number), `angulo_max` (Number), `pitch_m` (Number), `ancho_colector_m` (Number), `n_filas` (Integer), `backtracking` (Boolean). El argumento JSON `backtracking` se convierte en el enum en el borde.

**Doctor:** `projects/doctor/seguidores.rs::revisar_seguidor(var_txt, inf)`, llamado desde `revisar_variante`. Para un `FieldType` que empieza por `Track` o es `UnlimTracking`:
- error si `Pitch <= CollectorWidth`;
- error si `MinPhi >= MaxPhi`;
- aviso si `UsesBacktracking=True` y falta `Pitch` o `CollectorWidth`.

**Receta y prompt:** `convertir_a_seguidor`, con `proyecto*` y `variante` (defecto VC0). Plantilla: «Convierte {proyecto} variante {variante} a seguidor de un eje con proyecto accion:seguidor, ejecuta proyecto doctor y simula para comparar con la variante fija.»

### 3.4 Límite de inyección (`src/inyeccion/`)
- **Dominio:** `ParametrosInyeccion { potencia_kw: f64, perdida_separada: PerdidaSeparada, tan_phi: Option<f64> }` con `enum PerdidaSeparada { Si, No }` (defecto `No`). Validación: `potencia_kw > 0` y `-1 ≤ tan_phi ≤ 1`.
- `cambios(p)` produce `PLimGrid` (con 1 decimal), `LimGrSeparLoss` (`True`/`False`) y, si se indica, `TanPhi` (3 decimales).
- **Aplicación:** `abrir_variante` → `exigir(Red)` → `editar_objetos([pvSystem])` → `guardar_variante("proyecto limite_inyeccion")`.
- **Tool:** acción `"limite_inyeccion"`, que requiere `proyecto` y `potencia_kw`; propiedades `potencia_kw` (Number), `perdida_separada` (Boolean), `tan_phi` (Number). Para quitar el límite se usa `editar` con `PLimGrid: null`; esto se documenta en la descripción del schema del campo.

### 3.5 Almacenamiento con baterías (`src/almacenamiento/`)
- **Dominio:** `ParametrosAlmacenamiento { estrategia: Estrategia, bateria: String, n_serie: u32, n_paralelo: u32, p_max_carga_kw: f64, p_max_descarga_kw: f64, soc_inicial: f64 }` con `enum Estrategia { Autoconsumo, RecortePicos }`.
- Defectos: batería `Generic_LI_Module_180Ah.BTR`, `n_serie` 10, `n_paralelo` 20, `p_max_carga_kw` 380, `p_max_descarga_kw` 170, `soc_inicial` 0.6 (de `_DEMO_Commercial.VC4`).
- Validación: `soc ∈ (0,1)`, potencias > 0, enteros ≥ 1, nombre de batería terminado en `.BTR` y sin separadores de ruta.
- Mapeo de la estrategia: `Autoconsumo` → `Strategy=SelfConsumption`, `ChargingStrategy=PVAvailable`, `DischStrategy=Immediate`. `RecortePicos` → `PeakShaving`, `PVaboveThresh`, `Immediate`.
- `ediciones(p) -> Vec<EdicionObjeto>`:
  - `pvSystem`: `BatteryFile`, `NBattSerie`, `NBattParall`.
  - `pvGridStorage`: los tres campos de estrategia, `BattSoCInit`, `PMaxCharge`, `PMaxDisch` (3 decimales).
- PowerShift queda fuera porque necesita series `TParSimulSeries` sin un formato documentado.
- **Aplicación:**
  1. `abrir_variante` → `exigir(Red)`.
  2. Si no existe un span `pvGridStorage`, `aplicar_elemento(bytes, "baterias")`, que también inserta la carga.
  3. `editar_objetos(ediciones)`.
  4. Si es `RecortePicos` y `pvSystem` no tiene `PLimGrid` → `Error::Invalid("El recorte de picos usa PLimGrid como umbral: define antes proyecto accion:limite_inyeccion")`.
  5. Un único `guardar_variante("proyecto almacenamiento")`.
- La respuesta incluye el aviso «PVsyst debe conocer la batería {bateria}» cuando `components::resolve_component("BTR", …)` no la encuentra. No es error, porque las baterías genéricas viven en la base interna de PVsyst.
- **Tool:** acción `"almacenamiento"`, que requiere `proyecto` y `estrategia` (texto `autoconsumo` | `recorte_picos`), más las propiedades numéricas y `bateria`.
- **Receta/prompt:** `autoconsumo_con_baterias`, con `proyecto*` y `variante`.

### 3.6 Pérdidas de la variante (`src/perdidas/`)
Solo se tocan campos con semántica confirmada en los demos:

| arg | objeto.campo | conversión |
|---|---|---|
| `suciedad_pct` | `pvMainArray.SoilingLoss` | pct/100 con 3 decimales (fracción) |
| `uc_termico` | `pvMainArray.UFactor` | 1 decimal (W/m²K) |
| `lid_pct` | todos los `pvSubArray[i].LIDLoss` | 2 decimales (%) |
| `mismatch_pct` | todos los `pvSubArray[i].PMismMPP` | 2 decimales (%) |

- **Dominio:**
  - `ParametrosPerdidas`, con todos los campos `Option<f64>`. Si no llega ninguno → `args::missing("suciedad_pct", "o uc_termico, lid_pct, mismatch_pct")`.
  - Rangos: suciedad 0-50, uc 0-100, lid 0-20, mismatch 0-20.
  - `ediciones(p, n_subarrays: usize) -> Vec<EdicionObjeto>`. Genera los selectores `pvSubArray[i]` para `i in 0..n`. Confirmar con `_DEMO_Tracking.VC5` (2 campos) que `find_span` interpreta el índice sobre todas las apariciones del tipo; si no es así, generar el selector con el padre (`pvMainArray/pvSubArray[i]`).
- **Aplicación:**
  - Si la variante contiene la línea `Soiling=Soiling` (formato nuevo de 8.1.7) y llega `suciedad_pct` → `Error::Invalid("Formato nuevo de suciedad: edítalo con editar")`.
  - Si no, `n_subarrays` = número de spans `pvSubArray`, y luego `editar_objetos` y `guardar_variante("proyecto perdidas")`.
- **Tool:** acción `"perdidas"`, que requiere `proyecto`.

### 3.7 Bombeo (`src/bombeo/`)
- **Dominio:** `ParametrosBombeo { caudal_m3_dia: Option<f64>, profundidad_estatica_m: Option<f64>, bomba: Option<String> }`. Al menos uno es obligatorio. Caudal en (0, 10000], profundidad en (0, 1000], bomba terminada en `.PMP` y sin separadores.
- `ediciones` produce:
  - `pvSystem`: `PumpFile` y `StaticDepth = format!("-{p:.1}")`.
  - Cambios de sección `WaterUse`: `FlowModul=Yearly`, `FlowYear` (3 decimales) y `StaticDepth`, el mismo valor, porque la sección lo repite.
- **Aplicación:** `exigir(Bombeo)` → `editar_objetos` → `editar_campos_seccion(.., "WaterUse", ..)` → `guardar_variante("proyecto bombeo")`.
- **Tool:** acción `"bombeo"`, que requiere `proyecto`.
- Red DC y prediseño (`.PSG/.PSS/.PSP`) solo van por plantilla y receta, porque no hay archivos de muestra con los que aprender su formato. Optimizadores: no se implementan, porque PVsyst no expone campos de configuración en ningún demo.

### 3.8 Evaluación económica de PVsyst (`src/evaluacion_economica/`)
- **Dominio:** `ParametrosEconomicos { moneda: Option<String>, vida_anios: Option<u32>, tasa_descuento: Option<f64>, inflacion: Option<f64>, tarifa_venta_kwh: Option<f64> }`. Al menos uno es obligatorio.
- Validación: moneda de 3 letras mayúsculas, vida 1-100, tasas 0-1, tarifa 0-100.
- Mapeo: `Currency`, `ProjectLife`, `DiscountRate` (todas las apariciones, con 4 decimales), `InflationRate` (4 decimales), `SummerPeakSaleTariff` (5 decimales).
- **Aplicación:**
  1. Si `!existe_seccion(bytes, "EconomicEval")` → `Error::NotFound("La variante no tiene evaluación económica; créala una vez en la GUI de PVsyst (Economic evaluation) y luego ajústala con esta acción")`.
  2. Si llega `tarifa_venta_kwh` y `campo_seccion(.., "TariffType") != Some("taFixed")` → `Error::Invalid("Tarifa variable: edítala con editar")`.
  3. `editar_campos_seccion` → `guardar_variante("proyecto economia")`.
- **Tool:** acción `"economia"`, que requiere `proyecto`. `resultados economia` (el cálculo propio) no cambia.

### 3.9 Medido frente a simulado (`src/medicion/`)
- **Dominio:**
  ```rust
  pub(crate) struct SerieMensual([f64; 12]);   // nuevo(valores: &[f64]) -> Result: exactamente 12, finitos, >= 0
  pub(crate) struct FilaMes { mes: u8, simulado_kwh: f64, medido_kwh: f64, desviacion_pct: Option<f64> }
  pub(crate) struct ComparacionMensual { meses: Vec<FilaMes>, anual: FilaMes /* mes = 0 */, fuera_de_umbral: Vec<u8> }
  pub(crate) fn comparar(simulada: &SerieMensual, medida: &SerieMensual, umbral_pct: f64) -> ComparacionMensual
  pub(crate) fn serie_de_csv(texto: &str) -> Result<SerieMensual>
  ```
  - `desviacion = (medido - simulado) / simulado * 100`, con 2 decimales; `None` si `simulado == 0`.
  - `fuera_de_umbral`: los meses con `|desv| > umbral`.
  - `const UMBRAL_DEFECTO_PCT = 10.0`.
  - `serie_de_csv` acepta líneas `mes;energia_kwh`. La cabecera es opcional y se ignora si la segunda columna no es numérica. Acepta coma decimal y exige 12 filas.
- **Aplicación:**
  1. `medido_kwh` (array) o `medido_csv` (ruta confinada con `workspace::resolver_ruta` y `Alcance::de_args`). Si faltan los dos → `args::missing("medido_kwh", "o medido_csv")`.
  2. `engineering::caso(paths, args)` (hacerlo `pub(crate)` si no lo es) → `report_pdf::mensual(&caso.tokens)`.
  3. La columna cuyo nombre, ignorando mayúsculas, es `E_Grid`: unidad `MWh` → ×1000, `kWh` → ×1, cualquier otra → `Error::Format`.
  4. Si no hay columna → `Error::Format("El informe no tiene la columna E_Grid mensual")`.
  5. `comparar` y respuesta con `con_cabecera(&caso, json)`.
- **Tool:** `TOOL_RESULTADOS` añade `"medido_vs_simulado"`, con propiedades `medido_kwh` (Array), `medido_csv` (String) y `umbral_pct` (Number).
- **Receta/prompt:** `comparar_medido`, con `proyecto*` y `variante`.
- La decodificación de `.DAM` queda fuera, porque sus valores `Day_DDMM` usan una codificación no documentada.

### 3.10 Sistemas aislados en `disenar_sistema`
- Primero, partir `easy_mode/parametros.rs` (299 líneas): mover `parsear_f64_requerido`, `parsear_f64_opcional`, `parsear_f64_opcional_sin_defecto`, `parsear_u32_opcional` y `parsear_bool_opcional` a `easy_mode/lectura_argumentos.rs`.
- `DisenoParametros` gana `tipo_sistema: TipoDiseno` (`enum TipoDiseno { Red, Aislado }`, leído de `tipo_sistema` = `"red"` | `"aislado"`, defecto `Red`) y `variante: String`.
- Reglas de `plantilla` y `variante` (las constantes van en `easy_mode/parametros.rs`):
  - `Red`: el `plantilla` del argumento o `DEFAULT_PLANTILLA`; variante del argumento o `"VC0"`.
  - `Aislado`: el `plantilla` del argumento o `const PLANTILLA_AISLADO = "_DEMO_StandAlone"`; variante del argumento o `const VARIANTE_AISLADO = "VC3"`.
- Sustituir los `"VC0"` fijos de `easy_mode/dimensionado.rs:226`, `easy_mode/proyecto.rs:83` y `easy_mode/simulacion.rs:189`, y pasar la variante a `crear_proyecto` (que ya conserva el nombre de la variante) y a la lectura de `referencias_de_variante`.
- Con `Aislado`, `dimensionar_serie` ya devuelve `None` sin `GInverter`, y el criterio queda `ajuste_local`/`plantilla`; no hace falta lógica nueva.
- Argumentos opcionales nuevos para `Aislado`: `n_baterias_serie` y `n_baterias_paralelo`. Se aplican con `editar_objetos([pvSystem: NBattSerie, NBattParall])` después del dimensionado.
- **Tool:** `TOOL_DISENAR_SISTEMA` añade las propiedades `tipo_sistema` (String), `variante` (String), `n_baterias_serie` (Integer) y `n_baterias_paralelo` (Integer). La descripción ES añade «(red o aislado)» y la EN «(grid or stand-alone)».

### 3.11 Cobertura y recetas
- Actualizar `cobertura()` en `server/resources.rs` para que las áreas de seguidores, límite de inyección, almacenamiento, pérdidas, bombeo, economía, datos medidos y sistema aislado nombren las acciones nuevas.
- El test `cobertura_resource_lists_areas` ya comprueba que los nombres de tools existen.
- Añadir las recetas de 3.3, 3.5 y 3.9 a `RECETAS`; `prompts/list` pasa a devolver 10.

### 3.12 Publicar v0.3.0
Mismo procedimiento que 1.7, con la versión 0.3.0.

## Archivos críticos
- `src/lib.rs::serve` y `src/server/mod.rs`: el bucle nuevo con hilo trabajador, la clasificación, la versión negociada y `structuredContent` (2.4).
- `src/adapters/cli/proceso.rs::esperar_proceso`: el punto donde se comprueban la cancelación y el progreso (2.4).
- `src/server/tools/declaracion.rs`: `Herramienta`, `Ejecutor`, `Anotaciones`. Se parte primero (2.1).
- `src/format/{insert.rs,seccion.rs,edit.rs}`: las primitivas puras de las que depende toda la fase 3.
- `npm/bin/pvsyst-mcp.js` y `npm/actualizacion/aplicacion/actualizar.js`: la auto-actualización.

## Verificación
Directorio de trabajo: `C:\Users\Lenovo\Desktop\PVSYST MCP`. Antes de cada commit:

```powershell
cargo fmt --check; cargo clippy --all-targets -- -D warnings; cargo test
cargo test -- --ignored   # solo con PVsyst instalado; 2 de GUI requieren la GUI abierta
cargo audit; cargo deny check
cd npm; npm test; npm audit --audit-level=high; cd ..
```

**Fase 1:**
- `$env:PVSYST_WORKSPACE=""; target\release\pvsyst-mcp.exe` con `initialize` + `tools/call entorno` muestra en el diagnóstico el workspace autodetectado (`C:\Users\Lenovo\PVsyst8.1_Data`), no una ruta vacía.
- Auto-actualización real:
  1. Instalar `npm i -g pvsyst-mcp@0.1.0`. Ese launcher es antiguo y no se actualiza; por eso la prueba se hace con 0.1.1 contra 0.1.2.
  2. Publicar 0.1.1. Ejecutar `pvsyst-mcp` con `PVSYST_MCP_ALMACEN=%TEMP%\pmcp` y mandar `initialize`: responde `version 0.1.1`.
  3. Publicar un 0.1.2 de prueba solo si el usuario lo aprueba. Si no, apuntar `npm` a un registro local con `npm config set registry` o hacer la prueba de integración con puertos falsos.
  4. En el segundo arranque, `initialize` responde `0.1.2` y `%TEMP%\pmcp\versiones\0.1.2\.completa` existe.
  5. La primera sesión siguió respondiendo con 0.1.1 mientras se instalaba.
  6. `actualizacion.log` contiene `{"estado":"actualizada"}`.
- `PVSYST_MCP_AUTO_ACTUALIZAR=0`: el log no recibe líneas y no se crea staging.

**Fase 2:**
- Los tests de integración de 2.2, 2.3 y 2.4 pasan.
- Manual con Claude Desktop o con el `.mcpb`: `leer` no pide confirmación (`readOnlyHint`), los prompts aparecen en el menú, y una simulación larga muestra progreso y se puede cancelar.
- `cargo test` sigue sin red.
- GUI (2.11): con PVsyst abierto, `gui_observar que:ventanas` y `gui_actuar accion:cerrar` sobre el diálogo de Herramientas funcionan igual que antes.

**Fase 3** (cada feature con un test de integración sobre una copia en tempdir del demo correspondiente, `#[ignore]` porque necesita DataRO, y un unitario del dominio):
- `proyecto accion:seguidor proyecto:<copia de _DEMO_Commercial> variante:VC0 angulo_max:55` → el archivo tiene `FieldType=UnlimTracking`, `MinPhi=-55.0` y el mismo `NoOrient`, y `proyecto doctor` sale sin errores.
- `limite_inyeccion potencia_kw:300` → `PLimGrid=300.0`.
- `almacenamiento estrategia:recorte_picos` sin límite → error; después de `limite_inyeccion` → `Strategy=PeakShaving`.
- `perdidas lid_pct:1.5` sobre `_DEMO_Tracking.VC5` → los dos `pvSubArray` tienen `LIDLoss=1.50`.
- `bombeo caudal_m3_dia:25` sobre `_DEMO_Pumping.VC0` → `FlowYear=25.000` dentro de `WaterUse`.
- `economia tasa_descuento:0.03` sobre `_DEMO_Commercial.VC9` → las dos líneas tienen `DiscountRate=0.0300`; sobre `_DEMO_Commercial.VC0` → `NotFound`.
- `resultados medido_vs_simulado` con el fixture `tests/fixtures/results/MCP_E2E.pdf` y `medido_kwh` = la serie simulada ×1.2 → las 12 desviaciones valen 20.00 y `fuera_de_umbral` tiene 12 meses. Este test no se ignora, porque el fixture existe y no hace falta PVsyst.
- `disenar_sistema tipo_sistema:aislado latitud:7.07 longitud:-73.84 kwp:5 simular:false` → crea `Sistema_5kWp.VC3` con `SystemType=Battery`.
- **Simulación real** (gasta ejecuciones TRIAL; pedir confirmación al usuario antes de ejecutarla): `simular` sobre la variante convertida a seguidor y sobre la variante aislada; las dos producen CSV y PDF.

## Supuestos y contingencias
- **CI bloqueado.** GitHub Actions sigue bloqueado por facturación hasta que el usuario lo resuelva. Los releases se compilan en local (GNU) siguiendo 1.7.
- **Rechazo del CLI en 3.3 o 3.10.** Si PVsystCLI rechaza la simulación de una variante convertida a `UnlimTracking`, se prueba a copiar también los `pvShdFTable` de la plantilla. Si sigue fallando, la acción `seguidor` se publica como edición, con la nota «simular desde la GUI», y se informa al usuario. Mismo criterio para `aislado`: si el CLI no simula sistemas `Battery`, la respuesta de `disenar_sistema` devuelve el error del CLI tal cual y la receta lo indica.
- **Índices de selector.** Si `find_span` no admite `pvSubArray[i]` global (3.6), se usa la ruta con el padre. Si tampoco funciona, la acción solo edita el primer sub-array y lo dice en la respuesta.
- **`npm view` sin red.** En ese caso el estado es `error` y queda registrado; nunca bloquea el arranque.
- **`rust-version`.** Si `1.85` no compila, se pone la mínima que compile.
- **Publicación en npm.** Cada publicación requiere la sesión npm de `angelthebestone`, ya activa (`npm whoami`). Una versión publicada no se puede volver a publicar: si algo falla tras `npm publish`, se sube el número de parche.
