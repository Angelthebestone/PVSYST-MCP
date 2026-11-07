# Revisión de código con subagentes Haiku: hallazgos consolidados

Fecha: 2026-10-31. Rama revisada: `main` (commit `81b4f59`).

## Cómo leer este documento

- Es una revisión de **solo lectura**, hecha por 12 subagentes Haiku, guiada por `docs/REVISION_ULTRAREVIEW.txt` (que solo existe en el worktree `tarea-d-fila`) y por `agents.md`.
- **Nada de lo que sigue está verificado por ejecución**, salvo `cargo clippy --all-targets -- -D warnings`, que salió con 0 avisos (agente global). Cada hallazgo debe reproducirse antes de corregirlo.
- Marcas: [CONFIRMADO] lo demuestra el código leído; [PLAUSIBLE] es inferencia.
- Las líneas son las de `main`. El PR 2 (`worktree-tarea-d-fila`) puede tenerlas desplazadas.
- Los puntos de la guía que mencionan `escena_importar`, `TImportResultForm`, `GetSubMenu`, `spawn/kill/wait_with_output`, el tope de 64 MiB en `http.rs` y `ACCIONES` en `gui/mod.rs` **no existen en `main`**.
- Los hallazgos de «`#[expect]` sobrante» (`lib.rs`, `tests/protocol.rs`, `cliente_mcp`, etc.) probablemente son falsos: `clippy -D warnings` salió limpio y `clippy.toml` fija `too-many-arguments=3`.

## Cobertura

| Parte | Archivos | Líneas | Lectura |
|---|---|---|---|
| `src/adapters/` | 58 | 7.975 | completa |
| `server/`, `vista_agente/`, `gui_catalogo/`, `escena_3d/` | 94 | 9.772 | completa |
| `projects/`, `simulation/`, `workspace/`, `admin/` | 73 | ~8.800 | completa |
| `components/`, `sites/`, `format/`, `easy_mode/`, `history/` | 53 | ~8.500 | completa |
| raíz de `src/`, `bin/` y dominios pequeños | ~85 | ~6.800 | completa |
| `tests/` | ~145 | ~16.000 | completa, salvo `tests/mcp_smoke/gui.rs` (lectura parcial) |
| Vista global | 478 | 58.005 | `clippy` normal y pedántico más greps |

Código muerto: ningún símbolo privado ni `pub(crate)` sin uso según `rustc` y `clippy`. Todas las dependencias de `Cargo.toml` tienen uso. Los candidatos `pub(crate)` con uso solo interno están en la sección 5.

---

## 1. Gravedad ALTA

### A. Confinamiento y validación en el borde

| # | Dónde | Qué |
|---|---|---|
| A1 | `projects/mod.rs:244-268` | Nombres con raíz sin unidad (`\Users\x`) reemplazan la ruta en `join`. `eliminar_proyecto` mueve a la papelera un archivo fuera del workspace; exportar, renombrar, leer y `doctor` también actúan sobre él. [CONFIRMADO] |
| A2 | `projects/gui/mod.rs:106-110` | `proyecto` sin validar. La GUI escribe `..\..\X.PRJ` y «Guardar» escribe fuera del workspace. También en `gui/p50.rs:32` y `server/tools/proyectos.rs:249`. [CONFIRMADO] |
| A3 | `simulation/preparacion.rs:101-109` | `output_csv` y `report_pdf` se unen al workspace sin `resolver_ruta` ni `Alcance`. [CONFIRMADO] |
| A4 | `simulation/preparacion.rs:67-69` | `extra_options` puede sobrescribir `-w`, `-p`, `-v`, `-ocf`, `-rpf`, `-bpf`. [CONFIRMADO] |
| A5 | `horizonte_archivo/aplicacion.rs:82-93`, `serie_horaria/aplicacion.rs:128-137` | Candidatos de DataRO con `join(nombre)` sin `ruta_segura`; un nombre absoluto reemplaza la base y `..` no se rechaza. Los errores de decodificar devuelven la línea del archivo. [CONFIRMADO] |
| A6 | `plantilla_variables/aplicacion.rs:16-33, 137` | `resolver_ruta_rvt` devuelve rutas absolutas tal cual y `history::reemplazar` no valida: escritura fuera del workspace sin `permitir_fuera_del_workspace`. [CONFIRMADO] |
| A7 | `components/pan/mod.rs:108-112` | `direct.starts_with(compos_ro())` es léxico: `DataRO\..\..\x.PAN` pasa. `resolucion.rs:76` sí usa `es_base_solo_lectura`. [CONFIRMADO] |
| A8 | `simulation/batch/mod.rs:45-60`, `batch/plantilla.rs:79-80`, `batch/mod.rs:109-111, 162-173` | `;`, `\r` y `\n` sin validar en parámetros, iniciales, unidades, descripciones, proyecto, variante y nombres de variable (`Param_Modif.dat`, cabecera CSV, `.rvt`). `plantilla.rs:49` sí valida las celdas. [CONFIRMADO] |
| A9 | `projects/lifecycle/creacion.rs:106-121` | `sitio` va a `format!("{sitio}.SIT")` sin validar: oráculo de existencia y enlaza un `.SIT` externo. [CONFIRMADO] |
| A10 | `adapters/http.rs:19-29` | Sin tope de bytes: `--max-time 30` limita tiempo, no tamaño, y `Command::output()` bufferiza todo. [CONFIRMADO] |
| A11 | `adapters/http.rs:12`, `config.rs:179,190` | Fallback a `curl.exe`, `pvsystcli` y `PVsyst.exe` sin ruta: `CreateProcess` busca primero en el directorio actual. [PLAUSIBLE] |

### B. Éxito silencioso (ok, verificado o importado sin comprobación real)

| # | Dónde | Qué |
|---|---|---|
| B1 | `vista_agente/contrato/pendiente.rs:15-21, 37-53` | El «ensayo» se deduce de los argumentos (`dry_run`, `simular`), no de la salida. `preferencias_editar` (`prefs/ini.rs:209`), `escribir_serie` (`serie_horaria/aplicacion.rs:69`), `escribir_horizonte` (`horizonte_archivo/aplicacion.rs:29`), `simular preparar` (`simulacion.rs:127-164`) y `gui_actuar click simular:true` escriben y dicen «no se aplicó nada». [CONFIRMADO] |
| B2 | `gui_catalogo/espera.rs:86-99`, `aplicacion.rs:64-84` | Un error de `consultar_visibles` sale como «no aparecieron ventanas»; `abrir_funcion` da `estado: ok` aunque no llegue la ventana esperada (`construccion.rs:101-111` solo mira `ok` y `errores`). [CONFIRMADO] |
| B3 | `vista_agente/relleno.rs:118-119, 157-194` | `rellenar` escribe campo a campo; si falla el k-ésimo no informa qué quedó aplicado. Los resultados «distinto» y «simulado» no suben al nivel superior y la vista dice «ok». La vista se calcula después de escribir con `?`. [CONFIRMADO] |
| B4 | `objects/edicion.rs:23-37, 76-92` | `verificar_span` no comprueba nada si `parse_first` es `None` o el span no existe y escribe igual. La ruta legacy marca `verified_after_write = new_value` sin releer; la ruta normal relee `new_bytes` en memoria, no el archivo. [CONFIRMADO] |
| B5 | `easy_mode/mod.rs:173-180`, `sites/mod.rs:280-288`, `easy_mode/dimensionado.rs:231-238` | `correr_pasos` no comprueba `paso_sitio.ok` y sigue con orientación, dimensionado y simulación. El dimensionado devuelve `ok:true` fijo y, si falta `realised_kwp`, `exigir_tolerancia_kwp` devuelve `Ok`. [CONFIRMADO] |
| B6 | `projects/gui/dimensionado.rs:44-51`, `gui/horizonte.rs:42-54` | Devuelven OK si el archivo cambió, sin comparar con lo propuesto; `importar_pvgis` devuelve campos `null` si PVsyst no importó. [CONFIRMADO] |
| B7 | `adapters/gui/acciones.rs:151`, `ffi/escritorio.rs:232-240`, `gui/uia_vista.rs:210-238` | `clic_fisico` devuelve OK sin clic; `clic_emulado_en` descarta el error; `post_message(BM_CLICK)` ignora el `bool`; `intentar_uia` convierte cualquier error en `false` y la cascada sigue. [CONFIRMADO] |
| B8 | `components/ficha/pvsyst.rs:279-300` | `leer_guardado` devuelve éxito con lo leído del archivo sin contrastarlo con lo enviado. [CONFIRMADO] |
| B9 | `components/mod.rs:199-204` | Editar un PVObject sin `cambios` (o con `campos`, que la rama legacy sí acepta) devuelve `Ok` vacío y reescribe los mismos bytes. [CONFIRMADO] |
| B10 | `projects/anadir/catalogo.rs:154-159, 227-230` | Si el tipo ya existe no se inserta, pero se añade a `insertados` y se calcula `neutralizado` sobre esa lista. [CONFIRMADO] |
| B11 | `meteo_mef/legacy.rs:18, 65-98`, `dominio.rs:224-238` | Un MEF legacy sin variables legibles devuelve `Ok` con `total_variables: 0`. [CONFIRMADO] |
| B12 | `plantilla_variables/dominio.rs:186-197`, `aplicacion.rs:41` | `eliminar_variables` valida con `trim` pero filtra sin recortar y responde `ok:true`; `filter_map(as_str)` descarta en silencio elementos no textuales. [CONFIRMADO] |
| B13 | `adapters/ledger.rs:82-90, 113` | Un JSON corrupto hace que `estado()` devuelva `None` y no se exija confirmación del gasto de ejecuciones de prueba. [CONFIRMADO] |
| B14 | `simulation/ejecucion.rs:14-18` | `success` puede forzarse a `true` si hubo archivo nuevo aunque la CLI fallara. [PLAUSIBLE] |

### C. Estado de la GUI, teclas y clics

| # | Dónde | Qué |
|---|---|---|
| C1 | `gui/teclado.rs:95-118` | `teclas` sin `ventana` envía al primer plano de cualquier proceso (`scoped` se calcula y no se usa); el fallo de `traer_al_frente` se ignora y `sent` no verifica el foco. [CONFIRMADO] |
| C2 | `gui/ventanas.rs:177-186`, `119-124`, `gui/esperas.rs:24` | `resolve` acepta cualquier HWND hex válido sin mirar el alcance por proceso. `target_pids` trunca con `x as u32`, descarta entradas no numéricas y una lista vacía deja de filtrar por pid. [CONFIRMADO] |
| C3 | `gui/ffi/entrada.rs:72-109`, `gui/conduccion.rs:222-228` | `pulsar_tabulador` y `pulsar_combinacion` no comprueban el primer plano; si el foco no llegó, el Tab va a otra aplicación. [PLAUSIBLE] |
| C4 | `gui/menu_por_posicion.rs:131-148` | `salir_de_la_barra` descarta el error (`let _ = pulsar`): la barra de menús puede quedar armada o abierta. [CONFIRMADO] |
| C5 | `gui/sesion_pvsyst.rs:85-92`, `gui/rotulos.rs:20,55`, `gui/acciones.rs:82-88` | `aceptar_mensajes` y `cerrar` pulsan OK, Aceptar, Continuar o Sí sobre cualquier `TMessageForm`, incluidas preguntas Sí/No. Además es un bucle sin tope si el cierre falla. [CONFIRMADO / PLAUSIBLE] |
| C6 | `gui_catalogo/entradas/escena_3d.rs:38-60` | Funciones «no explorado» con efecto Modifica (exportar escena `.SHD`, Cancelar, Cerrar escena) expuestas como ejecutables vía `gui_actuar accion:funcion`. [CONFIRMADO] |
| C7 | `uia/elemento.rs:51-54, 62-68, 120-132` | `buscar` devuelve la primera coincidencia parcial; «Guardar» puede invocar «Guardar como…». [CONFIRMADO] |
| C8 | `gui/ffi/menus.rs:52-75` | `GetMenuItemID == -1` se clasifica como `Submenu`; `MF_ERROR` pasa a 0 («habilitado»). Un HMENU que deja de ser válido entre llamadas se lista como dato. [CONFIRMADO / PLAUSIBLE] |
| C9 | `gui/ffi/ventanas.rs:163-188`, `gui/acciones.rs:64-80` | `enviar_mensaje` acepta `mensaje`, `wparam` y `lparam` sin lista blanca; con mensajes que llevan puntero el `lparam` del llamador se usa como dirección. [PLAUSIBLE] |
| C10 | `gui/proceso.rs:24-43` | `cerrar_todo` aborta a medias con `?` y `cerradas` cuenta intentos, no cierres comprobados. [CONFIRMADO] |
| C11 | `gui/recuperacion.rs:134-147` | Si madExcept reinicia PVsyst pasados 60 s, se lanza una segunda instancia. [PLAUSIBLE] |
| C12 | `gui_catalogo/aplicacion.rs:126-148`, `entradas/inicio.rs:3-6` | `accion:menu` con `id` directo no se valida contra la barra; la doc dice que Licencia (74) es alcanzable. Regla 10: no pulsar «Activar». [PLAUSIBLE] |

### D. Procesos, recursos y concurrencia

| # | Dónde | Qué |
|---|---|---|
| D1 | `adapters/cli/proceso.rs:131-133` | `try_wait().map_err(...)?` sale sin `kill` ni `wait`: el hijo queda vivo. [CONFIRMADO] |
| D2 | `cli/proceso.rs:90-96`, `cli/captura.rs:58-72` | `reader.join()` sin plazo (un nieto con los pipes impide el EOF) y `drain` sale ante el primer error: el hijo bloquea hasta el timeout (hasta 3.600 s). [PLAUSIBLE / CONFIRMADO] |
| D3 | `admin/archive.rs:162-168, 196-209`, `75-94` | `stderr` de `tar` se lee solo tras agotar `stdout`: posible bloqueo. `wait()` y `.output()` sin plazo. Dos agentes coinciden. [PLAUSIBLE] |
| D4 | `simulation/ejecucion.rs:89-104` | El job se lanza antes de tomar el lock; con el mutex envenenado queda huérfano y el cliente puede relanzarlo y cobrarlo dos veces. [CONFIRMADO] |
| D5 | `simulation/mod.rs:132,143`, `ejecucion.rs:61`, `preparacion.rs:119-120` | `preparar` y el plan reciben `args` en vez de `args_con_salidas`. Con salidas por defecto `ocf/rpf = None`: la caché guarda sin CSV y devuelve aciertos que no lo restauran. [CONFIRMADO] |
| D6 | `simulation/cache/store.rs:75-80, 94-95` | `meta.json` con `fs::write` (no atómico) y `buscar()` propaga el error de parseo con `?`: la simulación entera falla en vez de ser un fallo de caché. [CONFIRMADO] |
| D7 | `simulation/cache/key.rs:12-13` | `OPCIONES_ENTRADA` no incluye `-s`: el contenido del SIT no entra en la clave. [CONFIRMADO] |
| D8 | `simulation/avisos_pvsyst.rs:114` | `read_line(...).unwrap_or(0)`: un byte no UTF-8 (cp1252) corta la lectura y se descarta el resto del log. [CONFIRMADO] |
| D9 | `adapters/cli/trabajo.rs:45-59` | Si `run_inner` entra en pánico, `resultado` no se fija y `estado_trabajo` responde «ejecutando» para siempre. [CONFIRMADO] |
| D10 | `gui/ocr/motor.rs:55-75` | `esperar_async!` sin plazo. [CONFIRMADO] |

### E. Pérdida o borrado de datos

| # | Dónde | Qué |
|---|---|---|
| E1 | `easy_mode/proyecto.rs:171-174, 194-204`, `mod.rs:156-165` | `deshacer_nuevos` borra cualquier archivo nuevo de `Projects`, incluidos los que PVsyst cree durante la corrida; si falla la foto inicial (`unwrap_or_default`) trata todo `Projects` como nuevo. El rollback tampoco revierte el `.SIT` creado. [CONFIRMADO / PLAUSIBLE] |
| E2 | `projects/doctor/mod.rs:148-150` | `revisar_variante(...)?` en el bucle: un fallo descarta `reparado` e `historial` ya escritos. [CONFIRMADO] |
| E3 | `projects/lifecycle/creacion.rs:159-165` | El par PRJ/VC se escribe antes de `vincular_sitio`; si falla queda el proyecto creado. [CONFIRMADO] |
| E4 | `format/object.rs:213-216` | Un documento truncado se parsea con los nodos abiertos mal anidados; `parse_first` devuelve el hijo interno. Se usa sin comprobar balance en `ficha/pvsyst.rs:287` y `components/mod.rs:156`. [CONFIRMADO] |
| E5 | `admin/componentes.rs:69-80`, `admin/proyecto.rs:46-59` | `fs::copy` y el zip se sobrescriben sin comprobar existencia (`exportar_workspace` sí comprueba). [CONFIRMADO] |
| E6 | `serie_horaria/aplicacion.rs:73,82`, `prefs/ini.rs:38-53, 233` | `from_utf8_lossy` y reescritura: se pierden bytes no UTF-8; `as_chunks` descarta el byte sobrante. [CONFIRMADO] |
| E7 | `plantilla_variables/dominio.rs:149-159, 244-246` | `parse_rvt` lee solo la primera línea y `serializar_rvt` reescribe el archivo con una línea. [PLAUSIBLE] |
| E8 | `admin/importacion.rs:27, 103-107`, `admin/workspace.rs:131-135`, `admin/proyecto.rs:34-36` | `read_dir().is_ok_and(...)` y `.flatten()` tratan errores como carpeta vacía; importación y exportación parciales reportan éxito. [CONFIRMADO] |
| E9 | `seguidores/aplicacion.rs:69-73, 102` | `texto_de_objeto` convierte el error en `None` con `.ok()`; `reemplazar_objeto` puede sustituir `pvShading`. [CONFIRMADO el mecanismo] |

### F. Tests que pasan sin verificar, o mal hechos

| # | Dónde | Qué |
|---|---|---|
| F1 | `tests/serie_horaria.rs:50-52, 86-88, 114-116, 141-143`; `tests/unit/serie_horaria/mod.rs:107-114, 143-150`; `tests/mcp_regresiones/diseno.rs:147-149, 166-168`; `tests/unit/components/tests.rs:61-63, 231-233`; `components/ficha/tests.rs:36-43`; `tests/unit/sites/solar/tests.rs:259-267` | `return` temprano si falta PVsyst, sin `#[ignore]`: pasan en verde sin verificar. [CONFIRMADO] |
| F2 | `tests/unit/admin/tests.rs:58-64` | `if let Err(err) = res`: si la exportación tiene éxito el test pasa sin aserción. [CONFIRMADO] |
| F3 | `tests/unit/components/pan/tests.rs:161-171` | No crea el archivo fuera del workspace y solo asevera `is_err()`. [CONFIRMADO] |
| F4 | `tests/unit/adapters/gui/teclado.rs:34` | `assert_eq!(vk_for("PGUP"), vk_for("PageUp"))` pasa si ambos son `None`. [CONFIRMADO] |
| F5 | `tests/cli_fake/resultados.rs:148-150`, `tests/fixture_workspace.rs:308` | `assert_ne!(isError, true)` pasa si la clave no existe. [CONFIRMADO] |
| F6 | `tests/unit/adapters/cli/captura.rs:41-50` | Prueba `JoinHandle::join` de std y compara con un texto que no existe en `src`. [CONFIRMADO] |
| F7 | `tests/unit/server/tools/validacion_tests.rs:150-158` | Llama a `call` con `gui_actuar` y HWND `0x1`: toca Win32 real. [CONFIRMADO] |
| F8 | `tests/mcp_smoke/gui.rs:66-108` | Cierra `TDEMOForm`/`TToolsForm` de la sesión real del usuario, usa `sleep` y descarta resultados. [CONFIRMADO] |
| F9 | `tests/cliente_mcp/mod.rs:133-138` | `recv` con `read_line` sin plazo. [CONFIRMADO] |
| F10 | `tests/mcp_smoke/proyectos.rs:57,78` | Usa la carpeta de datos de la instalación como workspace. [PLAUSIBLE] |
| F11 | `tests/roundtrip.rs:93-97` | `assert!(changed <= 1)` acepta cero cambios. [CONFIRMADO] |
| F12 | `tests/mcp_smoke/workspace_y_diseno.rs:54-152` | Un test de unas 99 líneas con 7 comportamientos y limpieza que no corre si hay pánico. [CONFIRMADO] |
| F13 | `tests/unit/gui_catalogo/mod.rs:68-73` | Test vacuo: `all()` sobre un resultado vacío pasa. [CONFIRMADO] |
| F14 | `tests/unit/components/pan/legacy_edicion_tests.rs:40-73` | `enc == re_enc` pasa con un codificador constante. [CONFIRMADO] |

---

## 2. Gravedad MEDIA

### Bugs y validación

- `format/insert.rs:84`: `rol` (de `projects/anadir/objetos.rs:95`) entra en `format!("{pad}PVObject_{nuevo_rol}={ty}")` sin validar; `\n` o `=` rompen la cabecera. [CONFIRMADO]
- `sites/readers.rs:30,36`: `split(',')` y luego `replace(',', '.')` nunca encuentra coma; «1,5» se parte en dos. [CONFIRMADO]
- `sites/pvgis/persistencia.rs:82-86`: `Err(_)` descarta el error del primer intento de `convertir_meteo`. [CONFIRMADO]
- `sites/pvgis/mod.rs:190`, `persistencia.rs:139-152`: `commit()?` sin `rollback()`; el staging queda en `Meteo\.staging`. [CONFIRMADO]
- `sites/mod.rs:95-98, 114`: `-wdf`, `-osf`, `-cof` al CLI sin `resolver_ruta` ni `exigir_archivo`. [CONFIRMADO]
- `easy_mode/simulacion.rs:184,190`: `extraer_kpis` lee `csv`, que `leer_kpis` nunca escribe; `kwp` siempre `null`. [CONFIRMADO]
- `easy_mode/simulacion.rs:202-213`, `parametros.rs:239-240`: rutas de salida sin alcance. [CONFIRMADO]
- `easy_mode/parametros.rs:169-171, 266-268`: plantilla y variante sin `validar_nombre`. [PLAUSIBLE]
- `easy_mode/dimensionado.rs:63-64, 77, 93`: `as u32` saturante; `ns_max = u32::MAX` itera unos 4.300 millones de veces; `ns0 + RANGO_SERIE_OFFSET` puede desbordar. [PLAUSIBLE]
- `history/validador.rs:14-19`: `validar_id` no rechaza `:`; `PathBuf::join("C:foo")` reemplaza la base. [PLAUSIBLE]
- `history/escritura_atomica.rs:135`: `fs::write` + `rename` sin `sync_all`. [PLAUSIBLE]
- `history/snapshot.rs:122-127`: id = ms + contador por proceso; dos procesos colisionan. [PLAUSIBLE]
- `components/ficha/inversor.rs:114-118`: `PNomConv` no numérico se escribe en el OND con factor 1.0 sin error. [CONFIRMADO]
- `components/ficha/pvsyst.rs:256-260`: el argumento `workspace` se ignora en la ruta PVsyst. [CONFIRMADO]
- `components/mod.rs:243-263`: `crear_componente` crea el directorio antes de `exigir_sobrescribir`. [CONFIRMADO]
- `server/tools/proyectos.rs:197-241`: `configurar` no pasa los pasos por `preparar_args`/`validar` (la doc dice que sí) y reconvierte cada error en `Invalid`. [CONFIRMADO]
- `server/tools/nativos.rs:165` frente a `restricciones_esquema.rs:86-99`: `gui_actuar.valores` es `Object` pero el esquema admite arrays. [CONFIRMADO]
- `server/tools/resultados.rs:112,125-127`: `include_text` y `archivo/report_pdf` cuentan `null` como presente, al revés que `validacion.rs`. [CONFIRMADO]
- `escena_3d/pedido/lectura.rs:9, 150-171`: `MAX_OBJETOS` (500) solo está en el esquema; `colision.rs:24-38` es O(n²). [CONFIRMADO]
- `escena_3d/pedido/informe.rs:76`: emite `origen: aproximacion_mcp` sin la compuerta `PVSYST_MCP_APROXIMACIONES`. [CONFIRMADO]
- `escena_3d/codec.rs:35`: `{:?}` imprime «Shd» en vez de «SHD». `codec.rs:121-144`: objetos sin `Version` reciben «4.0» y «$0000» fabricados. [CONFIRMADO / PLAUSIBLE]
- `vista_agente/vista.rs:91-102, 128-150`: un error de enumeración se trata como «ventana no abierta»; `gui_observar que:vista` sin ventana da `estado ok` con `entidad: null`. [CONFIRMADO]
- `gui_catalogo/aplicacion.rs:169-178`: el anfitrión se identifica por igualdad exacta del título en español. [CONFIRMADO]
- `gui_catalogo/aplicacion.rs:228-243` y `entradas/*`: se busca el botón por el texto «Sistema»; `hijo_visible_con_texto` toma el primero visible. [PLAUSIBLE]
- `projects/lifecycle/potencia_modulo.rs:30`: `.ok()` descarta cualquier error de `leer_modulo`. [CONFIRMADO]
- `projects/lifecycle/dimensionamiento.rs:54-59, 86-88, 169, 243-254`: `validar_n_strings` sin tope superior; `leer_kwp_opcional` ignora valores no numéricos; `edit_fields` puede devolver `not-found` y solo `baterias.rs:18` lo comprueba. [CONFIRMADO]
- `projects/lifecycle/componentes_variante.rs:21-33, 75-92`: se valida el valor completo (con ruta) pero se escribe solo el nombre base. [PLAUSIBLE]
- `projects/lifecycle/objetos.rs:28-41`: `campos_omitidos` cuenta también el ruido; las claves salen en orden alfabético (`serde_json` sin `preserve_order`). [CONFIRMADO]
- `projects/anadir/mod.rs:30-43`, `catalogo.rs:220`: `ruta_variante` sin `es_variante_valida`; `simulation/mod.rs:77-87`, `cache/key.rs:177`, `engineering/caso.rs:79`, `entregable.rs:193`, `comparacion.rs:81,149`: variante sin validar. [CONFIRMADO]
- `projects/gui/mod.rs:133-136`: tras «Guardar» se lee el archivo sin esperar el cierre del diálogo; un byte-idéntico da error falso. [CONFIRMADO]
- `projects/gui/dimensionado.rs:43`: `format!("{kwp}")` usa `.` decimal. [PLAUSIBLE]
- `simulation/results/parse.rs:37-49`: `column_total` ignora el paso temporal (subhora ×4 etiquetado kWh). [PLAUSIBLE]
- `simulation/results/parse.rs:8`, `report_pdf/parse.rs:9-23`: `parse::<f64>` acepta `NaN` e `inf`. [CONFIRMADO]
- `simulation/report_pdf/extract.rs:8,14, 88-118`: `decompress_to_vec_zlib` sin límite; los escapes `\n`/`\ddd` no se decodifican y los paréntesis anidados cortan la cadena. [CONFIRMADO]
- `simulation/engineering/caso.rs:104-110`: acepta `kwp=0` (inf serializa como `null`). [CONFIRMADO]
- `simulation/engineering/entregable.rs:178-179`: `.ok()` hace desaparecer secciones sin aviso. [CONFIRMADO]
- `simulation/engineering/incertidumbre.rs:51-59`: suma mensual por año sin comprobar 12 meses. [PLAUSIBLE]
- `simulation/preestimate.rs:193`: el gate de confirmación cuenta `n` ejecuciones y `preparacion.rs:169-175` cobra `EJECUCIONES_LOTE=2`. [CONFIRMADO]
- `simulation/results/mod.rs:47`: `size_bytes = text.len()` tras `from_utf8_lossy`. [CONFIRMADO]
- `simulation/estado.rs:19-34`: el mutex global `pendientes` se mantiene durante `finalizar()`. [CONFIRMADO]
- `workspace/rutas.rs:41-45`: `while !ancestro.exists()` salta enlaces colgantes; `ruta_segura` acepta un symlink roto fuera del workspace. [CONFIRMADO]
- `workspace/escritura.rs:72-89`: `validar_formato_pvsyst` no llama a `validar_nombre`; `CONIN$`/`CONOUT$` no están en `NOMBRES_RESERVADOS`. [CONFIRMADO / PLAUSIBLE]
- `workspace/recorrido.rs:77 vs 137`: límites de profundidad distintos (32 y 31). [CONFIRMADO]
- `admin/archive.rs:41-51, 108-122, 213-236`: el tope de 16 GiB usa tamaños declarados; el tope de 64 MiB se comprueba tras alocar la línea; los nombres van a `bsdtar` sin `--`. [PLAUSIBLE / CONFIRMADO]
- `admin/importacion.rs:98-118`: `recolectar` recursivo sin tope de profundidad. [PLAUSIBLE]
- `admin/proyecto.rs:14-17` frente a `projects/lifecycle/variantes.rs:14-19`: reglas de extensión de variante distintas. [CONFIRMADO]
- `admin/workspace.rs:63-65`: ruta absoluta (incluida UNC) en `crear_workspace` sin `Alcance`. [PLAUSIBLE]
- `horizonte_archivo`, `serie_horaria`, `meteo_mef`, `medicion`, `objects`, `prefs`:
  - `serie_horaria/decodificar.rs:137`: un `WRFactor` inválido pasa a 0 y luego al valor por defecto. [CONFIRMADO]
  - `serie_horaria/aplicacion.rs:176-186`: una matriz con filas = nº de variables se interpreta como columna×hora sin aviso. [CONFIRMADO]
  - `serie_horaria/aplicacion.rs:236`: variables repetidas, la segunda sobrescribe la primera. [CONFIRMADO]
  - `serie_horaria/aplicacion.rs:36-40`: `offset` y `limite` caen en silencio al valor por defecto. [CONFIRMADO]
  - `medicion/dominio.rs:115-119`: se omite la primera línea si su segundo campo no es numérico. [CONFIRMADO]
  - `meteo_mef/codec.rs:83,101-104,146,201`: conteo no numérico se salta la comprobación; `if c > 0` descarta la columna 0. [CONFIRMADO]
  - `prefs/diagnostico.rs:125-129`: `solo_errores` solo quita la cola, los `Warn` siguen. `19-32`: «más reciente» por orden lexicográfico. [CONFIRMADO / PLAUSIBLE]
  - `horizonte_archivo/aplicacion.rs:75-79, 85-87`: `if let Ok` oculta un error de confinamiento; el candidato `Shadings` no incluye `PVsyst8.1_Data`. [CONFIRMADO / PLAUSIBLE]
  - `objects/resolucion.rs:23-37, 51`: `flatten` sobre `read_dir`; `exists()` acepta directorios. [CONFIRMADO / PLAUSIBLE]

### Adapters (GUI, CLI, UIA, OCR)

- `gui/esperas.rs:67-91, 117-118`: si `SetWinEventHook` falla, `esperar_evento` devuelve `None` y el error es «timeout esperando». [CONFIRMADO]
- `gui/esperas.rs:122-131`, `campos.rs:173-176`, `uia_vista.rs:122-129`: esperas y profundidades sin tope superior; `timeout_ms` llega a 49 días; `as u32` trunca. [CONFIRMADO]
- `cli/proceso.rs:162-183`: `comando` y `opciones` van al argv sin lista blanca; las opciones que no son string ni objeto se descartan en silencio. [CONFIRMADO]
- `cli/trabajo.rs:31-41`: los trabajos terminados nunca se eliminan (cada `CliOutput` pesa hasta 2 MiB); con el lock envenenado se devuelve el id sin insertar. [PLAUSIBLE / CONFIRMADO]
- `gui/ffi/controles.rs:34,45,53-55`: `texto_control` trunca a 1024 unidades sin aviso. [CONFIRMADO]
- `gui/ffi/dpi.rs:111-115`: `escalar_a_dpi` devuelve `v` sin escalar en overflow. [CONFIRMADO]
- `gui/ocr/captura_ocr.rs:58`: `fs::read` del PNG sin tope. `70-71` y `ampliacion.rs:53-56`: con escala < 1 la región supera la imagen. [CONFIRMADO / PLAUSIBLE]
- `gui/ocr/seleccion.rs:14-22, 39-44`: compara etiquetas sin plegar acentos (`ubicacion.rs:11-32` sí). [CONFIRMADO]
- `gui/ocr/formulario.rs:226`: `declarar_dpi_por_monitor` en cada llamada aunque `dpi.rs:61-65` dice «una vez». [CONFIRMADO]
- `gui/resumen.rs:44`: cuenta solo `Button`, no `TBitBtn` ni `TPngBitBtn`. [CONFIRMADO]
- `gui/registro_pvsyst.rs:111`: `lines().map_while(Result::ok)` se corta en la primera línea no UTF-8. [PLAUSIBLE]
- `gui/mod.rs:86-88`, `gui/secuencia.rs:45-47`: todo error se reenvuelve como `Error::Gui`, perdiendo `Invalid` y `NotFound`. [CONFIRMADO]
- `gui/escritorio.rs:56-62`: una segunda llamada a `preparar` crea otro `HDESK` y lo descarta sin `CloseHandle`. [CONFIRMADO]
- `gui/captura.rs:176-187`: la ruta se valida después de capturar y el PNG ya está escrito si `textos_si_pedidos` falla. [CONFIRMADO]
- `uia/arbol.rs:157-164, 179-181`: la caché no expulsa entradas caducadas. [CONFIRMADO]
- `adapters/mod.rs:13-25`, `src/state/mod.rs:21,31`: caché de árboles en el estado global. [CONFIRMADO]

### Server, vista_agente, escena_3d

- `server/mod.rs:108`: `let _ = self.ejecutar_herramienta(...)`: un `tools/call` sin `id` ejecuta la acción y descarta el resultado. [PLAUSIBLE]
- `server/recetas/mod.rs:99`: `rellenar` sustituye secuencialmente; un valor con `{variante}` se vuelve a sustituir. [PLAUSIBLE]
- `server/tools/componentes.rs:47-51`: `origen` es obligatorio según la doc pero no está en `requeridos_por_accion`. [PLAUSIBLE]
- `server/tools/workspace_ejecucion.rs:131-139`: `es_proyecto` une `archivo` sin validar y consulta `exists()`. [PLAUSIBLE]
- `escena_3d/aplicacion.rs:20-23, 94-97`: `nombre` se une a `data_ro()` sin validar; el mensaje «PRUEBA tampoco guarda» es una afirmación no medida. [PLAUSIBLE]
- `vista_agente/plan.rs:103-117`: tolerancia relativa `EPSILON*|a|*8` muy estricta para decimales mostrados. [PLAUSIBLE]
- `vista_agente/esquemas/paneles.rs:14-58`: rangos y `clave_archivo` (por ejemplo `muISC` asignado a `mu_isc_ma_c`) sin fuente. [PLAUSIBLE]

### Violaciones claras de `agents.md`

- Archivos de más de 300 líneas, en `src/`: `sites/pvgis/mod.rs` (317), `sites/mod.rs` (312), `gui/uia_vista.rs` (312), `format/object.rs` (303), `components/mod.rs` (302). En `tests/`: `argumentos.rs` (388), `easy_mode/tests.rs` (371), `proyectos.rs` (359), `vista_tests.rs` (340), `protocol.rs` (318), `fixture_workspace.rs` (310), `workspace/tests.rs` (303).
- Tests dentro de `src/` fuera del enganche permitido: `server/tools/mod.rs:183-211` (`generar_bloque_alias_markdown`, `formatear_pares_markdown`) y `simulation/cache/store.rs:145-156` (`buscar_con_clave`).
- Funciones de más de 50 líneas: `history/validador.rs:44` (59), `history/restauracion.rs:62` (58), `easy_mode/sitio.rs:13` (58), `format/insert.rs:116` (55), `components/pan/legacy/dominio.rs:171` (52). Tests: `mcp_smoke/workspace_y_diseno.rs:54` (~99), `roundtrip.rs:38` (~82), `mcp_regresiones/workspace.rs:66` (~56).
- Funciones con más de 3 parámetros: `easy_mode/dimensionado.rs:40` (6), `sites/peticion.rs:6` (6), `simulation/engineering/entregable.rs:114` (7), `components/pan/legacy/edicion.rs:67` (5), `format/insert.rs:116` (5), `format/edit.rs:156` (5), entre otras.
- Estado global mutable: `gui/ffi/escritorio.rs:33` (`AISLADO: AtomicBool`) y `gui/escritorio.rs:33` (`ESCRITORIO: OnceLock`).
- Dependencias hacia fuera: `prefs/ini.rs:5,213` importa `adapters::gui::gui_running`; `components/ficha/pvsyst.rs:5-7` importa `adapters::gui`; `gui/ocr/dominio.rs:6` importa `ffi::Rect`; `gui_catalogo/dominio.rs:192-210` importa `crate::args`; `config.rs` mezcla `Paths` con `Arc<Estado>`.
- Nombres prohibidos: `server/recetas/datos.rs`, `components/pan/datos_modulo.rs`.
- `println!`/`dbg!`: ninguno fuera de `bin/fake_pvsystcli.rs` (`#![allow(print_stdout)]`).

### Duplicación con impacto

- Dos codificadores base64: `adapters/gui/base64.rs:1-40` y `workspace/base64.rs:1-79` (usos en `gui/captura.rs:3` y `workspace/lectura.rs:6`).
- Regla de variante en 4 sitios (`projects/mod.rs:32-47`, `lifecycle/variantes.rs:14-19`, `admin/proyecto.rs:14-17`, `doctor/mod.rs:100`, más `simulation/results/comparacion.rs:133`); listado de variantes en 3 (`projects/mod.rs:50-62`, `variantes.rs:63-78`, `doctor/mod.rs:87-108`); `mtime` en 4 (`projects/mod.rs:23-29`, `workspace/listado.rs:51-57`, `simulation/estado.rs:9-17`, `results/lookup.rs:15-21`).
- Dos `ruta_variante` con validación distinta: `lifecycle/variantes.rs:23-30` y `anadir/mod.rs:30-43`.
- `anadir/mod.rs:18-27` (`escribir_con_historial`) duplica `history::reemplazar`.
- En `format`: dos parsers del mismo formato (`object.rs:186-223` y `span.rs:110-149`) con semántica de cierre distinta; detección de CRLF repetida 5 veces (`edit.rs:166,226`, `insert.rs:132,186`, `seccion.rs:69`); `split/join/texto` idénticos en `insert.rs:20-30` y `delete.rs:11-21`; `indent_of` = `indent_width`; `tstart` idéntico en `object.rs:51` y `span.rs:81`; `sin_bom` en `insert.rs:14` y `components/ficha/mod.rs:27`; tablas de cierre en `cierre.rs:5-10, 28-33`.
- Parser numérico con coma decimal en 5 sitios: `components/buscar.rs:15`, `pan/mod.rs:26`, `sites/mod.rs:163`, `sites/solar/mod.rs:136, 156`.
- Rangos de lat/lon/tilt/azimut duplicados entre `easy_mode/parametros.rs:21-28, 142-194`, `sites/solar/mod.rs:36-51, 149-199` y `sites/entrada.rs:39-59`.
- Rutas de `Projects`, `PVmodules` y `Sites` como literal cuando existen `projects_dir()` y `modules_dir()` en `config.rs`.
- `CLASES_BOTON` ×3 con contenido distinto (`gui/acciones.rs:14`, `ocr/formulario.rs:21-28`, `vista_agente/cobertura/basica.rs:12`); `CLASES_TEXTO` divergentes (`campo_por_etiqueta.rs:14`, `ocr/formulario.rs:31`).
- `gui/proceso.rs:62-75` reimplementa `conduccion::esperar`; `menu_por_posicion.rs:92-103` y `proceso.rs:71-73` reimplementan `sondeo::sondear_hasta`.
- La lista de acciones de `gui` aparece en 4 sitios (`gui/mod.rs:47-48, 80-82, 85-113`).
- `server/tools/mod.rs:113-122`, `declaracion.rs:96-102, 140-143`, `validacion.rs:69-87`: la acción candidata se calcula 3 veces.
- Bloque de respuesta de variante repetido en 8 módulos: `almacenamiento/aplicacion.rs:36-44`, `bombeo/aplicacion.rs:31-38`, `evaluacion_economica/aplicacion.rs:32-39`, `inyeccion/aplicacion.rs:24-31`, `perdidas/aplicacion.rs:31-39`, `seguidores/aplicacion.rs:48-56`, `objects/edicion.rs:80-101`.
- `fn rango` casi idéntica en `bombeo/dominio.rs:36`, `evaluacion_economica/dominio.rs:40`, `perdidas/dominio.rs:36`, `seguidores/dominio.rs:77`, `sites/entrada.rs:52`.
- `validar_bomba` y `validar_bateria` iguales salvo la extensión (`bombeo/dominio.rs:48`, `almacenamiento/dominio.rs:86`).
- `ORIGEN_APROXIMACION` en `aproximaciones/mod.rs:17` y `vista_agente/contrato/construccion.rs:20`.
- `Sesion` de `simulation`: bloque `lock()` del mutex `pendientes` repetido 4 veces (`estado.rs:19, 38, 53`, `ejecucion.rs:91`).
- Tests: `Paths::resolve()` no hermético en unos 40 sitios; helpers `fake_env`, `FAKE`, `workspace`, `ws_str` ×3 en `tests/cli_fake/`; preparación de cliente repetida unas 30 veces en `mcp_regresiones/`; `vivo` duplicado (`tests/unit/vista_agente/mod.rs:16`, `cobertura/mod.rs:11`); `call` duplicado (`tests/unit/server/tools/tests.rs:6`, `vista_tests.rs:24`); `parametros`, `valor` y `cambios` idénticos en 5 módulos de dominio; cabecera PAN repetida 7 veces en `pan/tests.rs`; JSON base de `easy_mode/tests.rs` repetido 12 veces; `include_str!` del mismo fixture 11 veces en `clasificacion.rs`.

### Ineficiencia con impacto real

- `sites/nasa.rs:80-107`: `climatology` lee 3 rejillas completas en cada llamada (`sites/mod.rs:263`, `solar/mod.rs:130`, `posicion.rs:107`).
- `sites/reuse.rs:66-68`: `buscar_existente` lee cada `.SIT` completo, dos veces por diseño (`easy_mode/mod.rs:65`, `sites/mod.rs:224-225`).
- `admin/archive.rs:118, 182-193`: dos procesos `tar` (`-t` y `-tv`) sobre el mismo zip.
- `components/pan/mod.rs:172-204`, `ficha/modulo.rs:15-31`: leen y parsean completo cada `.PAN` solo para filtrar.
- Ocho `sleep` fijos iguales en `gui/conduccion.rs:142, 180, 200, 210, 226, 250, 265, 285`; `menu_por_posicion.rs:71, 81`; literales en `sesion_pvsyst.rs:107`, `projects/gui/horizonte.rs:36`, `projects/gui/mod.rs:60`, `projects/gui/p50.rs:39`; `gui_catalogo/aplicacion.rs:96` (5 s fijos).
- `server/respuesta.rs:33,36`: `value.clone()` y `to_string()`: dos copias de cada respuesta.
- `cli/proceso.rs:143-149`: `texto_sink` copia el stdout entero en cada crecimiento (hasta unas 20 veces por segundo). `cli/trabajo.rs:64-86` clona `CliOutput` en cada consulta.
- `gui/ventanas.rs:161-173`: `resolve` construye JSON completo de cada ventana solo para comparar título y clase.
- `config.rs:101-112, 182-191`: `data_ro()` y `resolve_gui()` recalculan entorno y `exists()` en cada llamada.
- `serie_horaria/decodificar.rs:170-195`: materializa 8.760 × N horas aunque se devuelvan 24 por defecto; `codificar.rs:98-99` clona un `Vec<i64>` de 8.760 elementos por variable.
- Lecturas enteras sin cota (unas 25), por ejemplo: `admin/componentes.rs:48`, `medicion/aplicacion.rs:74`, `horizonte_archivo/aplicacion.rs:17`, `serie_horaria/aplicacion.rs:97-106` (copia de `workspace::leer_bytes_acotados`), `simulation/engineering/caso.rs:88,93`, `simulation/cache/key.rs:100,124,186`, `projects/doctor/referencias.rs:14-25`, `projects/lifecycle/variantes.rs:120,175`, `creacion.rs:154-155`, `transacciones.rs:24`, `anadir/objetos.rs:89`, `catalogo.rs:226`, `prefs/diagnostico.rs:94,134`, `plantilla_variables/aplicacion.rs:55`, `components/ficha/inversor.rs:96`, `sites/readers.rs:17-27` (copia idéntica).
- `projects/doctor`: el mismo `var_txt`/`prj_txt` se parsea 4 o 5 veces por variante (`mod.rs:56,134`, `sintaxis.rs:88`, `electrico.rs:188`, `referencias.rs:82`, `seguidores.rs:17`).
- `simulation/results/lookup.rs:150, 212`: `sort_by_key` con `mtime_secs` (un `stat` por comparación).
- `simulation/avisos_pvsyst.rs:94, 102, 175`: deduplicación O(n²) con `Vec::contains`.
- `simulation/engineering/comparacion.rs:134-139, 163-168`: N+1: por cada variante se recorre `Results` y se relee PDF, CSV, PRJ y `variant_kwp`.
- `objects/listado.rs:26-31`: profundidad O(n²) sobre spans.

### Afirmaciones sin medir o versión

- «PVsyst 8.1.7» cableado en: `config.rs:108, 173-174, 187-188`, `vista_agente/esquemas/paneles.rs:1`, `gui_catalogo/entradas/inicio.rs:3`, `ayuda.rs:2`, `dialogos_variante.rs:3`, `dialogos_inicio.rs:3`, `escena_3d.rs:6`, `server/mod.rs:205`, `meteo_mef/aplicacion.rs:107` («8.1.2»). La memoria del proyecto sitúa las mediciones en 8.1.8 a 144 DPI.
- Anclas a 96 DPI sin declarar el DPI medido: `gui/conduccion.rs:6-7`, `components/ficha/pvsyst.rs:72-151`.
- Textos sin marca [INFERENCIA]: `gui/menu_por_posicion.rs:5-7`, `ffi/escritorio.rs:26-31`, `ocr/ampliacion.rs:12-13`, `captura.rs:71-73`, `ffi/dpi.rs:99-101`, `cli/clasificacion.rs:3`, `components/resolucion.rs:16`, `sites/readers.rs:13`, `pvgis/normalizacion.rs:10,22`, `easy_mode/diagnostico.rs:104`, `easy_mode/mod.rs:4-6`, `components/pan/legacy/edicion.rs:22-23`, `plantilla_variables/dominio.rs:12-13`.
- Comentarios falsos o desfasados: `server/tools/proyectos.rs:216-217`, `gui_catalogo/entradas/inicio.rs:3-6`, `vista_agente/cobertura/mapa.rs:22-23`, `components/buscar.rs:1-2` (cita `pan.rs` que no existe), `pan/legacy/dominio.rs:144-157`, `gui/ffi/puntos.rs:68` («el release siempre se envía»), `gui/escritorio.rs:61`.
- Idioma: `cli/clasificacion.rs:9-32` (`ERROR_MARKERS` en inglés sin versión medida), `simulation/preparacion.rs:57-58` frente a captions en inglés en `report_pdf/*`, `evaluacion_economica/aplicacion.rs:23-27` (`taFixed` sensible a mayúsculas), `aproximaciones/mod.rs:41-43` (comparación de rutas sensible a mayúsculas), `projects/gui/mod.rs:53` (`ends_with`).

### Privacidad

- `main.rs:21-25`: `info!` registra el workspace (`C:\Users\<usuario>\...`).
- `tests/unit/adapters/gui/escritorio.rs:27,31`: ruta de usuario de Windows con un nombre de persona (`C:\Users\<nombre>\Projects\...`). Si es un nombre inventado, no hay nada que hacer; si no, sustituirlo por uno ficticio.
- `server/recetas/datos.rs:59,62`, `tests/unit/sites/solar/tests.rs:14-20`, `simulation/preestimate.rs:3-9`: coordenadas fijas que podrían ser de un proyecto real; `tests/unit/simulation/cache/mod.rs:53` (nombre de un `.MET` de una localidad concreta), `tests/unit/projects/temporales.rs:9` (nombre con caracteres no ASCII). Comprobar si son datos reales y, de serlo, cambiarlos por ficticios.
- `adapters/http.rs:18`: `tracing::info!(url)` registra lat/lon del sitio. `gui/registro_pvsyst.rs:73`, `gui/proceso.rs:129`: devuelven rutas de usuario en respuestas. `simulation/engineering/entregable.rs:149`: escribe la ruta absoluta del PDF en el markdown.
- `bin/fake_pvsystcli.rs:95-115`: escribe en `%LOCALAPPDATA%\PVsystCLI\8.1\Admin` del usuario real.
- `prefs/ini.rs:79-85`: la máscara solo cubre key/password/token/secret. [PLAUSIBLE]
- `args.rs:113-115`: el argumento `workspace` reemplaza la raíz de confinamiento de `Alcance::Workspace`. Está documentado como decisión (`args.rs:109-112`), pero conviene aceptarlo de forma explícita.

---

## 3. Tests: otros hallazgos

- Nombres sin `should_X_when_Y`: unos 250 a 300 en casi todos los archivos. Casos casi totales: `easy_mode/tests.rs` (24 de 24), `cli_fake/ejecucion.rs` (11 de 11), `fixture_workspace.rs` (13 de 13), `protocol.rs` (22 de 24), `protocol_prompts.rs`, `roundtrip.rs`, `mcp_smoke/*`, `simulation/report_pdf/mod.rs` (12), `adapters/ledger.rs` (5), `simulation/cache/mod.rs` (5), `prefs/ini.rs` (4).
- `is_err()`/`is_ok()` sin motivo ni valor en decenas de sitios: `divisas/tests.rs:28-122`, `escena_3d/tests.rs:29-139`, `history/tests.rs:161-174`, `history/restauracion.rs:29-36`, `history/tests_externos.rs:18-21`, `gui_catalogo/mod.rs:170-191`, `meteo_mef/tests.rs:58-162`, `workspace/tests.rs:196-239`, `workspace/escritura.rs:50`, `sites/*`, `simulation/*`, `adapters/gui/secuencia.rs`, `campos.rs:43-46`.
- Tests que repiten la implementación: `format/insert.rs:139-158` (`reindentado`), `projects/doctor/electrico.rs:57-61`, `sites/solar/tests.rs:91-95`, `components/pan/legacy_edicion_tests.rs:40-73`.
- Nombres que prometen más de lo que prueban: `serie_horaria.rs:140` («verify_with_history»), `workspace.rs:173` («restore_replaced_files»), `diseno.rs:111` («at_the_sizing_boundary»), `mcp_smoke/proyectos.rs:146`, `unit/progreso/mod.rs:8-12` («silent»), `unit/sites/pvgis/tests.rs:130-166`, `unit/adapters/gui/proceso.rs:4-10` (`comprobar_proyecto_libre` no consulta ningún estado), `escena_3d/pedido.rs:131-134`.
- PVsyst real en tests unitarios (deberían ser `#[ignore]` o fixtures): `divisas/tests.rs:125`, `meteo_mef/tests.rs:166-199`, `projects/anadir/catalogo.rs` (7), `projects/anadir/horizonte.rs`, `horizonte_archivo/tests.rs:8-14`, y los `#[ignore]` de `aislado.rs`, `almacenamiento.rs`, `bombeo.rs`, `economia.rs`, `inyeccion.rs` que copian desde `C:\Program Files\PVsyst8.1.7`.
- Proptest sin semilla fija: `args.rs:125-146`, `format/*`, `calendario/mod.rs`. [PLAUSIBLE]
- `unit/history/tests.rs:176-223`: 16 hilos renombrando el mismo destino. [PLAUSIBLE]
- Limpieza con `let _ = remove_dir_all` en lugar de `tempfile`: `history/*`, `components/ficha/tests.rs`, `adapters/ledger.rs:6-10` (no limpia nunca), `simulation/cache/mod.rs`, `mcp_smoke/proyectos.rs`, `roundtrip.rs`.
- Fixtures sin referencia: `tests/fixtures/cli/export-logs-forbidden.txt`, `run-simulation-missing-project.txt`, `tests/fixtures/results/p50_tokens.json`. `tests/medicion.rs:23-27` copia `MCP_E2E.pdf` que ningún test usa; `cliente_mcp/mod.rs:29-45` (`protocol_server`, `protocol_call`) sin llamadores, ocultas por `#![allow(dead_code)]`.
- Ramas sin test (por lectura): `plantilla_variables` («variable ya presente»), `prefs` `FiltroLog::Todo`, `ledger` (banner sin días, modo licencia), `clasificar_salida` con salida vacía, `kind()` de 7 variantes de `Error`, límites válidos de bombeo, inyección y economía, temporal reciente en `limpiar_temporales_abandonados`, `crear_componente` (nombre inválido, sobrescritura), `borrar_componente` feliz, `eliminar_objeto` con selector ausente, `resolver_proyecto` feliz, `ya_abierta`.
- Funciones de la parte pura de los flujos GUI sin test: `sondeo::sondear_hasta`, `ffi/menus::tipo_de`, `menu_por_posicion::{es_menu_de_sistema, exigir_menu_valido, item_json}`, `cli/salida_json::{tail, filtrar_cuerpo_stdout, enriquecer_salida_json}`, `ventanas::{target_pids, resolve, matches}`, `ocr/captura_ocr::trasladar_linea`, `ffi/puntos::empaquetar` (coordenadas negativas), `uia/com::{resultado_accion, es_fin_de_lista}`, `sesion_pvsyst::formularios_de_usuario`, `recuperacion::fallo_de`.
- Tests con IO y disco real en `tests/unit` (la regla pide que sean sin IO): `history/*`, `workspace/tests.rs`, `escena_3d/tests.rs:93-118`, `components/tests.rs`, `admin/tests.rs`, `adapters/ledger.rs`, `horizonte_archivo/tests.rs:196-280`.
- Números mágicos en tests: `8760` (`serie_horaria.rs`), `64 MiB + 1` (`mcp_regresiones/workspace.rs:40`), conteos 9, 10, 82, 16 y 15 (`server/recetas.rs`, `cobertura/mod.rs`, `esquema_tests.rs:6`), `122` (`plantilla_variables/aplicacion.rs:19`).

---

## 4. Las 15 correcciones de mayor retorno

1. Un solo punto de resolución de rutas (`resolver_ruta` o `ruta_segura`) para A1, A2, A3, A5, A6, A7, A9. Cierra casi todas las fugas del workspace.
2. Validar `extra_options` y los campos de `batch` (A4, A8).
3. «Ensayo» según la salida de la acción, no según los argumentos (B1).
4. Cerrar el éxito silencioso con el mismo patrón en B2 a B8.
5. Teclado y `resolve`: exigir ventana o proceso de PVsyst y comprobar el primer plano antes de pulsar (C1, C2, C3).
6. `args_con_salidas` en `preparar`, el plan y la caché; `meta.json` atómico; fallo de caché que no tumbe la simulación; `-s` en la clave (D5, D6, D7).
7. Procesos hijo: `kill`/`wait` en cada salida de error, `stderr` en un hilo con plazo, lock antes de lanzar el job (D1, D2, D3, D4).
8. `easy_mode`: abortar si falla el sitio, exigir `realised_kwp`, no borrar de `Projects` lo que no creó la corrida (B5, E1).
9. Tests que pasan vacíos: `#[ignore = "motivo"]` o precondición que falle (F1 a F5).
10. `Paths::para_test` en vez de `Paths::resolve()` en los tests, y un único `Client::con_workspace` en `cliente_mcp`.
11. Unificar duplicados grandes: base64, regla y listado de variantes, helpers de `format`, parser numérico.
12. Topes: `--max-filesize`, descompresión con límite, `leer_bytes_acotados` en las lecturas sin cota.
13. Quitar del catálogo las funciones «no explorado» con efecto Modifica (C6).
14. Quitar los fallbacks sin ruta de `curl.exe`, `pvsystcli` y `PVsyst.exe` (A11).
15. Partir los archivos de más de 300 líneas y los tests con varios comportamientos.

## 5. Código muerto y simplificable (candidatos)

Verificar cada uno con `rg` antes de borrar.

- `adapters/cli/captura.rs:66`: `CapturaFlujo.error_io` solo se escribe (`state/registros.rs:14`). [CONFIRMADO]
- `plantilla_variables/mod.rs:9-13`: re-exportaciones sin llamadores (`anadir_variables`, `buscar_variable_valida`, `eliminar_variables`, `parse_rvt`, `reordenar_variables`, `serializar_rvt`, `VARIABLES_VALIDAS`) ocultas por `#[expect(unused_imports)]`. [CONFIRMADO]
- `projects/temporales.rs:59-85`: el `HashSet vistos` nunca deduplica. [CONFIRMADO]
- `components/ficha/modulo.rs:57-59`: rama `not-found` que nadie produce. [CONFIRMADO]
- Parámetros `_paths` sin uso: `gui/acciones.rs:190`, `ventanas.rs:210,240,270`, `campos.rs:83,118,171`, `proceso.rs:24`, `teclado.rs:93-94`, `simulation/preestimate.rs:105`, `sites/solar/posicion.rs:82`, `prefs/ini.rs:162`, `prefs/diagnostico.rs:119`.
- Wrappers que solo reenvían: `gui/acciones.rs:178-188` (`escritorio_interactivo`), `gui/campos.rs:78-80`, `tests/unit/escena_3d/tests.rs:12-15`, `divisas/tests.rs:12-14`, `meteo_mef/tests.rs:13-15`.
- `gui/rotulos.rs:3-7` y `components/ficha/pvsyst.rs:43-58`: `Idioma` con una sola variante.
- `admin/archive.rs:87-104`: `LimitesListado` de un solo uso. `escena_3d/dominio.rs:89-196`: pares de structs con los mismos campos. `evaluacion_economica/dominio.rs`, `inyeccion/dominio.rs`, `seguidores/dominio.rs`: enums de dos valores donde basta un `bool`.
- Ramas inalcanzables: `horizonte_archivo/codec.rs:55-61, 156-164`, `divisas/codec.rs:18-21`, `medicion/dominio.rs:33-35, 80`, `vista_agente/cobertura/basica.rs:80-82`, `uia/com.rs:194-201`, `simulation/cache/key.rs:45-67`.
- `pub(crate)` con uso solo interno: `gui/png.rs:64` (`crc32`), `cli/salida_json.rs:13`, `uia/arbol.rs:36`, `gui/ffi/dpi.rs:49`, `cli/captura.rs:15`, `components/ficha/inversor.rs:75`, `sites/solar/posicion.rs:19`, `sites/solar/produccion.rs:23`, `sites/nasa.rs:34`, `easy_mode/diagnostico.rs:110`, `sites/pvgis/mod.rs:294`, `pvgis/adquisicion.rs:9,19`, `pvgis/normalizacion.rs:10,13`, `components/pan/legacy/dominio.rs:10,19`, `plantilla_variables/dominio.rs:9`. Comprobar los tests antes de bajar la visibilidad.
- `format/selector.rs:17-21`: `impl Display for Seg` sin uso aparente. [PLAUSIBLE]
- Clippy pedántico: `map_unwrap_or` ×10, `unnecessary_wraps` ×8, `unused_self` ×3, `struct_excessive_bools` ×1, 69 avisos `cast_possible_truncation` (140 `as` numéricos en `src`). Candidatos a `clippy --fix`.
- 45 `let _ =` y 121 `.ok()` en `src`: los que ocultan fallos de E/S o de procesos merecen `tracing::warn!` (por ejemplo `admin/archive.rs:82`, `history/escritura_atomica.rs:81,111`, `history/snapshot.rs:173`, `easy_mode/proyecto.rs:199,202`, `sites/mod.rs:185`, `components/pan/mod.rs:163`, `simulation/cache/store.rs:26`).
- Magia: `0xF000` (`menu_por_posicion.rs:216`), `TEXT_BUFFER_LEN=512` (`ventanas.rs:8`), `"PVsyst no respondió en 3 s"` ×8 en `ffi/controles.rs`, `WindVel` 2.0 (`sites/nasa.rs:129`), `1000.0`, `0.05`, `800`, `20`, `25` (`sites/solar/produccion.rs:81-96`), `1440`/`60` (`pvgis/normalizacion.rs:93`), códigos 5 y 32 (`history/escritura_atomica.rs:154`), `2777.778` (`serie_horaria/dominio.rs:12-22`), `indent + 2` (`format/insert.rs:140`, `edit.rs:120`).

## 6. No comprobable sin PVsyst

Entrega real de teclas y clics (foco, UIPI, DPI 96 y 144), textos reales de botones y títulos («Salir» frente a «Salida»), comportamiento de HMENU ajenos, flujos de «Guardar», dimensionado, horizonte y P50, el formato del listado de `tar -tv` (índice `nth(4)`), la tolerancia de «distinto» en `rellenar`, la versión 8.1.7 frente a 8.1.8, el paso temporal de los CSV subhora, los rangos y `clave_archivo` de `esquemas/paneles.rs`, y el comportamiento real de `Path::join` en Windows (razonado desde la documentación de `std`, no ejecutado).
