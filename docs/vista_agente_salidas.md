# Vista de agente en la salida de las tools (Tarea A)

Cada respuesta de las 15 tools, correcta o con fallo, lleva un bloque `vista` con la misma forma. Las claves
que ya existían no cambian. Este documento recoge el contrato, lo que se midió y la matriz tool × acción.

## Contrato

| Clave | Contenido |
|---|---|
| `estado` | `ok`, `parcial` (algo se hizo y algo falló, o falta una confirmación) o `error`. |
| `resumen` | Una frase con lo hecho; en `parcial`/`error`, lo que falló. |
| `origen` | De dónde salen los `campos`: `pvsyst`, `mcp` o `aproximacion_mcp`. |
| `campos` | `[{clave, valor, unidad?, editable?}]`. `unidad` falta si no hay; `editable` falta si es de solo lectura. |
| `avisos` | Avisos de PVsyst o del MCP (como mucho 5, 160 caracteres cada uno). |
| `siguiente` | `[{tool, argumentos, motivo}]`. Cada paso lo acepta la tool de destino tal cual (se valida antes de enviarlo). |
| `recortado` | `[{clave, mostrados, total, pedir_resto}]`: lo que cortó `detalle` y cómo pedir el resto. |
| `error` | Solo en `parcial`/`error`: `{codigo, motivo, causa_probable, que_hacer, claves_validas?}`. |

Cambios respecto a la propuesta del handoff, con motivo:

- **`origen` sube de cada campo al bloque.** Todos los campos de una respuesta salen de la misma llamada. Repetirlo
  en cada campo costaba ~25 bytes por campo.
- **`unidad` y `editable` se omiten cuando no aplican** (sin unidad / solo lectura). Mismo motivo: tamaño.
- **`siguiente` solo trae pasos completos.** Si un valor que el paso necesita (`proyecto`, `trabajo`…) no está ni en
  la petición ni en la respuesta, el paso no se ofrece. La primera versión dejaba el paso sin ese valor y la
  prueba en vivo lo delató: `resultados que:perdidas` sin `proyecto` falla al ejecutarse.
- **Confirmación de la versión de PRUEBA.** Si la respuesta es `requiere_confirmacion`, el estado es `parcial`,
  `error.codigo` es `requiere_confirmacion` y el único `siguiente` es la misma llamada con `confirmar:true`.
  La primera versión la daba por `ok` con el resumen «Sistema diseñado…»: era falso, no se había ejecutado nada.
- **Ensayo (`dry_run:true`, o `simular:true` en `rellenar`).** `estado:ok`, el resumen dice que no se aplicó nada y
  el único `siguiente` es la misma llamada sin la marca de ensayo.
- **Flujo que falla a medias** (`ok:false` o `errores`): `parcial` si algún paso funcionó, `error` si ninguno. El
  `siguiente` es el diagnóstico (`entorno accion:diagnostico`), no los resultados de una simulación que no corrió.
- **Cliente sin `structuredContent`** (protocolo anterior a 2025-06-18): el fallo lleva el `vista` como segundo
  elemento de texto de `content`, con la forma `{"vista": {...}}`. El primero sigue siendo el mensaje.
- **Dónde vive el texto:** `src/vista_agente/contrato/catalogo/` (un archivo por feature). Un test falla si una
  tool declara una acción sin entrada, o si el catálogo conserva una acción que ya no existe. El `siguiente` de
  una línea que ya existía en la raíz sale de la misma entrada (`pista`).
- **`vista` (bloque de salida) y `gui_observar que:vista` (formularios por claves) son distintos.** Una respuesta de
  GUI puede llevar los dos. En una vista de formulario, `vista.campos` repite `{clave, valor, unidad, editable}` de
  los campos que la respuesta ya trae completos en `campos`; no se recortan.

## Inventario de lo que devolvía cada tool (claves de primer nivel que alimentan `campos`)

| Tool | Claves habituales hoy |
|---|---|
| `entorno` | `entorno{…, cli_exists, gui_exists, workspace}`, `preflight{checks}`; licencia: `license`, `exit_code`, `success`. |
| `disenar_sistema` | `ok`, `proyecto`, `variante`, `kwp_objetivo`, `kwp_realizado`, `pasos[]`, `errores[]`, `avisos[]`, `kpis`; sin confirmar: `requiere_confirmacion`, `ejecuciones_estimadas`, `restantes`, `mensaje`. |
| `listar` | `count`, `projects[]` / `archivos[]` / `objetos[]`, `advertencias[]`. |
| `leer` | `file`, `size_bytes`, `summary{…}`, `hint`. |
| `proyecto` | `project`, `variant`, `avisos[]`, `detail{applied[]|edits[]}`. |
| `simular` | `dry_run`, `cli_args`, `job_id`, `stdout_tail`, `csv`, `pdf`. |
| `resultados` | `kpis{…}`, `pr_pct`, `produccion_especifica_kwh_kwp`, series mensuales, pérdidas. |
| `gui_observar` / `gui_actuar` | `campos[]` (formularios), `ventanas[]`, `elapsed_ms`; errores: `error`, `tipo`. |

## Tamaño

Bytes del `result` completo (`content` + `structuredContent`) con el release, antes (v0.3.0 + commits locales) y
después, sobre `DataRO\PVsyst8.1_Data`. Medición: `.scratch/exp/medir_a.py`.

| Llamada | Antes | Después | Δ |
|---|---:|---:|---:|
| entorno | 5410 | 7217 | +1807 |
| entorno licencia | 1336 | 2640 | +1304 |
| entorno preferencias | 1284 | 1785 | +501 |
| listar proyectos | 5024 | 5741 | +717 |
| listar archivos | 3612 | 4542 | +930 |
| listar modulos | 894 | 1619 | +725 |
| listar objetos | 2898 | 3842 | +944 |
| leer (PRJ) | 1026 | 2641 | +1615 |
| leer (VC0) | 1070 | 2819 | +1749 |
| proyecto doctor | 3933 | 6478 | +2545 |
| resultados kpis (error) | 287 | 910 | +623 |
| simular ejecutar (dry_run) | 2277 | 3371 | +1094 |
| workspace historial | 211 | 674 | +463 |
| gui_observar esquema | 5175 | 5926 | +751 |
| gui_observar ventanas (error) | 281 | 945 | +664 |
| pvsyst_cli | 1037 | 2247 | +1210 |
| **Total** | **35 755** | **53 397** | **+17 642 (+49 %)** |

El bloque cuesta unos 0,5–1,3 KB por llamada en el cable, porque `structuredContent` repite el JSON y el bloque va
en las dos copias. Se acotó con 12 campos, 5 avisos, valores de texto de hasta 40 caracteres y sin aplanar objetos
con más de 8 escalares. En respuestas pequeñas el aumento relativo es grande y el absoluto, pequeño.
**La primera versión duplicaba un +106 %**; los recortes de campos y avisos lo bajaron a +49 %.

## Verificación

- **Unitaria (`cargo test --lib`):** cobertura de las 108 combinaciones tool × acción (resumen, estado `ok` y un
  `siguiente` que la tool de destino acepta), entradas huérfanas, estados `parcial`/`error`, confirmación, ensayo,
  recortes, avisos y errores con `claves_validas`.
- **Protocolo (`tests/protocol.rs`):** `vista` en un éxito y en un fallo con `structuredContent`.
- **Con `fake_pvsystcli` (`tests/cli_fake/encadenado.rs`):** un agente que solo sigue `siguiente[0]` encadena
  `proyecto crear` → `proyecto doctor` → `simular ejecutar` → `resultados kpis`, sin leer texto libre. La última
  llamada falla porque el CLI falso no escribe PDF; el fallo también trae `vista`.
- **En vivo (release, PVsyst 8.1.7 en PRUEBA, 2026-10-27):** 30 acciones de solo lectura o en ensayo (ver matriz).
  `disenar_sistema` real: primera llamada → `parcial` con `requiere_confirmacion`; `siguiente[0]` (la misma llamada con
  `confirmar:true`) se ejecutó tal cual y devolvió `parcial` con `error.motivo` = «paso 'simulacion' fallido»; el
  `siguiente` fue `entorno diagnostico`. **La simulación no corrió en esta máquina** (el módulo `MCP_Test_400.PAN` del
  workspace no tiene dimensiones; el módulo concentrador de DataRO exige seguimiento; Meteonorm no encuentra
  `DataMN9/station.db.enc`), así que el encadenado `disenar_sistema` → `resultados` con una simulación correcta
  **no se pudo demostrar en vivo**; sí con el CLI falso (arriba).

## Matriz tool × acción

Las 15 tools, las 108 combinaciones. «Unitaria» = lo cubre el test que enumera `Discriminador.valores`; no se
ejecutó la acción real (la mayoría escribe o gasta ejecuciones de prueba).

| Tool | Acción (`accion`/`que`) | `vista.resumen` | `vista.siguiente` (`*` = toma el valor de la petición o la respuesta) | Verificación |
|---|---|---|---|---|
| `componente` | `crear` | Componente creado a partir de una plantilla. | `? ` | unitaria (catálogo + validación) |
| `componente` | `editar` | Componente editado. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `componente` | `borrar` | Componente borrado. | `? ` | unitaria (catálogo + validación) |
| `componente` | `importar` | Componente importado al workspace. | `? ` | unitaria (catálogo + validación) |
| `componente` | `exportar` | Componente exportado fuera del workspace. | `? ` | unitaria (catálogo + validación) |
| `componente` | `crear_desde_ficha` | Componente creado desde los datos de una ficha técnica. | `? ` | unitaria (catálogo + validación) |
| `entorno` | `diagnostico` | Diagnóstico del entorno: PVsyst, CLI, workspace y herramientas. | `? `; `entorno accion=licencia` | en vivo: ok |
| `entorno` | `licencia` | Estado de la licencia de PVsystCLI. | `entorno accion=diagnostico` | en vivo: ok |
| `entorno` | `licencia_sync` | Licencia sincronizada con el servidor de PVsyst. | `entorno accion=licencia` | unitaria (catálogo + validación) |
| `entorno` | `licencia_activar` | Comando de activación preparado; el MCP no activa licencias. | `entorno accion=licencia` | en vivo: ok |
| `entorno` | `licencia_desactivar` | Comando de desactivación preparado; el MCP no desactiva licencias. | `entorno accion=licencia` | en vivo: ok |
| `entorno` | `exportar_logs` | Registros de PVsyst exportados. | `entorno accion=diagnostico` | en vivo: ok |
| `entorno` | `logs_cli` | Últimas líneas del registro de PVsystCLI. | `entorno accion=diagnostico` | en vivo: ok |
| `entorno` | `preferencias` | Preferencias de PVsyst leídas. | `entorno accion=diagnostico` | en vivo: ok |
| `entorno` | `preferencias_editar` | Preferencia de PVsyst modificada. | `entorno accion=preferencias` | en vivo: error `not_found` |
| `disenar_sistema` | `—` | Sistema diseñado de extremo a extremo: sitio, proyecto, dimensionado y simulación. | `? `; `? `; `entorno accion=diagnostico` | unitaria (catálogo + validación) |
| `pvsyst_cli` | `—` | Comando de PVsystCLI ejecutado. | `entorno accion=diagnostico` | en vivo: ok |
| `gui_observar` | `ventanas` | Ventanas de PVsyst listadas. | `gui_observar que=esquema`; `gui_observar que=funciones` | en vivo: error `gui` |
| `gui_observar` | `controles` | Controles de la ventana listados. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `leer` | Texto de la ventana leído. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `capturar` | Captura de la ventana obtenida. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `uia` | Árbol de UI Automation leído. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `arbol` | Árbol de controles leído. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `formulario` | Formulario leído por OCR. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `vista` | Formulario leído por claves estables. | `gui_observar que=esquema`; `gui_observar que=ventanas` | en vivo: ok |
| `gui_observar` | `esquema` | Esquemas de formulario con sus claves. | `gui_observar que=ventanas`; `gui_observar que=funciones` | en vivo: ok |
| `gui_observar` | `mensajes` | Mensajes de la ventana leídos. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_observar` | `funciones` | Funciones de la GUI listadas. | `gui_observar que=ventanas`; `gui_observar que=esquema` | en vivo: ok |
| `gui_observar` | `menus` | Menús de la ventana listados. | `gui_observar que=funciones`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `click` | Clic enviado al control. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `clic_fisico` | Clic físico enviado a la pantalla. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `mensaje` | Mensaje de Windows enviado a la ventana. | `gui_observar que=controles*`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `cerrar` | Ventana cerrada. | `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `activar` | Ventana traída al primer plano. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `teclas` | Teclas enviadas a la ventana en primer plano. | `gui_observar que=capturar*`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `escribir` | Texto escrito en el control. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `seleccionar` | Elemento de la lista seleccionado. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `pagina` | Pestaña del formulario cambiada. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `esperar_cierre` | Cierre de la ventana esperado. | `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `lanzar` | PVsyst lanzado. | `gui_observar que=ventanas`; `gui_observar que=funciones` | unitaria (catálogo + validación) |
| `gui_actuar` | `uia_invocar` | Elemento de UI Automation invocado. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `esperar` | Espera completada. | `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `paso` | Paso de GUI ejecutado. | `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `funcion` | Función de la GUI ejecutada. | `gui_observar que=ventanas`; `gui_observar que=esquema` | unitaria (catálogo + validación) |
| `gui_actuar` | `menu` | Entrada de menú pulsada. | `gui_observar que=ventanas`; `gui_observar que=esquema` | unitaria (catálogo + validación) |
| `gui_actuar` | `campo` | Campo del formulario escrito por etiqueta. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `rellenar` | Formulario rellenado por claves. | `gui_observar que=vista`; `gui_observar que=esquema` | unitaria (catálogo + validación) |
| `gui_actuar` | `clic_texto` | Clic en el texto visible de la ventana. | `gui_observar que=vista`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `reiniciar` | PVsyst reiniciado. | `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `cerrar_todo` | Ventanas de PVsyst cerradas. | `gui_observar que=ventanas` | en vivo: ok |
| `gui_actuar` | `escena` | Escena 3D validada: objetos, formas y colisiones, sin tocar PVsyst. | `gui_observar que=esquema`; `gui_observar que=ventanas` | unitaria (catálogo + validación) |
| `gui_actuar` | `secuencia` | Secuencia de pasos de GUI ejecutada. | `gui_observar que=ventanas`; `gui_observar que=esquema` | unitaria (catálogo + validación) |
| `proyecto` | `crear` | Proyecto creado desde una plantilla. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `eliminar` | Proyecto eliminado. | `? ` | unitaria (catálogo + validación) |
| `proyecto` | `renombrar` | Proyecto renombrado. | `proyecto accion=doctor*`; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `crear_variante` | Variante creada a partir de otra. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `dimensionar` | Proyecto dimensionado con módulos en serie y strings. | `? `; `? ` | en vivo: error `io` |
| `proyecto` | `doctor` | Proyecto revisado: coherencia de la variante y sus archivos. | `? `; `? ` | en vivo: ok |
| `proyecto` | `anadir` | Elemento añadido a la variante. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `seguidor` | Variante convertida a seguidor. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `limite_inyeccion` | Límite de inyección a red aplicado. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `almacenamiento` | Almacenamiento con baterías configurado. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `perdidas` | Pérdidas detalladas configuradas. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `bombeo` | Sistema de bombeo configurado. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `economia` | Evaluación económica configurada. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `configurar` | Cambios de variante aplicados en una sola llamada. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `p50_p90` | Parámetros de incertidumbre P50/P90 configurados. | `? `; `? ` | unitaria (catálogo + validación) |
| `proyecto` | `limpiar_temporales` | Archivos temporales del MCP eliminados. | `? ` | unitaria (catálogo + validación) |
| `simular` | `ejecutar` | Simulación ejecutada con PVsystCLI. | `? `; `? `; `simular accion=estado*`; `? ` | en vivo: ok |
| `simular` | `preparar` | Simulación preparada sin ejecutarla. | `? `; `simular accion=estado*`; `? ` | unitaria (catálogo + validación) |
| `simular` | `estado` | Estado de las simulaciones consultado. | `? `; `? ` | en vivo: error `not_found` |
| `simular` | `preestimar` | Producción estimada antes de simular. | `? `; `simular accion=estado*`; `? ` | unitaria (catálogo + validación) |
| `simular` | `barrido` | Barrido de simulaciones ejecutado. | `resultados que=comparar*`; `simular accion=estado*`; `? ` | unitaria (catálogo + validación) |
| `resultados` | `kpis` | Indicadores de la simulación leídos del informe. | `? `; `? ` | en vivo: error `not_found` |
| `resultados` | `csv` | Resultados horarios leídos del CSV. | `? `; `? ` | en vivo: error `not_found` |
| `resultados` | `perdidas` | Desglose de pérdidas del sistema leído. | `resultados que=mensual*`; `? ` | en vivo: error `not_found` |
| `resultados` | `mensual` | Producción mensual leída. | `? `; `? ` | en vivo: error `not_found` |
| `resultados` | `comparar` | Variantes del proyecto comparadas. | `? `; `? ` | unitaria (catálogo + validación) |
| `resultados` | `medido_vs_simulado` | Datos medidos comparados con la simulación. | `? `; `? ` | unitaria (catálogo + validación) |
| `resultados` | `p50_p90` | Producción P50/P90 calculada de la simulación. | `? `; `? ` | en vivo: error `not_found` |
| `resultados` | `carbono` | Balance de carbono calculado. | `? `; `? ` | unitaria (catálogo + validación) |
| `resultados` | `economia` | Resultados económicos leídos. | `? `; `? ` | unitaria (catálogo + validación) |
| `resultados` | `entregable` | Entregable del proyecto generado. | `? `; `? ` | unitaria (catálogo + validación) |
| `resultados` | `resumen` | Resumen de resultados del proyecto. | `? `; `? ` | unitaria (catálogo + validación) |
| `sitio` | `crear` | Sitio creado con la meteorología de la ubicación. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `sitio` | `convertir_meteo` | Archivo meteorológico convertido al formato de PVsyst. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `sitio` | `geometria_solar` | Geometría solar calculada para la ubicación. | `? ` | unitaria (catálogo + validación) |
| `sitio` | `horizonte_pvgis` | Horizonte descargado de PVGIS. | `? ` | unitaria (catálogo + validación) |
| `sitio` | `meteo_pvgis` | Meteorología descargada de PVGIS. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `sitio` | `escribir_horizonte` | Horizonte escrito en el sitio. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `sitio` | `escribir_serie` | Serie meteorológica escrita en el sitio. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `workspace` | `crear` | Workspace creado. | `? `; `? ` | unitaria (catálogo + validación) |
| `workspace` | `exportar` | Workspace exportado a un archivo. | `? `; `? ` | unitaria (catálogo + validación) |
| `workspace` | `importar` | Workspace importado. | `? `; `? ` | unitaria (catálogo + validación) |
| `workspace` | `exportar_proyecto` | Proyecto exportado a un archivo. | `? `; `? ` | unitaria (catálogo + validación) |
| `workspace` | `historial` | Historial de cambios del workspace. | `? ` | en vivo: ok |
| `workspace` | `restaurar` | Cambio restaurado desde el historial. | `? `; `? ` | unitaria (catálogo + validación) |
| `listar` | `proyectos` | Proyectos del workspace listados. | `? `; `? ` | en vivo: ok |
| `listar` | `componentes` | Componentes del tipo pedido listados. | `? `; `? ` | en vivo: ok |
| `listar` | `modulos` | Módulos fotovoltaicos listados. | `? `; `? ` | en vivo: ok |
| `listar` | `archivos` | Archivos del workspace listados. | `? `; `? ` | en vivo: ok |
| `listar` | `objetos` | Objetos del archivo PVsyst listados. | `leer *`; `? ` | en vivo: ok |
| `leer` | `—` | Archivo de PVsyst leído. | `listar que=objetos*`; `? ` | en vivo: ok |
| `editar` | `campos` | Campos editados con copia en el historial. | `? `; `leer *`; `? ` | unitaria (catálogo + validación) |
| `editar` | `insertar_objeto` | Objeto insertado en el archivo. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `editar` | `eliminar_objeto` | Objeto eliminado del archivo. | `leer *`; `? ` | unitaria (catálogo + validación) |
| `editar` | `rvt` | Operación sobre el archivo RVT aplicada. | `? ` | unitaria (catálogo + validación) |
| `editar` | `plantilla_rvt` | Plantilla RVT generada. | `? ` | unitaria (catálogo + validación) |
| `escribir_archivo` | `—` | Archivo escrito en el workspace con copia en el historial. | `leer *`; `? ` | unitaria (catálogo + validación) |
