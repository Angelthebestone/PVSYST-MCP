# Informe E2E — servidor MCP `pvsyst` (cobertura total)

Fecha: 2026-10-25 · Binario: `C:\pvsyst-mcp\pvsyst-mcp.exe` (sha256 `db2fa518…9ab6`, verificado) · Servidor `pvsyst-mcp 0.3.0` · PVsyst 8.1.7 (modo PRUEBA / TRIAL).

## 1. Método y límites

| Vía | Uso |
|---|---|
| Cliente MCP montado (`xd://mcp__pvsyst_*`) | Casos A–D de la primera ronda y un smoke de las 15 tools al final |
| Segunda instancia del mismo exe por stdio JSON-RPC (Python) | Barrido sistemático (~800 llamadas): esquemas crudos, argumentos, casos que el cliente valida antes o no deja enviar (`n:-1`, `name:5`, `contenido:""`), y toda la GUI |

- Workspace de pruebas: `%TEMP%\mcp_e2e_full_20261025_002627` (y el de la primera ronda). Lecturas del workspace real y de DataRO: solo lectura.
- Se mantuvieron las reglas de seguridad: ninguna simulación real exitosa, ninguna pulsación de Guardar/Eliminar/Exportar/OK/Aplicar/Sí/Imprimir/Informe/Ejecutar simulación.
- Verdad de referencia usada para validar cifras: CSV de `MCP_E2E_VC0.csv` (suma `E_Grid` = 11 243,38 kWh; enero = 358,48 kWh). `resultados kpis` da 11 243,0 kWh y `mensual` enero 358,0 → coinciden.

### Efectos secundarios (todos revertidos salvo indicación)
| Efecto | Estado |
|---|---|
| `entorno preferencias_editar` (cambio `Dialog BeepSound` 1→0→1) | Revertido; `PVsyst.ini` restaurado byte a byte desde copia |
| Ese mismo comando escribió historial en el **workspace real** (`UserData\mcp_history\20261025_0527*`) | Dos carpetas borradas (eran mías) |
| PVsyst GUI modificó `PVsyst.ini` (NRun 51→52, vació `Grid/DCGrid/Battery/Pumping Project`, `LastHostID`) al abrir ventanas de proyecto nuevo | `PVsyst.ini` restaurado desde copia (estado original) |
| `.Fav` del workspace real (Battery/Inverter/Pump/PVModule/Regulator/Sites) | Solo cambió mtime (0 bytes antes y después); lo toca PVsyst al abrir las bases de datos |
| Ejecuciones TRIAL consumidas | **170 → 165** (5): 2× `create-site`, 1× `convert-meteo`, 1× `meteo_pvgis`, 1× intento fallido de `run-simulation` desde `disenar_sistema simular:true` en workspace temporal |
| Archivos fuera de workspace creados por mí (`%TEMP%\e2e_*`, `C:\Users\Lenovo\x_nope`) | Borrados |
| PVsyst.exe | Cerrado; `tasklist` sin PVsyst.exe (solo los `pvsyst-mcp.exe`) |

## 2. Primera ronda (casos 0–20 del plan original)

| Caso | Resultado |
|---|---|
| 0 hash + 15 tools | OK |
| 1 alias canónicos (`modules`, `projects`, `file`, `action:create`) | OK |
| 2 sin palabras vetadas en descripciones/`via`/enums (esquema crudo) | OK (ver F-26 para fugas fuera de descripciones) |
| 3 `listar.texto` sin combos; `minimum` en `workspace.n` y `gui_observar.lineas` | OK |
| 4 `latitud:"abc"` | OK: `'latitud' debe ser un número finito (recibido 'abc')` |
| 5 `contenido:""` | **FALLO por cliente montado** (F-11); OK por stdio (`bytes_written:0`) |
| 6 vacío / truncado (.PRJ y .SIT) | OK: `Archivo vacío` / `Archivo truncado: 1 objeto(s) PVObject sin cerrar [pvProject]` |
| 7 sin falsos positivos | OK: 192 archivos (98 VCx, 44 MET, 32 PRJ, 18 SIT) sin error |
| 8 `csv` / sin `csv` / conflicto | OK |
| 9 `n:-1` | OK (cliente: `n: must be >= 1`; servidor: `Valor -1 fuera de rango para 'n' (mínimo 1)`) |
| 10 `name:5` | OK (por stdio) `Falta 'name' (string) en tools/call` |
| 11 hints | OK |
| 12 tipos incorrectos | OK (5 tools; 13 casos por stdio) |
| 13–16, 18–20 GUI | OK (veto de pestaña, guardia modal, compacto 1252 B vs 3357 B, capturas sin márgenes, cierre ordenado) |
| 17 rellenar + `pagina 1` + volver a 0 | `rellenar` y `pagina 1` OK; **volver a 0 vetado por PVsyst** (Rderivación/Rserie en pestaña "Parámetros Modelo") — no completado |

Tiempos: caso 15 (`formulario`) 1 201 ms; caso 18 (`vista`) 37 ms normal / 29 ms compacto.

## 3. Matriz de cobertura (esta ronda)

Leyenda: **OK** probado y correcto · **OK\*** correcto con hallazgo asociado · **PARCIAL** · **NO** no probado (motivo en §6).

### 3.1 Tools de datos
| Tool | Acción / variante | Estado |
|---|---|---|
| `entorno` | diagnostico (resumen/normal/completo) | OK |
| | licencia (`lic-info`), logs_cli (`n`, `solo_errores`) | OK |
| | preferencias (sección / clave / sección inexistente) | OK\* F-20 |
| | preferencias_editar (dry_run, real, restaurado) | OK\* F-10 |
| | licencia_sync (dry y real: `ambiguo`, licencia ausente) | OK |
| | licencia_activar / licencia_desactivar | OK (solo devuelven instrucciones; nunca ejecutan, por diseño) |
| | exportar_logs (dry, real con `permitir_fuera_del_workspace`) | OK |
| `listar` | proyectos, componentes (9 tipos), modulos, archivos, objetos, filtros, fuera de workspace | OK\* F-21 |
| `leer` | PRJ/VCx/BAK/PAN/OND/SIT/MET(serie, offset, limite)/MEF/HOR/SHD/SHO/BLD/DAF/DAM/DAT/TFT/LOD/SFI/RVT/REV/CM0/.Fav/CSV/MD/JSON/BIN/PDF/PNG, base64, offset, max_bytes, plano, objeto, tipo, traversal, fuera de workspace | OK\* F-22 |
| `escribir_archivo` | texto, UTF-8, base64 (+inválido), sobrescribir (historial), alias, nombres raros/ilegales, traversal, fuera de workspace | OK\* F-11 |
| `editar` | campos (objeto / ediciones / null / tipos / multilínea), insertar_objeto, eliminar_objeto, rvt (listar, catalogo, crear, anadir, eliminar, reordenar), plantilla_rvt | OK\* F-03 F-28 |
| `workspace` | crear, exportar, importar (zip-slip rechazado, corrupto, sobrescribir), exportar_proyecto, historial, restaurar | OK\* F-23 |
| `sitio` | crear (dry/real), convertir_meteo (dry/real), meteo_pvgis (real, red disponible), escribir_horizonte, escribir_serie | OK\* F-02 F-07 F-25 |
| | horizonte_pvgis | PARCIAL F-09 (solo error) |
| `componente` | importar, exportar, crear, editar, borrar | OK\* F-03 F-24 |
| | crear_desde_ficha | PARCIAL (validaciones; ejecución real NO) |
| `proyecto` | crear, crear_variante, renombrar, eliminar, doctor (+reparar), dimensionar (n_mod_serie/n_strings), anadir (9 de 10 elementos), seguidor, limite_inyeccion, almacenamiento, perdidas, bombeo, economia, limpiar_temporales | OK\* F-01 F-04 F-05 F-06 F-29 |
| | dimensionar solo `kwp`, p50_p90 | PARCIAL F-09 |
| `simular` | ejecutar (dry_run con todos los parámetros), preparar (filas / parametros), estado | OK\* F-25 |
| | ejecutar real, segundo_plano real | NO |
| `resultados` | kpis, csv, perdidas, mensual, comparar, entregable | OK\* F-08 F-26 |
| | p50_p90, carbono, economia (solo errores) | PARCIAL |
| `disenar_sistema` | red (n_mod_serie/n_strings), aislado (error esperado), tilt/azimut/pnom/plantilla/variante/region, validaciones | OK\* F-09 F-29 |
| `pvsyst_cli` | version, help, lic-info, comando inexistente, opciones array/objeto, dry_run, timeout_s, inyección de shell | OK\* F-27 |

### 3.2 Protocolo MCP
| Elemento | Estado |
|---|---|
| `initialize`, `ping`, `tools/list` (15), `tools/call` con tool o args inválidos, método inexistente (-32601) | OK |
| `resources/list` + `resources/read` de los 5 recursos; URI inexistente (-32602) | OK |
| `prompts/list` + `prompts/get` de los 9 prompts; sin argumentos / inexistente (-32602) | OK\* F-26 |

### 3.3 GUI (PVsyst real)
| Elemento | Estado |
|---|---|
| `gui_observar`: ventanas, controles, leer, capturar (ruta, recorte, ocr, hwnd), formulario (ocr, incluir_uia), vista (compacto, ocr), esquema, arbol, uia (max_nodos), mensajes (`lineas`), funciones (203), menus (10,7 s), `pids`, `proceso` | OK\* F-18 |
| `gui_actuar`: lanzar, activar, esperar, esperar_cierre, mensaje (WM_NULL), click, clic_fisico, clic_texto, escribir, seleccionar, campo, rellenar, pagina, teclas, menu (id y posición), uia_invocar (6 operaciones), paso, funcion, cerrar | OK\* F-12 F-13 F-14 F-15 F-16 F-17 F-30 |
| `gui_actuar reiniciar` | NO (exige diálogo de fallo de PVsyst; error correcto verificado) |
| Formularios recorridos (§4) | 45 aperturas, ver tabla |

## 4. Formularios y pestañas PVsyst recorridos

Cada fila: función abierta con `gui_actuar funcion`, formulario observado, pestañas recorridas con `pagina` y cierre con `Cancelar/Cerrar`. Todas las ventanas se cerraron hasta dejar solo THomeForm.

| Función | Formulario | Campos | Pestañas |
|---|---|---|---|
| herramientas.optimizacion_voltaje | TVoltageOptimiseForm | 20 | – |
| herramientas.comportamiento_electrico | TArrayCharactForm | 19 | 3, todas OK |
| herramientas.factor_transposicion | TTranspositionForm | 17 | – |
| herramientas.calculo_mensual_meteo | TMeteoMensForm | 12 | 1 ("General") |
| herramientas.parametros_solares | TParamSolForm | 11 | 3, todas OK |
| datos_medidos.importar_archivo_personalizado | TMeteoConversionForm | 8 | – |
| datos_medidos.tablas_graficos | TMeteoTablesGraphsForm | 7 | – |
| datos_medidos.transformar_archivo | TDatamesFusionForm | 4 | 3, todas OK |
| datos_medidos.analisis | TProjectForm (New.PRJ) | 6 | – |
| bases_de_datos.precios | TEcoPriceListForm | 2 | – |
| bases_de_datos.importar_archivo_personalizado | TMetWizard | 0 | – |
| bases_de_datos.importar_formato_conocido | TMeteoImportForm | 15 | – |
| bases_de_datos.comparar_meteo | TMeteoCompareForm | 11 | – |
| bases_de_datos.tablas_graficos_meteo | TMeteoTablesGraphsForm | 25 | – |
| bases_de_datos.datos_sinteticos | TGenerSyntheForm | 11 | – |
| bases_de_datos.generacion_tmy | TMessageForm | 0 | – |
| bases_de_datos.{modulos_pv, inversores_red, baterias, controladores_independientes, bombas, controladores_bombeo, generadores, fabricantes_minoristas, sitios_geograficos} | TChoixObjetPVForm ×9 | 3–10 | – |
| …modulos_pv.nuevo | TPVModuleForm | 21 | 5; **vetos en 1–4** (validación de PVsyst) |
| …inversores_red.nuevo | TGridInverterForm | 29 | 6, todas OK |
| …baterias.nuevo | TBatteryForm | 19 | 5, todas OK |
| …controladores_independientes.nuevo | TSAControllerForm | 23 | 4, todas OK |
| …bombas.nuevo | TPumpForm | 20 | 4, todas OK |
| …controladores_bombeo.nuevo | TPumpControllerForm | 15 | 2, todas OK |
| …generadores.nuevo | TGeneratorForm | 9 | 2, todas OK |
| …fabricantes_minoristas.nuevo | TContactForm | 22 | – |
| …sitios_geograficos.nuevo | TGeoSiteForm | 0 | 3, todas OK |
| archivo.espacio_de_trabajo / inicio.administrar_espacio | TWorkspaceForm | 0 | – |
| configuraciones.preferencias | TPreferencesForm | 0 | 9, todas OK |
| configuraciones.parametros_avanzados | TParamEditForm | 4 | – |
| licencia.detalles | TLicenseMainForm | 2 | – |
| ayuda.acerca_de | TAboutForm | 0 | – |
| proyecto.nuevo_{conectado_a_la_red, independiente, bombeo, red_cc} | TProjectForm | 5 | – |
| proyecto.cargar | TProjectLoadForm | 1 | – |
| diseno_preliminar.{conectado_a_la_red, independiente, bombeo} | TDimensForm ×3 | 0 | – |
| variante.resumen_sistema | TSummaryForm | 2 | – |
| inicio.datos_medidos | TDamMainForm | 0 | – |
| inicio.herramientas (vía `click` y `clic_fisico`) | TToolsForm | 0 | – |

Observación: en TPVModuleForm el contenido de las pestañas 2–4 (Datos adicionales, Measured Data, Comercial) **no se pudo ver**: PVsyst veta el cambio mientras el formulario no es válido, y hacerlo válido exige la pestaña "Parámetros del modelo", también vetada (ver F-14).

## 5. Hallazgos

Severidad: **A** pérdida/corrupción de datos · **M** comportamiento incorrecto o engañoso · **B** menor/UX.

### Alta
**F-01 (A) `proyecto eliminar` con `variante` borra el proyecto entero.**
Repro: `proyecto {accion:"eliminar", proyecto:"E2E_P2", variante:"VC2"}` → `recycled` = PRJ + VC0 + VC1 + VC2 (`count:4`). `variante` se ignora. Recuperable desde `UserRecycleBin`.

**F-02 (A) `sitio escribir_serie` acepta variables inexistentes y corrompe el MET.**
Repro: `sitio {accion:"escribir_serie", archivo:"Meteo/e2e.MET", variables:["ZZ_NoExiste"], valores:[[1],…×8760]}` → `variables_actualizadas:["ZZ_NoExiste"]`; `leer serie:true` muestra la columna nueva con unidad `"u"`. Recuperado con `workspace restaurar`.

### Media
**F-03 `editar campos` / `componente editar|crear` aceptan claves con `=`.** `campos:{"A=B":"1"}` → `ok`, pero el archivo queda `A=B=1` y se lee como `A:"B=1"`; `verified_after_write:{"A=B":null}` no se trata como error.

**F-04 `proyecto anadir elemento:envejecimiento` falla siempre.** `Estructura PVObject desbalanceada (2 cierre(s) sin apertura): mutación rechazada` (VC0, VC1, VC2 y `_DEMO_Residential.VC3`). Los otros 9 elementos funcionan.

**F-05 `proyecto dimensionar` ignora `modulo` e `inversor`.** Con `ZZZ_NoExiste.PAN/OND` responde OK y reporta el módulo anterior (`applied` solo toca NModSerie/NStringCh/NStrOrient1/PNomSizing/NInverter). `pnom_modulo` solo cambia el kWp reportado, no el módulo.

**F-06 `proyecto crear` ignora `comentario`.** `applied:[]`; el comentario queda "Residential DEMO" y `doctor` avisa "Comment … parece de demo".

**F-07 `sitio crear` reutiliza sitios por coordenadas e ignora `nombre_sitio`.** Pedir `E2E_Site3` en (7,1) devuelve `site:"E2E_Site2"`, `reutilizado:true` y no crea `E2E_Site3.SIT`; por el cliente montado, (7.07,-73.84) devolvió `E2E_Dis12_Site`. Además: lat 95 / lon -200 dan `Cell 95/-74 not found in Global_World.txt` (sin validar rango) y `region:"marte"` pasa a la CLI (`-r:marte`).

**F-08 `resultados que:comparar` con override `workspace` falla.** `resultados {que:"comparar", proyecto:"Planta_Solar_50kW", variantes:["VC0","VC1"], workspace:<temp>}` → `Ruta fuera del workspace: '<temp>\Projects\Planta_Solar_50kW.VC0'`; con el workspace real funciona.

**F-09 `workspace` ignorado en operaciones que usan la GUI de PVsyst.**
- `sitio horizonte_pvgis` y `proyecto p50_p90` ignoran `workspace` en silencio y leen `C:\Users\Lenovo\PVsyst8.1_Data\Projects\…`; `p50_p90` además **lanza PVsyst** (13,9 s) y lo deja abierto.
- `proyecto dimensionar` solo-`kwp` sí avisa: `El pre-dimensionado lo hace PVsyst en su propio espacio de trabajo: no admite 'workspace'`.
- `disenar_sistema` sin `n_mod_serie/n_strings` abre la GUI en el workspace real (16–20 s) y falla: `PVsyst abrió 'Proyecto:  _DEMO_Tracking.PRJ' en lugar de E2E_Dis3.PRJ`.
Inconsistente: debería rechazar `workspace` como `dimensionar`.

**F-10 `entorno preferencias_editar` escribe historial en el workspace real** (`UserData\mcp_history\…`) aunque la tool no tiene `workspace`.

**F-11 `escribir_archivo contenido:""` por el cliente montado:** `Missing required argument 'contenido' (escribir_archivo)` (3 de 3 intentos). Por stdio: `bytes_written:0` y archivo vacío. [INFERENCIA] el cliente/bridge elimina strings vacíos.

**F-12 `gui_actuar lanzar` no es idempotente.** Con PVsyst abierto lanza una 2.ª instancia (PID 752), tarda 30 s, aparece `TMessageForm "PVsyst is already running."` y devuelve `ok, ready:false`. Hubo que pulsar "No" y verificar que el PID desaparecía.

**F-13 `escribir` / `seleccionar` / `paso escribir` sobre el formulario (no el control) devuelven OK sin efecto.** `escribir ventana:TProjectLoadForm texto:abc` → OK, campo sigue vacío; `seleccionar … texto:ZZZNoExiste` sobre TPVModuleForm → OK, combo sin cambio; `paso … operacion:escribir` → `verificado:false` con OK. Con el hwnd del control funcionan (y `seleccionar` valida: `PVsyst no ofrece 'ZZZNoExiste' en esa lista`).

**F-14 `uia_invocar … operacion:seleccionar` sobre una pestaña vetada da falso positivo.** Devuelve OK y `formulario.pestanas` marca la nueva pestaña como `seleccionada:true` (Comercial, Measured Data, Datos adicionales), pero las capturas muestran el contenido de "Datos básicos". Seleccionar la pestaña ya activa devuelve `ISERROR La operación se completó correctamente. (0x00000000)`.

### Baja
**F-15** `teclas "ZZZ+Q"` → OK `sent:"ZZZ+Q"`; se escribió `q` en Fabricante (modificador inválido ignorado). Además usa `keybd_event` (teclas reales al primer plano).
**F-16** `esperar` ignora `esperar_segundos` (esperó 10 s con 1); `timeout_ms` sí funciona.
**F-17** `clic_texto` (OCR) falla con `'Cerrar' no está dibujado` en TToolsForm/TArrayCharactForm aunque el botón existe; una vez devolvió OK sin cerrar (TGenerSyntheForm).
**F-18** `gui_observar`: `controles` ignora `todos`, `visibles` y `completo:true` (devuelve 13 de 25 salvo `detalle:"completo"`; incluye controles ocultos); igual `funciones` (13/203) y `listar objetos` (13/43); `funciones ventana:` ignora el filtro (`funcion:` prefijo sí); `vista incluir_uia` y `capturar ocr/recorte/imagen:false` sin efecto; `esquema entidad:"zzz"` / `components.inverters` devuelve el esquema de panels sin error (mientras `rellenar` sí rechaza: `usa ["components.panels"]`).
**F-20** `entorno preferencias clave:` ignora `clave` (devuelve la sección completa).
**F-21** `listar proyectos texto:` ignora el filtro (11 resultados con `texto:"zzzzzz"`); `detalle` sin efecto; `listar objetos tipo:pvProject` lo trata como tipo de componente (`Componente 'MCP_E2E' no encontrado`).
**F-22** `leer` de texto no-PVObject (.csv, .md, .json, .txt, .ini, .py, .bat) sin `max_bytes` responde `Binary file; use leer with base64:true`; ruta relativa con backslash (`Projects\X.VC0`) → `Proyecto … no encontrado`; sitio por nombre → `Cannot read metadata … (os error 2)`; en `serie`, `hora` es índice absoluto (`dia:"0501", hora:100`).
**F-23** `workspace exportar`: ignora `sobrescribir:false` con destino existente; un zip dentro del workspace se incluye a sí mismo (`UserData/ws_export.zip` y `*.mcp_tmp_*.zip`); sin `destino` escribe el zip junto al workspace sin `permitir_fuera_del_workspace`. `workspace crear` acepta `..` y creó un directorio fuera (`C:\Users\Lenovo\x_nope`).
**F-24** `componente importar` ignora `destino` y sobrescribe sin aviso; `componente crear` sobrescribe un destino existente (`proyecto crear` sí da `Project exists`).
**F-25** `simular ejecutar` (dry_run) no valida `time_step:"minute"`, fechas (`31/02/2024`, fin < inicio), `paginas_reporte:["zzz"]`; `log_level:9` → `-ll:3` en silencio; rutas relativas de csv/pdf sin resolver contra el workspace; con variante inexistente no valida e invoca la CLI (3,9 s) devolviendo OK con `clasificacion:"error"`. `convertir_meteo` dry_run no valida entradas ni `input_mef_timeshift` fuera de [-30,30].
**F-26** Fugas de funciones gated por `PVSYST_MCP_APROXIMACIONES`: `resultados que:zzz` lista `medido_vs_simulado` entre valores válidos; el esquema expone `medido_csv/medido_kwh/umbral_pct`; el prompt `disenar_desde_coordenadas` y `instructions` mencionan la "aproximación". Además: `resultados kpis include_text` sin efecto; `resultados {}` sin args devuelve un proyecto arbitrario; `kpis proyecto:NoExiste` → `Informe PDF … no encontrado` (engañoso).
**F-27** `pvsyst_cli help run-simulation` se clasifica `error` (`detected_error:"Required option"`) aunque la salida es correcta.
**F-28** `editar insertar_objeto` ignora `campos` cuando se usa `desde`; `editar rvt` con `variables` y sin `operacion` ejecuta `listar` en silencio (`total:0`, incluso en archivo inexistente).
**F-29** `proyecto bombeo` tras `anadir circuito_hidraulico`: `Sección 'WaterUse' no encontrada` (solo funciona sobre plantillas de bombeo); `disenar_sistema` que falla a mitad deja proyecto parcial (`E2E_Dis.PRJ/VC0` tras `Módulo 'Generic_Mono_300W.PAN' no encontrado`).
**F-30** `pagina indice:9` → `PVsyst no cambió a la pestaña 9: … vetó el cambio` (debería ser índice fuera de rango); `indice:-1` → `Missing required argument 'indice'`.

## 6. Lo que no se pudo probar y por qué

| Elemento | Motivo |
|---|---|
| `simular ejecutar` real, `segundo_plano` real, `estado` con trabajo real, `abrir_gui`, `confirmar` | Regla: no gastar ejecuciones TRIAL; además la CLI solo acepta el workspace configurado en PVsyst (en el temporal: `This directory does not contain a workspace for PVsyst`) |
| `disenar_sistema simular:true` exitoso y `kpis` | Idem |
| `resultados p50_p90`, `carbono`, `economia` (camino exitoso) | Ninguno de los 10 PDF de `Results` (copiados al temporal) contiene esas páginas y generarlos exige simular |
| `proyecto p50_p90`, `dimensionar` solo-`kwp`, `sitio horizonte_pvgis`, `componente crear_desde_ficha` (ejecución) | Operan sobre el workspace real vía GUI de PVsyst y guardan; prohibido escribir ahí |
| `proyecto economia` desde cero | Exige crear la evaluación en la GUI (OK/Guardar); probado el ajuste sobre `_DEMO_Residential.VC6` y el error en variantes sin evaluación |
| GUI `variante.*` (Orientación, Sistema, Pérdidas, Autoconsumo, Almacenamiento, Horizonte, Sombreados, Energía, Economía, Simulación avanzada, Resultados) y sus sub-ventanas, escena 3D (`escena_3d.*`), `ventana_proyecto.*` | Requieren un proyecto cargado; cargarlo exige OK en TProjectLoadForm. En un proyecto nuevo los botones están deshabilitados: `Botón 'Orientación' deshabilitado` (5 s de espera) |
| Pestañas 2–4 de TPVModuleForm (contenido) | Veto de validación de PVsyst (F-14) |
| Funciones GUI con efecto `modifica` (idioma, importar/exportar, actualizar bases, borrar listas recientes, ejecutar simulación, informe) y `inicio.cambiar_espacio`, `inicio.conectado_a_la_red/independiente/bombeo`, ayuda web (`sitio_web`, `foro`, tutoriales, `buscar_actualizaciones`) | Modifican estado/configuración, abren navegador o están en la lista prohibida |
| `gui_actuar reiniciar` | Solo actúa con diálogo madExcept/inestabilidad; error correcto verificado |
| `licencia_activar/desactivar` real | La tool nunca ejecuta (devuelve instrucciones); `licencia_sync` real devolvió `ambiguo` (sin archivo de licencia) |
| Concurrencia, cancelación, límites de tamaño, idiomas distintos de español | Fuera del alcance de esta ronda |

## 7. Resumen

- ~800 llamadas por stdio + 31 por cliente montado; 15/15 tools, 5/5 recursos, 9/9 prompts ejercitados.
- Funcionalidad base sólida: lectura/escritura de archivos, historial/restauración, validaciones de rango, zip-slip rechazado, sin inyección de shell, cifras de resultados coinciden con el CSV de referencia, recorrido de 45 formularios de PVsyst con cierre limpio.
- Prioridad de corrección: F-01 (borrado de proyecto completo), F-02 (corrupción de MET), F-03 (claves con `=`), F-09/F-10 (operaciones que ignoran `workspace` y escriben en el real), F-14 (falso positivo de pestaña), F-11 (cliente/bridge y strings vacíos).

## 8. Fallos en las instrucciones del MCP (instructions, recursos, prompts, esquemas)

Evidencia: textos leídos del `initialize`, `pvsyst://recetas`, los 9 `prompts/get`, los esquemas de `tools/list` y el comportamiento observado en las secciones 2–5.

| ID | Dónde | Fallo | Evidencia |
|---|---|---|---|
| I-01 | `instructions` (initialize) | Omite lo que más condiciona el uso: (a) que `workspace` es un override por llamada y que varias acciones lo ignoran (F-09); (b) que `create-site`, `convert-meteo` y `meteo_pvgis` gastan ejecuciones TRIAL (1 c/u; solo la caché por coordenadas es gratis); (c) que las listas se truncan a 13 elementos salvo `detalle:"completo"`; (d) que la GUI es instancia única y `funcion` resuelve sola los prerrequisitos. En cambio gasta texto en la etiqueta interna `origen:aproximacion_mcp` y en una variable de entorno que el LLM no puede cambiar. | 5 ejecuciones gastadas sin aviso previo; `listar objetos` 13/43, `controles` 13/25, `funciones` 13/203 |
| I-02 | `pvsyst://recetas` ("consejo") | "Si dudas, llama a `entorno` y luego a `disenar_sistema`": en un workspace sin `Generic_Mono_300W.PAN`/`Generic_7.5kW.OND` como archivo falla con `Error al leer el módulo 'Generic_Mono_300W.PAN': … no encontrado` y deja un proyecto parcial; sin `n_mod_serie/n_strings` recurre a la GUI y falló 3 de 3 (`PVsyst abrió 'Proyecto:  _DEMO_Tracking.PRJ' en lugar de …`). El ejemplo de la receta no incluye `modulo`/`inversor`. En el workspace real `ComposPV` solo tiene `MCP_Test_400.PAN`, por lo que [INFERENCIA] el camino recomendado falla también allí. | `dis.red`, `dis.autodim`, `listar modulos` real |
| I-03 | prompt `disenar_desde_coordenadas` | Dice que el dimensionado automático "solo existe con `PVSYST_MCP_APROXIMACIONES=1`" y manda "pedir o proponer" strings: obliga a una ida y vuelta con el usuario, y contradice a `disenar_sistema`, que sin strings sí intenta un pre-dimensionado (vía GUI). | texto del prompt vs `dis.autodim` |
| I-04 | `disenar_sistema` (contrato) | Devuelve `ok:true` con un proyecto que **conserva sitio, meteo y comentario de la plantilla** (Geneva 46.2383/6.1094, "Residential DEMO") aunque creó el `.SIT` de las coordenadas pedidas; `leer`/`listar` posteriores y un `simular` posterior sin `site` usan Geneva. Tampoco falla con `kwp_realizado 73,5` frente a 10 pedidos (+635 %), solo avisa. [INFERENCIA] con `simular:true` aplica el sitio por `-s`; el paso `simulacion` no lo muestra. | `c.leer.prj` |
| I-05 | receta/prompt `estudio_paso_a_paso` | Enumera `sitio crear → proyecto crear → editar campos → …` sin los argumentos obligatorios (`plantilla`, `nombre_nuevo`), sin decir que la plantilla debe existir **dentro** del workspace (`Proyecto '_DEMO_Commercial' no encontrado; no hay ninguno en <WS>\Projects`) y sin ningún paso que vincule el sitio al proyecto. Añade "Confírmame cada fase antes de simular": 6 turnos secuenciales. | `proy.crear.plantilla.mala`, `c.crear.plantilla.RO` |
| I-06 | prompt `convertir_a_seguidor` | Pide convertir `VC0` y "comparar con la variante fija", pero `proyecto seguidor` modifica la variante en sitio (`NoOrient 2→1`, `Pitch`, `NSheds`…): la variante fija desaparece salvo que antes se llame a `crear_variante`, paso que el prompt no menciona. | `proy.seg` (`cambios`) |
| I-07 | prompts `leer_resultados`, `explorar_archivo`, `operar_gui` | `leer_resultados` encadena `kpis`, `csv` y `perdidas` (3 llamadas; `perdidas` ya trae energía y kWp, `csv` solo metadatos). `explorar_archivo` encadena `leer` + `listar objetos` (truncado a 13) + `editar`. `operar_gui` es una frase genérica: no menciona descartar `TDEMOForm`, usar `funcion` en vez de clics, que `escribir`/`seleccionar` exigen el hwnd del control, ni cerrar las ventanas al final. | textos de los prompts |
| I-08 | `prompts/optimizar_orientacion` | "dime cuántas ejecuciones gastará, lánzalas con `simular ejecutar` y `filas`": no hay forma barata de saber las restantes (`entorno licencia` tarda 4 s) y `simular` real no funciona en workspaces distintos del configurado en PVsyst; ese aviso (`Data-only workspace: PVsystCLI run-simulation only accepts the workspace configured in PVsyst itself`) aparece solo en la respuesta de `workspace crear`. | `ws.crear`, `sim.real.variante.noexiste` |
| I-09 | Esquemas de `tools/list` | Descripción de una línea por tool y parámetros solo con "(alias: x)". Esquemas planos: `proyecto` 50 parámetros/15 acciones, `simular` 48, `sitio` 37, `gui_actuar` 34/20 acciones; nada indica qué parámetros aplican a cada acción ni cuáles son obligatorios (`required: []` en `listar`, `leer`, `componente`). El LLM descubre los requisitos por error. | §8.1, tabla de descubrimiento |
| I-10 | Esquema ↔ comportamiento | Parámetros anunciados que no hacen nada: `esperar_segundos`, `etiqueta` en `escribir`/`seleccionar`, `comentario` en `proyecto crear`, `modulo`/`inversor` en `dimensionar`, `variante` en `eliminar`, `destino` en `componente importar`, `texto` en `listar proyectos`, `todos`/`visibles`/`completo` en `controles`, `incluir_uia` en `vista`, `ocr`/`recorte` en `capturar`, `ventana` en `funciones`, `entidad` en `esquema`, `include_text` en `kpis`, `sobrescribir:false` en `workspace exportar`, `campos` con `desde` en `insertar_objeto`, `variables` sin `operacion` en `editar rvt`. Un parámetro ignorado que devuelve OK es peor que uno inexistente (F-01, F-05, F-06, F-13). | F-01…F-30 |
| I-11 | Campo `siguiente` | Sugerencias que fallan: `resultados kpis` → "`resultados que:comparar`" con una sola variante (`Se necesitan al menos 2 variantes con resultados`); `proyecto crear` → "`proyecto dimensionar {proyecto,kwp}`" cuyo modo solo-`kwp` rechaza `workspace`; `resultados {}` sin argumentos devuelve el KPI de un proyecto arbitrario. | `res.comparar`, `proy.dim.kwp` |
| I-12 | Nombres de argumentos | Inconsistentes dentro de un mismo tool: `proyecto crear` usa `nombre_nuevo` (no `proyecto`); `editar campos` pide `archivo`, pero `ediciones` pide `proyecto`+`variante`; `leer` acepta `E2E_P3` o ruta absoluta, pero rechaza `Projects/E2E_P3.PRJ` y `Projects\X.VC0`; `workspace importar` exige `destino` sin decirlo; `listar componentes` exige `tipo` con códigos (`PAN`, `OND`…) que el LLM tiende a escribir como `inverters`/`PVModules`. | primeras llamadas de cada acción |

### 8.1 Costes de descubrimiento medidos (llamadas hasta acertar)
Cada error de "falta argumento" informa de **un** solo argumento:

| Acción | Llamadas fallidas antes del éxito | Argumentos descubiertos uno a uno |
|---|---|---|
| `componente crear` | 3 | `nombre_nuevo` → `plantilla` → `cambios` |
| `componente crear_desde_ficha` | 4 | `tipo` → `nombre_nuevo` → `datos` → lista de 9 campos (`Model`, `Manufacturer`, `PNom`, `Technol`, `Isc`, `Voc`, `Imp`, `Vmp`, `NCelS`) |
| `proyecto crear` | 2 | `plantilla` → `nombre_nuevo` |
| `workspace importar` | 2 | `zip` → `destino` |
| `sitio escribir_serie` | 3 | `archivo` → `variables` → `valores` |
| `editar insertar_objeto` | 3 | `archivo` → `padre` → `desde` |
| `editar rvt` | 1 + trampa | `batch_rvt_file`/`archivo`; sin `operacion` ejecuta `listar` en silencio (los valores válidos solo aparecen al fallar con una operación inválida) |
| `gui_actuar uia_invocar` | 3 | `ventana` → `operacion` → `nombre`/`rol` |

## 9. Cómo reducir pasos y llamadas (propuestas priorizadas)

### 9.1 Contratos de error y esquema (coste casi nulo, ahorro alto)
1. **Devolver todos los argumentos que faltan de una vez**, con un ejemplo mínimo: `Faltan: nombre_nuevo, plantilla, cambios. Ejemplo: {...}`. Ahorra 1–3 llamadas por acción nueva (§8.1).
2. **Esquema discriminado por acción** (`oneOf` por `accion`/`que`) o, como mínimo, listar en la descripción de `accion` los argumentos obligatorios de cada valor. Elimina el prueba-y-error en `proyecto`, `simular`, `sitio`, `gui_actuar`.
3. **Rechazar (o avisar en un campo `ignorados:[…]`) los parámetros que no aplican** en vez de devolver OK (I-10). Evita reintentos por falsos éxitos.
4. **Un único nombre por concepto**: aceptar `proyecto` como alias de `nombre_nuevo` en `crear`; aceptar `proyecto`+`variante` en `editar` para todas las acciones; normalizar `Projects\…`, `Projects/…` y `.PRJ` en `leer`.
5. **Candidatos en todos los "no encontrado"** (hoy solo en proyectos/componentes): `leer` de sitio por nombre, `horizonte_pvgis`, `p50_p90` devuelven `os error 2`.
6. **Truncado explícito**: cuando una lista se corta, incluir `truncado:true, total:N, hint:"detalle:completo"` (hoy solo algunas respuestas traen `*_total`) o paginar con `limite/offset`.

### 9.2 Menos llamadas en el flujo principal
| Flujo | Llamadas hoy | Propuesta | Llamadas |
|---|---|---|---|
| Workspace nuevo con plantilla DEMO | `workspace crear` + (`exportar_proyecto` + `importar`) por plantilla + `componente importar` ×2 | `proyecto crear` y `componente crear` resuelven `plantilla` también en `DataRO` (ya lo hacen `listar modulos` y `leer`); `workspace crear` con `sembrar:true` | 1 |
| Diseñar y obtener KPIs | `sitio crear` → `proyecto crear` → `dimensionar` → (`editar`) → `simular` → `resultados kpis` | Hacer de `disenar_sistema` el camino canónico: módulo/inversor por defecto tomados de DataRO o del primer componente disponible, vincular el sitio al `.PRJ`, `simular:true` devuelve KPIs (ya lo prevé) | 1–2 |
| Ajustar una variante (pérdidas + límite + almacenamiento + seguidor) | 4 llamadas, 4 snapshots de historial sobre el mismo `.VCx` | `proyecto configurar` con todos los parámetros opcionales y una sola escritura atómica | 1 |
| Leer resultados | `kpis` + `mensual` + `perdidas` (+ `csv`) | `resultados que:resumen` con kpis, mensual, 3 mayores pérdidas y avisos | 1 |
| Crear sitio + proyecto | `sitio crear` + `proyecto crear` + `editar` para fijar sitio/meteo | `proyecto crear` acepta `sitio` (nombre o coordenadas), `comentario`, `kwp`, `n_mod_serie`, `n_strings`, `modulo`, `inversor`, `tilt`, `azimut` | 1 |
| Variante nueva + cambio | `crear_variante` + `seguidor`/`perdidas`/… | `crear_variante` acepta los mismos parámetros de configuración | 1 |
| Re-simular tras editar | `editar` + `simular` + `resultados` | `simular` con `devolver:"kpis"` y `cambios:{…}` opcionales | 1 |
| Preflight de sesión | `entorno diagnostico` + `entorno licencia` (4 s) | Incluir en la primera respuesta de cualquier tool (o en `diagnostico`) `gui_running`, ejecuciones restantes y tipo de workspace; el dato de ejecuciones ya existe en `costo.restantes` | 0 extra |

### 9.3 GUI: de ~9 llamadas a 2–3
Secuencia observada para "abrir formulario, rellenar, comprobar, cerrar": `lanzar` → `cerrar TDEMOForm` → `funcion` → `formulario` → `rellenar` → `vista` → `clic_texto Cancelar` → `cerrar` selector → `cerrar` TDatabaseForm (≈9).
1. `lanzar` con `cerrar_demo:true` (descartar `TDEMOForm` automáticamente) y que sea **idempotente** (F-12).
2. `funcion` devuelve también el resumen del formulario (`botones`, `pestañas`, claves de campos); hoy solo `ventanas_nuevas`.
3. `rellenar` devuelve la `vista` compacta posterior (11 campos tardaron 9,7 s y hace falta otra llamada para ver qué quedó inválido).
4. `cerrar` con `cancelar:true` y `cerrar_todo`/`restaurar_inicio` que cierra la cadena en orden (selector → bases de datos → home). Yo tuve que escribir ese bucle; tras `Cancelar` reaparece el selector oculto, lo que rompe cualquier script ingenuo.
5. Acción `plan`/`secuencia` que acepte un array de pasos (`funcion`, `rellenar`, `pagina`, `observar`, `cerrar`) y se detenga en el primer fallo: 1 llamada en lugar de 6–9.
6. `clic_texto`: probar primero el control por título (11 ms, fiable) y usar OCR solo como respaldo (0,6 s y falló con `'Cerrar' no está dibujado` aunque el botón existe, F-17). `escribir`/`seleccionar` deberían aceptar `etiqueta` (el esquema ya la expone) para no necesitar `formulario` + hwnd; hoy `campo`/`rellenar` son los únicos caminos de una llamada y la documentación no lo dice.
7. `gui_observar formulario` tarda 0,6–1,2 s y devuelve campos + textos + botones; `vista compacto` tarda 29 ms. Hacer `compacto` el valor por defecto y `formulario` solo bajo `detalle:"completo"`.
8. Mensajes de veto: incluir en el error de `pagina` el motivo que hoy hay que pedir con `vista ocr:true` (p. ej. `La corriente MPP es demasiado alta con respecto al ISC`), que ahorra una llamada de 0,6 s y una captura.

### 9.4 Menos tokens por llamada
- `detalle` no cambia el tamaño en `listar`, `leer`, `simular` y `editar` (2 462 / 2 463 / 2 462 caracteres para `resumen/normal/completo`): hacer que `resumen` devuelva de verdad menos.
- Campos duplicados en la misma respuesta: `pr` y `pr_pct`; `produccion_especifica_kwh_kwp` y `specific_yield_kwh_per_kwp`; `cli_args` y `command_line`; rutas absolutas repetidas en `workspace`, `project_file`, `variant_file`, `pdf`, `csv`. Devolver rutas relativas al workspace y un solo nombre por magnitud.
- `leer` de texto plano debería devolver el texto (hoy "Binary file; use leer with base64:true" para `.csv/.md/.json/.txt`), que evita una segunda llamada con `max_bytes`.

### 9.5 Seguridad operativa que también ahorra reintentos
- `dry_run` con `costo_estimado:{ejecuciones:N}` y una guarda `max_ejecuciones` (env o parámetro): en esta ronda se gastaron 5 ejecuciones sin poder anticiparlo.
- `disenar_sistema` y `workspace` override: fallar al inicio, con un solo mensaje, si la operación requiere la GUI/workspace de PVsyst (F-09), en vez de abrir PVsyst durante 16–20 s para fallar.
- Operaciones multi-paso (`disenar_sistema`) transaccionales: si falla un paso, no dejar `E2E_Dis.PRJ/VC0` parciales (F-29).

### 9.6 Orden recomendado de implementación
| Prioridad | Cambio | Esfuerzo | Ahorro |
|---|---|---|---|
| 1 | Errores con todos los argumentos faltantes + ignorados explícitos (9.1 · 1, 3) | Bajo | 1–3 llamadas por acción nueva y menos falsos éxitos |
| 2 | `plantilla` resuelta en DataRO + `disenar_sistema` con componentes por defecto y sitio vinculado (I-02, I-04) | Medio | flujo principal de 6–10 llamadas a 1–2 |
| 3 | `resultados que:resumen` y `proyecto configurar` | Medio | 3–4 llamadas por consulta/ajuste |
| 4 | GUI: `cerrar_todo`, `lanzar` idempotente con `cerrar_demo`, `funcion` con resumen, `secuencia` | Medio | ~9 → 2–3 llamadas por interacción |
| 5 | Reescribir `instructions`, receta "consejo" y los prompts (I-01…I-08) con los pasos y argumentos reales | Bajo | evita los primeros 5–10 intentos fallidos de cada sesión |
