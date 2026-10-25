# Línea base de la limpieza (paso 1: zonas dirigidas)

Medida el 2026-10-22 sobre `b4149ac`, antes de tocar código. Herramientas: `cargo clippy 0.1.99`,
recuento de líneas y `grep` sobre `src/` y `tests/`.
Resultado del paso 1 (antes y después, bugs y pendientes): `docs/limpieza_informe.md`.

## Tamaño
| Medida | Valor |
|---|---|
| Líneas `src/` | 35.523 |
| Líneas `tests/` | 12.522 |
| Archivos `src/` > 300 líneas | `sites/pvgis/mod.rs` (323), `gui_catalogo/aplicacion.rs` (306), `adapters/gui/ffi/controles.rs` (305) |
| Archivos de tests > 300 líneas | `mcp_smoke.rs` (743), `mcp_regresiones.rs` (484), `unit/server/tools/tests.rs` (420), `cli_fake.rs` (332), `unit/easy_mode/tests.rs` (312) |
| Tests de librería | 613 pasan, 14 ignorados (requieren PVsyst) |

## Zonas del paso 1
| Zona | Archivos | Líneas `src` | Tests | Líneas de test |
|---|---|---|---|---|
| `server/tools` | 14 | 1.945 | 21 | 420 |
| `vista_agente` | 9 | 768 | 11 | 136 |
| `adapters/gui` (con `ffi/`, `ocr/`) | 37 | 5.339 | 75 | 937 |
| `gui_catalogo` | 14 | 1.720 | 20 | 231 |
| `escena_3d` | 5 | 586 | 11 | 170 |

## Reglas mecánicas (clippy como medidor)
Lints añadidos: `indexing_slicing`, `excessive_nesting`, `needless_pass_by_ref_mut`;
`clippy.toml`: `too-many-arguments-threshold = 3`, `excessive-nesting-threshold = 3`,
`allow-indexing-slicing-in-tests = true`.

**Por qué el umbral de anidamiento es 3 y no 1 ni 2** (calibrado en un crate de prueba): clippy cuenta el
bloque `impl` como un nivel. Con 2, un método con un solo `if` (un nivel de indentación, conforme a AGENTS)
ya se marca; con 1, cualquier `if` dentro de una función libre. 3 equivale a «un nivel» dentro de métodos y
deja dos en funciones libres. Con 1 salían 1.839 avisos; con 2, 666; con 3, 168.

| Hallazgo (todo el crate) | Avisos |
|---|---|
| Anidamiento > umbral 3 | 168 |
| Indexación que puede entrar en pánico (fuera de tests) | ≈410 |
| Funciones con > 3 parámetros (`self` cuenta) | 68 |
| Funciones > 50 líneas | 0 (`too_many_lines` ya activo) |

En las zonas del paso 1 quedan 113 avisos (los de cada archivo, abajo). El resto del crate lleva un
`#[expect(..., reason = "…paso 2…")]` en su declaración `mod`: el compilador avisará cuando se limpie.

| Zona | Indexación | Anidamiento | Parámetros > 3 |
|---|---|---|---|
| `adapters/gui` | 30 | 19 | 6 |
| `escena_3d` | 21 | 0 | 0 |
| `vista_agente` | 10 | 1 | 3 |
| `server/tools` | 2 | 11 | 1 |
| `gui_catalogo` | 1 | 1 | 1 |

## Otras medidas
| Medida | Valor |
|---|---|
| `.unwrap()`/`.expect(` en `src` | 1, en un doctest de `server/mod.rs` (no es producción) |
| `#[allow]` en `src` | 5 (1 justificado en `bin/fake_pvsystcli.rs`; 3 `unused_imports` en `serie_horaria/mod.rs`; 1 `dead_code` en `simulation/engineering/caso.rs`) |
| `panic!`/`unreachable!` en `src` | 0 |
| `print*!` en `src` | 10, todos en `bin/fake_pvsystcli.rs` (stdout es su contrato) |
| Bloques `unsafe {` | 72 |
| Funciones con parámetro `bool` | 16; 5 en zonas: `menu_por_posicion::salir_de_la_barra`, `ffi::escritorio::{fijar_aislado, tecla_emulada}`, `registro_pvsyst::acumular`, `vista_agente::relleno::resultado_del_paso` |
| Estado global mutable | `AISLADO: AtomicBool`, `ESCRITORIO: OnceLock`, `thread_local!` en `esperas.rs`; contadores atómicos en `history/` y `sites/pvgis/staging.rs` |

## No medido con herramienta
- Números mágicos y duplicación: sin lint fiable; se revisan a mano por zona.
