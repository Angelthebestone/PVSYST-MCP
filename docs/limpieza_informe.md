# Informe de la limpieza, paso 1 (§8 del handoff)

Base: `585d2f7` (lints activos, línea base en `docs/limpieza_linea_base.md`, medida en `b4149ac`).
Final: el commit que añade este informe. Todo lo de abajo está medido sobre el árbol final, salvo lo
marcado como `[INFERENCIA]`.

## 1. Antes y después
| Medida | Antes | Después |
|---|---|---|
| Líneas `src/` | 35.523 | 37.032 |
| Líneas `tests/` | 12.522 | 13.188 |
| Archivos `src/` > 300 líneas | 3 | 1 (`sites/pvgis/mod.rs`, paso 2) |
| Archivos de tests > 300 líneas | 5 | 1 (`unit/easy_mode/tests.rs`, paso 2) |
| Funciones > 50 líneas | 0 | 0 |
| Avisos de clippy (indexación, anidamiento, > 3 parámetros) en las zonas | 113 | 0 |
| Los mismos avisos en el resto del crate | ≈516 | sin cambios: 32 `#[expect(…paso 2…)]` |
| `#[allow]` en `src` | 5 | 1 (`bin/fake_pvsystcli.rs`: su stdout es el contrato) |
| `.unwrap()`/`.expect(` en producción | 0 (1 en un doctest) | 0 (igual) |
| Bloques `unsafe {` (total / en zonas) | 72 / 57 | 68 / 55 |
| Bloques de lógica duplicada (≥ 3 líneas) en las zonas | 34 | 12 (ver §3) |
| Tests de librería | 613 + 14 ignorados | 628 + 14 ignorados |
| Funciones `#[test]` en `tests/` (unitarios + integración) | ≈737 | 755 |

`src` crece porque se añadieron una tabla de sinónimos por tool, tablas de rótulos y envoltorios FFI
con su `SAFETY`, y la validación de entradas que antes podía entrar en pánico. Con los mismos commits
desaparecieron unas 250 líneas duplicadas o muertas.

## 2. Rendimiento (release, escritorio aislado, PVsyst 8.1.7)
| Camino | Antes | Después | Presupuesto |
|---|---|---|---|
| `gui_observar que:formulario` (ya leído) | 0,79–0,81 s | 0,78–0,86 s | < 1 s |
| `gui_observar que:vista` | ≈30 ms | ≈30–47 ms | < 100 ms |
| `gui_actuar rellenar`, 4 campos reales | 5,1 s | 3,2–4,1 s | < 1,5 s por campo |
| `rellenar` con `simular:true` | 22 ms | 31 ms | < 100 ms |
| Lectura de `.PAN` / MEF / escena 3D | — | 0,54–0,73 ms | < 2 ms |
| Tamaño del `.exe` | 4.149.248 B | 4.177.920 B | perfil sin cambios |

No se optimizó nada: ninguna medición señaló un cuello de botella. El handoff decía ≈0,4 s para
`formulario`; aquí se midió 0,8 s antes y después.

## 3. Simplificación y duplicación
- **Quitado:**
  - envoltorios: `RectOcr` (igual que `ffi::Rect`), `send_message`/`send_message_res` (tragaban el
    timeout), `SINONIMOS_VACIOS` y la función muerta `pais_simulado`;
  - DTO de un solo uso: `PuntoBarra`, `DestinosPreparacion`, `DescriptoresFuncion`;
  - el método `Paths::con_aproximaciones(bool)`;
  - tres reexports sin uso en `serie_horaria`;
  - unos 15 alias sueltos en llamadas a `req_str`/`opt_str`, más los de `sites`, `easy_mode` y
    `simulation`.
- **Unificado:**
  - todos los `SendMessageTimeoutW` en un envoltorio, con un `SAFETY` y una constante;
  - `Toolhelp` en `ffi/procesos.rs`, con una sola función segura `pids_de_ejecutable`;
  - el sondeo con plazo en `sondeo.rs`;
  - `json_identidad_ventana`, `hijos_visibles`, `leer_control`, `nodos_sin_hwnd`,
    `LineaOcr::con_rects`, `Discriminador::extraer`, `extraer_aliases`, `extension_mayuscula`,
    `truncar_cadena`, `valor_no_nulo`, `validar_rango`;
  - `Paths::para_test`, que sustituye 8 copias de `Paths { … }` en los tests.
- **Se conservan, con motivo:**
  - `Mensaje`: el concepto Win32 (id, wparam, lparam), usado desde 8 módulos;
  - `OrigenSalida`, `EstadoMensaje`, `ModoEjecucion`: evitan pasar un `bool` como parámetro;
  - `ContextoRecorte`, `SolicitudEspera`;
  - las tablas de rótulos con `Idioma` (§4b.7).
- **Duplicados que quedan (12 ventanas):** cierres de bloque, dos accesores de `escena_3d/dominio`
  y dos ramas del códec 3D (los códecs no se reestructuran, §2).

## 4. Tests podados
| Test | Motivo |
|---|---|
| `simulated_country_comes_from_results_csv` | probaba código muerto (`pais_simulado`) |
| `should_reject_invalid_discriminator_value` | duplicado de `should_reject_unknown_discriminator_value_and_list_admitted` |
| `should_expose_click_by_text_restart_and_menu_listing_in_the_gui_tools` | aserciones ya cubiertas por `should_expose_the_new_gui_field_actions` |
| 4 tests `should_return_none_when_label_*` | mismo cuerpo; ahora es un test con 4 casos |
| `should_not_panic_when_typing_or_changing_page_on_invalid_handle` | «no falla»; ahora `should_fail_when_…` comprueba el error |
| aserción duplicada de la tabla de alias del manifiesto | dos tests comprobaban lo mismo |
| (sin borrar, reescritos) `should_reject_an_empty_request`, `should_reject_truncated_3d_file`, `should_return_only_that_function_when_key_matches_exactly`, `should_carry_no_user_paths_…` | ahora comprueban el error o el comportamiento, no datos fijos; el último tenía una condición siempre verdadera |

## 5. Bugs
| Zona | Síntoma | Causa | Test | Commit |
|---|---|---|---|---|
| gui/captura | pánico al recortar una ventana de tamaño 0 | `clamp(0, ancho-1)` con ancho 0 | `should_handle_zero_dimensions_without_panic` | `b207ee6` |
| gui/captura, png | pánico `chunks(0)` al capturar un control de tamaño 0 | el recorte vacío llegaba a `png_rgb` | `should_reject_zero_dimensions_without_panic` | `ded4aa1` |
| gui/ffi | el servidor podía colgarse si PVsyst no respondía | `SendMessageW` sin límite | no reproducido en vivo | `769c3d6` |
| gui/ffi | un timeout se tomaba como éxito o como `0` (marcado falso, teclas perdidas) | `unwrap_or(0)` y descarte del resultado | `should_fail_when_typing_or_changing_page_on_invalid_handle` | `af80292` |
| sites, easy_mode | los tests dejaban `MCP_*_Site_Monthly.csv` y `mcp_cache` en `PVsyst8.1_Data` | `workspace` no se propagaba a la llamada interna | `should_write_ledger_cache_to_requested_workspace_not_server_default` (falla en la base) | `abbe31f` |
| tests | los tests resolvían el PVsyst y el PVsystCLI reales | el cliente de tests no fijaba `PVSYST_GUI`/`PVSYST_CLI` | `default_client_rejects_launching_gui_with_missing_executable` | `d3e1ba2`, `6edde24`, `3c8ab38` |
| server/tools | los alias en inglés nunca llegaban al código | se validaba antes de normalizar | `should_accept_file_as_alias_for_archivo_in_leer` (falla en la base) | `e1cc0a1` |
| server/tools | `sitio {source}` fallaba con «Origen no válido» | el alias `source` tapaba una propiedad declarada | `should_not_corrupt_declared_property_source_in_sitio` | `cefa85c` |
| simulation | dos `-ocf` en el comando y la caché seguía otro archivo | los alias partidos en filas sueltas | `should_normalize_simulation_short_flags_and_reject_conflicting_values` | `b53a883` |
| sites, easy_mode | alias sueltos sin validar ni anunciar | quedaban fuera de la tabla | `should_accept_loose_aliases_in_sitio_and_disenar_sistema` | `a534601` |
| server/tools | las acciones aproximadas seguían anunciadas sin el flag | la descripción se generaba antes de filtrar | `should_hide_in_house_actions_without_the_flag` | `3c1e971` |
| server/tools | Gemini rechazaba toda la lista de tools (400) | arrays sin `items` | `should_declare_items_for_all_array_schemas` | `6edcd1c` |
| mcpb | la descripción de `proyecto` no coincidía con el código | se mantenía a mano | `should_match_manifest_tools_and_declarations` | `afb0e14` |
| mcpb | `mcpb validate` fallaba | clave `tool_aliases` no admitida en la raíz | `mcpb validate` | `d4cfc96` |
| scripts | `verificar.ps1` se cortaba en silencio y no fallaba con clippy | manejadores de eventos fuera del runspace (PS 5.1) | ejecución completa | `50d7a68` |
| scripts | `verificar.ps1` nunca mataba los PVsyst huérfanos | `$pid` es de solo lectura | lanzar PVsyst y llamar a `Kill-NewPVsystPids` | `9b0af3b` |

**No reproducido:** el cuelgue del `cargo test` completo (§4b.8). Ningún test no ignorado acciona la
GUI. La causa más probable es que los tests resolvieran el PVsyst real y se engancharan a uno abierto
`[INFERENCIA]`; eso está corregido.

## 6. Reglas de AGENTS.md que siguen sin cumplirse
- **Resto del crate (paso 2):** unos 516 avisos de indexación, anidamiento y parámetros en 32 módulos;
  `sites/pvgis/mod.rs` (323 líneas) y `tests/unit/easy_mode/tests.rs` (312). Están bloqueados hasta el
  handoff de vista 3D (§1).
- **`#[expect(too_many_arguments)]` en `CampoEsquema::numero`/`entero`:** el rango es obligatorio en
  el constructor para que el tipo no quede sin validar.
- **Estado global** (`AISLADO`, `ESCRITORIO`, `thread_local!` de `esperas.rs`):
  - el modo de escritorio es fijo para todo el proceso;
  - el callback de `SetWinEventHook` no admite un puntero de contexto.
  - Está documentado en el código.
- **Parámetros `bool` fuera de las zonas:** `resources`/`recetas` (`aproximaciones`), `de_bool` de
  varios dominios y `parsear_bool_opcional`. Quedan para el paso 2.
- **Nombres en inglés:** 172 funciones; propuesta y coste en `docs/convencion_nombres.md`.
- **Salidas y errores solo en español:** quedan fuera de alcance hasta el contrato `vista` (§4b.15).

## 7. Idempotencia (§4b.10)
| Operación | Al repetirla |
|---|---|
| `escribir_archivo` | mismo resultado; snapshot previo |
| `componente crear` | reemplaza con snapshot; idempotente en contenido |
| `componente borrar` | segunda vez `NotFound`; no corrompe |
| `proyecto crear`, `workspace crear` | error si existe; idempotente con `sobrescribir:true` |
| GUI `rellenar` | escribe los mismos valores; idempotente |
| GUI `funcion` | abre otra ventana si ya estaba abierta [INFERENCIA]: no es idempotente, pero no corrompe datos |

## 8. Verificación final
- `scripts/verificar.ps1`: `fmt`, `clippy -D warnings`, `test --lib`, los 16 binarios de integración y
  los doctests en OK, sin PVsyst abierto al empezar y sin huérfanos al terminar.
- **Release en vivo (escritorio aislado):** se lanzaron 4 instancias de PVsyst, matadas al terminar.
  - `lanzar`, `funcion bases_de_datos.modulos_pv.nuevo`, `formulario`, `vista`, `capturar` y
    `rellenar` con 4 campos reales: correctos.
  - `menus` responde con su error documentado en escritorio aislado.
  - No se pulsó Guardar ni nada equivalente.
- `tools/list`: ningún array sin `items`. `mcpb validate` en OK.
- **Revisión externa `cmdc`:** 13 hallazgos (7 de servidor, 6 de GUI), todos comprobados contra el
  código y aplicados. Los dos sobre comentarios `SAFETY` (búferes y guardas `IsWindow`) se resolvieron
  reescribiendo el comentario: `IsWindow` es una comprobación con carrera, no una garantía.

## 9. Decisiones para el usuario
- Publicar los commits (no se ha subido nada).
- El renombrado al español, según `docs/convencion_nombres.md`.
- El paso 2, después del handoff de vista de agente + 3D.
