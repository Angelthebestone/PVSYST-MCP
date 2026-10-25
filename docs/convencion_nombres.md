# Convención de nombres: propuesta (§4b.13)

**Estado:** propuesta aprobada por el usuario (español); el renombrado está pendiente y no se ha hecho.
Mientras tanto, ningún módulo recibe nombres nuevos en inglés.

## Regla
1. **Identificadores propios del crate en español:** funciones, tipos, módulos, constantes y campos.
   Sin tildes ni `ñ` en identificadores (`anadir`, `pagina`), como ya hace el código.
2. **Se quedan como están:**
   - nombres impuestos por fuera: traits de `std`/`serde` (`new`, `from`, `fmt`, `default`, `as_str`);
   - FFI de Win32 (`SendMessageTimeoutW`, `PROCESSENTRY32W`) y sus campos `#[repr(C)]`;
   - claves de formatos de PVsyst (`PNom`, `Technol`);
   - nombres de argumentos CLI de PVsystCLI (`output_csv`, `-ocf`);
   - API pública del crate (`Server`, `Paths`, `serve`, `handle_line`): renombrarla es un cambio de
     contrato y se decide aparte.
3. **Contrato MCP:** los nombres de tools y de argumentos ya son españoles; los alias en inglés viven en
   la tabla de sinónimos (`src/server/tools/declaracion.rs`).

## Coste medido (2026-10-24)
Recuento de funciones `fn` con alguna palabra inglesa en el nombre, sin contar los nombres impuestos
por traits:

| Medida | Valor |
|---|---|
| Funciones con nombre inglés | 172 |
| Referencias a esas funciones (src + tests) | ≈1.600 |
| Del total, en las zonas del paso 1 | 53 funciones, ≈430 referencias |
| Tipos con nombre inglés (sin FFI) | ≈10 (`CliOutput`, `ParseState`, `LegacyHeader`, `ItemMenu`…) |

Las que más se usan en las zonas del paso 1: `resolve` (75), `class_of` (50), `rect_of` (23),
`text_of` (20), `pid_of` (15), `parse_escena_3d` (15), `window_json` (13), `post_message` (13),
`enum_children`/`enum_windows` (10 c/u), `send_message_timeout` (11).
Fuera de ellas, la mayor parte está en `format` (35), `adapters/cli` y `uia`, `sites`, `components` y
`simulation`.

## Plan propuesto
- **Un commit `refactor(<zona>)` por zona**, con el renombrado hecho por el servidor de lenguaje
  (`rename`), nunca con buscar y reemplazar, y verificado con `scripts/verificar.ps1`.
- **Orden:** primero `adapters/gui` (lo más usado y ya tocado: `class_of` → `clase_de`,
  `text_of` → `texto_de`, `rect_of` → `rect_de`, `pid_of` → `pid_de`, `window_json` → `ventana_json`,
  `post_message` → `publicar_mensaje`, `send_message_timeout` → `enviar_mensaje_con_plazo`); después el
  resto del crate, junto con el paso 2.
- **Riesgo:** bajo, porque son símbolos `pub(crate)` y el compilador detecta cualquier referencia que
  falte. Los únicos nombres visibles desde fuera (tools, argumentos y JSON de salida) no cambian.
