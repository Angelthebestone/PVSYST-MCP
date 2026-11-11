# Informe de idioma de la superficie MCP — 2026-11-09

- **Rama:** `worktree-tarea-d-fila` (PR #2, <https://github.com/Angelthebestone/PVSYST-MCP/pull/2>)
- **HEAD auditado:** `9e4c5e5` (2026-11-09)
- **Rango principal:** `2f105d3..HEAD` (65 commits). Se amplió a `81b4f59..HEAD` (323 commits) al confirmar el patrón.
- **Binario probado:** `target/debug/pvsyst-mcp.exe`, toolchain `stable-x86_64-pc-windows-gnu`, sin GUI de PVsyst y sin ejecuciones TRIAL.
- **Estado:** la tabla de §4 describe el estado **antes** de la corrección. Los commits que corrigen los hallazgos se listan en §11.

## 1. Qué entra en el alcance

El CHANGELOG (`## [Unreleased]`, «Configurable client-facing language») define el contrato así:

> `PVSYST_MCP_IDIOMA` (`es` o `en`, por defecto `en`) selecciona el idioma de **toda cadena visible por el cliente**: descripciones y títulos de tools, anotaciones de esquema, recetas (recurso y prompts), nombres de recursos `pvsyst://`, las `instructions` de `initialize` y los mensajes de ejecución (argumentos que faltan, tipos incorrectos, valores fuera de rango, conflictos de alias, tool o recurso desconocido, instrucciones de licencia, rechazos de workspace y avisos de aproximación). Los identificadores de protocolo (`accion`/`que`, nombres de argumentos, valores de `detalle`) no se traducen.

Entran, por tanto, también las cadenas que el CHANGELOG no enumera pero que el cliente lee: `vista.resumen`, `vista.avisos`, `vista.siguiente[].motivo`, `vista.error.causa_probable`, `vista.error.que_hacer`, `vista.recortado.pedir_resto`, los `aviso`/`mensaje` de las respuestas y los textos de rechazo de los adaptadores (CLI, GUI, workspace, proyectos, objetos, simulación).

Quedan fuera, y así se han tratado:

- **Logs a stderr** (`tracing`), que no son protocolo.
- **Texto propio de PVsyst**: `detected_error`, `stdout_tail`, `stderr_tail`, `avisos_pvsyst`, etiquetas de ventanas y menús que hay que pulsar tal cual (p. ej. `Ayuda > Acerca de PVsyst`).
- **Identificadores de protocolo**: `accion`, `que`, `detalle`, nombres de argumentos, claves de `vista.campos`, valores de `origen`/`estado`.
- **Cuerpos de recursos de referencia**: `pvsyst://formato` y `pvsyst://cli` son documentación incluida; `pvsyst://cobertura` y `pvsyst://gui` son datos de referencia. El CHANGELOG traduce los **nombres** de los recursos, no sus cuerpos. La nota nueva de `pvsyst://cobertura` está en inglés y es coherente con el resto del mapa.
- **Texto de arranque** (`src/lib.rs`, `src/config.rs`): se imprime antes de que exista una sesión MCP; no es una respuesta al cliente.

## 2. Método

1. Lectura de `agents.md`, `src/idioma.rs`, `src/error.rs`, `src/config.rs`, `src/server/**`, `src/vista_agente/**` y del CHANGELOG.
2. `git log --oneline 2f105d3..HEAD` y `git diff 2f105d3..HEAD -- src/`; extracción de literales nuevos con espacios y letras; `git blame` y `git log -S` para atribuir cada uno al commit que lo introdujo (no al refactor que lo movió).
3. Parseo estático de los 183 `Texto { es, en }` de `src/` para buscar pares idénticos o intercambiados.
4. Compilación y ejecución real del servidor por stdio con `PVSYST_MCP_IDIOMA=es` y `=en`, sobre una copia de `tests/fixtures/workspace`, con `PVSYST_CLI` apuntando a `target/debug/fake_pvsystcli.exe` y `FAKE_PVSYST_FIXTURES=tests/fixtures/cli`. Se hicieron `initialize`, `tools/list`, `prompts/list`, `resources/list`, `resources/read` y 46 `tools/call` (ver §9), más dos llamadas puntuales de `editar`/`sitio`. No se lanzó la GUI ni se gastó ninguna ejecución TRIAL; los `dry_run` y los rechazos no gastan.
5. Lectura de los tests que tocan `Idioma`/`idioma` para decidir qué regresión no detectan.

## 3. Resumen ejecutivo

El mecanismo funciona en la capa de servidor: descripciones y títulos de tools, anotaciones de esquema, recetas, prompts, nombres de recursos, `instructions`, argumentos que faltan, tipos incorrectos, valores fuera de rango, acciones inválidas, tool/recurso desconocido y el rechazo de `sobrescribir` se midieron correctamente en ambos idiomas.

No funciona en tres capas:

1. **El bloque `vista` es español en ambos idiomas.** El catálogo (`src/vista_agente/contrato/catalogo/`, ~112 entradas), los resúmenes de `construccion.rs`, las causas/remedios de `errores.rs` y los textos de `pendiente.rs` son `&'static str` en español, no `Texto`. Es el defecto de mayor impacto: **toda** respuesta correcta e incorrecta lo lleva. Viene de `4856acd` (2026-10-27), anterior al commit de idioma `7bcd971` (2026-11-06), que no lo cubrió.
2. **Los adaptadores y el dominio lanzan `Error::Invalid`/`NotFound`/`Format`/`Cli`/`Gui` con literales en un solo idioma.** Muchos son anteriores al rango, pero los commits de QA `a925953`, `6443711`, `030acb1`, `4d0e693`, `08e7d61`, `13bafb8`, `5be248b`, `90b1953`, `b36d8ac`, `d03928b`, `5efd172`, `f8bd6ee`, `682f96d`, `8dc8a28` añadieron mensajes nuevos por la misma vía.
3. **Las pruebas nuevas fijan el español.** `tests/mcp_regresiones/mensajes.rs` (commit `a925953`) afirma `"ya existe"` y `"se pidieron 1"` con el servidor en su idioma por defecto (`en`). No solo no detecta la regresión: la convierte en expectativa.

Recuento: 183 pares `Texto`; 3 idénticos (`simular ejecutar`, `PVsystCLI`, `Workspace`, nombres propios aceptables); 0 intercambiados por heurística de vocabulario; ~40 literales nuevos o movidos en el rango que no pasan por `Texto`; ~112 entradas de catálogo y ~50 mensajes de adaptador/dominio fuera del mecanismo.

## 4. Hallazgos

Convenciones de la tabla:

- **Commit**: el que introdujo el texto. Si un refactor posterior lo movió, se indica entre paréntesis.
- **Medido**: la cadena se obtuvo del servidor real por stdio. **Código**: no se pudo disparar sin GUI/PVsyst; se leyó en el fuente y se marca la limitación.
- **Severidad**: P1 = lo ve el cliente en un camino normal y en el idioma equivocado; P2 = rechazo/aviso menos frecuente o de una sola rama; P3 = interno, raro o inalcanzable por la superficie MCP.

| # | Archivo:línea | Commit | Cadena (representativa) | Con `es` | Con `en` | Sev | Arreglo recomendado |
|---|---|---|---|---|---|---|---|
| 1 | `src/vista_agente/contrato/catalogo/mod.rs:133` y todo `catalogo/*.rs` | `4856acd`; `6cb6e78` añadió «Ejecutar el lote preparado» | `resumen` de cada acción y `motivo` de cada paso: «Proyectos del workspace listados.», «Ver los archivos del workspace», «Ejecutar el lote preparado» | Español (medido en `listar`, `leer`, `workspace`, `pvsyst_cli`, `simular`) | **Español** (medido: misma cadena) | P1 | Cambiar `Entrada.resumen`, `Entrada.pista` y `Paso.motivo` a `Texto`; `construir`/`siguiente_de` reciben `Idioma` y llaman `en_idioma`. Actualizar las ~112 entradas. |
| 2 | `src/vista_agente/contrato/errores.rs:9-64,74` | `4856acd` | `vista.error.causa_probable` / `que_hacer`: «El archivo, proyecto o componente no existe o está fuera del workspace.», «Lista los candidatos con listar…»; `resumen`: «No se completó: …» | Español (medido en `leer`, `proyecto`, `componente`, `pvsyst_cli`) | **Español** (medido: misma cadena) | P1 | `CAUSAS` y `CAUSA_DESCONOCIDA` como `Texto`; `desde_error(..., idioma)`; `resumen` con `Texto`. |
| 3 | `src/vista_agente/contrato/pendiente.rs:14,63,65,75,77,96,97` | `4856acd` | «Ensayo, no se aplicó nada. Al aplicarlo: …», «Aplicar de verdad lo ensayado», «Confirmar el gasto de ejecuciones de prueba» | Español (medido con `pvsyst_cli dry_run:true`) | **Español** (medido: misma cadena) | P1 | `Texto` para `PRUEBA_CONFIRMAR`, resúmenes, causa y motivos; `Pendiente::resumen/error/siguiente` reciben `Idioma`. |
| 4 | `src/vista_agente/contrato/construccion.rs:26,139,156,157,163,166,266` | `4856acd`; `b67d1b3`/`b4fd568` añadieron dos | «La acción informó un fallo o problemas en su resultado…», «Corrige la causa indicada en motivo y avisos…», «Acción completada.», «leer avisos completos en la propia respuesta» | Español (medido en `pvsyst_cli` fallido) | **Español** (medido: misma cadena) | P1 | `Texto` para el fallback, causa, remedio, resumen y `PEDIR_AVISOS`; `construir(..., idioma)`. |
| 5 | `src/args.rs:169,175,191` | `a925953` (la rama de 169 es del commit inicial; 175/191 son nuevas) | «Proyecto 'NOPE' no encontrado en …; hay 1: SYNTH_Plant», «Archivo 'NOPE.xyz' no encontrado…» | Español (medido en `leer`, `proyecto doctor`, `componente editar`, `resultados`) | **Español** (medido: misma cadena) | P1 | `Texto` para las tres ramas; `no_encontrado(..., idioma)` y pasar `paths.idioma` desde los llamadores. |
| 6 | `src/projects/lifecycle/creacion.rs:118` | `a925953` | «'…\\SYNTH_Plant.PRJ' ya existe: pasa 'sobrescribir': true para reemplazarlo» | Español (medido con `proyecto crear` duplicado) | **Español** (medido: misma cadena) | P1 | `Texto` + `paths.idioma` (la función ya recibe `paths`). |
| 7 | `src/simulation/engineering/comparacion.rs:173,179` | `a925953` | «comparar necesita al menos 2 variantes: se pidieron 1 (VC0)», «Se necesitan al menos 2 variantes con resultados en…» | Español (medido con `resultados comparar`) | **Español** (medido: misma cadena) | P1 | `Texto` + `paths.idioma`. |
| 8 | `src/adapters/cli/entrada.rs:80,85,111` | `6443711` (movidas por `651e37a`) | «'dry_run' debe ser true o false; null ejecutaría el comando de verdad…», «'comando' no puede estar vacío…», «'comando' es el nombre del comando…» | Español (medido con `dry_run:null` y `comando:" help"`; el `trim` evita la segunda) | **Español** (medido: misma cadena) | P1 | `Texto` + `paths.idioma`; `leer(args, idioma)` y `opciones_de(args, idioma)`. |
| 9 | `src/adapters/cli/entrada.rs:100` | `030acb1` (movida por `651e37a`) | «'lic-activate' cambia la licencia y pvsyst_cli no la toca: usa entorno {accion:"licencia_activar"}…» | Español (medido) | **Español** (medido: misma cadena) | P1 | `Texto` + `paths.idioma`; es una instrucción de licencia, explícitamente en alcance. |
| 10 | `src/adapters/cli/entrada.rs:129,141,146,151,169,184` | `6443711` (movidas por `651e37a`) | «'opciones' debe ser una lista o un objeto…», «'opciones' solo admite textos…», «'opciones' repite '-x'…», «'opciones.a' debe ser texto, número o booleano…» | Español (medido) | **Español** (medido: misma cadena) | P2 | `Texto` + `paths.idioma`. |
| 11 | `src/objects/edicion.rs:111` | `13bafb8` | «Campos que el objeto 'pvVCalcul' no tiene: ["NoExiste"]. editar campos cambia valores existentes; pasa nuevo:true…» | Español (medido) | **Español** (medido: misma cadena) | P1 | `Texto` + `paths.idioma` (la ruta ya recibe `paths`). |
| 12 | `src/vista_agente/vista.rs:141,144,190` | `1213833`, `1f2c22b`, `6d41580` | «ninguna ventana de PVsyst abierta; esquemas con gui_observar que:esquema», «ninguna de las ventanas abiertas (…) tiene esquema de campos…», «la ventana no muestra los campos del esquema…» | Español (código; requiere una ventana GUI) | **Español** (código: misma cadena) | P1 | `Texto` + `paths.idioma`; `aviso_sin_entidad(paths, args)` y `vista(paths, args)`. |
| 13 | `src/projects/mod.rs:218` | `a925953` | `hint`: «Pass detalle:"completo" for the full tree, or leer with 'objeto' for one object.» | **Inglés** (medido con `leer` de un `.VC0`) | Inglés (medido) | P2 | `Texto` + `paths.idioma`; en `es` debe decir «Pasa detalle:"completo"…». |
| 14 | `src/admin/importacion.rs:26` | `da42576` | «Zip not found: … (resolved against the workspace …: pass an absolute path, or a 'zip' relative to that workspace)» | **Inglés** (medido con `workspace importar` sin zip) | Inglés (medido) | P2 | `Texto` + `paths.idioma`. El resto del archivo mezcla español e inglés (líneas 38, 45, 51, 153, 158, 178, 182, 188, 191, 214, 221, 243, 247, 259); unificarlo en el mismo `Texto`. |
| 15 | `src/adapters/cli/clasificacion.rs:182` | `8a16235` (previo) | «Proceso terminó con código de error 2» | Español (medido con el CLI falso fallando) | **Español** (medido: misma cadena) | P2 | `Texto` + `idioma`; se propaga a `detected_error`, `vista.avisos` y `vista.error.motivo`. |
| 16 | `src/adapters/gui/ventanas.rs:140` | `eeede5e` (previo) | «Process 'PVsyst.exe' is not running. Launch the PVsyst GUI first (or pass 'proceso').» | **Inglés** (medido con `gui_observar que:vista` sin GUI) | Inglés (medido) | P2 | `Texto` + `paths.idioma`; es el rechazo más común de la GUI. |
| 17 | `src/adapters/gui/acciones.rs:161,174` | `5be248b` | «'texto' no selecciona el control en click/clic_fisico/cerrar/activar/mensaje…», «El control … está deshabilitado…» | Español (medido con `gui_actuar click texto`) | **Español** (medido: misma cadena) | P2 | `Texto` + `paths.idioma`. |
| 18 | `src/adapters/gui/teclado.rs:129` | `b36d8ac` | «No control of 0x… holds the keyboard focus: the keys would be lost…» | **Inglés** (código; requiere GUI) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 19 | `src/adapters/gui/clic_fisico.rs:43,53,75` | `5be248b` (movidas por `2be310d`) | «'objetivo' y 'dx'/'dy' son dos formas de elegir el punto…», «El punto de pantalla … no está sobre la ventana…» | Español (código; requiere GUI) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 20 | `src/adapters/gui/cierre.rs:122,130` | `2be310d` (previo al split) | «{clase} is still open after asking it to close…», «… Answer it with gui_actuar accion:click…» | **Inglés** (código; requiere GUI) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 21 | `src/adapters/gui/conduccion.rs:143` | `94db09c` | «No {} at ({}, {}) of '{}': did the PVsyst version change?» | **Inglés** (código; requiere GUI) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 22 | `src/adapters/gui/campos.rs:217` | `90b1953` | «{} sigue abierta tras {espera_ms} ms (gui_observar que:ventanas)» | Español (código; requiere GUI) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 23 | `src/adapters/gui/clic_por_texto.rs:61` | `90b1953` | «El OCR no encontró '{texto}' (coincidencias:0): se pulsó el elemento UIA…» | Español (código; requiere GUI/OCR) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 24 | `src/adapters/gui/escena/exportar.rs:81` | `d03928b` | «The file name did not reach the save dialog: it holds…» | **Inglés** (código; requiere GUI) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 25 | `src/adapters/uia/elemento.rs:101`; `src/adapters/uia/limite.rs:26,31` | `5efd172` | «UIA '{accion}' no volvió en {} s…», «No se pudo crear el hilo UIA…», «La operación UIA terminó de forma inesperada…» | Español (código; requiere GUI/UIA) | **Español** (código) | P2/P3 | `Texto` + `paths.idioma`. |
| 26 | `src/adapters/cli/proceso.rs:154` | `08e7d61` | «Comando cancelado por el cliente» | Español (código; cancelación no disparada) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 27 | `src/adapters/cli/proceso.rs:85` | `b16e50d` | «Failed to launch {}: {e}. Set PVSYST_CLI.» | **Inglés** (código; no se forzó el fallo de `spawn`) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 28 | `src/projects/gui/lanzar.rs:43,63` | `f8bd6ee` | «'{pedido}' no es el nombre de un proyecto», «PVsyst 8.1.8 ignora el proyecto de su línea de comandos…» | Español (código; requiere GUI) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 29 | `src/horizonte_archivo/aplicacion.rs:100` | `682f96d` | «'archivo' debe nombrar un archivo: '.'» | Español (medido con `sitio escribir_horizonte`) | **Español** (medido: misma cadena) | P2 | `Texto` + `paths.idioma`. |
| 30 | `src/horizonte_archivo/aplicacion.rs:86,118,125`; `codec.rs:182` | `eeede5e`/`682f96d` (previos) | «Archivo de horizonte '…' no encontrado», «Punto de horizonte sin campo numérico 'altura'», «Horizonte inválido: se requieren al menos 2 puntos…» | Español (medido con `leer`/`sitio`) | **Español** (medido: misma cadena) | P2 | `Texto` + `paths.idioma`. |
| 31 | `src/adapters/gui/escena/mod.rs:121` | `8dc8a28` | «ruta de la malla (3DS, DAE, PVC, PVC2, PVZ2), fuera de Program Files» | Español (código; descripción de esquema) | **Español** (código) | P2 | `Texto` + `paths.idioma` en la anotación del esquema. |
| 32 | `src/adapters/gui/menu_por_posicion/listado.rs:58` | `2be310d` (previo al split) | «más de {MAX_MENUS} menús» | Español (código; requiere GUI) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 33 | `src/adapters/cli/linea_comando.rs:126,134` | `4d0e693` | «'timeout_s' debe ser un entero entre 1 y 3600…», «'timeout_s' debe estar entre 1 y 3600 segundos…» | Español (código; **inalcanzable**: el esquema valida antes y responde «Valor 99999 fuera de rango…» en ambos idiomas) | Español (código; inalcanzable) | P3 | `Texto` + `paths.idioma`; útil si otra ruta llama a `timeout_de_comando`. |
| 34 | `src/adapters/cli/trabajo.rs:40` | `ed21f1d` | «Mutex de trabajos envenenado: no se lanzó el trabajo» | Español (código; requiere un pánico previo) | **Español** (código) | P3 | `Texto` + `paths.idioma`. |
| 35 | `src/easy_mode/foto_proyecto.rs:15` | `ecbafcf` (previo; movido por `82df645`) | «No se pudo leer {dir}» | Español (código; no se forzó el fallo de lectura) | **Español** (código) | P3 | `Texto` + `paths.idioma`. |
| 36 | `src/adapters/http.rs:26,29,46,61` | `e0fc80a`/`eeede5e` (previos) | «No se pudo leer la respuesta…», «La respuesta supera el máximo…», «HTTP GET {url} falló…» | Español (código; no se hizo red) | **Español** (código) | P2 | `Texto` + `paths.idioma`. |
| 37 | `src/adapters/gui/proceso.rs:159,183` | `d2c3f0a`/`1c71e12` (previos) | «'proyecto' cannot be opened: PVsyst is already running…», «PVsyst GUI not found at '…'. Set PVSYST_GUI.» | **Inglés** (código; requiere GUI) | Inglés (código) | P2 | `Texto` + `paths.idioma`. |
| 38 | `src/adapters/ledger.rs:240` | previo | «Esta acción consume {n} ejecuciones de prueba; repítela con confirmar:true» | Español (código; el CLI falso no emite `requiere_confirmacion`) | **Español** (código) | P2 | `Texto` + `paths.idioma`; es la confirmación de TRIAL que el CHANGELOG nombra. |
| 39 | `tests/mcp_regresiones/mensajes.rs:32,42,58` | `a925953` | `assert!(msg.contains("ya existe"))`, `assert!(msg.contains("se pidieron 1"))` con el cliente por defecto (`en`) | — | — | P1 (prueba) | Reescribir las aserciones para que comprueben el idioma configurado (o el texto en inglés por defecto) y añadir el caso `es`. |
| 40 | `src/workspace/escritura.rs:27,70`; `src/format/edit.rs:76`; `src/format/campos_numericos.rs:106,112`; `src/error.rs:73,78` | `2c20626`/`7bcd971` | Usan `Idioma::de_entorno()` en vez de `paths.idioma` | Correcto en producción (ambos leen la misma variable) | Correcto en producción | P3 (diseño) | Pasar `Idioma` desde `paths.idioma`; evita que un `Paths` con idioma explícito (tests, futura configuración) se ignore. |

### 4.1 Casos que el patrón deja fuera de la tabla pero son la misma causa

Los siguientes archivos tienen el mismo defecto y no se listan uno a uno para no repetir la recomendación: `src/adapters/gui/{captura,escritorio,combinacion,secuencia,uia_paso,uia_vista,modal,recuperacion,esperas,gdi,png}.rs`, `src/adapters/gui/catalogo/{aplicacion,dominio,recurso}.rs`, `src/adapters/gui/menu_por_posicion.rs`, `src/projects/{anadir,doctor,gui,lifecycle,tipo_sistema}.rs`, `src/simulation/{batch,cache,ejecucion,estado,preestimate,report_pdf,engineering}/**`, `src/objects/{edicion,lectura}.rs`, `src/horizonte_archivo/{aplicacion,codec}.rs`, `src/almacenamiento/**`, `src/perdidas/dominio.rs`, `src/components/**`, `src/args.rs:45-102,220`. Son mensajes de `Error::*` que el cliente lee; el arreglo es el mismo: `Texto` + `Idioma` en el borde.

## 5. Cadenas comprobadas y correctas

Medidas en el servidor real, con `PVSYST_MCP_IDIOMA=es` y `=en`:

| Superficie | `es` | `en` |
|---|---|---|
| `tools/list` descripción de `entorno` | «Entorno PVsyst: diagnostico, licencia, logs y preferencias.» | «PVsyst environment: diagnostics, licence, logs and preferences.» |
| `tools/list` título/anotación de `entorno` | «Entorno» | «Environment» |
| `tools/list` descripción de `disenar_sistema` | «E2E en una llamada: …» | «End-to-end in one call: …» |
| Esquema `pvsyst_cli.comando` | «Solo el nombre del comando (`help`, `version`, `lic-info`)…» | «Only the command name (`help`, `version`, `lic-info`)…» |
| Esquema `pvsyst_cli.opciones` | «Argumentos de PVsystCLI. Lista: … Objeto: `--nombre:valor`…» | «PVsystCLI arguments. List: … Object: `--name:value`…» |
| Esquema `pvsyst_cli.dry_run` | «true: ensayo, no ejecuta nada… null no se acepta…» | «true: rehearsal, runs nothing… null is refused…» |
| Esquema `pvsyst_cli.timeout_s` | «Segundos antes de matar el proceso (de 1 a 3600)…» | «Seconds before the process is killed (1 to 3600)…» |
| `initialize.instructions` | «Opera PVsyst 8.1 (verificado en 8.1.7 y 8.1.8)…» | «Operate PVsyst 8.1 (measured on 8.1.7 and 8.1.8)…» |
| `prompts/list` | «Disenar sistema desde coordenadas (recomendada)», «Latitud del sitio en grados» | «Design system from coordinates (recommended)», «Site latitude in degrees» |
| `resources/list` nombres | «Recetas», «Cobertura», «Formato de archivos PVsyst», «Referencia PVsystCLI», «Mapa de la GUI de PVsyst» | «Recipes», «Coverage», «PVsyst file format», «PVsystCLI reference», «PVsyst GUI map» |
| `pvsyst://recetas` cuerpo | «Recetas listas para usar…», «Empieza con entorno accion:diagnostico…» | «Ready-to-use recipes…», «Start with entorno accion:diagnostico…» |
| `resources/read` URI desconocida | «Recurso desconocido: pvsyst://nope» | «Unknown resource: pvsyst://nope» |
| Argumentos que faltan (`componente crear`) | «Faltan: nombre_nuevo, plantilla. Ejemplo: …» | «Missing: nombre_nuevo, plantilla. Example: …» |
| Argumento que falta con hint (`editar insertar_objeto`) | «Falta el argumento obligatorio 'desde.archivo' (desde:{archivo, objeto})» | «Missing required argument 'desde.archivo' (desde:{archivo, objeto})» |
| Tipo incorrecto (`timeout_s:"abc"`) | «Tipo incorrecto para 'timeout_s' (esperado integer, recibido string)» | «Wrong type for 'timeout_s' (expected integer, got string)» |
| Fuera de rango (`timeout_s:99999`) | «Valor 99999 fuera de rango para 'timeout_s' (entre 1 y 3600)» | «Value 99999 out of range for 'timeout_s' (between 1 and 3600)» |
| Acción inválida (`proyecto accion:"leer"`) | «Valor 'leer' no válido para proyecto (accion); valores: …» | «Value 'leer' not valid for proyecto (accion); values: …» |
| Tool desconocida | «Tool desconocida 'no_existe'» | «Unknown tool 'no_existe'» |
| `escribir_archivo` sin `sobrescribir` | «'…\\probe3.txt' ya existe: pasa 'sobrescribir': true…» | «'…\\probe3.txt' already exists: pass 'sobrescribir': true…» |
| `workspace` acción inválida | «Valor 'escribir_archivo' no válido para workspace (accion); valores: …» | «Value 'escribir_archivo' not valid for workspace (accion); values: …» |
| Rechazo de aproximación / dimensionado sin `n_mod_serie` | `Texto` en `src/easy_mode/diseno.rs` (código) | `Texto` (código) |
| Avisos de aproximación (`aproximaciones::marcar`) | `Texto` en `src/aproximaciones/mod.rs` (código) | `Texto` (código) |

Pares `Texto`: 183 con `es` y `en`; 3 idénticos, todos nombres propios o valores de protocolo (`simular ejecutar`, `PVsystCLI`, `Workspace`); 0 con acentos españoles en `en`; 0 intercambiados por la heurística de vocabulario. No se encontró un `Texto` con `es`/`en` invertidos.

## 6. Pruebas que no detectarían una regresión

- `tests/mcp_regresiones/mensajes.rs` (nuevo en `a925953`): **fija el español**. El cliente `Client::start` no define `PVSYST_MCP_IDIOMA`, así que el servidor corre en `en` por defecto; las aserciones `contains("ya existe")` y `contains("se pidieron 1")` solo pasan porque el mensaje está hard-codeado en español. Si se corrige el mensaje, esta prueba falla: hay que reescribirla, no conservarla.
- `tests/unit/server/tools/manifiesto_tests.rs`: solo compara `name`/`description` del registro con `mcpb/manifest.json`, con `paths.idioma = Idioma::En`. No mira títulos, anotaciones, mensajes ni `vista`.
- `tests/unit/server/tools/sinonimos_tests.rs`: solo comprueba el conflicto de alias con `Idioma::En`; no prueba la rama `es` ni el mensaje de conflicto en español.
- `tests/unit/easy_mode/fallo_simulacion.rs` y `tests/unit/easy_mode/meteonorm.rs`: pasan `Idioma::En` explícito; no prueban `PVSYST_MCP_IDIOMA` ni la selección por entorno.
- No hay ninguna prueba que arranque el servidor con `PVSYST_MCP_IDIOMA=es` y compruebe una respuesta; `grep` de `PVSYST_MCP_IDIOMA` en `tests/` no devuelve nada. `tools/generar_contrato.py` fija `en`.
- Los tests unitarios de `src/idioma.rs` se borraron en `9a3640f` («drop tests that pin implementation details»). Hoy `Texto::en_idioma` y `Texto::texto_con` no tienen prueba directa; solo se ejercitan de forma indirecta.
- Ninguna prueba cubre `vista.resumen`, `vista.avisos`, `vista.siguiente[].motivo`, `vista.error.causa_probable` ni `vista.recortado.pedir_resto`.

## 7. Lo que no se pudo verificar y por qué

- **Caminos que necesitan una ventana real de PVsyst**: `vista.rs` (aviso sin entidad/sin campos), `campos.rs` (espera que sigue abierta), `clic_por_texto.rs` (fallback OCR), `teclado.rs` (foco), `cierre.rs`, `conduccion.rs`, `escena/exportar.rs`, `uia/*`, `menu_por_posicion/*`, `catalogo/aplicacion.rs`, `captura.rs`, `escritorio.rs`, `proyectos/gui/lanzar.rs`. La instrucción del encargo prohíbe lanzar la GUI; se citan por lectura de código y se marcan «código» en la tabla.
- **Confirmación de modo TRIAL** (`requiere_confirmacion`): el CLI falso de `tests/fixtures/cli` no emite esa marca. Se midió la rama de ensayo (`dry_run:true`), no la de confirmación. El texto de `pendiente.rs` es el mismo en ambas ramas.
- **Cancelación** (`proceso.rs:154`): no se encontró una forma determinista de cancelar un `pvsyst_cli` por stdio sin dormir el proceso; no se forzó.
- **`timeout_s` interno** (`linea_comando.rs:126,134`): el esquema y `validacion.rs` rechazan antes el valor; la ruta es inalcanzable desde `tools/call`. Se marca P3.
- **Red** (`http.rs`): no se hizo ninguna petición.
- **Rollback de `workspace importar`** (`admin/importacion.rs:153-259`): requeriría un zip que falle a mitad de publicación; no se construyó.
- **Deshacer de `easy_mode`** (`foto_proyecto.rs`): requeriría un fallo después de crear el sitio/proyecto; no se forzó.
- **Dominio de simulación** (`report_pdf`, `cache`, `batch`): muchos mensajes solo se alcanzan con informes/PDF reales o cachés concretas; se revisaron por lectura, no todos en ejecución.
- **Cuerpos de `pvsyst://cobertura` y `pvsyst://gui`**: son datos de referencia; no se juzgan como traducción. El nombre del recurso sí se midió y es correcto.
- **Texto propio de PVsyst**: `detected_error` («Invalid command syntax»), `stdout_tail`, `stderr_tail` y las etiquetas de menú no se traducen por diseño.

## 8. Plan de arreglo recomendado

Ordenado por impacto y por unidades de commit (una preocupación por commit):

1. **`vista` (P1, el mayor).** Cambiar los tipos del contrato a `Texto`:
   - `Entrada.resumen`, `Entrada.pista`, `Paso.motivo` → `Texto`.
   - `ErrorVista.causa_probable`, `ErrorVista.que_hacer` → `Texto`.
   - `Recorte.pedir_resto` → `Texto`.
   - `construir`, `siguiente_de`, `desde_error`, `Pendiente::resumen/error/siguiente` reciben `Idioma` (desde `paths.idioma`).
   - Actualizar las ~112 entradas del catálogo y los textos de `construccion.rs`, `errores.rs`, `pendiente.rs`.
   - Prueba: una llamada `listar` con `es` devuelve `vista.resumen` en español y con `en` en inglés; una llamada fallida devuelve `vista.error.causa_probable` en el idioma pedido.
2. **Mensajes nuevos de QA (P1).** `args.rs::no_encontrado`, `creacion.rs`, `comparacion.rs`, `entrada.rs`, `objects/edicion.rs`: constantes `Texto` y `Idioma` en el borde. Prueba por tabla: el mismo caso en `es` y `en`.
3. **Rechazos de GUI (P2).** Pasar `paths.idioma` a `acciones`, `teclado`, `clic_fisico`, `cierre`, `conduccion`, `campos`, `clic_por_texto`, `escena`, `uia`, `menu_por_posicion`, `proceso`, `ventanas`. Prueba unitaria de las funciones puras de mensaje; las que necesitan GUI quedan `#[ignore]`.
4. **Resto de adaptadores/dominio (P2/P3).** `admin/importacion.rs`, `horizonte_archivo`, `http`, `ledger`, `clasificacion`, `foto_proyecto`, `trabajo`. Misma receta.
5. **Pruebas (P1 de calidad).** Reescribir `tests/mcp_regresiones/mensajes.rs` para que compruebe el idioma configurado; añadir un test de integración que arranque el servidor con `PVSYST_MCP_IDIOMA=es` y otro con `en` y compare una respuesta de `listar` y una de error; restaurar una prueba mínima de `Texto::en_idioma`/`texto_con` (sin volver a las pruebas de implementación que se borraron).
6. **Documentación.** El CHANGELOG ya promete «toda cadena visible»; no hace falta cambiarlo, pero la entrada de `a925953` que dice «says in Spanish» debe corregirse cuando el mensaje se traduzca.

## 9. Comandos ejecutados

```powershell
# Contexto
git log --oneline 2f105d3..HEAD
git log --oneline 81b4f59..HEAD
git diff 2f105d3..HEAD -- src/
git blame -L <n>,<n> --date=short <archivo>
git log -S "<cadena>" -- src/

# Compilación (toolchain GNU; el MSVC no tiene linker aquí)
$env:PATH = "$env:USERPROFILE\.cargo\bin;C:\Users\Lenovo\AppData\Local\Microsoft\WinGet\Packages\BrechtSanders.WinLibs.POSIX.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe\mingw64\bin;$env:PATH"
cargo +stable-x86_64-pc-windows-gnu build

# Ejecución por stdio (dos idiomas, misma copia de tests/fixtures/workspace)
$env:PVSYST_MCP_IDIOMA = "es"   # y luego "en"
$env:PVSYST_WORKSPACE = "$env:TEMP\pvsyst_lang_ws"
$env:PVSYST_CLI = "target\debug\fake_pvsystcli.exe"
$env:FAKE_PVSYST_FIXTURES = "tests\fixtures\cli"
python $env:TEMP\pvsyst_lang\drive.py  es
python $env:TEMP\pvsyst_lang\drive.py  en
python $env:TEMP\pvsyst_lang\drive2.py es
python $env:TEMP\pvsyst_lang\drive2.py en
python $env:TEMP\pvsyst_lang\drive3.py es
python $env:TEMP\pvsyst_lang\drive3.py en
python $env:TEMP\pvsyst_lang\drive4.py es
python $env:TEMP\pvsyst_lang\drive4.py en

# Parseo estático de Texto y de literales
python (script de extracción de `Texto { es, en }` y de literales con espacios)
```

Llamadas `tools/call` ejecutadas en ambos idiomas: `listar` (`proyectos`, `archivos`, `modulos`), `leer` (archivo existente, inexistente, `.VC0`, `.HOR`), `pvsyst_cli` (`help`, `version`, `nope`, `dry_run:null`, `dry_run:true`, `timeout_s:99999`, `timeout_s:"abc"`, `lic-activate`, `opciones` vacías/repetidas/de tipo objeto), `workspace` (`historial`, `importar` sin zip, acción inválida), `escribir_archivo` (nuevo y existente), `proyecto` (`doctor` inexistente, `dimensionar` sin `kwp`, `crear` duplicado), `resultados` (`kpis`/`resumen`/`comparar`), `componente` (`crear` sin claves, `editar` inexistente), `editar` (`campos` con clave inexistente, `insertar_objeto` sin `desde.archivo`), `sitio` (`escribir_horizonte` con nombre inválido, punto incompleto y menos de 2 puntos), `gui_observar` (`vista`, `esquema`), `gui_actuar` (`click` con `texto`), `entorno` (`diagnostico`), `no_existe`.

## 10. Conclusión

El mecanismo de idioma está bien diseñado y bien aplicado en la capa de servidor, pero la promesa del CHANGELOG («toda cadena visible») no se cumple. El bloque `vista` —presente en todas las respuestas— y una parte grande de los adaptadores y del dominio siguen en un solo idioma. Los commits de QA de esta rama no introdujeron el patrón, pero lo repitieron en al menos 20 mensajes nuevos y añadieron una prueba que fija el español. La corrección es mecánica y se puede dividir en las unidades del §8; la de mayor relación impacto/esfuerzo es la primera (`vista`), porque arregla el idioma de todas las respuestas correctas y fallidas a la vez.

## 11. Estado de corrección (actualización 2026-11-09)

Los hallazgos 1–38 se corrigieron en esta rama, en cuatro tandas:

| Commits | Alcance |
|---|---|
| `c2f0246` | Bloque `vista` completo: catálogo (~112 entradas), `construccion.rs`, `errores.rs`, `pendiente.rs`, `recorte.rs`, avisos de `vista_agente/vista.rs`. |
| `d6ca9ff`, `5167456`, `10e5b2b`, `9b10eef`, `57ed7aa`, `8cb142f` | Rechazos y avisos de `adapters/gui/**` y `adapters/uia/**`; `catalogo/aplicacion.rs` se partió en `catalogo/abrir.rs` para no pasar de 300 líneas. |
| `756771a` | `args.rs`, `adapters/cli/**`, `adapters/http.rs`, `adapters/ledger.rs`, `admin/importacion.rs`, `easy_mode/foto_proyecto.rs`, `horizonte_archivo/**`, `objects/**`, `projects/**`, `simulation/{engineering/comparacion,opciones}.rs`, `workspace/{escritura,lectura}.rs`, `components/resolucion.rs`, `format/**`, `server/tools/workspace_ejecucion.rs`. |
| `5483ca2` | Mensajes de `horizonte_archivo/dominio.rs` y alineación de los tests existentes con las firmas nuevas; no se creó ningún test. |

Verificación en ejecución, con el binario reconstruido y las mismas llamadas de §9:

- `listar` → `vista.resumen`: «Proyectos del workspace listados.» / «Workspace projects listed.»; `siguiente[].motivo` también cambia.
- `leer` de un proyecto inexistente: «Proyecto … no encontrado … hay 1: …» / «Project … not found … there are 1: …»; `vista.error.causa_probable` y `que_hacer` cambian.
- `pvsyst_cli` con `dry_run:null`, `lic-activate` y `opciones` inválidas: español / inglés.
- `proyecto crear` duplicado y `resultados comparar` con una variante: español / inglés.
- `editar campos` con una clave inexistente: «Campos que el objeto … no tiene…» / «Fields the object … does not have…».
- `gui_actuar click` con `texto`: español / inglés.
- `workspace importar` sin zip y `sitio escribir_horizonte` con un punto: español / inglés.
- `pvsyst_cli` con el CLI falso fallando: `detected_error` y `vista.avisos` en el idioma pedido.

Gate ejecutado después de la última edición: `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` (306 unitarias + 34 + 35 + 8 + 2 + 1 + 1 + 2 doc-tests, todas en verde), `python tools/check_deps.py`, `python -m ruff check tools`, `python -m ruff format --check tools`, `python -m mypy --strict tools`, `python -m pytest -q tools`. Sin GUI y sin ejecuciones TRIAL.

Queda pendiente, y así se reporta: las ~150 cadenas de prosa del MCP en `adapters/gui/**`/`uia/**` que la receta no enumeraba (lista en el reporte del subagente GUI), y el defecto de diseño P3 de `Idioma::de_entorno()` en módulos sin `Paths` (se usa donde la firma no permite pasar `paths.idioma`; en producción lee la misma variable). No se creó ningún test nuevo; los tests existentes se ajustaron al idioma por defecto (`en`) o se les pasó `Idioma::Es` para conservar sus aserciones en español.
