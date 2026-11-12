# Informe de pruebas exhaustivas — `pvsyst-mcp` 0.3.0

**Fecha:** 2026-11-11
**Build probado:** `85a2450` (`docs(i18n): record the correction status of the language audit`), rama `worktree-tarea-d-fila`
**Ejecutable:** `C:\pvsyst-mcp\pvsyst-mcp.exe`
**PVsyst:** 8.1.8 (rev. 49527), Windows, modo TRIAL
**Superficie:** 15 tools, 107 acciones/`que` canónicas (104 visibles sin `PVSYST_MCP_APROXIMACIONES=1`) más 4 tools sin discriminador; 126 pares tool/acción en el índice de llamadas (incluye tools sin discriminador y alias no normalizados); 5 recursos `pvsyst://`, 9 prompts, GUI de PVsyst
**Llamadas registradas:** 749 (`initialize` + `tools/call`), en `.qa/exhaustiva/out_*.jsonl`

---

## 0. Método, alcance y límites

### 0.1 Cómo se llamó al servidor

Se usó el arnés `.qa/exhaustiva/qa.py`, un cliente MCP por stdio (JSON-RPC por líneas: `initialize`, `notifications/initialized`, `tools/call`). Habla con el mismo `pvsyst-mcp.exe` y el mismo protocolo que un cliente nativo. Se cruzaron cinco llamadas con las tools nativas `mcp__pvsyst__*` y devolvieron lo mismo: `entorno diagnostico` (workspace real, GUI en ejecución), `listar que:proyectos` (18 proyectos, 13 mostrados), `leer MCP_FullTest.PRJ` (mismo resumen), `resultados que:kpis` de `QA_Real_20261111b` (11902 kWh, PR 87.14) y `gui_observar que:ventanas` (2 ventanas). La diferencia es la validación del cliente, no el servidor.

El arnés se usó por tres razones:

1. **Entorno por lote:** la instancia nativa está fijada al workspace real y al idioma por defecto. Para probar el workspace de pruebas y `PVSYST_MCP_IDIOMA=es/en` hacía falta arrancar el servidor con esas variables.
2. **Evidencia cruda:** cada llamada y respuesta queda en `.qa/exhaustiva/out_*.jsonl`, que es lo que pide el entregable.
3. **Casos que un cliente estricto rechaza antes de enviar:** enum inexistente, tipo erróneo, `dry_run` no declarado, `workspace` en rutas puras. El arnés los manda al servidor; un cliente con validación de esquema no llegaría a probarlos.

**Matiz honesto:** los casos inválidos probados por arnés son «contrato del servidor», no «cliente nativo». Desde un cliente estricto podrían ni siquiera enviarse. Los casos válidos se contrastaron también con las tools nativas.

### 0.2 Workspaces

- **Workspace de pruebas:** `C:\Users\Lenovo\AppData\Local\Temp\pvsyst_qa_20261111\PVsyst8.1_Data`, creado con `workspace accion:crear`. Ahí se hicieron todas las escrituras, ediciones, restauraciones e importaciones. Se copiaron proyectos, variantes, sitios, meteo, componentes y resultados de fixtures y de demos.
- **Workspace real:** `C:\Users\Lenovo\PVsyst8.1_Data`, solo para la GUI, la simulación real y `p50_p90`. No se modificó ningún proyecto preexistente del usuario. Se crearon proyectos QA nuevos (`QA_Real_20261111`, `QA_Real_20261111b`), un inversor QA, una escena QA y se copió `Geneva_MN90.SIT`; todo se retiró al terminar (ver §7).

### 0.3 GUI

PVsyst se lanzó una vez al principio (la acción `gui_actuar accion:lanzar` lo hizo durante una prueba de `dry_run`; ver BUG-03). Después de `cerrar_todo` se relanzó para terminar las pruebas de `clic_fisico`. Siempre hubo una sola instancia manejada por esta sesión. No había otro PVsyst abierto por otra sesión.

**Límite de verificación visual:** el modelo de esta sesión no puede leer imágenes. Las capturas se guardaron en `docs/capturas_2026-11-11/`, pero **no se verificaron visualmente**. Como sustituto se usaron `gui_observar que:formulario` (OCR), `que:vista` (claves estables), `que:controles`, `que:ventanas`, `que:uia` y `que:menus`. Las celdas de GUI se marcan como «sin verificación visual» cuando la única prueba posible era la imagen.

### 0.4 Datos y seguridad

- No se ejecutaron `lic-activate`, `lic-deactivate` ni `lic-sync` de verdad. `pvsyst_cli` los rechaza; `entorno accion:licencia_sync` **no** los rechaza (BUG-01), pero solo se probó con `dry_run:true` para no tocar la licencia.
- No se incluyen claves, contenido de licencia ni identificadores de hardware en este informe. La salida de `lic-info` los contiene; se omiten.
- No se modificó código ni se hizo ningún commit.

---

## 1. Matriz de cobertura

Convenciones:

- **Probado:** se envió al menos una llamada válida y se contrastó el resultado (archivo, CLI, GUI u otra tool).
- **Probado (error):** se envió el caso y se comprobó que el rechazo es el esperado.
- **No probado:** no se envió; se indica la razón.
- **Sin verificación visual:** la GUI se leyó por OCR/`vista`, no por imagen.

Las llamadas crudas están en `.qa/exhaustiva/calls/` y las respuestas en `.qa/exhaustiva/out_*.jsonl`; el índice está en `.qa/exhaustiva/resumen_llamadas.md`.

### 1.1 `entorno`

| Acción | Parámetros probados | Enum / alias probado | Estado |
|---|---|---|---|
| `diagnostico` | `accion`, `dry_run` (rechazado), `foo` (ignorado) | `diagnostico`, `diagnostic`, `diagnostics` | Probado |
| `licencia` | `accion`, `dry_run` | `licencia`, `license` | Probado |
| `licencia_sync` | `accion`, `dry_run` | `licencia_sync`, `license_sync` | Probado solo con `dry_run`; no se ejecutó por seguridad (BUG-01) |
| `licencia_activar` | `accion`, `dry_run` (rechazado) | `licencia_activar`, `license_activate` | Probado (devuelve el comando, no ejecuta) |
| `licencia_desactivar` | `accion`, `dry_run` (rechazado) | `licencia_desactivar`, `license_deactivate` | Probado (devuelve el comando, no ejecuta) |
| `exportar_logs` | `accion`, `carpeta_destino`, `dry_run` | `exportar_logs`, `export_logs` | Probado; éxito con carpeta existente, error con carpeta inexistente, escritura fuera del workspace sin destino (BUG-04) |
| `logs_cli` | `accion`, `n`, `solo_errores` | `logs_cli` | Probado |
| `preferencias` | `accion`, `seccion`, `clave` | `preferencias`, `preferences` | Probado; sección/clave inexistente rechazadas |
| `preferencias_editar` | `accion`, `seccion`, `clave`, `valor` | `preferencias_editar`, `preferences_edit` | Probado; `NRun` 101→86 y restaurado desde copia |

**Parámetros no probados:** `permitir_fuera_del_workspace` (solo aplica a rutas; no se usó en `entorno`).

### 1.2 `disenar_sistema`

| Parámetro | Valores probados | Estado |
|---|---|---|
| `latitud`, `longitud` | 7.07/-73.84, 8.2/-74.5, 9.123/-75.456, 999 (error) | Probado |
| `kwp` | 10, 8, 5, 0 (error) | Probado |
| `n_mod_serie`, `n_strings` | 10/2, 10/1; sin ambos (error) | Probado |
| `nombre` | `QA_Design`, `QA_Dis` | Probado |
| `modulo`, `inversor` | `Synth_400W`, `Synth_10kW`, `NoExiste` (error) | Probado |
| `tilt`, `azimut`, `altitud`, `timezone`, `country_code`, `region` | 25/0/100/-5/CO/south_america | Probado |
| `simular`, `output_csv`, `report_pdf`, `dry_run`, `confirmar` | true/false, rutas | Probado |
| `tipo_sistema` | `red`, `aislado` | Probado |
| `n_baterias_serie`, `n_baterias_paralelo` | 2/4 | Probado |
| `synthetic_weather` | true | Probado (plan) |
| `tolerancia_kwp_pct` | 5 | Probado |
| `via` | `aproximacion` (rechazado sin `PVSYST_MCP_APROXIMACIONES=1`) | Probado (error) |
| `plantilla`, `variante`, `pnom_modulo`, `sobrescribir`, `workspace`, `permitir_fuera_del_workspace` | — | No probado: no se necesitaron para el caso E2E; `workspace` se probó por el entorno del servidor |

**Resultado real:** dos intentos reales (`9.123,-75.456` y `8.2,-74.5`) fallaron en el paso de sitio con `PVsystCLI no creó el sitio ...` (BUG-05). No se completó el E2E real.

### 1.3 `listar`

| `que` | Parámetros probados | Enum / alias probado | Estado |
|---|---|---|---|
| `proyectos` | `texto`, `include_variants`, `recursivo`, `detalle`, `dry_run` (ignorado) | `proyectos`, `projects` | Probado |
| `componentes` | `tipo` (PAN, OND, BTR, PMP, REV, modulo, modules, inversor, baterias, pumps), `pnom_min`, `pnom_max`, `texto` | `componentes`, `components` | Probado |
| `modulos` | `pnom_min`, `pnom_max`, `texto` | `modulos`, `modules` | Probado |
| `archivos` | `carpeta`, `extension`, `recursivo`, `todos`, `texto`, `detalle` | `archivos`, `files` | Probado |
| `objetos` | `archivo`, `tipo` (rechazado: `tipo` es de componente), `detalle` | `objetos`, `objects` | Probado |

**No probado:** `workspace` y `permitir_fuera_del_workspace` (el workspace se cambió por entorno del servidor).

### 1.4 `leer`

| Parámetro | Valores probados | Estado |
|---|---|---|
| `archivo` | `.PRJ`, `.VC0`, `.PAN`, `.SIT`, `.MET`, `.HOR`, `.DAM`, `.MEF`, `.SHO`, `.SHD`, `.REV`, `.OND`, `.BTR`, `.CSV`, `.PDF`, `.xyz`, directorio | Probado |
| `objeto` | `pvSite` | Probado |
| `plano` | true | Probado |
| `serie` | true (MET/DAM) | Probado |
| `limite` | 2, 3 | Probado |
| `offset`, `max_bytes`, `base64` | 0/20/true | Probado; falla con nombre de proyecto sin `Projects/` (BUG-13) |
| `tipo` | `PAN` | Probado |
| `detalle` | `completo` | Probado |
| `permitir_fuera_del_workspace` | true/false | Probado (`C:\Windows\win.ini` rechazado/aceptado) |
| `dry_run` | true | Ignorado y reportado en `ignorados` (no es acción con ensayo) |

**No probado:** `workspace` (se cambió por entorno).

### 1.5 `editar`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `campos` (PVObject) | `archivo`, `objeto`, `campos` (`Comment`), `dry_run` (ignorado) | Probado; `dry_run` no evita la escritura (BUG-02) |
| `campos` (variante) | `proyecto`, `variante`, `ediciones` (`pvSubArray.NStringCh`, `PVModule`, `GInverter`), `ediciones:[]` | Probado |
| `insertar_objeto` | `archivo`, `padre`, `desde:{archivo,objeto}`, `reemplazar`, `campos` (rechazado) | Probado |
| `eliminar_objeto` | `archivo`, `objeto`; raíz (rechazada); inexistente (error) | Probado |
| `rvt` | `operacion` (crear, listar, anadir, eliminar, reordenar, catalogo, nope), `archivo`, `variables`, `detalle` | Probado |
| `plantilla_rvt` | `operacion:listar`, `archivo` | Probado |

**No probado:** `tipo`, `nuevo`, `batch_rvt_file`, `workspace`, `permitir_fuera_del_workspace`.

### 1.6 `escribir_archivo`

| Parámetro | Valores probados | Estado |
|---|---|---|
| `archivo`, `contenido` | `QA.txt`, `CON`, `NUL`, `../fuera.txt`, ruta absoluta | Probado |
| `permitir_fuera_del_workspace` | true/false | Probado |
| `sobrescribir` | — | No probado (el archivo nuevo no existía; el caso de reemplazo no se envió) |
| `base64` | — | No probado |
| `workspace` | — | No probado |

### 1.7 `proyecto`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `crear` | `plantilla`, `nombre_nuevo`, `variante`, `modulo`, `inversor`, `n_mod_serie`, `n_strings`, `comentario`, `sobrescribir`, `dry_run` (rechazado) | Probado |
| `eliminar` | `proyecto` (sin `variante` y con variante en pruebas de limpieza) | Probado |
| `renombrar` | `proyecto`, `nombre_nuevo` | Probado |
| `crear_variante` | `proyecto`, `variante_nueva`; duplicada (error) | Probado |
| `dimensionar` | `proyecto`, `n_mod_serie`, `n_strings`, `kwp` (GUI), `dry_run` (rechazado) | Probado; `kwp` falla si el proyecto no está cargado en la GUI |
| `doctor` | `proyecto`, `variante`, `kwp`/`moneda`/`n_mod_serie` (ignorados no reportados, BUG-11) | Probado |
| `anadir` | `elemento` (`horizonte`, `noexiste`), `proyecto`, `variante` | Probado |
| `seguidor` | `azimut_eje`, `angulo_max`, `pitch_m`, `ancho_colector_m`, `n_filas`, `backtracking`; pitch inválido (error) | Probado |
| `limite_inyeccion` | `potencia_kw`, `perdida_separada`, `tan_phi`; potencia negativa (error) | Probado |
| `almacenamiento` | `estrategia` (`autoconsumo`; `recorte_picos` no), `bateria`, `n_serie`, `n_paralelo`, `p_max_carga_kw`, `p_max_descarga_kw`, `soc_inicial`; SOC 50 (error) | Probado |
| `perdidas` | `suciedad_pct`, `uc_termico`, `lid_pct`, `mismatch_pct`; 99 (error) | Probado |
| `bombeo` | `caudal_m3_dia`, `profundidad_estatica_m`; negativo (error) | Probado |
| `economia` | `moneda`, `vida_anios`, `tasa_descuento`, `inflacion`, `tarifa_venta_kwh`; moneda inválida (error); tarifa variable (rechazada) | Probado |
| `configurar` | `pasos` (2 pasos), paso `crear` (rechazado) | Probado |
| `p50_p90` | `proyecto`, `variante`, `dry_run`; proyecto inexistente (plan erróneo, BUG-09); real en `QA_Real_20261111b` | Probado |
| `limpiar_temporales` | sin args | Probado |

**No probado:** `sitio`, `variante_origen`, `pnom_modulo`, `reparar`, `n_inversores`, `bomba`, `workspace`, `permitir_fuera_del_workspace`, `via`. Razón: no eran necesarios para el caso o requieren datos que no estaban en el workspace de pruebas.

### 1.8 `componente`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `crear` | `tipo` (PAN, OND, NOPE), `nombre_nuevo`, `plantilla`, `sobrescribir` (no), `dry_run` (rechazado) | Probado; `tipo` es obligatorio en runtime pero no está en el esquema (BUG-12) |
| `editar` | `componente`, `cambios` (`PNom`, campo inexistente) | Probado |
| `borrar` | `componente`; inexistente (error) | Probado |
| `importar` | `origen`, `nombre_nuevo`, `tipo`, `destino` (rechazado) | Probado |
| `exportar` | `componente`, `destino`; inexistente (error) | Probado |
| `crear_desde_ficha` | `tipo` (PAN, OND, NOPE), `nombre_nuevo`, `datos` | Probado; OND real creado, PAN falla por dimensiones (BUG-14) |

**No probado:** `campos`, `sobrescribir`, `via`, `workspace`, `permitir_fuera_del_workspace`.

### 1.9 `sitio`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `crear` (coordenadas) | `nombre_sitio`, `latitud`, `longitud`, `altitud`, `timezone`, `country_code`, `region`, `dry_run`, `log_level`; nombre `CON` (error), lat 999 (error), región inválida (error) | Probado; un `create-site` real falló con crash (BUG-05) |
| `crear` (`origen:csv`) | `origen`, `nombre_sitio`, `latitud`, `longitud`, `input_csv_file` (rechazado: falta `weather_data_file`) | Probado (error); `weather_data_file` no probado con archivo válido |
| `convertir_meteo` | `input_csv_file`, `input_mef_file`, `input_sit_file`, `output_met_file`, `input_mef_timeshift`, `dry_run`; archivos inexistentes (error) | Probado; dos intentos reales fallaron con `Invalid date` (BUG-07) |
| `horizonte_pvgis` | `proyecto`, `dry_run` (rechazado), proyecto inexistente (error) | Probado (error/plan) |
| `meteo_pvgis` | `latitud`, `longitud`, `nombre_sitio`, `confirmar`, `dry_run` (rechazado); lat 999 (confirmación antes de validar) | Probado; intento real falló con crash (BUG-08) |
| `escribir_horizonte` | `archivo`, `titulo`, `puntos` (`[{azimut,altura}]`), `dry_run` (rechazado), puntos inválidos (error) | Probado |
| `escribir_serie` | `archivo`, `variables`, `valores` (objeto/columnas), variable inexistente (error) | Probado |
| `geometria_solar` | `latitud`, `longitud`, `fecha`; sin `PVSYST_MCP_APROXIMACIONES=1` (rechazado) | Probado (error) |

**No probado:** `albedo`, `source`, `output_sit_file`, `cmd_options_file`, `timeout_s`, `weather_data_file`, `sit_file` como entrada, `via`, `workspace`, `permitir_fuera_del_workspace`, `puntos` con `azimut_grados`/`altura_grados` (se probó el formato principal).

### 1.10 `simular`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `ejecutar` | `proyecto`, `variante`, `output_csv`, `report_pdf`, `paginas_reporte`, `idioma_reporte`, `extra_options`, `timeout_s`, `dry_run`, `filas`+`dry_run` (rechazado), `parametros`+`dry_run` (rechazado), `archivo_rvt` (error), `input_met_file`, `input_sfi_file`, `cmd_options_file`, `input_param_file`, `batch_params_file`, `batch_rvt_file`, `site`, `synthetic_weather`, `time_step`, `start_date`, `end_date`, `recalcular_sombreado`, `overwrite_default_csv`, `sin_cache`, `segundo_plano`, `abrir_gui`, `n`, `latitud`, `longitud`, `kwp`, `tilt`, `azimut`, `descripciones`, `iniciales`, `unidades`, `destino`, `operacion`, `trabajo`, `confirmar` | Probado; los parámetros geográficos y varios más se ignoran sin reportarse (BUG-10); simulación real exitosa en el workspace real |
| `preparar` | `proyecto`, `variante`, `filas`, `parametros`, ambos, ninguno (error), fila con clave inválida (error) | Probado |
| `estado` | `trabajo` (inexistente, vacío) | Probado |
| `preestimar` | `latitud`, `longitud`, `kwp`; sin aproximaciones (rechazado) | Probado (error) |
| `barrido` | `latitud`, `longitud`, `kwp`; sin aproximaciones (rechazado) | Probado (error) |

**No probado:** `log_level`, `workspace`, `permitir_fuera_del_workspace`, `input_rvt_file`, `variables` en lote, `iniciales`/`descripciones`/`unidades` con `parametros` (solo se probaron como ignorados).

### 1.11 `resultados`

| `que` | Parámetros probados | Estado |
|---|---|---|
| `kpis` | `proyecto`, `archivo`, `dry_run` (ignorado) | Probado |
| `csv` | `proyecto`, `archivo`, `dir`, `count`, `include_text`, `internet` (ignorado) | Probado |
| `perdidas` | `proyecto`, `archivo` | Probado |
| `mensual` | `proyecto`, `archivo` | Probado |
| `comparar` | `proyecto`, `variantes` (`VC0`,`VC1`; `VC9` error) | Probado |
| `p50_p90` | `proyecto`, `sigma_meteo_pct`, `sigma_modelo_pct` | Probado (error: PDF sin página) |
| `carbono` | `proyecto`, `archivo` | Probado (error: PDF sin página) |
| `economia` | `proyecto`, `archivo` | Probado (error: PDF sin página) |
| `entregable` | `proyecto`, `variante` | Probado |
| `resumen` | `proyecto`, `archivo` | Probado |

**No probado:** `detalle` en `resultados`, `kwp`, `via`, `workspace`, `permitir_fuera_del_workspace`. Razón: no eran necesarios; `detalle` se probó en `listar`/`rvt`.

### 1.12 `workspace`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `crear` | `ruta`; existente sin/con `sobrescribir` | Probado |
| `exportar` | `destino`, `sobrescribir`, `dry_run` (rechazado), sin destino (error) | Probado |
| `importar` | `zip`, `destino`; zip inexistente (error), zip con `../evil.txt` (rechazado), ruta fuera (rechazada) | Probado |
| `exportar_proyecto` | `proyecto`, `destino`; proyecto inexistente (error) | Probado |
| `historial` | `n` (3, 0 error), `dry_run` (rechazado) | Probado |
| `restaurar` | `id` (válido, inexistente), `dry_run` | Probado |

**No probado:** `workspace`, `permitir_fuera_del_workspace`.

### 1.13 `pvsyst_cli`

| Parámetro | Valores probados | Estado |
|---|---|---|
| `comando` | `version`, `help`, `lic-info`, `no-existe`, `help version` (error) | Probado |
| `opciones` | lista `["-x:1","-y:2"]`, objeto `{a:1,b:2,flag:true,off:false,nulo:null}` | Probado |
| `dry_run` | true | Probado |
| `timeout_s` | 0 (error), 1 (timeout), 99999 (error), 120 | Probado |
| `log_level` | — | No probado |
| `lic-activate`, `lic-deactivate`, `lic-sync` | rechazados | Probado (error) |

### 1.14 `gui_observar`

| `que` | Parámetros probados | Estado |
|---|---|---|
| `ventanas` | `dry_run` (ignorado) | Probado |
| `controles` | `ventana` | Probado |
| `leer` | `ventana`, `ocr` | Probado; `ocr` no cambia el resultado (BUG-18) |
| `capturar` | `ventana`, `ruta` | Probado (PNG guardado; sin verificación visual) |
| `uia` | `ventana`, `profundidad` | Probado |
| `arbol` | `ventana`, `profundidad` | Probado |
| `formulario` | `ventana`, `ocr` | Probado |
| `vista` | `ventana`, `detalle` | Probado |
| `esquema` | sin args | Probado (89 ventanas, 4 estados) |
| `mensajes` | sin args | Probado (0 mensajes) |
| `funciones` | sin args | Probado (13 funciones de THomeForm) |
| `menus` | `ventana` | Probado |

**No probado:** `proceso`, `pids`, `compacto`, `completo`, `visibles`, `todos`, `incluir_uia`, `max_nodos`, `lineas`, `imagen`, `recorte`, `permitir_fuera_del_workspace`, `funcion`, `entidad`. Razón: no eran necesarios para la cobertura funcional; `profundidad` y `detalle` sí se probaron.

### 1.15 `gui_actuar`

| Acción | Parámetros probados | Estado |
|---|---|---|
| `click` | `ventana` (hwnd, título `OK`), `dry_run` (ignorado en `lanzar`) | Probado |
| `clic_fisico` | `ventana` (centro); `dx`/`dy` no | Probado (error por TDEMOForm; no se repitió con la ventana libre) |
| `mensaje` | `ventana`, `mensaje:0` | Probado |
| `cerrar` | `ventana`, `cancelar` | Probado (TDEMOForm, TSettingsForm, TSystemegridForm, TPVModuleForm, TProjectForm, TPerspectiveForm, TShadingsForm, TDatabaseForm, TChoixObjetPVForm, TToolsForm) |
| `activar` | `ventana` | Probado |
| `teclas` | `ESC`, `RETURN` | Probado |
| `fila` | `ventana`, `clase` (`TVirtualStringTree`), `indice` | Probado; sin `clase` falla (BUG-21) |
| `escribir` | `ventana`, `campo:{clase,x,y}`, `texto` | Probado |
| `seleccionar` | `ventana` (hwnd del combo), `texto` | Probado |
| `pagina` | `ventana`, `indice` (0,1,2) | Probado; tab 2 vetada por validación de PVsyst |
| `esperar_cierre` | `ventana`, `timeout_ms` | Probado |
| `lanzar` | `dry_run` (ignorado, BUG-03) | Probado |
| `uia_invocar` | `ventana`, `nombre`, `rol`, `operacion` | Probado (timeout y ventana equivocada, BUG-15) |
| `esperar` | `clase`, `timeout_ms` | Probado |
| `paso` | `objetivo:{nombre,rol}` | Probado (abrió TToolsForm) |
| `funcion` | `proyecto.cargar`, `bases_de_datos.modulos_pv.nuevo`, `variante.sombreados_cercanos.construccion_perspectiva`, `variante.gestion_energia` | Probado |
| `menu` | `ventana`, `menu:0`, `item:8`; solo `menu` (error) | Probado; falta `id`/`item` en el esquema (BUG-20) |
| `campo` | `ventana`, `etiqueta`, `valor`; etiqueta ambigua (error) | Probado; sin desambiguador (BUG-19) |
| `rellenar` | `entidad` (`variante.system`, `components.panels`), `valores` | Probado |
| `clic_texto` | `ventana`, `texto`, `indice` | Probado |
| `reiniciar` | sin args | Probado (error: no hay diálogo de fallo) |
| `cerrar_todo` | sin args | Probado (cerró 2 ventanas) |
| `secuencia` | `pasos` (2 pasos), paso sin `accion` (error) | Probado |
| `escena` | `escena`, `objetos` (`rectangulo`), `simular` | Probado (plan y real; `.SHD` creado) |
| `escena_leer` | `escena` | Probado (real; `.SHD` temporal borrado) |
| `escena_importar` | `archivo`, `escena`, `simular` | Probado (error: archivo inexistente; no hay malla válida) |

**No probado:** `proceso`, `pids`, `titulo`, `verificar`, `proyecto`, `esperar_segundos`, `permitir_fuera_del_workspace`, `cerrar_demo`, `id`, `subitem`, `dx`/`dy`, `unidades`. Razón: no eran necesarios o requieren una ventana/archivo que no estaba disponible.

### 1.16 Enumeraciones: valores probados

| Tool | Enum | Valores probados | No probados |
|---|---|---|---|
| `entorno` | `accion` | diagnostico, licencia, licencia_sync (dry), licencia_activar, licencia_desactivar, exportar_logs, logs_cli, preferencias, preferencias_editar + todos los alias | — |
| `listar` | `que` | proyectos, componentes, modulos, archivos, objetos + alias projects/components/modules/files/objects | — |
| `editar` | `accion` | campos, insertar_objeto, eliminar_objeto, rvt, plantilla_rvt + alias fields/insert_object/delete_object/remove_object | — |
| `proyecto` | `accion` | crear, eliminar, renombrar, crear_variante, dimensionar, doctor, anadir, seguidor, limite_inyeccion, almacenamiento, perdidas, bombeo, economia, configurar, p50_p90, limpiar_temporales + alias create/delete/remove/rename/create_variant/clone_variant/size/sizing/add/add_object/tracker/convert_tracker/injection_limit/storage/losses/pumping/economics/economy/clean_temps | — |
| `componente` | `accion` | crear, editar, borrar, importar, exportar, crear_desde_ficha + alias create/edit/delete/remove/import/export/datasheet/create_from_datasheet | — |
| `sitio` | `accion` | crear, convertir_meteo, horizonte_pvgis, meteo_pvgis, escribir_horizonte, escribir_serie + alias create/convert_meteo/pvgis_horizon/pvgis_meteo/write_horizon/write_series; geometria_solar (rechazada sin aproximaciones) | — |
| `simular` | `accion` | ejecutar, preparar, estado + alias run/execute/prepare/status; preestimar/barrido (rechazadas sin aproximaciones) | — |
| `resultados` | `que` | kpis, csv, perdidas, mensual, comparar, p50_p90, carbono, economia, entregable, resumen + alias losses/monthly/compare/carbon/economics/economy/deliverable | — |
| `workspace` | `accion` | crear, exportar, importar, exportar_proyecto, historial, restaurar + alias create/export/import/export_project/history/restore | — |
| `gui_observar` | `que` | ventanas, controles, leer, capturar, uia, arbol, formulario, vista, esquema, mensajes, funciones, menus + alias windows/controls/read/capture/tree/form/view/schema/messages/functions | — |
| `gui_actuar` | `accion` | click, clic_fisico, mensaje, cerrar, activar, teclas, fila, escribir, seleccionar, pagina, esperar_cierre, lanzar, uia_invocar, esperar, paso, funcion, menu, campo, rellenar, clic_texto, reiniciar, cerrar_todo, secuencia, escena, escena_leer, escena_importar + alias press/type/write/select/keys/close_dialog/wait/fill | — |
| `detalle` | enum | resumen, normal, completo | — |
| `via` | enum | `pvsyst`; `aproximacion` rechazada sin `PVSYST_MCP_APROXIMACIONES=1` | — |

### 1.17 Manifest, recursos y prompts

- **`mcpb/manifest.json`:** el bloque `_meta.pvsyst-mcp.tool_aliases` declara los alias de argumentos y valores para `componente`, `disenar_sistema`, `editar`, `entorno`, `escribir_archivo`, `gui_actuar`, `gui_observar`, `leer`, `listar`, `proyecto`, `pvsyst_cli`, `resultados`, `simular`, `sitio` y `workspace`. Se contrastaron los alias de `componente` (`action`→`accion`, `file`/`component`→`componente`, `create`→`crear`, etc.) y `entorno` (`license_activate`→`licencia_activar`, etc.) contra el comportamiento del servidor: coinciden con `Sinonimos` y con los valores aceptados en las llamadas.
- **Recursos `pvsyst://`:** `recetas` (3.979 caracteres), `cobertura` (11.083), `formato` (33.219), `cli` (14.605) y `gui` (357.534). Los cinco `resources/read` respondieron con su `mimeType` y contenido; `resources/templates/list` está vacío.
- **Prompts:** 9 (`disenar_desde_coordenadas`, `estudio_paso_a_paso`, `optimizar_orientacion`, `convertir_a_seguidor`, `autoconsumo_con_baterias`, `leer_resultados`, `operar_gui`, `explorar_archivo`, `diagnostico_entorno`). Los 9 `prompts/get` devolvieron un mensaje; los argumentos obligatorios se validaron con valores de ejemplo.

---

## 2. Bugs encontrados

Ordenados por severidad. Cada uno se reprodujo al menos dos veces salvo que se indique. Las llamadas y respuestas completas están en el `label` citado dentro de `.qa/exhaustiva/out_*.jsonl`.

### BUG-01 — `entorno accion:licencia_sync` ejecuta `lic-sync` (Alta)

**Llamada:**
```json
{"tool":"entorno","args":{"accion":"licencia_sync","dry_run":true}}
```
**Respuesta (extracto):**
```json
{"accion":"sync","command_line":"\"C:\Program Files\PVsyst8.1.8\PVsystCLI.exe\" lic-sync","dry_run":true}
```
**Esperado:** el mismo rechazo que da `pvsyst_cli`:
```json
{"tool":"pvsyst_cli","args":{"comando":"lic-sync"}}
→ 'lic-sync' changes the licence and pvsyst_cli does not touch it: use entorno {accion:"licencia_sync"}, which returns the command so you can run it yourself
```
**Real:** `entorno` no devuelve el comando: lo ejecuta. `src/server/tools/entorno.rs:163` mapea `licencia_sync` a `admin::licencia(..., "sync")`, y `src/admin/licencia.rs:19` ejecuta `lic-sync`. El mensaje de `pvsyst_cli` afirma lo contrario.

**Evidencia:** `ent_lic_sync_dry`, `empty_ent_sync_dry`.

**Severidad:** Alta (seguridad de licencia). **Reproducido:** 2 (`ent_lic_sync_dry`, `empty_ent_sync_dry`). **No se ejecutó de verdad** para no tocar la licencia.

### BUG-02 — `editar` ignora `dry_run` y escribe (Alta)

**Llamada:**
```json
{"tool":"editar","args":{"accion":"campos","archivo":"SYNTH_Plant.PRJ","objeto":"pvSite","campos":{"Comment":"DRYRUN_SHOULD_NOT_APPLY"},"dry_run":true}}
```
**Respuesta (extracto):**
```json
{"changes":[{"action":"replaced","key":"Comment","new":"DRYRUN_SHOULD_NOT_APPLY","old":"alias"}],
 "ignorados":["dry_run"],"historial_id":"20261111_054449_751_0000"}
```
**Comprobación:** `leer` posterior devuelve `Comment:"DRYRUN_SHOULD_NOT_APPLY"`. El archivo cambió con `dry_run:true`.

**Esperado:** no escribir, o rechazar `dry_run` por nombre como hacen `entorno`/`proyecto`/`sitio`/`simular`/`workspace`.

**Causa probable:** `src/vista_agente/contrato/ensayo.rs:12` (`implementa_ensayo`) y `src/vista_agente/contrato/pendiente.rs:53` (`CLAVE_DE_ENSAYO`) no incluyen `editar`; `src/server/tools/workspace_ejecucion.rs:192` ejecuta la edición sin mirar `dry_run`.

**Evidencia:** `edit_dry_real`, `leer_after_dry`.

**Severidad:** Alta. **Reproducido:** 2 (cambio vacío y cambio real). El archivo se restauró con `workspace accion:restaurar`.

### BUG-03 — `gui_actuar accion:lanzar` ignora `dry_run` y lanza PVsyst (Alta)

**Llamada:**
```json
{"tool":"gui_actuar","args":{"accion":"lanzar","dry_run":true}}
```
**Respuesta (extracto):**
```json
{"pid":23232,"ready":true,"ignorados":["dry_run"],"windows":{"count":2,...}}
```
**Esperado:** rechazar `dry_run` por nombre. La clave de ensayo de `gui_actuar` es `simular`, no `dry_run` (`src/vista_agente/contrato/pendiente.rs:61`); el esquema no lo documenta. Un `dry_run` de una acción que no lo implementa no debe tener efectos.

**Evidencia:** `dry_gui_act_lanzar`, `lanzar_dry_2`.

**Severidad:** Alta (efecto lateral durante un ensayo). **Reproducido:** 2 (`dry_gui_act_lanzar`, `lanzar_dry_2`). La GUI quedó abierta y se usó para el resto de pruebas.

### BUG-04 — `exportar_logs` sin destino escribe fuera del workspace (Alta)

**Llamada:**
```json
{"tool":"entorno","args":{"accion":"exportar_logs"}}
```
**Respuesta (extracto):**
```json
{"clasificacion":"exito","stdout_tail":"Exporting logs to C:\Users\Lenovo\ ...\nLogs ZIP file successfully created: C:\Users\Lenovo\2026_11_11-00_44_09-PVsystCLI818-logs.zip"}
```
**Esperado:** exigir `carpeta_destino` o usar una carpeta dentro del workspace. `src/admin/logs.rs:12-20` solo añade `-t` si el llamador pasa `carpeta_destino`; sin él, el CLI elige `%USERPROFILE%`. El archivo se borró tras documentar la ruta.

**Evidencia:** `empty_ent_exportar_logs`.

**Severidad:** Alta (confinamiento). **Reproducido:** 1.

### BUG-05 — `disenar_sistema` no crea el sitio; `sitio crear` con los mismos argumentos hace crash (Alta)

**Llamada real 1:**
```json
{"tool":"disenar_sistema","args":{"latitud":9.123,"longitud":-75.456,"kwp":8,"n_mod_serie":10,"n_strings":2,"nombre":"QA_Design_20261111","tilt":25,"azimut":0,"altitud":100,"timezone":-5,"country_code":"CO","region":"south_america","simular":true,"confirmar":true}}
```
**Respuesta:** `PVsystCLI no creó el sitio 'QA_Design_20261111_Site': revisa el paso de sitio antes de crear el proyecto`

**Llamada real 2:** mismos argumentos con `latitud:8.2`, `longitud:-74.5`, `nombre:"QA_Dis"` → `PVsystCLI no creó el sitio 'QA_Dis_Site': ...`

**Contraste:** `sitio crear` con los mismos argumentos (`QA_Dis_Site`, 8.2/-74.5) devuelve:
```json
{"annual_ghi_kwh_m2":1880.1,
 "create_site":{"clasificacion":"error","crash":"Instruction privilégiée","stdout_tail":"","success":false}}
```
La climatología NASA sí se calculó; el `create-site` del CLI crashea con `Instruction privilégiée` y stdout vacío. `disenar_sistema` pierde ese detalle y solo dice que no creó el sitio.

**Causa probable:** `src/easy_mode/sitio.rs:41` llama a `sites::crear_sitio_coordenadas`; `src/easy_mode/sitio.rs:73` (`exigir_sitio_creado`) solo mira `success`/`sit_file` y no adjunta la salida del CLI. El crash está en `src/sites/mod.rs:52` (`ejecutar`).

**Evidencia:** `real_disenar`, `real_disenar2`, `sit_crear_same`.

**Severidad:** Alta (el E2E principal no completa). **Reproducido:** 2 `disenar_sistema` + 1 `sitio crear` con los mismos argumentos.

### BUG-06 — `simular ejecutar` en un workspace no configurado crashea en vez de rechazarlo (Alta/Media)

**Llamada:** servidor con `PVSYST_WORKSPACE=C:\Users\Lenovo\AppData\Local\Temp\pvsyst_qa_20261111\PVsyst8.1_Data`:
```json
{"tool":"simular","args":{"accion":"ejecutar","proyecto":"QA_Proj2","variante":"VC0","output_csv":"Results/QA_Proj2_VC0.csv","report_pdf":"Results/QA_Proj2_VC0.pdf","timeout_s":300}}
```
**Respuesta:** `clasificacion:"error"`, `crash:"Instruction privilégiée"`, `stdout_tail:"Checking license..."`, `success:false`.

**Contraste:** el mismo proyecto, copiado al workspace real, simula bien (`QA_Real_20261111b`, 13.2 s, CSV+PDF). El CLI acepta `-w:<workspace de pruebas>` pero crashea al cargar; el servidor no comprueba que sea el workspace configurado en PVsyst.

**Causa probable:** `src/simulation/preparacion.rs:107-115` construye `-w:<workspace>` sin validar que sea el workspace de PVsyst.

**Evidencia:** `real_sim_qa`, `real_sim_qa2`, `real_sim_real_ws2`.

**Severidad:** Alta/Media. **Reproducido:** 2 (`real_sim_qa`, `real_sim_qa2`).

### BUG-07 — `convertir_meteo` gastó 1 ejecución TRIAL y reportó `costo.ejecuciones:0` (Media)

**Llamada:**
```json
{"tool":"sitio","args":{"accion":"convertir_meteo","input_csv_file":"Meteo/QA_Sig.csv","input_mef_file":"Meteo/QA_Geneva.MEF","input_sit_file":"Sites/QA_Geneva.SIT","output_met_file":"Meteo/QA_Convert.MET"}}
```
**Respuesta:** `clasificacion:"error"`, `detected_error:"Invalid date Line"`, `costo:{"ejecuciones":0,"restantes":248}`.
**Ledger:** antes 249, después 248. El banner del CLI imprimió 248. La ejecución se gastó, pero la respuesta dice 0.

**Causa probable:** `src/sites/mod.rs:33` (`costo_de`) cuenta solo si la clasificación es éxito; `src/adapters/ledger.rs` (`registrar`) usa el banner. El fallo del CLI también consume.

**Evidencia:** `real_sit_conv2`.

**Severidad:** Media (presupuesto). **Reproducido:** 2 intentos; el segundo gastó 1.

### BUG-08 — `meteo_pvgis` falla con crash en la conversión (Media)

**Llamada real:**
```json
{"tool":"sitio","args":{"accion":"meteo_pvgis","latitud":8.123,"longitud":-74.456,"nombre_sitio":"QA_PVGIS_20261111","confirmar":true}}
```
**Respuesta (extracto):**
```json
{"ok":false,"error":"crash: Instruction privilégiée","horas":8760,
 "csv":"...\Meteo\QA_PVGIS_20261111_PVGIS.csv",
 "mef":"...\Meteo\.staging\...\MCP_PVGIS_....MEF"}
```
El CSV de PVGIS se descargó (8760 h), pero el `convert-meteo` crasheó y no se creó el `.MET` final. El ledger no cambió (248).

**Causa probable:** `src/sites/pvgis/mod.rs:145` (`meteo_pvgis`) y `:114` (`construir_respuesta_meteo`); la conversión transaccional no distingue el crash de un error de formato.

**Evidencia:** `real_sit_pvgis`.

**Severidad:** Media. **Reproducido:** 1.

### BUG-09 — `p50_p90` en `dry_run` planifica un proyecto inexistente (Media)

**Llamada:**
```json
{"tool":"proyecto","args":{"accion":"p50_p90","proyecto":"NoExiste","variante":"VC0","dry_run":true}}
```
**Respuesta (extracto):**
```json
{"accion":"p50_p90","dry_run":true,"estado":"ok","variant_file":"...\NoExiste.VC0",
 "variant_file_exists":false,"siguiente":[{"tool":"proyecto","argumentos":{"accion":"p50_p90","proyecto":"NoExiste"}}]}
```
**Esperado:** error de proyecto/variante inexistente, como en `doctor` o `simular`.

**Causa probable:** `src/server/tools/proyectos.rs:197` llama a `projects::gui::p50::plan` sin comprobar existencia; `src/projects/gui/p50.rs:34` solo rellena `variant_file_exists:false`.

**Severidad:** Media. **Reproducido:** 2 (`proy_p50_bad`, `p50_bad2`).

### BUG-10 — `simular ejecutar` ignora parámetros sin reportarlos en `ignorados` (Media)

**Llamada:**
```json
{"tool":"simular","args":{"accion":"ejecutar","proyecto":"QA_Proj2","variante":"VC0","latitud":46.2,"longitud":6.1,"kwp":8,"tilt":30,"azimut":0,"n":2,"descripciones":{"a":"b"},"iniciales":{"a":1},"unidades":{"a":"kW"},"destino":"x","operacion":"y","trabajo":"z","abrir_gui":true,"confirmar":true,"dry_run":true}}
```
**Respuesta:** `ignorados:["unidades"]`; `cli_args` no cambia; el `siguiente` repite todos los argumentos, como si fueran a aplicarse.

**Esperado:** `ignorados` con `latitud`, `longitud`, `kwp`, `tilt`, `azimut`, `n`, `descripciones`, `iniciales`, `destino`, `operacion`, `trabajo`, `abrir_gui`, `confirmar` (o usarlos).

**Causa probable:** `src/simulation/preparacion.rs` (`push_options`) no lee esos campos; `src/server/tools/requeridos.rs:157` (`NO_LEIDOS`) no los declara para `simular ejecutar`.

**Severidad:** Media (contrato). **Reproducido:** 1 (`sim_ejec_geo2`).

### BUG-11 — `proyecto doctor` no reporta claves declaradas que no lee (Media)

**Llamadas:** `doctor` con `kwp:5`, `moneda:"USD"`, `n_mod_serie:10`.
**Respuesta:** no aparece `ignorados` en ninguna.
**Esperado:** `ignorados:["kwp"]`, `["moneda"]`, `["n_mod_serie"]`.

**Causa probable:** `src/server/tools/requeridos.rs:157` (`NO_LEIDOS`) solo lista un subconjunto; el resto de claves declaradas para otras acciones de `proyecto` no se reportan.

**Severidad:** Media. **Reproducido:** 3 (`proy_doctor_ignorado`, `...2`, `...3`).

### BUG-12 — El esquema no declara todos los requeridos por acción (Media)

Casos comprobados:

| Llamada | Error real | `Required by action` del esquema |
|---|---|---|
| `componente crear` sin `tipo` | `Missing required argument 'tipo' (p.ej. "PAN")` | solo `nombre_nuevo`, `plantilla` |
| `sitio crear` sin coordenadas | `Missing required argument 'latitud'` | no lista `crear` |
| `sitio convertir_meteo` sin archivos | `Missing required argument 'input_csv_file'` | no lista `convertir_meteo` |
| `simular ejecutar` sin proyecto | `Missing required argument 'proyecto'` | no lista `ejecutar` |
| `simular preparar` sin filas/parametros | `Missing required argument 'filas' (o 'parametros')` | no lista `preparar` |
| `workspace crear` sin ruta | `Missing required argument 'ruta'` | no lista `crear` |
| `workspace exportar` sin destino | error pidiendo `destino`/`permitir_fuera_del_workspace` | no lista `exportar` |
| `workspace importar` sin zip | `Missing required argument 'zip'` | no lista `importar` |
| `workspace exportar_proyecto` sin proyecto | `Missing required argument 'proyecto'` | no lista `exportar_proyecto` |
| `resultados kpis` sin proyecto/archivo | `'proyecto' is required for que:kpis` | no lista `kpis`/`resumen` |
| `editar campos` sin archivo | `Missing required argument 'archivo'` | no lista `campos` |
| `gui_actuar uia_invocar` sin operacion | `Missing required argument 'operacion'` | no lista `uia_invocar` |
| `gui_actuar menu` sin id/item | `accion 'menu' needs 'id', or the pair 'menu' and 'item'` | no lista `menu` |
| `gui_actuar fila` sin clase en `TProjectLoadForm` | pide `clase` | solo lista `ventana`, `indice` |

**Causa probable:** `requeridos_por_accion` incompleto en `src/server/tools/componentes.rs:48`, `src/server/tools/sitios.rs:71`, `src/server/tools/nativos.rs:191`, etc.

**Severidad:** Media (un agente que confíe en `tools/list` enviará llamadas incompletas). **Reproducido:** todos los casos de la tabla.

### BUG-13 — `leer` con `offset`/`max_bytes`/`base64` no busca en `Projects/` (Media)

**Llamada:**
```json
{"tool":"leer","args":{"archivo":"SYNTH_Plant.PRJ","offset":0,"max_bytes":20}}
```
**Respuesta:** `File 'SYNTH_Plant.PRJ' not found; candidates: Projects/SYNTH_Plant.PRJ`
**Contraste:** `leer` sin `offset`/`max_bytes`/`base64` sí lo encuentra.

**Causa probable:** `src/server/tools/workspace_ejecucion.rs:79-92`: la rama de bytes crudos llama a `crate::workspace::leer_archivo` sin el fallback de `Projects/` que usa la rama de proyecto.

**Severidad:** Media. **Reproducido:** 2 (`leer_offset`, `leer_base64`).

### BUG-14 — `crear_desde_ficha` de un módulo PAN no produce un módulo usable (Media)

**Llamada:**
```json
{"tool":"componente","args":{"accion":"crear_desde_ficha","tipo":"PAN","nombre_nuevo":"QA_Ficha","datos":{"Model":"QA 410","Manufacturer":"QA","PNom":410,"Technol":"mtSiMono","Isc":10.5,"Voc":50,"Imp":9.8,"Vmp":42,"NCelS":72}}}
```
**Respuesta tras 34 s:** `PVsyst no guardó ... PVsyst said: ... Los tamaños de este módulo no están definidos. No puede usarlo en la simulación!`

**Causa probable:** `src/vista_agente/esquemas/paneles.rs` no tiene campos de dimensiones (Length/Width) y `src/components/ficha/datos.rs` (`valores_de`) solo rellena las claves del esquema. La ventana de PVsyst no recibe los tamaños y se niega a guardar.

**Evidencia:** `comp_ficha`.

**Severidad:** Media (la acción no cumple su promesa para PAN). **Reproducido:** 1. Para OND sí funcionó (`QA_Inverter_10kW.OND`, 20 s).

### BUG-15 — `uia_invocar` se cuelga 10 s y abre la ventana equivocada (Media)

**Llamada:**
```json
{"tool":"gui_actuar","args":{"accion":"uia_invocar","ventana":"THomeForm","nombre":"Herramientas","rol":"button","operacion":"invocar"}}
```
**Respuesta:** `error:"UIA 'invocar' did not return in 10 s: PVsyst probably opened a modal form that is still open..."`.
**Ventanas después:** `TDatabaseForm` + `TChoixObjetPVForm`, no `TToolsForm`. La acción `paso` con el mismo objetivo sí abrió `TToolsForm` (con retraso).

**Causa probable:** `src/adapters/gui/uia_paso.rs:19` (`uia_invocar`) resuelve el elemento UIA y no verifica qué ventana abrió; el árbol UIA de THomeForm puede devolver otro botón con el mismo nombre/rol.

**Severidad:** Media. **Reproducido:** 1 (`gui_uia_herr`); `gui_uia_sistema` dio `Pattern not found`.

### BUG-16 — Cadenas en inglés con `PVSYST_MCP_IDIOMA=es` (Baja)

| Llamada | Respuesta en `es` | Esperado |
|---|---|---|
| `workspace crear` existente | `Workspace already exists: ... (pass sobrescribir=true to reuse)` | español |
| `componente crear tipo:"NOPE"` | `Unknown component kind 'NOPE'` | español |
| `gui_observar que:capturar ventana:"NoExiste"` | `Window 'NoExiste' not found` | español |
| `editar insertar_objeto` objeto inexistente | `Object 'NoExiste' not found` | español |

**Causa probable:** literales en `src/admin/workspace.rs:97`, `src/components/mod.rs`, `src/adapters/gui/acciones.rs`, `src/projects/anadir/objetos.rs`.

**Severidad:** Baja. **Reproducido:** 4.

### 2.1 Bugs menores y observaciones

| ID | Severidad | Qué | Evidencia | Archivo probable |
|---|---|---|---|---|
| BUG-17 | Baja | `entorno licencia` devolvió una vez `Empty output for 'lic-info'` con la GUI abierta; dos llamadas posteriores funcionaron | `empty_ent_licencia` | `src/adapters/cli/salida_json.rs`, `src/admin/licencia.rs` |
| BUG-18 | Baja | `gui_observar que:leer` ignora `ocr:true` y solo devuelve el título | `gui_leer_home`, `gui_leer_home_ocr` | `src/adapters/gui/ventanas.rs:233` |
| BUG-19 | Baja | `campo` falla con etiquetas ambiguas (`Núm. de inversores` ×3) y no acepta `clase`/`indice` para desambiguar | `gui_campo_inv` | `src/adapters/gui/campo_por_etiqueta.rs` |
| BUG-20 | Baja | `menu` exige `id` o `menu`+`item`, pero el esquema solo declara `ventana` | `gui_menu0` | `src/server/tools/nativos.rs:191` |
| BUG-21 | Baja | `fila` exige `clase` en `TProjectLoadForm`; sin ella falla y solo sugiere los controles medidos | `gui_fila0` vs `gui_fila0b` | `src/adapters/gui/fila/mod.rs` |
| BUG-22 | Baja | `p50_p90` real devuelve los 10 campos en `p50_p90` pero `vista.campos:[]` | `real_p50_2` | `src/projects/gui/p50.rs`, `src/server/tools/vista.rs` |
| BUG-23 | Baja | `exportar_logs` con `carpeta_destino` inexistente falla; no crea la carpeta | `ent_logs_export` | `src/admin/logs.rs` |
| BUG-24 | Baja | `simular` con `batch_params_file` estima 2 ejecuciones para un lote de 1 fila | `sim_ejec_dry_bpf` | `src/simulation/preparacion.rs` (`EJECUCIONES_LOTE`) |
| BUG-25 | Baja | `pvsyst_cli` devuelve `confirmed_by_output:false` para `version`/`lic-info` aunque la salida confirma | `cli_version`, `ent_lic` | `src/adapters/cli/clasificacion.rs:226` |
| BUG-26 | Baja | `leer` de un `.MEF` legacy devuelve `estado:"parcial"` y un error aunque sí lee metadatos | `leer_mef` | `src/meteo_mef`, `src/server/tools/vista.rs` |
| BUG-27 | Baja | `reiniciar` solo actúa si hay diálogo de fallo; no reinicia una instancia sana (puede ser intencional, pero no está documentado) | `gui_reiniciar` | `src/adapters/gui/recuperacion.rs` |

---

## 3. Lo verificado y correcto

- **Superficie MCP:** `tools/list` devuelve 15 tools, 0 recursos con plantilla, 5 recursos `pvsyst://` y 9 prompts; `resources/read` de los 5 recursos y `prompts/get` de los 9 prompts responden.
- **Contrato de argumentos:** alias en inglés (`action`, `what`, `window`, `file`, `project`, `type`, `keys`, `text`, `label`, etc.) resuelven al canónico; los valores de enum con alias (`create`, `projects`, `windows`, `press`, `close_dialog`, etc.) también.
- **Validación:** tipos erróneos, enums inexistentes, `null` en el discriminador, `detalle` inválido, `timeout_s` fuera de rango y `region` inválida se rechazan con mensajes que listan alternativas.
- **Rutas:** `..`, rutas absolutas fuera del workspace, `CON`/`NUL`, directorios y escritura fuera sin `permitir_fuera_del_workspace` se rechazan; con el permiso explícito se aceptan.
- **`workspace`:** crear, exportar, importar, exportar_proyecto, historial y restaurar funcionan; el zip con `../evil.txt` se rechaza; la restauración real repone el archivo y deja historial.
- **`editar`:** campos de PVObject y de variante, insertar/eliminar objetos, RVT (crear/listar/añadir/eliminar/reordenar/catálogo) y `plantilla_rvt` funcionan; la edición preserva bytes no tocados y verifica el resultado.
- **`proyecto`:** crear, renombrar, crear_variante, dimensionar con `n_mod_serie`+`n_strings`, doctor, añadir horizonte, seguidor, límite de inyección, almacenamiento, pérdidas, bombeo, economía (con sección existente), configurar en pasos y p50_p90 real funcionan.
- **`componente`:** crear desde plantilla, editar, borrar, importar, exportar y `crear_desde_ficha` de OND funcionan; los nombres sin extensión se normalizan y se avisa.
- **`sitio`:** `crear` desde coordenadas (con NASA) funcionó una vez; `escribir_horizonte` y `escribir_serie` escriben y preservan el resto; `convertir_meteo` valida rutas y `dry_run`; `meteo_pvgis` descarga 8760 h y valida el nombre.
- **`simular`:** `preparar` genera batch/param file; `estado` informa; `dry_run` muestra la línea de comando y `costo_estimado`; `paginas_reporte`, `idioma_reporte`, `extra_options`, `site`, `synthetic_weather`, `time_step`, fechas, `recalcular_sombreado` y `overwrite_default_csv` se reflejan en la línea de comando; la simulación real en el workspace real produjo CSV y PDF.
- **`resultados`:** kpis, csv, pérdidas, mensual, comparar (VC0/VC1), resumen y entregable leen los PDF/CSV; los errores de página ausente (P50/P90, carbono, economía) son claros y nombran la alternativa.
- **`pvsyst_cli`:** `version`, `help`, `lic-info`, opciones en lista/objeto, `dry_run`, `timeout_s` y rechazo de `lic-*` funcionan.
- **GUI:** se abrieron y leyeron THomeForm, TProjectLoadForm, TProjectForm, TSystemegridForm, TPVModuleForm, TSystemeAddForm (incluida la pestaña P50/P90), TPerspectiveForm, TSettingsForm, TDatabaseForm, TChoixObjetPVForm y TToolsForm. Funcionan `ventanas`, `controles`, `capturar`, `uia`, `arbol`, `formulario`, `vista`, `esquema`, `mensajes`, `funciones`, `menus`, `click`, `cerrar`, `activar`, `teclas`, `fila`, `escribir`, `seleccionar`, `pagina`, `esperar`, `esperar_cierre`, `funcion`, `menu`, `campo`, `rellenar`, `clic_texto`, `secuencia`, `escena`, `escena_leer`, `cerrar_todo`. La escena 3D real se creó y se leyó.
- **i18n:** con `PVSYST_MCP_IDIOMA=es` los mensajes principales (validación, doctor, resultados, CLI, GUI) salen en español; con `en` salen en inglés. Las excepciones están en BUG-16.
- **`detalle`:** 13 por defecto, 50 con `normal`, todos con `completo` (probado en `listar archivos` 175 y `rvt catalogo` 122).
- **`vista.siguiente`:** se enviaron 8 pasos tal cual (`listar archivos`, `listar objetos`, `resultados perdidas`, `proyecto doctor`, `workspace historial`, `entorno licencia`, `gui_observar esquema`, `gui_observar ventanas`) y funcionaron.

---

## 4. Lo que no se pudo probar y por qué

| Área | Qué falta | Razón |
|---|---|---|
| Verificación visual | Ninguna captura se miró con los ojos | El modelo de esta sesión no acepta imágenes; se usaron OCR, `vista`, `controles`, `ventanas` y `uia`. Las capturas quedan como evidencia, marcadas «sin verificación visual». |
| `gui_actuar clic_fisico` | El caso exitoso | El primer intento coincidió con `TDEMOForm` modal; no se repitió con la ventana libre para no seguir moviendo la GUI. El caso de error se probó. |
| `gui_actuar escena_importar` | Importación real de una malla | No hay una malla válida (3DS/DAE/PVC/PVC2/PVZ2) en el workspace ni en `DataRO`. Se probó el rechazo de archivo inexistente. |
| `gui_actuar reiniciar` | Reinicio exitoso | Solo actúa ante un diálogo de fallo (madExcept/inestabilidad); no había ninguno. El rechazo se probó. |
| `gui_actuar uia_invocar` | Invocación correcta de un botón | La llamada se colgó 10 s y abrió `TDatabaseForm`/`TChoixObjetPVForm` en vez de `TToolsForm` (BUG-15). `paso` sí abrió `TToolsForm`. |
| `entorno licencia_sync` | Ejecución real | Prohibido por la tarea: `lic-sync` no debe ejecutarse. Solo se probó con `dry_run:true`; el bug es que no lo rechaza. |
| `sitio convertir_meteo` | Conversión exitosa | Dos CSV/MEF de ejemplo fallaron con `Invalid date Line`; el CLI gastó 1 ejecución en el segundo. No se probó un tercer formato para no gastar más. |
| `sitio meteo_pvgis` | MET final | El CSV de PVGIS se descargó, pero `convert-meteo` crasheó (BUG-08). |
| `disenar_sistema` | E2E completo | Dos intentos reales fallaron en el sitio (BUG-05). |
| `proyecto p50_p90` + informe | Página P50/P90 en el PDF | Se configuró la evaluación, pero no se volvió a simular con `paginas_reporte:["p50p90"]` para no gastar otra ejecución TRIAL. `resultados que:p50_p90` se probó contra un PDF sin la página y devolvió el error esperado. |
| `resultados carbono/economia` | Lectura real | Los PDF disponibles no tienen esas páginas; se probó el error claro. Crear las páginas exigía otra simulación TRIAL. |
| `componente crear_desde_ficha` PAN | Módulo usable | PVsyst rechaza guardar porque faltan dimensiones (BUG-14). OND sí funcionó. |
| `escribir_archivo` | `base64`, `sobrescribir` | No se enviaron; el caso principal y las rutas peligrosas sí. |
| `leer`/`listar`/`resultados` | `workspace` como override | Se cambió el workspace por entorno del servidor; no se probó el argumento por llamada en estas tools. |
| `simular` | `log_level`, `input_rvt_file`, `variables` en lote | No eran necesarios para la cobertura funcional; el resto de opciones sí se probó. |
| `gui_observar` | `proceso`, `pids`, `compacto`, `completo`, `visibles`, `todos`, `incluir_uia`, `max_nodos`, `lineas`, `imagen`, `recorte`, `funcion`, `entidad` | No eran necesarios; `profundidad`, `detalle`, `ocr` y `ruta` sí se probaron. |
| `gui_actuar` | `proceso`, `pids`, `titulo`, `verificar`, `proyecto`, `esperar_segundos`, `cerrar_demo`, `id`, `subitem`, `dx`/`dy`, `unidades` | No eran necesarios o requieren una ventana/archivo que no estaba disponible. |
| `pvsyst_cli` | `log_level` | No se probó; el resto de opciones sí. |

### 4.1 Recomendaciones para reducir los clics nulos

**Para el agente que maneja la GUI**

1. Antes de un clic físico (`clic_fisico`, `clic_texto`, `fila`), llama a `gui_actuar accion:activar` sobre la ventana y comprueba en `gui_observar que:ventanas` que esté `visible:true`, `minimized:false` y `enabled:true`. Si sigue minimizada, no sigas: el clic caerá fuera.
2. Revisa `que:ventanas` antes de cada paso. Si hay `TDEMOForm`, `TMessageForm`, `TChoixObjetPVForm` o `TDatabaseForm`, ciérralo o contéstalo primero. Un modal deshabilita la ventana de debajo y convierte el clic en nulo.
3. Prefiere las rutas sin clic físico: `que:vista` + `rellenar` (claves estables), `campo` (etiqueta), `seleccionar` (combo) y `escribir` (control). `clic_texto`/`clic_fisico` solo cuando no exista clave o etiqueta.
4. Para listas y árboles: `fila` con `clase` e `indice`, y después `teclas:"RETURN"` o `click` sobre el OK por hwnd. `fila` solo selecciona; no abre el elemento.
5. Si un texto se repite (`Sistema`, `OK`, `Núm. de inversores`), pasa `indice` a `clic_texto` o `clase`/`campo:{clase,x,y}` a `campo`/`escribir`; si el título se repite, pasa el hwnd.
6. Verifica el efecto después de cada acción: `que:ventanas` (¿abrió/cerró?), `que:formulario`/`que:vista` (¿cambió el campo?), `que:capturar` (evidencia). No encadenes acciones a ciegas.
7. No minimices PVsyst durante la sesión. Si lo haces, `activar` antes de continuar.
8. Para cargar un proyecto: `funcion:proyecto.cargar`, escribe el nombre en `edit_39_1017`, `fila` sobre `TVirtualStringTree` fila 0 y `RETURN`; el OK por clic no siempre carga.
9. Para P50/P90: `funcion:variante.gestion_energia`, `pagina` índice 2, y lee las casillas con `que:vista`/`que:formulario`; `rellenar` con las claves `checkbox_...`/`edit_...` si hay que cambiarlas.
10. Usa `que:capturar` como evidencia: no necesita foco y funciona aunque el clic falle.

**Para el servidor (mejoras que evitarían esta clase de fallo)**

1. `gui_actuar` debería restaurar y traer al frente la ventana antes de `clic_fisico`/`clic_texto`/`fila`, y devolver un error nombrado si `IsIconic` sigue activo, en vez de un clic nulo.
2. Detectar modales (`TDEMOForm`, `TMessageForm`, `TChoixObjetPVForm`, `TDatabaseForm`) antes de actuar y devolver `modal: <clase> <hwnd>`; hoy algunos se detectan y otros no.
3. `fila` debería aceptar `abrir:true`/`confirmar:true` para pulsar Enter/OK, o documentar en la descripción que solo selecciona. Hoy el llamador puede creer que la fila se abrió.
4. `clic_texto` debería verificar el efecto (ventana abierta/cerrada, campo cambiado) y devolver `verificado:false` si no hubo cambio; hoy devuelve éxito aunque no pase nada.
5. `uia_invocar` debería verificar el elemento y la ventana resultante; el caso `Herramientas` abrió `TDatabaseForm` en vez de `TToolsForm` (BUG-15).
6. `campo` debería aceptar `clase`/`indice` para desambiguar etiquetas repetidas (BUG-19).
7. `click` por título debería exigir hwnd cuando hay varios controles con ese título, o resolver solo dentro de la ventana nombrada; `ventana:"OK"` puede pulsar el OK de otra ventana.
8. `que:leer` debería honrar `ocr:true` o documentar que solo lee el título (BUG-18).
9. `que:ventanas` debería incluir `foreground` y `modal` además de `minimized`/`visible`/`enabled`, para que el agente decida antes de hacer clic.
10. `fila` debería verificar el texto de la fila, no solo la banda resaltada; hoy confirma que la fila 0 está resaltada, no que sea el proyecto buscado.
11. El OK de `TProjectLoadForm` (`TPngBitBtn`) no cargó el proyecto con `click`; `RETURN` sí. Conviene probar el mecanismo de clic en ese botón o documentar el atajo.

### 4.2 Diseño para un inventario completo de la GUI

El objetivo es separar **inventario** (qué controles existen, visibles u ocultos) de **clasificación** (cuáles son botones/campos). Hoy `que:formulario` hace las dos cosas a la vez y parte de «hijos visibles + lista blanca»; por eso se quedan fuera pestañas inactivas, clases no listadas, controles ocultos y celdas.

| Gap | Cambio propuesto | Dónde |
|---|---|---|
| Pestañas inactivas | `que:formulario` con `todos:true`: `EnumChildWindows` ya devuelve los hijos ocultos; dejar de filtrar por `visible` y añadir `pestana`/`parent`. No hace falta `pagina` para inventariar. | `src/adapters/gui/ocr/formulario.rs` (`hijos_visibles_con_clase`), `src/adapters/gui/ventanas.rs` (`hijos_visibles`) |
| Clases fuera de la lista blanca | Añadir `otros` con todo lo no clasificado (`TGridPanel`, `TVirtualStringTree`, `TPageControl`, botones custom) y `clase`/`hwnd`/`rect`/`texto`/`visible`/`enabled`. | `src/adapters/gui/ocr/formulario.rs` (`clasificar_controles`) |
| Controles ocultos/condicionales | Incluirlos con `visible:false`/`enabled:false`; `condicionado_por` heurístico. Los que se crean al activar la condición se documentan como `creado_al_activar`. | `src/adapters/gui/ocr/formulario.rs` |
| Celdas de árboles/grids/menús | `que:elementos` con UIA `FindAll`/`ItemContainerPattern`/`VirtualizedItemPattern` y MSAA `IAccessible`; `gui_actuar accion:elemento` o `clic_texto` con `indice`. Si no hay UIA/MSAA, `solo_vision:true` y fallback `fila`/`menu`. | `src/adapters/uia`, `src/adapters/gui/uia_vista.rs`, `src/adapters/gui/fila` |
| Truncación | `detalle:completo` por defecto en `que:formulario`/`que:controles`/`que:vista` cuando se pide inventario; límite alto (500) y `truncado:true` con `total`. | `src/server/tools/recorte.rs`, `src/server/tools/nativos.rs` |
| OCR | Orden `WM_GETTEXT` → UIA `Name`/`LabeledBy` → MSAA `accName` → OCR → `null`; añadir `etiqueta_fuente`. | `src/adapters/gui/ocr/formulario.rs`, `src/adapters/gui/uia_vista.rs` |
| Controles dinámicos | `que:ventanas` con `foreground`/`modal`/`iconic`; `esperar`/`paso` con `verificar`; relectura tras cada interacción. | `src/adapters/gui/ventanas.rs`, `src/adapters/gui/esperas.rs`, `src/adapters/gui/uia_paso.rs` |
| Mapa estático | `pvsyst://gui` con `measured_on`, `pvsyst_version`, `windows_total`, `cobertura`; aviso si la versión en ejecución difiere. | `src/adapters/gui/catalogo/recurso.rs`, `docs/gui_mapa.json` |

**API propuesta**

```json
{"tool":"gui_observar","args":{"que":"inventario","ventana":"TSystemeAddForm","todos":true,"detalle":"completo"}}
```
```text
{"ventana":"0x1209FE","clase":"TSystemeAddForm","titulo":"Gestión de la energía...",
 "pestanas":[{"nombre":"Estimación P50 - P90","seleccionada":false,"controles":["0x80AB8","0x90A0A"]}],
 "controles":[
   {"clase":"TCheckBox","hwnd":"0x80AB8","rect":{"left":120,"top":233,"right":567,"bottom":256},
    "texto":"Activo","visible":true,"enabled":true,"parent":"0x...",
    "pestana":"Estimación P50 - P90","rol":"checkbox","etiqueta":"Activo",
    "etiqueta_fuente":"window_text","marcado":true,"patrones":["toggle"]}
 ],
 "total":21,"truncado":false}
```

- `que:formulario` sigue siendo la vista OCR/humana; `que:inventario` es la vista máquina.
- `que:controles` sigue siendo el volcado Win32 crudo; `que:uia`/`que:arbol` el volcado UIA.
- `que:elementos` cubre celdas/items que no son HWND.

**Orden de implementación**

1. `todos:true` + `otros` + `etiqueta_fuente` + `pestana`/`parent` (pequeño, sin tool nueva).
2. `que:inventario` que fusione Win32 + UIA (mediano).
3. `que:elementos` + MSAA para celdas (grande).
4. Regenerar `docs/gui_mapa.json` por versión y añadir un test de cobertura (datos/test).

**Tests de aceptación**

- P50/P90: `que:inventario todos:true` sobre `TSystemeAddForm` en la pestaña 0 devuelve las 21 casillas/edits con `pestana:"Estimación P50 - P90"`, sin `pagina`.
- `TPVModuleForm`: los controles de dimensiones aparecen en `otros`/`inventario` aunque `components.panels` no los tenga en el esquema.
- `TSystemegridForm`: los 3 controles «Núm. de inversores» aparecen con `clase`/`hwnd`/`rect`, para que `campo` desambigüe por `clase`/`indice`.
- `TVirtualStringTree`: los items aparecen en `que:elementos` con `nombre`/`rect`/`indice`.
- `crear_desde_ficha` PAN: si el esquema recibe dimensiones, el inventario las muestra y el módulo se guarda.

**Riesgos y límites**

- Enumerar ocultos puede devolver cientos de controles; usar `detalle:completo` con tope y paginar por ventana/pestaña.
- Algunos controles VCL se crean perezosamente; `todos:true` puede no verlos hasta activar la pestaña.
- Owner-drawn sin UIA/MSAA: `solo_vision:true`; no hay garantía del 100 %.
- Cambiar de pestaña para inventariar puede disparar vetos de validación (`TPVModuleForm`); preferir la enumeración de hijos ocultos.
- El mapa estático sigue siendo una foto: la fuente de verdad de una ventana abierta es el inventario en vivo.

### 4.3 Independencia del idioma de la GUI

**Problema:** si las pestañas, botones y etiquetas estáticas se guardan por su texto en español, un PVsyst en inglés/francés no las encuentra. El servidor ya tiene este acoplamiento:

- `src/adapters/gui/rotulos.rs` solo define `Idioma::Es`.
- `src/projects/gui/p50.rs:16` usa `rotulos_p50(Idioma::Es)` y busca «Gestión de la energía», «Activo», etc.
- `src/adapters/gui/catalogo/entradas/*.rs` usa captions en español (`Sistema`, `Orientación`, `Gestión de la energía`).
- `docs/gui_mapa.json` guarda títulos y pestañas medidos en español.
- `PVSYST_MCP_IDIOMA` solo cambia los mensajes del MCP, **no** el idioma de la GUI de PVsyst.

**Regla:** lo estático es la estructura; el texto se lee en vivo.

| Estable entre idiomas | Dependiente del idioma |
|---|---|
| Clase de ventana (`TProjectForm`, `TSystemeAddForm`) | Título de la ventana |
| Clase de control (`TCheckBox`, `TEdit`, `TPngBitBtn`) | Caption del botón |
| `hwnd`, `rect`, ancla a 96 DPI | Texto de la etiqueta |
| Índice de pestaña (0,1,2…) | Nombre de la pestaña |
| `id`/posición de menú | Texto del menú |
| Claves del esquema (`components.panels`, `variante.system`) | — |
| Claves de `vista` (`checkbox_134_34`, `edit_264_238`) | — |
| `fila` con geometría medida | — |

**Cambios concretos**

1. `pestanas` con `indice`; `pagina` por índice. `p50` usa `pagina indice:2`, no el nombre.
2. `campo`/`escribir`/`seleccionar` por `clase`+`x,y` o por `vista`; la etiqueta solo como comodidad leída en vivo.
3. `funcion` por `id`/posición de menú o por clase+ancla; `Funcion::boton` por caption solo con alias de idioma.
4. `rotulos.rs` con `Idioma::Es|En|Fr|De` para los captions inevitables (OK/Aceptar, Cancelar/Cancel, Cerrar/Close) y para `p50`.
5. Detectar el idioma de la GUI (`PVsyst.ini` o un caption conocido) y exponer `gui_observar que:idioma`; no confundir con `PVSYST_MCP_IDIOMA`.
6. `gui_mapa.json` con `clase`, `indice_pestana`, `clase_control`, `ancla`, `rol`, `patron`; títulos/nombres opcionales (`titulo_es`, `titulo_en`) o fuera del mapa.
7. Resolución de etiqueta `WM_GETTEXT` → UIA `Name`/`LabeledBy` → MSAA `accName` → OCR; `etiqueta_fuente`.
8. `que:esquema`/`pvsyst://gui` con `idioma_medido` y aviso si la GUI en ejecución usa otro.

**Tests de aceptación**

- `p50_p90` en un PVsyst en inglés: `variante.gestion_energia` + `pagina indice:2` + `que:vista`; las casillas se encuentran por índice/clase, no por «Activo».
- `proyecto.cargar` en inglés: `edit_39_1017` + `fila` + `RETURN`; no depende del título «Proyectos».
- `TSystemegridForm` en inglés: `variante.system` + `que:vista`; los 3 controles «Núm. de inversores» se distinguen por `clase`/`hwnd`, no por el caption.
- `componente crear_desde_ficha` en inglés: `components.panels`/`components.inverters` por claves, no por «Modelo»/«Fabricante».

**Riesgos**

- El orden de pestañas puede cambiar entre versiones; el índice es más estable que el nombre, pero no inmutable.
- Las posiciones de menú pueden moverse; el `id` es más estable que la posición.
- OCR y traducciones automáticas no son fiables; por eso el texto es el último recurso.
- Un control sin `hwnd`/UIA/MSAA sigue siendo `solo_vision:true`.

### 4.4 Eficiencia: menos llamadas y más rápidas

**Lo que medí en esta campaña:** 749 llamadas. No es un flujo normal: es QA exhaustivo. Los mayores consumidores fueron `gui_observar que:ventanas` (62), `leer` (78 en total, muchos repetidos), `simular ejecutar` (35, muchos `dry_run`), `gui_observar que:formulario` (28), `listar que:archivos` (28), `gui_actuar funcion` (14), `entorno diagnostico` (14), `proyecto doctor` (12) y `gui_observar que:capturar` (15). Un flujo E2E normal debería ser 3-5 llamadas.

**Tabla de reducción**

| Tarea | Llamadas naive | Llamada optimizada | Ahorro |
|---|---|---|---|
| Diseño E2E | `sitio crear` + `proyecto crear` + `dimensionar` + `simular` + `resultados kpis` (5) | `disenar_sistema` (1) | 4 |
| Resumen de resultados | `kpis` + `mensual` + `perdidas` (3) | `resultados que:resumen` (1) | 2 |
| Editar N campos de una variante | N `editar campos` | 1 `editar campos` con `ediciones:[...]` | N-1 |
| Configurar N cambios de variante | N `proyecto ...` | 1 `proyecto configurar` con `pasos:[...]` | N-1 |
| Leer un formulario GUI | `controles` + `formulario` + `vista` + `uia` (4) | `que:inventario` (1) | 3 |
| Rellenar un formulario | N `campo`/`escribir` | 1 `rellenar` | N-1 |
| Secuencia GUI | N `gui_actuar` | 1 `secuencia` con `pasos:[...]` | N-1 |
| Escena 3D con N objetos | N `escena` | 1 `escena` con `objetos:[...]` | N-1 |
| Simulación por lotes | N `simular ejecutar` | 1 `simular ejecutar` con `filas:[...]` | N-1 (pero N ejecuciones TRIAL) |
| Comparar N variantes | N `kpis` | 1 `resultados que:comparar` con `variantes:[...]` | N-1 |
| Listar componentes | N `leer` | 1 `listar que:componentes` con `tipo`/`pnom_min`/`pnom_max`/`texto` | N-1 |
| Cargar proyecto en GUI | `funcion` + `escribir` + `fila` + `teclas` (4) | 1 `secuencia` con 4 pasos | 3 |
| P50/P90 | `funcion` + `pagina` + `vista` (3) | 1 `secuencia` (funcion+pagina) + 1 `vista` | 1 |

**Qué cachear durante la sesión**

- `tools/list` y `pvsyst://gui`/`pvsyst://cobertura`: no cambian; leer una vez.
- `entorno diagnostico`: workspace, CLI y GUI; una vez al inicio.
- `listar que:proyectos` con `include_variants`: una vez; reutilizar nombres y variantes.
- `gui_observar que:esquema`/`que:funciones`: una vez; el catálogo no cambia.
- `gui_observar que:ventanas`: solo después de una acción que pueda abrir/cerrar ventanas; no después de `escribir`/`seleccionar`/`campo`.
- `leer` de un archivo que no cambió: no repetir; si la tool ya devolvió `verified_after_write`, no re-leer.
- `proyecto doctor`: una vez tras los cambios, no antes de cada paso.

**Qué evitar**

- `gui_observar que:menus`: 5.9 s y payload grande; solo si la función no está en el catálogo.
- `que:uia`/`que:arbol` con `profundidad` alta: 187-473 ms y árboles grandes; usar `max_nodos`/`profundidad` ajustados.
- `que:capturar` para verificar: 100-200 ms y no se puede leer si el modelo no acepta imágenes; solo evidencia.
- `que:ventanas` después de cada paso: 62 llamadas en esta campaña, la mayoría redundantes.
- `simular ejecutar` con `dry_run` repetido para opciones que no cambian la línea de comando (`n`, `latitud`, `kwp`… se ignoran; BUG-10).
- `detalle:completo` cuando basta `resumen`/`normal`: el payload crece.
- `leer` de un PDF/CSV entero cuando `resultados que:csv`/`kpis`/`resumen` ya extrae lo necesario.
- `pvsyst_cli` para comandos que ya tienen tool (`simular`, `sitio`, `resultados`).

**Reglas de oro**

1. Una llamada de intención antes de varias atómicas: `disenar_sistema`, `resultados resumen`, `proyecto configurar`, `editar ediciones`, `simular filas`.
2. Cachea lo estable; no re-descubras.
3. Usa `vista.siguiente` como plan: ya trae el siguiente paso con argumentos.
4. Usa `dry_run` para validar y `costo_estimado` para decidir; no repitas el mismo dry_run.
5. Ajusta `detalle` y `limite`/`max_bytes` al tamaño necesario.
6. Para GUI, `que:vista` + `rellenar` (1-2 llamadas) en vez de `que:formulario` + N `campo`.
7. Para archivos, `leer` con `objeto`/`plano`/`serie`/`limite` en vez de leer todo.
8. Para componentes, `listar` con filtros en vez de `leer` cada archivo.
9. Para resultados, `resumen`/`comparar` en vez de N `kpis`.
10. Para proyectos, `crear` con `modulo`/`inversor`/`n_mod_serie`/`n_strings`/`sitio` en vez de crear+dimensionar+vincular.
11. Para simulación, `preparar` una vez y reutilizar `batch_params_file`/`input_param_file`.
12. Para GUI, `secuencia` para acciones dependientes y `cerrar_todo` al final.
13. Para workspace, `exportar`/`importar` en bloque y `historial` una vez.
14. No cambies `PVSYST_MCP_IDIOMA` a mitad de sesión; arranca con el idioma.
15. El arnés `.qa` es para QA, no para uso normal; usa las tools nativas.

**Medición de tiempos (esta máquina)**

| Llamada | Tiempo típico | Nota |
|---|---|---|
| `gui_observar que:ventanas` | 10-15 ms | barata, pero 62 llamadas suman |
| `gui_observar que:formulario` | 0.5-1.1 s | OCR; usar `vista`/`inventario` si basta |
| `gui_observar que:capturar` | 100-200 ms | PNG; no verifica |
| `gui_observar que:uia` | 187 ms | árbol grande |
| `gui_observar que:arbol` | 473 ms | árbol grande |
| `gui_observar que:menus` | 5.9 s | evitar si hay `funcion` |
| `resultados que:resumen` | 649 ms | 1 llamada vs 3 (~663 ms) |
| `resultados que:comparar` (2 variantes) | 7.6 s | 1 llamada vs N |
| `simular ejecutar` real | 13.2 s | 1 ejecución TRIAL |
| `proyecto p50_p90` real | 15.3 s | GUI |
| `componente crear_desde_ficha` OND | 20 s | GUI |
| `componente crear_desde_ficha` PAN | 34 s (falla) | GUI + validación |

**Presupuesto recomendado**

- Diseño E2E: 1 `disenar_sistema` + 1 `resultados resumen` + 1 `workspace historial` = 3 llamadas.
- Ajuste de un proyecto existente: 1 `leer` + 1 `editar ediciones`/`proyecto configurar` + 1 `doctor` = 3.
- Simulación: 1 `simular preparar` + 1 `simular ejecutar` + 1 `resultados resumen` = 3.
- GUI: 1 `que:ventanas` + 1 `que:vista` + 1 `rellenar` + 1 `que:ventanas` de verificación = 4.
- Si una tarea necesita más de 8-10 llamadas, probablemente hay una tool de intención que la colapsa.

**Antipatrones observados en esta campaña**

- 62 `que:ventanas` (la mayoría para confirmar lo que ya se sabía).
- 35 `simular ejecutar` (muchos `dry_run` con opciones que no cambian la línea de comando).
- 28 `que:formulario` (OCR lento; `vista`/`inventario` habría bastado en varios).
- 15 `que:capturar` (evidencia, no verificación).
- 78 `leer` (muchos repetidos del mismo archivo).
- 14 `entorno diagnostico` y 12 `proyecto doctor` (cacheables).

---

## 5. Ejecuciones TRIAL gastadas

### 5.1 Ledger antes y después

| Momento | Workspace | `ledger.json` | Banner del CLI |
|---|---|---|---|
| Inicio pruebas | pruebas | 250 | — |
| Tras `sitio crear` real | pruebas | 249 | 249 |
| Tras `convertir_meteo` (CSV mensual) | pruebas | 249 | 249 |
| Tras `convertir_meteo` (CSV Sig) | pruebas | 248 | 248 |
| Tras `meteo_pvgis` | pruebas | 248 | 248 |
| Tras `simular` QA_Proj2 (2 intentos) | pruebas | 248 | 248 |
| Inicio pruebas reales | real | 246 | — |
| Tras `simular` QA_Real_20261111 (módulo sin dimensiones) | real | 246 | — |
| Tras `simular` QA_Real_20261111b (sitio ausente) | real | 246 | — |
| Tras `simular` QA_Real_20261111b (éxito) | real | 246 | 247 antes → 246 después |
| Tras `disenar_sistema` (2 intentos) | real | 246 | — |
| Tras `p50_p90` real | real | 246 | — |
| Tras `componente crear_desde_ficha` OND | real | 246 | — |

### 5.2 Coste estimado y gasto real

| Acción | `costo_estimado` | Gasto real | Comentario |
|---|---|---|---|
| `sitio crear` (coordenadas) | 1 | 1 | Éxito; 250→249 |
| `sitio convertir_meteo` | 1 | 1 | Dos intentos; el segundo gastó 1 (249→248) y falló |
| `sitio meteo_pvgis` | 2 | 0 | Falló en la conversión; el ledger no cambió |
| `simular ejecutar` | 1 (2 con `-bpf`) | 1 | Éxito en el workspace real; el banner pasó de 247 a 246. Los intentos en el workspace de pruebas no cambiaron el ledger |
| `disenar_sistema` | 2 | 0 | Dos intentos; falló en el sitio antes de simular |
| `proyecto p50_p90` | 0 | 0 | Configura la variante; la simulación posterior es la que gasta |
| `componente crear_desde_ficha` | 0 | 0 | No es TRIAL |

**Total gastado: 3 ejecuciones TRIAL** (2 en el workspace de pruebas, 1 en el real). El contador del CLI salta entre estados (documentado en `src/adapters/ledger.rs`); por eso el ledger real quedó en 246 aunque el banner bajó de 247 a 246.

### 5.3 `dry_run` usados

Se usaron `dry_run` en `simular ejecutar` (muchas combinaciones), `sitio crear`, `sitio convertir_meteo`, `proyecto p50_p90`, `disenar_sistema`, `workspace restaurar`, `pvsyst_cli`, `entorno licencia`/`exportar_logs` y `gui_actuar escena`/`rellenar`. Los `dry_run` de acciones que no los implementan se probaron y se documentaron (BUG-02, BUG-03, BUG-10).

---

## 6. Capturas de pantalla

Guardadas en `docs/capturas_2026-11-11/`. **Todas están sin verificación visual directa**: el modelo de esta sesión no puede leer imágenes. Se enlazan como evidencia y se describe qué ventana muestran; la comprobación funcional se hizo con `gui_observar que:formulario`/`vista`/`controles`.

| Captura | Ventana / momento | Qué muestra | Verificación |
|---|---|---|---|
| [01_home.png](capturas_2026-11-11/01_home.png) | `THomeForm` al lanzar | Pantalla de inicio de PVsyst 8.1.8 en modo PRUEBA | Sin verificación visual; `que:formulario` leyó botones y textos |
| [03_before_gui.png](capturas_2026-11-11/03_before_gui.png) | `THomeForm` antes de las acciones | Estado previo a `activar`/`mensaje`/`teclas` | Sin verificación visual |
| [04_project_load.png](capturas_2026-11-11/04_project_load.png) | `TProjectLoadForm` | Diálogo «Proyectos» con el buscador y la lista | Sin verificación visual; `que:formulario` leyó textos y botones |
| [06_project_form.png](capturas_2026-11-11/06_project_form.png) | `TProjectForm` | Proyecto `MCP_FullTest.PRJ` cargado, variante VC0 | Sin verificación visual; `que:vista` leyó descripción, meteo y variante |
| [08_system_form.png](capturas_2026-11-11/08_system_form.png) | `TSystemegridForm` | Definición del sistema de red, subconjunto PV Array | Sin verificación visual; `que:vista` leyó `n_modulos_serie`, `n_cadenas`, `n_inversores` |
| [09_module_form.png](capturas_2026-11-11/09_module_form.png) | `TPVModuleForm` | Formulario de módulo FV, pestaña «Datos básicos» | Sin verificación visual; `que:vista` leyó modelo, fabricante, PNom, Isc, Voc… |
| [10_3d_editor.png](capturas_2026-11-11/10_3d_editor.png) | `TPerspectiveForm` | Editor 3D de sombreado | Sin verificación visual; `que:formulario` leyó botones y textos |
| [11_energy_form.png](capturas_2026-11-11/11_energy_form.png) | `TSystemeAddForm` | «Gestión de la energía», pestaña «Temperatura inversor» | Sin verificación visual; `que:formulario` leyó las casillas de temperatura |
| [12_p50_tab.png](capturas_2026-11-11/12_p50_tab.png) | `TSystemeAddForm` | Pestaña «Estimación P50 - P90» con las casillas y valores | Sin verificación visual; `que:formulario` y `que:vista` leyeron las 21 claves |

La captura `12_p50_tab.png` es la que responde a la pregunta sobre las casillas de P50/P90: sí aparecen, y `que:vista` las expone como `checkbox_134_34`, `combobox_216_178`, `edit_240_238`, `edit_264_238`, `checkbox_266_310`, `edit_318_238`, `checkbox_320_310`, `edit_342_238`, `checkbox_344_310`, `radiobutton_364_633`, `edit_366_238`, `checkbox_368_310`, `radiobutton_382_633`, `edit_390_238`, `checkbox_392_310`, `edit_414_34`, `edit_414_238`, `edit_416_380`, `edit_439_380`, `checkbox_503_34`, `checkbox_524_34`.

---

## 7. Limpieza

- Se restauró `PVsyst.ini` desde `.qa/exhaustiva/PVsyst.ini.before` después de probar `preferencias_editar`.
- Se retiraron del workspace real los proyectos QA (`QA_Real_20261111`, `QA_Real_20261111b`), sus resultados, el inversor `QA_Inverter_10kW.OND`, el módulo parcial `QA_Ficha.PAN`, la escena `QA_Escena.SHD`, la copia `Geneva_MN90.SIT`, `logs_qa/` y `capturas/`.
- El workspace de pruebas `C:\Users\Lenovo\AppData\Local\Temp\pvsyst_qa_20261111` se eliminó al terminar.
- La GUI se cerró con `gui_actuar accion:cerrar_todo`; el proceso de PVsyst se terminó al final.
- No se tocó ningún proyecto preexistente del usuario. Los archivos `ComposPV/Inverters/Inverter.Fav` y `ComposPV/PVmodules/PVModule.Fav` cambiaron de fecha al abrir la ventana de bases de datos de la GUI; todos los `.Fav` del workspace están a 0 bytes (también Battery/Pump/Regulator), así que no se perdió contenido.

---

## 8. Datos crudos

| Ruta | Contenido |
|---|---|
| `.qa/exhaustiva/calls/*.json` | Lotes de llamadas enviadas (00–83) |
| `.qa/exhaustiva/out_*.jsonl` | Una línea por llamada: `{label, tool, args, wall_ms, response}` (749 entradas) |
| `.qa/exhaustiva/resumen_llamadas.md` | Índice tabular de las 749 llamadas |
| `.qa/exhaustiva/resumen_llamadas.json` | El mismo índice en JSON |
| `.qa/exhaustiva/cobertura.json` | Claves y valores usados por tool/acción, y enums del esquema |
| `.qa/exhaustiva/tools.json` | `tools/list` íntegro |
| `.qa/exhaustiva/capabilities.json` | `resources/list`, `resources/templates/list`, `prompts/list` |
| `.qa/exhaustiva/resources_prompts.json` | `resources/read` de los 5 recursos y `prompts/get` de los 9 prompts |
| `.qa/exhaustiva/inventario.md` | Inventario legible de tools, props y enums |
| `.qa/exhaustiva/PVsyst.ini.before` | Copia previa a `preferencias_editar` (restaurada) |
| `.qa/exhaustiva/malicioso.zip` | Zip con `../evil.txt` usado para probar el rechazo de importación |

---

## 9. Conclusión

El servidor cubre bien el contrato básico: las 15 tools responden, los alias y enums funcionan, las rutas se confinan, las ediciones preservan bytes, los resultados salen de PVsyst y la GUI se puede leer y manejar por OCR/UIA. Los fallos más serios son de **seguridad de acciones** (`licencia_sync`, `editar` con `dry_run`, `lanzar` con `dry_run`, `exportar_logs` fuera del workspace) y de **fiabilidad del E2E real** (`disenar_sistema`, `convertir_meteo`, `meteo_pvgis`, `simular` en workspace no configurado). El esquema también promete menos de lo que exige en varias acciones, y algunas claves declaradas se ignoran sin aparecer en `ignorados`.

Antes de usar el servidor con datos reales, conviene corregir al menos BUG-01 a BUG-06. Los bugs de esquema (BUG-12) y de `ignorados` (BUG-10, BUG-11) hacen que un agente pueda creer que aplicó parámetros que en realidad no se leyeron.
