# Informe de pruebas de la GUI del MCP `pvsyst` (PVsyst 8.1.8)

> **Fecha de la sesión:** 2026-11-08, 03:49–05:05 (hora local; el nombre del
> archivo conserva la fecha pedida en el encargo).
>
> **Alcance:** QA de caja negra de **todas** las rutas de `gui_observar` (12
> acciones) y `gui_actuar` (26 acciones): ruta feliz y ruta de error/borde.
> No se modificó `src/`, no se hizo ningún commit y el PR #2 no se tocó.
>
> **Binario probado:** `C:\pvsyst-mcp\pvsyst-mcp.exe`
> `sha256 = 501ec8ffabc020fb720bb58b16f9f3ff362dd97fb6a8268b2977e68bffa8abc0`
> (5 015 552 bytes, 2026-11-07 21:31). Código correspondiente: worktree
> `.claude/worktrees/tarea-d-fila`, rama `worktree-tarea-d-fila`, HEAD `5dbfdd0`.
>
> **Verificación de binario nuevo:** la descripción de `disenar_sistema` que
> devuelve `tools/list` contiene `proyecto_comparable` y la frase «A project an
> earlier call left behind with the same parameters is reused instead of
> rebuilt». ✔ Es el binario del PR #2.
>
> **Escritorios usados:** fase 1 en **aislado**
> (`PVSYST_MCP_ESCRITORIO=aislado`, escritorio privado `pvsyst_mcp_agente`);
> fase 2 en **compartido** para las rutas que el aislado rechaza por diseño
> (`fila`, `menu`, `menus`) y para todo lo que dependía de un proyecto cargado
> con `fila` (`vista`/`rellenar` de orientación y sistema, `escena*`, `teclas`
> en compartido). Al final se cerró PVsyst y no quedó ninguna ventana ni
> artefacto `QA_GUI_*`.

---

## 1. Resumen ejecutivo

| Métrica | Valor |
|---|---|
| Acciones de `gui_observar` ejercitadas | **12 / 12** |
| Acciones de `gui_actuar` ejercitadas | **26 / 26** |
| Rutas (acción × feliz/error) registradas en la matriz | **189** (93 felices, 96 de error/borde; incluye variantes H1b) |
| Rutas felices | **93** |
| Rutas de error/borde | **96** |
| Llamadas registradas por el arnés stdio | **459** (183 `gui_observar`, 264 `gui_actuar`, 12 auxiliares) |
| Respuestas con `isError` | **151** (casi todas errores/validaciones deliberadas) |
| Resultado `ok` (ruta feliz o validación correcta) | **~300** |
| Bugs confirmados | **20** (2 × P1, 7 × P2, 11 × P3) — 0 × P0 |
| Bugs del informe previo re-verificados | BUG-03, BUG-16, BUG-17, BUG-23, BUG-24 (5); BUG-18 **no reproducido** |
| Rutas bloqueadas | **3** (`escena`, `escena_leer`, `escena_importar` felices, por GUI-03/GUI-04) + `reiniciar` con fallo real (no reproducible sin dañar nada) |
| Simulaciones / TRIAL | **0** (no se lanzó ninguna simulación ni se usó `pvsyst_cli`) |
| Proyecto desechable | `QA_GUI_20261108_0351` (creado 03:51, borrado al final) |
| Duración | ≈ 4 h 15 min (03:49–05:05), incluidas preparación, dos escritorios y limpieza |

**Lo que funciona bien.** Las 12 rutas de `gui_observar` responden con datos
reales y las 26 de `gui_actuar` tienen efecto observable en la mayoría de los
casos. Destacan: `fila` (selección de fila 0 en `TProjectLoadForm` y en
`TChoixObjetPVForm`, verificada por la banda resaltada), `menu` (menú y
submenú reales, con `id` 23 y 40), `uia_invocar` (`invocar`, `fijar_valor`,
`expandir`, `contraer`, `seleccionar` y `alternar`), `rellenar` sobre
`proyecto.ventana`, `variante.orientation`, `variante.system` y
`components.panels`, `pagina` sobre `TPreferencesForm` (pestañas 1, 2 y 3),
`escribir`/`seleccionar` con coordenadas de cliente, `capturar` con PNG
verificado (bytes, IHDR, base64), y `mensajes` con un aviso real de PVsyst.

**Lo que no funciona.**

1. **Coordenadas de `vista`/`esquema` y `campo`/`seleccionar` no coinciden**
   (P1): las claves que publica `vista` son relativas a la ventana, pero
   `escribir`/`seleccionar` las interpretan como relativas al cliente. Escribir
   en el ancla documentada de `modelo` (121, 75) cambió **`nombre_archivo`**, no
   `modelo`; con (121, 52) sí cambió `modelo`. Es una escritura silenciosa en el
   campo equivocado.
2. **`uia_invocar invocar` puede colgar el servidor** (P1): invocar con UIA el
   botón «Sitio y datos meteo» de `TDimensForm` abrió `TDimensSituationForm`
   pero la llamada no volvió en >10 min; el proceso MCP quedó bloqueado y hubo
   que matarlo. `Invoke()` no tiene timeout (`src/adapters/uia/com.rs:212`).
3. **`escena`/`escena_leer`/`escena_importar` no pueden completar** (P2): el
   diálogo «Exportar escena (.SHD)» se cierra sin escribir el archivo y la tool
   falla con `PVsyst did not save …\_mcp_<escena>.SHD: no other way is
   attempted` tras 22–41 s. La creación manual de un objeto sí funciona (la
   lista de objetos muestra `QA_Caja`), y `escena_importar` sí importa la malla
   (`Objeto 57` aparece en la lista), pero la exportación no escribe nada.
4. **`click` ignora `texto` e `id`** (P2): `{accion:"click", ventana:"TDimensForm",
   texto:"Sitio y datos meteo"}` devuelve `invoked:"TDimensForm"` y no pulsa el
   botón. El botón solo se pulsa pasando su título como `ventana` o su `hwnd`.
5. **`lanzar proyecto:…` no abre el proyecto** (P2): el proceso arranca con la
   ruta en la línea de comandos, pero tras cerrar `TDEMOForm` solo queda
   `THomeForm`; no aparece `TProjectForm`.
6. **El contrato `vista` sigue mintiendo** (P2, BUG-03): `esperar_cierre`
   devuelve `cerrado:false` con `vista.estado:"ok"` y `clic_texto` con
   `coincidencias:0` también.

---

## 2. Matriz completa acción × ruta

Convenciones: **H** = ruta feliz, **E** = ruta de error/borde. La evidencia
está recortada; el JSON completo de cada llamada quedó en el arnés
(`%TEMP%\qa_gui_2026\results\*.json` y `calls.jsonl`). «Verificación» es cómo se
comprobó el efecto, no lo que la tool dijo.

### 2.1 `gui_observar` (12 acciones, 57 rutas)

#### `ventanas` (3 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"ventanas"}` | ok, 20 ms | `count:4`, `THomeForm` («PVsyst 8.1.8 - DEMO»), `TApplication`, 2 ventanas UAC | Enumeración real del proceso 29668. |
| H2 | `{que:"ventanas", completo:true}` | ok | Mismo `count:4` con `rect`, `pid`, `enabled` | Campos completos. |
| H3 | `{que:"ventanas", visibles:false, todos:true}` | ok | `count:40`, `windows_total:40`, `truncado:true`, 13 elementos | El recorte genérico avisa y da el total. |
| E1 | `{que:"ventanas"}` con PVsyst cerrado | error | `Process 'PVsyst.exe' is not running. Launch the PVsyst GUI first (or pass 'proceso').` | Mensaje accionable. |
| E2 | `{que:"ventanas", proceso:"NoExiste.exe"}` | ok, 0 | `count:0`, `processes:[]` | No es error: el proceso se nombró a propósito. |
| E3 | `{que:"noexiste"}` | error | `Value 'noexiste' not valid for gui_observar (que); values: …` | Lista de valores válidos. |

#### `controles` (4 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"controles", ventana:"THomeForm"}` | ok, 25 ms | `count:24`, `controls_total:24`, `truncado:true`, **13** controles | **BUG-16**: `count` no es la lista devuelta. |
| H2 | `{…, detalle:"completo"}` | ok | `count:24`, **24** controles | Lista completa. |
| H3 | `{…, detalle:"completo", incluir_uia:true}` | ok, 463 ms | 24 controles + `uia:[6]` | Fusión Win32/UIA. |
| H4 | `{que:"controles"}` (sin `ventana`) | ok | `count:25`, 13 mostrados, `controls_total:25` | Enumera los hijos de todas las ventanas del proceso. |
| E1 | `{que:"controles", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E2 | `{que:"controles", ventana:"0xDEAD"}` | error | `Window '0xDEAD' not found` | — |

#### `leer` (2 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"leer", ventana:"THomeForm"}` | ok, 20 ms | `{class:"THomeForm", text:"PVsyst 8.1.8 - DEMO"}` | Texto real. |
| H2 | `{que:"leer", ventana:"0x2207D6"}` (TEdit) | ok | `text:"CommentEdit"` | **GUI-10**: devuelve el nombre del control, no el valor; `vista` del mismo formulario muestra `QA_GUI_Sitio`. |
| E1 | `{que:"leer"}` | error | `Missing required argument 'ventana' (hwnd, class or title)` | — |
| E2 | `{que:"leer", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |

#### `capturar` (5 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"capturar", ventana:"THomeForm"}` | ok, 207 ms | `path:…\Temp\pvsyst_gui_THomeForm.png`, `bytes:88434`, `width:801`, `height:701` | PNG real: magic `89 50 4E 47`, IHDR 801×701, 88 434 B en disco. |
| H2 | `{…, imagen:true}` | ok | Segundo ítem `image/png`; base64 decodifica a **88 434 B**, idéntico al archivo | `content[1].data` verificado. |
| H3 | `{…, ventana:"0x4008E8", recorte:true}` (botón) | ok, 427 ms | `bytes:1914`, `height:62` | Recorte real del botón, no de la ventana. |
| H4 | `{…, ventana:"THomeForm", recorte:true}` | ok | `bytes:88434`, 801×701 | La raíz es la propia ventana: el recorte no cambia nada. |
| H5 | `{…, ocr:true}` | ok, 1,1 s | `textos:["e PVsyst8.1.8 -DEMO","Archivo","Diseño preliminar",…]` | OCR sobre la captura. |
| E1 | `{…, ruta:"C:/Windows/Temp/QA_GUI_fuera.png"}` | error | `Ruta fuera del workspace: 'C:/Windows/Temp/QA_GUI_fuera.png'` | Confinamiento correcto. |
| E2 | `{que:"capturar", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E3 | `{que:"capturar", ventana:"0x3F078A"}` (TPanel oculto) | error | `0x3F078A (TPanel) está oculto: no hay nada que capturar` | — |

#### `uia` (1 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"uia", ventana:"THomeForm", profundidad:3, max_nodos:60}` | ok, 224 ms | `arbol` con 31 nodos, `truncado:false`, `nodos_con_error:0` | `profundidad` y `max_nodos` respetados. |
| H1b | `{…, ventana:"TDimensForm", profundidad:6, max_nodos:200}` | ok | 5 botones con `patrones:["invoke"]` y `rect` | Árbol UIA real. |
| E1 | `{que:"uia", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E2 | `{que:"uia"}` | error | `Missing required argument 'ventana' (hwnd, class or title)` | — |

#### `arbol` (1 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"arbol", ventana:"THomeForm"}` | ok, 195 ms | `ventanas:[25]`, `uia_sin_hwnd:[6]` | Fusión Win32 + UIA. |
| E1 | `{que:"arbol", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E2 | `{que:"arbol"}` | error | `Missing required argument 'ventana' (hwnd, class or title)` | — |

#### `formulario` (2 H / 1 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"formulario", ventana:"TDimensForm"}` | ok, 597 ms | `botones:[{texto:"Horizonte",habilitado:false},{texto:"Sistema"},{texto:"Resultados"},{texto:"Sitio y datos meteo",habilitado:true},…]` | OCR + Win32; etiquetas y `rect`. |
| H2 | `{…, ventana:"TPVModuleForm", detalle:"completo"}` | ok | `pestanas:[Datos básicos,Dimensiones y tecnología,Datos adicionales,Measured Data,Comercial]`; `campos` con etiquetas «Modelo», «Fabricante», «Potencia nom.», «Isc», «Voc»… | Pestañas y campos reales. |
| E1 | `{que:"formulario", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |

#### `vista` (7 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"vista", ventana:"TDimensForm"}` | ok, 43 ms | `entidad:"ventana.TDimensForm"`, `botones:[Horizonte,Sistema,Resultados,Sitio y datos meteo]`, `abierta_con:[diseno_preliminar…]` | Ventana registrada sin esquema. |
| H2 | `{que:"vista", entidad:"proyecto.ventana", completo:true}` | ok | `descripcion:"Nuevo Proyecto"`, `archivo_meteo:""`, `variante_activa:"VC0 : Nueva variante de simulación"`; `sin_mapear` con los radio buttons | Campos editables y valores reales. |
| H3 | `{que:"vista", entidad:"variante.orientation", completo:true}` | ok | `tipo_de_campo:"Plano inclinado fijo"`, `inclinacion_plano_deg:"15.0"`, `azimut_deg:"20.0"`, `pendiente_base_deg:"0.0"`; 5 campos | Proyecto `_DEMO_Commercial` cargado. |
| H4 | `{que:"vista", entidad:"variante.system", completo:true}` | ok | `n_inversores:"6"`, `n_modulos_serie:"24"`, `n_cadenas:"48"`, `potencia_planeada_kwp:"400.0"`; 7 campos | — |
| H5 | `{que:"vista", entidad:"components.panels", completo:true}` | ok | 16 campos, `tecnologia` con 15 opciones, `nombre_archivo:"New.PAN"` | — |
| H6 | `{que:"vista", ventana:"THomeForm"}` | ok, 40 ms | `entidad:null`, `aviso:"ninguna ventana conocida abierta; esquemas con gui_observar que:esquema"` | **BUG-17**: THomeForm está abierta. |
| H7 | `{que:"vista"}` (sin argumentos) | ok | Mismo `aviso`, `entidad:null` | — |
| E1 | `{que:"vista", entidad:"no.existe"}` | error | `Entidad 'no.existe' desconocida: usa ["components.panels",…,"proyecto.ventana"]` | — |
| E2 | `{que:"vista", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E3 | `{que:"vista", incluir_uia:true}` | error | `'incluir_uia' no aplica a 'vista': use gui_observar que:controles incluir_uia:true o que:uia` | — |

#### `esquema` (2 H / 1 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"esquema", detalle:"completo"}` | ok, 186 ms | `count:6` (`components.panels`, `components.inverters`, `components.batteries`, `variante.orientation`, `variante.system`, `proyecto.ventana`); `cobertura:{total:89,por_estado:{2:71,3:14,4:4}}` | 33,9 KB; esquema completo. |
| H2 | `{que:"esquema", entidad:"variante.orientation"}` | ok | `count:1`, 5 campos | Filtro por entidad. |
| E1 | `{que:"esquema", entidad:"no.existe"}` | error | `Entidad 'no.existe' desconocida: usa […]` | — |

#### `mensajes` (2 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"mensajes", lineas:50}` tras rellenar un módulo con valores de prueba | ok | `count:1`, `mensajes:["El exponente eV/KT es demasiado grande (76.333), reducido a -0.001"]`, `ruta:…\20261108_0415Err.log` | Aviso **real** escrito por PVsyst; no lo inventó el MCP. |
| H2 | `{que:"mensajes", lineas:1}` | ok | `count:0` en ese instante, misma `ruta` | Límite inferior válido. |
| E1 | `{que:"mensajes", lineas:501}` | error | `Value 501 out of range for 'lineas' (between 1 and 500)` | Esquema. |
| E2 | `{que:"mensajes", lineas:0}` | error | `Value 0 out of range for 'lineas' (between 1 and 500)` | Esquema. |

#### `funciones` (3 H / 1 E lógico)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"funciones", detalle:"completo"}` | ok, 3,7 KB | `count:209`; p. ej. `{clave:"diseno_preliminar.conectado_a_la_red",abre:"TDimensForm",via:"menu"}` | Catálogo completo. |
| H2 | `{que:"funciones", funcion:"proyecto.cargar"}` | ok | `count:1`, `destino:{id:68}`, `abre:"TProjectLoadForm"` | Filtro por clave. |
| H3 | `{que:"funciones", ventana:"TProjectForm"}` | ok | `count:39` | Filtro por ventana anfitriona. |
| E1 | `{que:"funciones", funcion:"no.existe"}` | ok, 0 | `count:0`, `funciones:[]` | **No hay ruta de error**: un filtro sin coincidencias devuelve 0, no error. Se documenta como comportamiento, no como bug. |

#### `menus` (1 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{que:"menus", ventana:"THomeForm"}` en escritorio **compartido** | ok, 10,7 s | `count:8`, `completo:true`, `error:null`; menú 0 con 14 entradas (`items_total:14`, 13 mostradas), menú 7 = menú de sistema | **BUG-18 no reproducido** esta sesión. |
| E1 | `{que:"menus", ventana:"THomeForm"}` en **aislado** | error | `Los menús se abren con el teclado y el escritorio aislado ignora las teclas: usa el botón equivalente (accion:clic_texto) o PVSYST_MCP_ESCRITORIO sin aislar` | Rechazo por diseño. |
| E2 | `{que:"menus", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found` | — |
| E3 | `{que:"menus"}` | error | `Missing required argument 'ventana' (hwnd, class or title)` | — |

**Nota de `menus`:** devuelve posiciones e `id`, pero **no los textos** de los
menús ni de las entradas; el llamador necesita `funciones` o `capturar` para
saber qué es cada posición (GUI-12). Además lista 8 menús (incluido el de
sistema) mientras `menu` solo acepta 0..6 (GUI-11).

---

### 2.2 `gui_actuar` (26 acciones, 49 rutas)

#### `click` (4 H / 4 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"click", ventana:"Herramientas"}` | ok, 28 ms | `invoked:"Herramientas"`, `hwnd:"0x4008E8"`, `window.class:"TPngBitBtn"` | Abrió `TToolsForm` (comprobado con `ventanas`). |
| H2 | `{accion:"click", ventana:"Sitio y datos meteo"}` | ok | `invoked:"Sitio y datos meteo"`, `hwnd:"0xA0952"` | Abrió `TDimensSituationForm`. |
| H3 | `{accion:"click", ventana:"OK"}` en `TProjectLoadForm` | ok | `invoked:"OK"` | Cargó `_DEMO_Commercial` (`TProjectForm` con ese título). |
| H4 | `{accion:"click", ventana:"0x14091C"}` (botón) | ok | `invoked:"0x14091C"` | Botón deshabilitado: sin efecto (ver E2). |
| E1 | `{accion:"click", ventana:"TDimensForm", texto:"Sitio y datos meteo"}` | **bug** | `invoked:"TDimensForm"`, `hwnd:"0x1C08EA"`; no se abrió ninguna ventana | **GUI-06**: `texto` se ignora; el clic fue a la ventana, no al botón. |
| E2 | `{accion:"click", ventana:"0x14091C"}` («Ejecutar simulación», `enabled:false`) | **bug** | `vista.estado:"ok"`, `resumen:"Clic enviado al control."`, `window.enabled:false` | **GUI-07**: no comprueba que el control esté habilitado. |
| E3 | `{accion:"click", ventana:"TNoExisteForm", texto:"Cerrar"}` | error | `Control 'TNoExisteForm' not found (accion click)` | — |
| E4 | `{accion:"click", ventana:"No"}` con `TMessageForm` «Advertencia» encima | **bug** | `TShadingsForm is disabled: a modal dialog is open on top of it (TMessageForm)` | **GUI-08**: «No» resuelve a `TShadingsForm` porque su título contiene «cerca**no**s»; el botón correcto era `0xE09EA`. |

#### `clic_fisico` (1 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"clic_fisico", ventana:"Herramientas"}` (aislado) | ok, 19 ms | `invoked:"Herramientas"`, `hwnd:"0xA0A28"` | Abrió `TToolsForm`. |
| E1 | `{accion:"clic_fisico", ventana:"THomeForm", objetivo:{x:919,y:565}}` | **bug** | `invoked:"THomeForm"`, `hwnd:"0xB09AE"`; se abrió `TProjectForm` «New.PRJ» | **BUG-24**: ignora `objetivo` y pulsa el centro de la ventana; el punto pedido era el botón «Herramientas». |
| E2 | `{accion:"clic_fisico"}` | error | `Missing required argument 'ventana' (hwnd, class or title of the control to invoke)` | — |
| E3 | `{accion:"clic_fisico", ventana:"TNoExisteForm"}` | error | `Control 'TNoExisteForm' not found` | — |

#### `mensaje` (1 H / 1 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"mensaje", ventana:"TToolsForm", mensaje:16}` (WM_CLOSE) dentro de una `secuencia` | ok | `action:"mensaje"`, `hwnd:"0xC0978"` | La ventana desapareció; el paso siguiente `esperar_cierre` dio `cerrado:true`. |
| E1 | `{accion:"mensaje", ventana:"THomeForm"}` sin `mensaje` | error | `accion 'mensaje' requires numeric 'mensaje' (accion mensaje)` | — |

#### `cerrar` (3 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"cerrar", ventana:"TDEMOForm"}` | ok, 651 ms | `window.closed:true`, `window.visible:false` | El modal desapareció. |
| H2 | `{accion:"cerrar", ventana:"TToolsForm"}` | ok | `window.closed:true` | — |
| H3 | `{accion:"cerrar", ventana:"TWorkspaceForm"}` | ok | `window.closed:true` | — |
| E1 | `{accion:"cerrar", ventana:"TPerspectiveForm"}` con escena modificada | error | `TPerspectiveForm is still open after asking it to close: it may have opened a confirmation (gui_observar que:ventanas)` | Apareció `TMessageForm` «¿Quiere mantener sus modificaciones en su escena de sombreado?»; el llamador debe leerla y elegir. |
| E2 | `{accion:"cerrar", ventana:"TNoExisteForm"}` | error | `Window 'TNoExisteForm' not found (already closed?)` | — |

#### `activar` (1 H / 1 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"activar", ventana:"TDimensForm"}` | ok, 111 ms | `action:"activar"`, `window.visible:true` | La ventana quedó al frente. |
| E1 | `{accion:"activar", ventana:"TNoExisteForm"}` | error | `Control 'TNoExisteForm' not found (accion activar)` | — |

#### `teclas` (2 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"teclas", ventana:"0x609E6", teclas:"RETURN"}` (aislado) | ok | `sent:"RETURN"`, `foco:true`, `via:"mensajes"` | La lista de `TProjectLoadForm` se filtró (efecto observable). |
| H2 | `{accion:"teclas", ventana:"TToolsForm", teclas:"ESC"}` (compartido) | **bug** | `sent:"ESC"`, `foco:false`, `via:"keybd_event"`, `aviso:"The target is the window…"`, `vista.estado:"ok"` | **BUG-23**: TToolsForm **no** se cerró. |
| E1 | `{accion:"teclas", ventana:"THomeForm", teclas:"CTRL+S"}` (aislado) | error | `On the isolated desktop only Alt is accepted as a modifier: Ctrl and Shift cannot be sent as messages` | — |
| E2 | `{accion:"teclas", ventana:"THomeForm", teclas:"NOEXISTE"}` | error | `Tecla no reconocida 'NOEXISTE' en 'NOEXISTE'` | — |
| E3 | `{accion:"teclas", ventana:"THomeForm"}` | error | `Missing required argument 'teclas' (e.g. "F1" or "CTRL+S")` | — |

#### `fila` (2 H / 6 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"fila", ventana:"TProjectLoadForm", clase:"TVirtualStringTree", indice:0}` (compartido) | ok, 640 ms | `fila:0`, `leida:0`, `verificado:"banda_resaltada"`, `dpi:144`, `dy_px96:30` | Fila 0 resaltada; `click OK` cargó `_DEMO_Commercial`. |
| H2 | `{accion:"fila", ventana:"TChoixObjetPVForm", clase:"TListBox", indice:0}` | ok, 657 ms | `fila:0`, `leida:0`, `verificado:"banda_resaltada"`, `dy_px96:8` | Selector de módulos. |
| E1 | `{accion:"fila", ventana:"THomeForm", indice:0}` | error | `THomeForm of THomeForm has no measured row geometry; measured on: TVirtualStringTree of TProjectLoadForm, TListBox of TChoixObjetPVForm, TVirtualStringTree of TShdObjectSelectionForm` | — |
| E2 | `{…, ventana:"TProjectLoadForm", clase:"TVirtualStringTree", indice:999}` | error | `Row 999 does not exist: the control shows 29 rows (0..=28)` | — |
| E3 | `{accion:"fila", ventana:"THomeForm"}` | error | `Missing: indice. Example: {…}` | — |
| E4 | `{accion:"fila", indice:0}` | error | `Missing: ventana. Example: {…}` | — |
| E5 | `{…, ventana:"TProjectLoadForm", clase:"TNoExiste", indice:0}` | error | `No visible child of class 'TNoExiste' in that window. Visible classes: TEdit, TPngBitBtn, TVirtualStringTree` | — |
| E6 | `{accion:"fila", ventana:"THomeForm", indice:0}` en **aislado** | error | `fila uses a physical click, which is not measured on the isolated desktop: use accion:clic_texto or PVSYST_MCP_ESCRITORIO without isolation` | Rechazo por diseño; se probó en compartido. |

#### `escribir` (2 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"escribir", ventana:"TProjectLoadForm", …}` sobre el `TEdit` `0x609E6`, texto `QA_GUI` | ok | `hwnd:"0x609E6"`, `texto:"QA_GUI"` | `vista` del formulario: `edit_39_1017:"QA_GUI"`. |
| H2 | `{accion:"escribir", ventana:"TPVModuleForm", campo:{clase:"TEdit",x:121,y:52}, texto:"QA_GUI_M125"}` | ok | `clase:"TPVModuleForm"`, `texto:"QA_GUI_M125"` | `vista components.panels`: `modelo:"QA_GUI_M125"`. |
| E1 | `{…, campo:{clase:"TEdit",x:121,y:75}, texto:"QA_GUI_Mod3"}` (ancla documentada de `modelo`) | **bug** | ok, `texto:"QA_GUI_Mod3"` | **GUI-01**: `vista` muestra `nombre_archivo:"QA_GUI_Mod3"` y `modelo:"QA_GUI_Mod2"`; escribió en el campo equivocado. |
| E2 | `{accion:"escribir", ventana:"TPVModuleForm", texto:"x"}` (sin `campo`) | error | `'escribir' on class TPVModuleForm has no effect: pass 'campo' (clase, x, y) of the control or a window of class TEdit/TMemo/TLabeledEdit/TMaskEdit/TSpinEdit` | — |
| E3 | `{accion:"escribir", ventana:"0x30AB0", texto:"QA_GUI_File"}` | error | `'escribir' on class TPVModuleForm has no effect: …` | El hwnd era el de la ventana, no el del control; el mensaje es correcto. |

#### `seleccionar` (1 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"seleccionar", ventana:"TPVModuleForm", campo:{clase:"TComboBox",x:112,y:188}, texto:"Si-poly"}` | ok, 428 ms | `texto:"Si-poly"` | `vista components.panels`: `tecnologia:"Si-poly"`. |
| E1 | `{…, campo:{clase:"TComboBox",x:116,y:211}, texto:"Si-poly"}` (ancla documentada) | error | `No TComboBox at (116, 211) of 'Definición de un módulo FV': did the PVsyst version change?` | **GUI-01**: el ancla de `vista` no es coordenada de cliente. |
| E2 | `{accion:"seleccionar", ventana:"TPVModuleForm", campo:{…}, texto:"NoExiste"}` | error | `PVsyst offers no 'NoExiste' in that list (items are matched by prefix, not by substring)` | — |
| E3 | `{accion:"seleccionar", ventana:"TWorkspaceForm", texto:"x"}` | error | `'seleccionar' on class TWorkspaceForm has no effect: pass 'campo' … or a window of class TComboBox/TComboBoxEx/TListBox` | — |

#### `pagina` (2 H / 4 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"pagina", ventana:"TPVModuleForm", indice:1}` tras rellenar los campos obligatorios | ok, 514 ms | `indice:1`; `formulario.pestanas`: «Dimensiones y tecnología» `seleccionada:true` | Cambio real de pestaña. |
| H2 | `{accion:"pagina", ventana:"TPreferencesForm", indice:1/2/3}` | ok, ~450 ms cada uno | `indice:1`, `2`, `3` | Pestañas seleccionadas; en la 2 se leyeron los `TCheckBox`. |
| E1 | `{accion:"pagina", ventana:"TPVModuleForm", indice:1}` con campos obligatorios vacíos | error | `PVsyst did not switch to tab 1: it is still on 0. The form vetoed the change (validation of the current tab fields); … Form texts: […]` | PVsyst vetó el cambio; el error explica. |
| E2 | `{accion:"pagina", ventana:"TPVModuleForm", indice:99}` | error | `'indice' 99 out of range: 'Definición de un módulo FV' has 5 tab pages (0 to 4)` | — |
| E3 | `{accion:"pagina", ventana:"TPVModuleForm"}` | error | `Missing: indice. Example: {…}` | — |
| E4 | `{accion:"pagina", ventana:"THomeForm", indice:0}` | error | `'PVsyst 8.1.8 - DEMO' has no tab pages. Form texts: […]` | — |

#### `esperar_cierre` (2 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `secuencia` `[{mensaje WM_CLOSE TToolsForm},{esperar_cierre TToolsForm, timeout_ms:5000}]` | ok | `completados:2`; `cerrado:true`, `ya_cerrada:false`, `espera_ms:5000` | Cierre real durante la espera. |
| H2 | `{accion:"esperar_cierre", ventana:"TToolsForm", timeout_ms:1000}` ya cerrada | ok | `cerrado:true`, `ya_cerrada:true`, `aviso:"No window matches that reference now…"` | Éxito trivial documentado. |
| E1 | `{accion:"esperar_cierre", ventana:"TToolsForm", timeout_ms:2000}` con la ventana abierta | **bug** | `cerrado:false`, `espera_ms:2000`, `vista.estado:"ok"`, `resumen:"Cierre de la ventana esperado."` | **BUG-03**: la vista dice ok con `cerrado:false`. |
| E2 | `{accion:"esperar_cierre", timeout_ms:1000}` | error | `Missing: ventana. Example: {…}` | — |

#### `lanzar` (2 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"lanzar", esperar_segundos:45}` con PVsyst cerrado | ok, 16,6 s | `pid:29668`, `ready:true`, `windows.count:4` | `THomeForm` visible; `TDEMOForm` apareció después. |
| H2 | `{accion:"lanzar"}` con PVsyst ya corriendo | ok, 44 ms | `pid:30904`, `ready:true`, `ya_en_ejecucion:true` | Instancia única respetada. |
| E1 | `{accion:"lanzar", proyecto:"QA_GUI_20261108_0351"}` con PVsyst corriendo | error | `'proyecto' cannot be opened: PVsyst is already running. Close it (gui_actuar accion:cerrar_todo) and repeat lanzar with proyecto` | — |
| E2 | `{accion:"lanzar", proyecto:"C:\\…\\QA_GUI_20261108_0351.PRJ", cerrar_demo:true, esperar_segundos:60}` con PVsyst cerrado | **bug** | `pid:29384`, `ready:true`; tras cerrar `TDEMOForm` solo `THomeForm` | **GUI-15**: el proyecto no se abre; la línea de comandos sí llevaba la ruta. |

#### `uia_invocar` (6 H / 4 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"uia_invocar", ventana:"TDimensSituationForm", nombre:"Cancelar", rol:"button", operacion:"invocar"}` | ok, 2,1 s | `operacion:"invocar"`, `nombre:"Cancelar"` | La ventana se cerró. |
| H2 | `{…, nombre:"", rol:"edit", operacion:"fijar_valor", valor:"QA_GUI_Sitio"}` | ok, 2,2 s | `operacion:"fijar_valor"` | `vista TDimensSituationForm`: `edit_68_157:"QA_GUI_Sitio"`. |
| H3 | `{…, nombre:"", rol:"combo box", operacion:"expandir"}` | ok, 2,1 s | `operacion:"expandir"` | El árbol UIA mostró `list item` «Todos los países», «Favoritos», «Europa»… |
| H4 | `{…, nombre:"", rol:"combo box", operacion:"contraer"}` | ok, 2,1 s | `operacion:"contraer"` | El desplegable se cerró. |
| H5 | `{…, nombre:"Europa", rol:"list item", operacion:"seleccionar"}` | ok, 668 ms | `operacion:"seleccionar"` | `vista`: `combobox_142_157:"Europa"`. |
| H6 | `{…, ventana:"TPreferencesForm", nombre:"Activación del bip del sistema", rol:"checkbox", operacion:"alternar"}` | ok, 2,1 s | `operacion:"alternar"` | `formulario`: `marcado` pasó de `true` a `false`; se volvió a alternar. |
| E1 | `{…, nombre:"Activación del bip del sistema", rol:"check box", operacion:"alternar"}` | error | `Elemento UIA 'Activación del bip del sistema' no encontrado; cercanos: …` | El rol real es `checkbox`, no `check box`. |
| E2 | `{accion:"uia_invocar", ventana:"TDimensForm"}` | error | `Missing required argument 'operacion' (invocar\|fijar_valor\|alternar\|seleccionar\|expandir\|contraer)` | — |
| E3 | `{…, nombre:"NoExiste", operacion:"invocar"}` | error | `Elemento UIA 'NoExiste' no encontrado; cercanos: …` | — |
| E4 | `{accion:"uia_invocar", ventana:"TDimensForm", nombre:"Sitio y datos meteo", operacion:"invocar"}` | **bug** | La llamada **no volvió en >10 min**; el proceso MCP quedó bloqueado; `TDimensSituationForm` sí se abrió (visto desde otro MCP) | **GUI-02**: `Invoke()` sin timeout; cerrar el modal no desbloqueó. |

#### `esperar` (1 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"esperar", clase:"TDimensSituationForm", timeout_ms:3000}` con la ventana abierta | ok | `window.class:"TDimensSituationForm"`, `tarde:false` | — |
| E1 | `{accion:"esperar", clase:"TNoExisteForm", timeout_ms:1000}` | error | `timeout waiting for TNoExisteForm: it did not appear in 1000 ms nor in the 1500 ms recheck` | — |
| E2 | `{accion:"esperar", timeout_ms:1000}` | error | `Pass 'clase' or 'titulo' to wait for a window` | — |

#### `paso` (1 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"paso", ventana:"THomeForm", objetivo:{nombre:"Datos medidos"}, verificar:{clase:"TDamMainForm"}, timeout_ms:8000}` | ok, 2,5 s | `via:"uia"`, `verificado:true` | `TDamMainForm` «Datos medidos» abierta. |
| E1 | `{accion:"paso", pasos:[]}` | error | `Missing required argument 'objetivo' (nombre, rol, clase o titulo)` | — |
| E2 | `{accion:"paso", objetivo:{}}` | error | `'objetivo' necesita nombre, rol, clase o titulo` | — |
| E3 | `{accion:"paso", objetivo:{nombre:"Cerrar"}, operacion:"invocar"}` | error | `'operacion' no aplica a 'paso': la cascada elige la acción; use uia_invocar operacion:… o gui_actuar accion:campo/seleccionar` | — |

#### `funcion` (8 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"funcion", funcion:"diseno_preliminar.conectado_a_la_red"}` | ok, 660 ms | `ventanas_nuevas:[TDimensForm]` | — |
| H2 | `{…, funcion:"proyecto.cargar"}` | ok, 664 ms | `ventanas_nuevas:[TProjectLoadForm]` | — |
| H3 | `{…, funcion:"variante.orientacion"}` | ok, 848 ms | `ventanas_nuevas:[TOrientationForm]` | — |
| H4 | `{…, funcion:"variante.sistema"}` | ok | `ventanas_nuevas:[TSystemegridForm]` | — |
| H5 | `{…, funcion:"variante.sombreados_cercanos"}` | ok | `ventanas_nuevas:[TShadingsForm]` | — |
| H6 | `{…, funcion:"variante.sombreados_cercanos.construccion_perspectiva"}` | ok, 1,7 s | `ventanas_nuevas:[TPerspectiveForm]` | — |
| H7 | `{…, funcion:"bases_de_datos.modulos_pv.nuevo"}` | ok, 15,7 s | `ventanas_nuevas:[TPVModuleForm]` | — |
| H8 | `{…, funcion:"configuraciones.preferencias"}` | ok, 852 ms | `ventanas_nuevas:[TPreferencesForm]` | — |
| E1 | `{…, funcion:"variante.orientacion"}` con `TToolsForm` modal encima | error | `THomeForm is disabled: a modal dialog is open on top of it (UAC_InputIndicatorOverlayWnd). Close it …` | **GUI-13**: nombra la ventana UAC, no el modal real (`TToolsForm`). |
| E2 | `{…, funcion:"no.existe"}` | error | `Función 'no.existe' fuera del catálogo (gui_observar que:funciones)` | — |

#### `menu` (2 H / 5 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"menu", ventana:"THomeForm", menu:0, item:0}` (compartido) | ok, 12,5 s | `id:23`, `ventanas_nuevas:[TWorkspaceForm]` | Abrió «Espacio de trabajo de usuario». |
| H2 | `{accion:"menu", ventana:"THomeForm", menu:0, item:4, subitem:0}` | ok, 13,6 s | `id:40`, `ventanas_nuevas:[#32770 «Ruta de la fuente»]` | Submenú «Importar componentes > desde carpeta». |
| E1 | `{…, menu:99, item:0}` | error | `La barra de THomeForm tiene 7 menús (0..6): 'menu' 99 daría la vuelta` | — |
| E2 | `{…, menu:0, item:99}` | error | `El menú 0 solo tiene 14 entradas` | — |
| E3 | `{…, menu:7, item:0}` | error | `La barra de THomeForm tiene 7 menús (0..6): 'menu' 7 daría la vuelta` | **GUI-11**: `menus` lista el 7 (sistema), `menu` lo rechaza. |
| E4 | `{accion:"menu", ventana:"THomeForm", menu:0, item:0}` en **aislado** | error | `Los menús se abren con el teclado y el escritorio aislado ignora las teclas: …` | Rechazo por diseño. |
| E5 | `{accion:"menu", menu:0, item:0}` | error | `Missing: ventana. Example: {…}` | — |

#### `campo` (2 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"campo", ventana:"TPVModuleForm", etiqueta:"Modelo", valor:"QA_GUI_Mod2"}` | ok, 1,8 s | `antes:""`, `despues:"QA_GUI_Mod2"`, `clase:"TEdit"` | `vista`: `modelo:"QA_GUI_Mod2"`. |
| H2 | `{accion:"campo", ventana:"TElementaryObjectForm", etiqueta:"Nombre"/"Anchura (DX)"/"Longitud (DY)"/"Albura (DZ)"}` | ok, ~2,3 s cada uno | `antes:"Paralelepípedo"` → `despues:"QA_Caja"`; `1.00` → `10.00`, `5.00`, `3.00` | La lista de objetos mostró `QA_Caja` tipo `Paralelepípedo`. |
| E1 | `{…, etiqueta:"NoExiste", valor:"1"}` | error | `No field is called 'noexiste'. Labels seen: Toper; GOper; … Use gui_actuar accion:rellenar with entidad 'components.panels' and keys: …` | — |
| E2 | `{accion:"campo", ventana:"TOrientationForm", etiqueta:"Inclinación del plano", valor:"25"}` | error | `'inclinación del plano' matches several fields: Inclinación del plano; Inclinación del plano` | Ambigüedad real (dos etiquetas iguales); no adivina. |
| E3 | `{accion:"campo", ventana:"TPVModuleForm"}` | error | `Missing: etiqueta, valor` | — |

#### `rellenar` (5 H / 2 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"rellenar", entidad:"proyecto.ventana", valores:{descripcion:"QA_GUI_Simulado"}, simular:true}` | ok, 40 ms | `simulado:true`, `resultados:[{estado:"simulado", antes:"Nuevo Proyecto", despues:null}]` | No escribió: `vista` siguió con «Nuevo Proyecto». |
| H2 | `{…, valores:{descripcion:"QA_GUI_Descripcion"}}` | ok, 874 ms | `resultados:[{estado:"ok", antes:"Nuevo Proyecto", despues:"QA_GUI_Descripcion"}]` | `vista`: `descripcion:"QA_GUI_Descripcion"`. |
| H3 | `{…, entidad:"variante.orientation", valores:{inclinacion_plano_deg:30, azimut_deg:-10}}` | ok, 2,0 s | `inclinacion_plano_deg:"30.0"`, `azimut_deg:"-10.0"` | `vista`: 30.0 / -10.0. |
| H4 | `{…, entidad:"variante.system", valores:{n_modulos_serie:22, n_cadenas:40}}` | ok, 2,0 s | `n_modulos_serie:"22"`, `n_cadenas:"40"`; `recalculados:[{…false→true}]` | `vista`: 22 / 40. |
| H5 | `{…, entidad:"components.panels", valores:{modelo:"QA_GUI_Mod", fabricante:"QA", potencia_nominal_w:400, isc_a:10.5, voc_v:45.2, impp_a:9.8, vmpp_v:37.5, tecnologia:"Si-mono"}}` | ok, 5,7 s | 8 `resultados` con `estado:"ok"`; `campos_que_difieren:0` | `vista`: todos los valores. |
| E1 | `{…, entidad:"no.existe", valores:{x:1}}` | error | `Entidad 'no.existe' desconocida: usa […]` | — |
| E2 | `{…, entidad:"components.panels", valores:{potencia_nominal_w:99999}}` | error | `components.panels: 'potencia_nominal_w' 99999 fuera de rango [0.1, 2000]. Claves editables: […]` | — |
| E3 | `{…, valores:{no_existe:1}}` | error | `components.panels: 'no_existe' no existe. Claves editables: […]` | — |

#### `clic_texto` (2 H / 3 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"clic_texto", ventana:"TPreferencesForm", texto:"Cancelar"}` | ok, 624 ms | `coincidencias:1`, `control:{clase:"TPngBitBtn",hwnd:"0xB09E4"}`, `punto:{x:900,y:689}` | La ventana se cerró. |
| H2 | `{accion:"clic_texto", ventana:"THomeForm", texto:"Herramientas"}` | ok, 836 ms | `coincidencias:1`, `control.hwnd:"0xA0A28"` | Abrió `TToolsForm`. |
| E1 | `{accion:"clic_texto", ventana:"TProjectLoadForm", texto:"QA_GUI_20261108_0351"}` | error | `'QA_GUI_20261108_0351' no está dibujado en la ventana (el OCR puede haberlo leído mal: gui_observar que:capturar)` | — |
| E2 | `{accion:"clic_texto", ventana:"THomeForm"}` | error | `Missing: texto. Example: {…}` | — |
| E3 | `{accion:"clic_texto", ventana:"THomeForm", texto:"Herramientas", indice:5}` | **bug** | ok, `coincidencias:1`, `punto:{x:919,y:565}` | **GUI-09**: con una sola coincidencia e `indice:5`, en vez de error cayó al fallback UIA y pulsó la única caja. |

#### `reiniciar` (1 H / 0 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"reiniciar"}` sin diálogo de fallo | error (guarda) | `PVsyst shows no failure dialog (madExcept or the instability warning): there is nothing to restart. Use cerrar or lanzar` | Guarda correcta. |
| E1 | Fallo real de PVsyst | **no probado** | — | Provocar un crash de PVsyst no es reproducible de forma segura; se documenta como bloqueado (§4). |

#### `cerrar_todo` (2 H / 0 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"cerrar_todo"}` en aislado con 5 ventanas | ok, 625 ms | `cerradas:5`, `quedan:[]`, `rondas:2` | PVsyst terminó. |
| H2 | `{accion:"cerrar_todo"}` en compartido con 4 ventanas | ok, 616 ms | `cerradas:4`, `quedan:[]`, `rondas:2` | — |
| E1 | Sin ventanas | no ejecutado | — | No hay ruta de error: `cerrar_todo` es idempotente por código; no se ejecutó sin ventanas. |

#### `secuencia` (2 H / 4 E)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"secuencia", pasos:[{mensaje WM_CLOSE TToolsForm},{esperar_cierre TToolsForm}]}` | ok, 245 ms | `completados:2`, `resultados:[…]`; `cerrado:true` | Cierre real. |
| H2 | `{accion:"secuencia", pasos:[{activar THomeForm},{esperar THomeForm},{click Herramientas}]}` | ok, 56 ms | `completados:3`; `TToolsForm` abierta | Efecto real de 3 pasos. |
| E1 | `{accion:"secuencia", pasos:[]}` | error | `'pasos' no puede estar vacío` | — |
| E2 | `{accion:"secuencia", pasos:[{accion:"secuencia", …}]}` | error | `'pasos'[0]: las secuencias no se anidan` | — |
| E3 | `{accion:"secuencia", pasos:[{ventana:"THomeForm"}]}` | error | `'pasos'[0] necesita 'accion'` | — |
| E4 | `{accion:"secuencia", pasos:[{activar TNoExisteForm},{activar THomeForm}]}` | error | `secuencia detenida en el paso 0 (activar): Control 'TNoExisteForm' not found (accion activar)` | Se detiene en el primer fallo. |

#### `escena` (1 H / 2 E + feliz bloqueada)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"escena", escena:"QA_Escena", objetos:[{tipo:"paralelepipedo",…}], simular:true}` | ok, 0 ms | `creado:false`, `objetos:[{campos:[…]}]`, `avisos:[…]`, `colisiones:[]` | Validación sin tocar PVsyst. |
| H2 | `{accion:"escena", escena:"QA_Escena", objetos:[…]}` con `TPerspectiveForm` abierto | **bug/bloqueada** | Tras 22,5 s: `PVsyst did not save C:\Users\Lenovo\PVsyst8.1_Data\Shadings\_mcp_previa_QA_Escena.SHD: no other way is attempted` | **GUI-03/GUI-04**: la lista de objetos quedó vacía; la creación manual sí añadió `QA_Caja`. |
| E1 | `{…, objetos:[{tipo:"paralelepipedo", …, ancho:10}]}` | error | `objetos[0]: 'ancho' no existe (claves de paralelepipedo: ["tipo","nombre","x","y","z","rotacion_azimut_deg","anchura_dx","longitud_dy","altura_dz"]); objetos[0]: falta 'anchura_dx'` | — |
| E2 | `{…}` sin editor 3D | error | `No 3D editor is open: open it with gui_actuar accion:funcion funcion:variante.sombreados_cercanos.construccion_perspectiva and retry` | — |

#### `escena_leer` (0 H / 1 E + feliz bloqueada)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"escena_leer", escena:"QA_Lectura"}` con editor abierto | **bug/bloqueada** | Tras 22,7 s: `PVsyst did not save …\_mcp_QA_Lectura.SHD: no other way is attempted` | **GUI-03**: mismo fallo de exportación. |
| E1 | `{accion:"escena_leer", escena:"QA_Escena"}` sin editor | error | `No 3D editor is open: …` | — |

#### `escena_importar` (0 H / 3 E + feliz bloqueada)

| Ruta | Llamada | Resultado | Evidencia | Verificación |
|---|---|---|---|---|
| H1 | `{accion:"escena_importar", escena:"QA_Import", archivo:"C:\\Users\\Lenovo\\PVsyst8.1_Data\\QA_GUI_malla.pvc"}` con editor abierto | **bug/bloqueada** | Tras 40,8 s: `PVsyst did not save …\_mcp_QA_Import.SHD: no other way is attempted` | La importación **sí** ocurrió: la lista de objetos mostró `Objeto 57` (malla) y los `Campo de tabla`. |
| E1 | `{…, archivo:"QA_GUI_malla.pvc"}` (relativa) | **bug** | `No existe QA_GUI_malla.pvc` (el archivo existe en el workspace) | **GUI-05**: resuelve contra el CWD del proceso, no contra el workspace. |
| E2 | `{…, archivo:"no_existe.pvc"}` | error | `No existe no_existe.pvc` | — |
| E3 | `{…, archivo:"…\\QA_GUI_malla.pvc"}` sin editor | error | `No 3D editor is open: …` | — |

---

## 3. Bugs

Severidad: **P0** catástrofe, **P1** alto (pérdida/corrupción o bloqueo), **P2**
funcionalidad principal rota o contrato que miente, **P3** molestia o mensaje.

### GUI-01 · P1 · `vista`/`esquema` publican coordenadas de ventana y `campo`/`seleccionar` las leen como de cliente

**Tool/acción:** `gui_observar que:vista` + `gui_actuar accion:escribir|seleccionar`.

**Reproducción**

1. Abrir `TPVModuleForm` (`funcion:bases_de_datos.modulos_pv.nuevo`).
2. `gui_observar {que:"vista", entidad:"components.panels", completo:true}` →
   `modelo` está en `(121, 75)`.
3. `gui_actuar {accion:"escribir", ventana:"TPVModuleForm",
   campo:{clase:"TEdit", x:121, y:75}, texto:"QA_GUI_Mod3"}` → ok.
4. `gui_observar {que:"vista", entidad:"components.panels"}` →
   `modelo:"QA_GUI_Mod2"` y **`nombre_archivo:"QA_GUI_Mod3"`**.
5. Con `y:52` (75 − 23 px de barra de título) el mismo `escribir` cambia
   `modelo` a `QA_GUI_M125`.

**Esperado:** que la clave publicada por `vista`/`esquema` escriba el campo que
nombra, o que la tool rechace la coordenada. El servidor dice literalmente
«las claves de vista como combobox_100_138 son coordenadas de cliente que
puedes pasar a campo/seleccionar».

**Real:** `campos_vivos` calcula la posición con `rect_of(h)` (rectángulo de la
**ventana**, incluida la barra de título) y `Formulario::control` suma el ancla
a `origen_cliente` (área de **cliente**). La diferencia (≈23 px verticales) hace
que el punto caiga en otro control; en el caso medido, en `nombre_archivo`.

**Evidencia:** `{"accion":"escribir","clase":"TPVModuleForm","hwnd":"0x30AB0",
"texto":"QA_GUI_Mod3"}` y luego `vista`:
`[("modelo","QA_GUI_Mod2"),("nombre_archivo","QA_GUI_Mod3")]`. Con `y:52`:
`[("modelo","QA_GUI_M125")]`.

**Archivo/línea:** `src/vista_agente/campos_vivos.rs:55-68` (origen
`rect_of(h)`) frente a `src/adapters/gui/conduccion.rs:131-145`
(`origen_cliente` + ancla). También afecta a `seleccionar`
(`x:116,y:211` → `No TComboBox at (116, 211)`).

**Regresión:** no hay prueba diferencial en esta sesión; el informe previo no
ejercitó la ruta feliz de `escribir`/`seleccionar` (§7.5). Por el código, el
desajuste está en el worktree del PR.

### GUI-02 · P1 · `uia_invocar` con `invocar` puede colgar el servidor para siempre

**Tool/acción:** `gui_actuar accion:uia_invocar operacion:invocar`.

**Reproducción:** con `TDimensForm` abierto,
`{accion:"uia_invocar", ventana:"TDimensForm", nombre:"Sitio y datos meteo",
operacion:"invocar"}`. La llamada abrió `TDimensSituationForm` (visible desde
otro MCP) pero **no volvió**; a los 10 minutos el proceso seguía bloqueado.
Cerrar `TDimensSituationForm` desde otro MCP no lo desbloqueó; hubo que matar
el proceso. `uia_invocar` sobre «Cancelar» (que cierra, no abre) volvió en
2,1 s.

**Esperado:** timeout o retorno tras invocar; nunca bloquear el servidor.

**Real:** `IUIAutomationInvokePattern::Invoke()` es síncrono y no tiene timeout;
si el manejador de PVsyst entra en un bucle modal/largo, la llamada no vuelve.
El servidor procesa las tools en un hilo y queda bloqueado.

**Evidencia:** el cliente stdio agotó 300 s; `ping` seguía respondiendo (el
proceso vivía) pero cualquier `tools/call` quedaba esperando; el `uia_invocar`
de «Cancelar» devolvió `{operacion:"invocar", nombre:"Cancelar"}` en 2 075 ms.

**Archivo/línea:** `src/adapters/uia/elemento.rs:82` →
`src/adapters/uia/com.rs:212` (`.Invoke()` sin timeout).

**Regresión:** no determinado; la ruta no se había ejercitado en el informe
previo.

### GUI-03 · P2 · `escena`/`escena_leer`/`escena_importar` no pueden exportar la escena

**Tool/acción:** `gui_actuar accion:escena|escena_leer|escena_importar`.

**Reproducción**

1. Cargar `_DEMO_Commercial` en la GUI (con `fila` + `OK`).
2. Abrir `variante.sombreados_cercanos` y
   `variante.sombreados_cercanos.construccion_perspectiva` (`TPerspectiveForm`).
3. `{accion:"escena", escena:"QA_Escena", objetos:[{tipo:"paralelepipedo",
   nombre:"QA_Caja", x:0,y:0,z:0, rotacion_azimut_deg:0, anchura_dx:10,
   longitud_dy:5, altura_dz:3}]}`.
4. `{accion:"escena_leer", escena:"QA_Lectura"}`.
5. `{accion:"escena_importar", escena:"QA_Import",
   archivo:"C:\\Users\\Lenovo\\PVsyst8.1_Data\\QA_GUI_malla.pvc"}`.

**Esperado:** que PVsyst escriba `Shadings/<escena>.SHD` y la tool lo lea.

**Real:** las tres fallan tras 22,5 s / 22,7 s / 40,8 s con
`PVsyst did not save C:\Users\Lenovo\PVsyst8.1_Data\Shadings\_mcp_<escena>.SHD:
no other way is attempted`. El diálogo `TSaveObjPVForm` se abre, se rellena el
nombre y se pulsa «Guardar», pero **no aparece ningún `.SHD`** en `Shadings`
(comprobado con `find`), ni en el resto del workspace, y `mensajes` no registra
error. La exportación manual (escribir `_mcp_previa_QA_Escena` en el `TEdit`
0x80A3E y pulsar «Guardar») tampoco escribe el archivo. La importación de
`escena_importar` **sí** ocurre: la lista de objetos muestra `Objeto 57` y los
`Campo de tabla`; solo falla la exportación/lectura posterior.

**Evidencia**

```
escena        → "PVsyst did not save …\_mcp_previa_QA_Escena.SHD: no other way is attempted" (22 485 ms)
escena_leer   → "PVsyst did not save …\_mcp_QA_Lectura.SHD: no other way is attempted" (22 665 ms)
escena_importar → "PVsyst did not save …\_mcp_QA_Import.SHD: no other way is attempted" (40 781 ms)
```

**Archivo/línea:** `src/adapters/gui/escena/exportar.rs:48-66`
(`guardar_temporal` / `esperar_escrito`) y `:101-112`. El diálogo
`TSaveObjPVForm` tiene «Descripción» (0x809D4, arriba) y «Nombre de archivo»
(0x80A3E, abajo); `hijo_n("TEdit",1)` selecciona el de abajo (correcto por
`rect.top`), pero el guardado no produce archivo. La causa exacta del silencio
de PVsyst no se pudo aislar (no hay error en `Err.log`); se documenta como
bloqueo técnico.

**Regresión:** no determinado; el informe previo no llegó a la exportación
(§7.5).

### GUI-04 · P2 · `escena` da por creado un objeto que no está en la escena

**Tool/acción:** `gui_actuar accion:escena`.

**Reproducción:** ejecutar `escena` con un `paralelepipedo` y, tras el fallo de
exportación, abrir `escena_3d.herramientas.lista_de_objetos`.

**Esperado:** o el objeto está en la lista, o `escena` falla al crearlo.

**Real:** la lista muestra «Selección vacía» (sin filas); `escena` siguió hasta
la exportación como si el objeto existiera. La creación manual con
`accion:campo` (Nombre/Anchura/Longitud/Albura) + `click` «Cerrar objeto» sí
añade `QA_Caja`. El `escena` no verifica el número de objetos tras
`objeto::crear`.

**Evidencia:** lista OCR tras `escena`: `['e','Lista y gestión de objetos',
'# A','TIPO','Selección vacía','I','o','Nombre','x','z','Color']`; lista tras
la creación manual: `['…','QA_Caja','x','0.00','o','0.00','1','QA_Caja','TIPO',
'Paralelepípedo',…]`.

**Archivo/línea:** `src/adapters/gui/escena/objeto.rs:17-30` (solo comprueba
que el diálogo se cierre, no que el objeto se haya añadido).

**Regresión:** no determinado; ruta no ejercitada antes.

### GUI-05 · P2 · `escena_importar` resuelve rutas relativas contra el CWD, no contra el workspace

**Tool/acción:** `gui_actuar accion:escena_importar`.

**Reproducción:** `{accion:"escena_importar", escena:"QA_Import",
archivo:"QA_GUI_malla.pvc"}` con el archivo en
`C:\Users\Lenovo\PVsyst8.1_Data\QA_GUI_malla.pvc`.

**Esperado:** resolver contra el workspace (como el resto de rutas de la tool).

**Real:** `No existe QA_GUI_malla.pvc`; con la ruta absoluta pasa la validación
y llega al editor. `std::path::absolute` usa el CWD del proceso
(`C:\pvsyst-mcp`), no `paths.workspace`.

**Evidencia:** relativa → `No existe QA_GUI_malla.pvc`; absoluta → `No 3D
editor is open: …` (con el editor cerrado) y, con el editor abierto, la
importación llega a `Objeto 57`.

**Archivo/línea:** `src/adapters/gui/escena/importar.rs:49`.

**Regresión:** el informe previo ya vio este doble bloqueo (§2.14, «No existe
UserRecycleBin/Z_malla.pvc»); **preexistente**.

### GUI-06 · P2 · `click` acepta `texto`/`id` y los ignora

**Tool/acción:** `gui_actuar accion:click`.

**Reproducción:** `{accion:"click", ventana:"TDimensForm",
texto:"Sitio y datos meteo"}`.

**Esperado:** pulsar el botón «Sitio y datos meteo» o rechazar `texto` con un
error que diga que se use `clic_texto`/`ventana`.

**Real:** `invoked:"TDimensForm"`, `hwnd:"0x1C08EA"`, `vista.estado:"ok"`; no
se abrió ninguna ventana. `click` solo lee `ventana`; `texto` e `id` no se
consumen ni se validan.

**Evidencia:** `{"action":"click","invoked":"TDimensForm","hwnd":"0x1C08EA",
"window":{"class":"TDimensForm",…}}`; `ventanas` seguía con
`THomeForm/TDimensForm/TApplication` (sin `TDimensSituationForm`).

**Archivo/línea:** `src/adapters/gui/acciones.rs:191-210` (solo
`req_str(args, &["ventana"], …)`).

**Regresión:** no determinado; el informe previo usó esta misma llamada y
concluyó «clic enviado; no se observó apertura», sin identificar que `texto` se
ignoraba (§2.14).

### GUI-07 · P3 · `click` sobre un control deshabilitado devuelve `ok`

**Tool/acción:** `gui_actuar accion:click`.

**Reproducción:** `{accion:"click", ventana:"0x14091C"}` («Ejecutar
simulación», `enabled:false`).

**Esperado:** error «el control está deshabilitado» o, al menos,
`vista.estado` distinto de `ok`.

**Real:** `vista.estado:"ok"`, `resumen:"Clic enviado al control."`,
`window.enabled:false`; no ocurre nada. `modal::exigir_habilitada` comprueba
la ventana raíz, no el control.

**Evidencia:** `{"action":"click","hwnd":"0x14091C","window":{"enabled":false,
…},"vista":{"estado":"ok","resumen":"Clic enviado al control."}}`.

**Archivo/línea:** `src/adapters/gui/acciones.rs:204` +
`src/adapters/gui/modal.rs:52-64`.

**Regresión:** no determinado; misma familia que BUG-03.

### GUI-08 · P3 · `click ventana:"No"` resuelve a la ventana padre por subcadena

**Tool/acción:** `gui_actuar accion:click` (y `resolve` en general).

**Reproducción:** con `TMessageForm` «Advertencia» (botones Cancelar/No/Sí)
encima de `TShadingsForm`, `{accion:"click", ventana:"No"}`.

**Esperado:** pulsar el botón «No» del `TMessageForm`.

**Real:** `TShadingsForm is disabled: a modal dialog is open on top of it
(TMessageForm)`. `resolve` busca primero en las ventanas de nivel superior por
subcadena de título/clase; «No» está dentro de «cerca**no**s» en el título de
`TShadingsForm`. El botón correcto era `0xE09EA`.

**Evidencia:** error citado; con `{ventana:"0xE09EA"}` el `TMessageForm` se
cerró y `TPerspectiveForm` desapareció.

**Archivo/línea:** `src/adapters/gui/ventanas.rs:162-205` (`matches` por
subcadena; `resolve` recorre primero los top-level).

**Regresión:** no determinado; el informe previo no probó este caso.

### GUI-09 · P3 · `clic_texto` con `indice` fuera de rango pulsa la única coincidencia

**Tool/acción:** `gui_actuar accion:clic_texto`.

**Reproducción:** `{accion:"clic_texto", ventana:"THomeForm",
texto:"Herramientas", indice:5}` (solo hay 1 coincidencia).

**Esperado:** `indice fuera de rango (0..1)`.

**Real:** ok, `coincidencias:1`, `punto:{x:919,y:565}`; abrió `TToolsForm`.
`punto_de_clic` intenta OCR, `elegir_caja` falla por índice, y el `Err(ocr)` cae
al fallback `uia::centro(h, Some(texto), None)`, que ignora el índice.

**Evidencia:** `{"accion":"clic_texto","coincidencias":1,"punto":{"x":919,
"y":565},"texto":"Herramientas"}`.

**Archivo/línea:** `src/adapters/gui/clic_por_texto.rs:55-66`.

**Regresión:** no determinado.

### GUI-10 · P3 · `leer` sobre un `TEdit` devuelve el nombre del control, no su valor

**Tool/acción:** `gui_observar que:leer`.

**Reproducción:** `{que:"leer", ventana:"0x2207D6"}` (TEdit de
`TDimensSituationForm`) tras `uia_invocar fijar_valor` con `QA_GUI_Sitio`.

**Esperado:** el texto del campo (`QA_GUI_Sitio`).

**Real:** `text:"CommentEdit"` (el nombre interno del control). `vista` del
mismo formulario sí muestra `edit_68_157:"QA_GUI_Sitio"` porque usa
`WM_GETTEXT` (`ffi::texto_control`), mientras `leer` usa `GetWindowTextW`
(`text_of`), que en un control de otro proceso devuelve el título/nombre.

**Evidencia:** `{"class":"TEdit","hwnd":"0x2207D6","text":"CommentEdit"}`;
`vista TDimensSituationForm` → `{"clave":"edit_68_157","valor":"QA_GUI_Sitio"}`.

**Archivo/línea:** `src/adapters/gui/ventanas.rs:12-28` (`text_of`).

**Regresión:** no determinado; el informe previo solo leyó títulos de ventana.

### GUI-11 · P3 · `menus` lista 8 menús (incluido el de sistema) pero `menu` solo acepta 0..6

**Tool/acción:** `gui_observar que:menus` + `gui_actuar accion:menu`.

**Reproducción:** `{que:"menus", ventana:"THomeForm"}` → `menus` con
`menu:0..7` (el 7 con `sistema:true`); `{accion:"menu", ventana:"THomeForm",
menu:7, item:0}` → error.

**Esperado:** que `menu` acepte el 7 (menú de sistema) o que `menus` no lo
liste como una entrada más.

**Real:** `La barra de THomeForm tiene 7 menús (0..6): 'menu' 7 daría la
vuelta`.

**Evidencia:** `menus` → `[{menu:0,…},{…},{menu:7,sistema:true,items:7}]`;
`menu` 7 → error citado.

**Archivo/línea:** `src/adapters/gui/menu_por_posicion.rs:259` (`contar_menus`)
frente a `:270` (`listar`).

**Regresión:** no determinado.

### GUI-12 · P3 · `menus` no devuelve los textos de menús ni de entradas

**Tool/acción:** `gui_observar que:menus`.

**Reproducción:** `{que:"menus", ventana:"THomeForm"}`.

**Esperado:** al menos el título de cada menú («Archivo», «Proyecto», …) y el
texto de cada entrada, para poder elegir `menu`/`item` sin OCR.

**Real:** cada menú es `{menu:0, sistema:false, items:[{id,posicion,tipo,
deshabilitado}], items_total:14}`; no hay `texto`. El llamador debe cruzar con
`funciones` o capturar.

**Evidencia:** `{"items":[{"id":23,"posicion":0,"tipo":"comando"},…],
"items_total":14,"menu":0,"sistema":false}`.

**Archivo/línea:** `src/adapters/gui/menu_por_posicion.rs:270-294`.

**Regresión:** no determinado; el informe previo ya lo usó así.

### GUI-13 · P3 · El error de modal nombra la ventana UAC en vez del diálogo real

**Tool/acción:** cualquier `gui_actuar` bloqueado por un modal.

**Reproducción:** con `TToolsForm` (modal) encima de `THomeForm`,
`{accion:"funcion", funcion:"variante.orientacion"}`.

**Esperado:** `THomeForm is disabled: a modal dialog is open on top of it
(TToolsForm)`.

**Real:** `… (UAC_InputIndicatorOverlayWnd)`. `bloqueante_de` recorre las
ventanas del proceso en Z-order y la primera habilitada es la ventana UAC
(`UAC_InputIndicatorOverlayWnd`), no el modal de PVsyst.

**Evidencia:** error citado; `ventanas` mostraba `TToolsForm` y
`THomeForm`, y las dos UAC.

**Archivo/línea:** `src/adapters/gui/modal.rs:25-49` (`bloqueante_de`).

**Regresión:** no determinado; el informe previo vio el mismo nombre en
§2.14 (`funcion` con modal) sin identificarlo como bug.

### GUI-14 · P3 · `cerrar` deja el `TMessageForm` de confirmación y falla con «still open»

**Tool/acción:** `gui_actuar accion:cerrar`.

**Reproducción:** con `TPerspectiveForm` y una escena modificada,
`{accion:"cerrar", ventana:"TPerspectiveForm"}`.

**Esperado:** o cerrar descartando, o decir explícitamente «hay una
confirmación; responde con click».

**Real:** `TPerspectiveForm is still open after asking it to close: it may have
opened a confirmation (gui_observar que:ventanas)`; el `TMessageForm`
«¿Quiere mantener sus modificaciones en su escena de sombreado?» queda abierto.
Es el comportamiento documentado (no adivina), pero obliga a una llamada extra
y el mensaje no nombra el `TMessageForm` ni sus botones.

**Evidencia:** error citado; `formulario TMessageForm` →
`["Advertencia","¿Quiere mantener sus modificaciones en su escena de
sombreado?"]`, botones `Cancelar/No/Sí`.

**Archivo/línea:** `src/adapters/gui/acciones.rs:230-240` (`exigir_cerrada`).

**Regresión:** no determinado.

### GUI-15 · P2 · `lanzar proyecto:…` no abre el proyecto

**Tool/acción:** `gui_actuar accion:lanzar` con `proyecto`.

**Reproducción:** con PVsyst cerrado,
`{accion:"lanzar", proyecto:"C:\\Users\\Lenovo\\PVsyst8.1_Data\\Projects\\
QA_GUI_20261108_0351.PRJ", cerrar_demo:true, esperar_segundos:60}`.

**Esperado:** `TProjectForm` con el proyecto.

**Real:** `pid:29384`, `ready:true`; tras cerrar `TDEMOForm` solo queda
`THomeForm`. La línea de comandos del proceso sí contiene la ruta
(`"C:\Program Files\PVsyst8.1.8\PVsyst.exe" "C:\Users\Lenovo\PVsyst8.1_Data\
Projects/QA_GUI_20261108_0351.PRJ"`). Con la ruta relativa
(`proyecto:"QA_GUI_20261108_0351"`) tampoco. La ruta se pasa como argumento
suelto; PVsyst 8.1.8 no la abre (o el `TDEMOForm` interrumpe la carga y no se
reintenta).

**Evidencia:** `ventanas` tras el lanzamiento y tras cerrar `TDEMOForm`:
`[THomeForm, TApplication]`; nunca `TProjectForm`. La carga por GUI
(`proyecto.cargar` + `fila 0` + `OK`) sí funciona.

**Archivo/línea:** `src/adapters/gui/proceso.rs:126-145` (`iniciar_proceso`) y
`:180-205` (`lanzar`).

**Regresión:** no determinado; el informe previo no probó `lanzar` con
`proyecto` (solo `lanzar` sin proyecto).

### BUG-03 · P2 · `vista.estado:"ok"` cuando la acción no hizo nada (re-verificado)

**Tool/acción:** `gui_actuar accion:esperar_cierre` y `clic_texto`.

**Reproducción:** `{accion:"esperar_cierre", ventana:"TToolsForm",
timeout_ms:2000}` con la ventana abierta → `{cerrado:false, espera_ms:2000}` y
`vista.estado:"ok"`, `resumen:"Cierre de la ventana esperado."`.
`{accion:"clic_texto", ventana:"TWorkspaceForm", texto:"NoExiste"}` →
`coincidencias:0` y `vista.estado:"ok"` (informe previo).

**Esperado:** `vista.estado:"error"` o `parcial` con `motivo`.

**Real:** `estado_de` solo mira `ok:false`, `errores`, `error` y
`completo:false`; `cerrado:false` y `coincidencias:0` no cuentan.

**Evidencia:** `{"cerrado":false,"espera_ms":2000,"vista":{"estado":"ok",
"resumen":"Cierre de la ventana esperado."}}`.

**Archivo/línea:** `src/adapters/gui/campos.rs:199-206` (no marca error) +
`src/vista_agente/contrato/construccion.rs:120-130` (`estado_de`).

**Regresión:** **preexistente** según §8.2 del informe previo.

### BUG-16 · P3 · `controles` devuelve `count` = total con la lista truncada (re-verificado)

**Reproducción:** `{que:"controles", ventana:"THomeForm"}` (sin `detalle`) →
`count:24`, `controls_total:24`, `truncado:true`, **13** controles.

**Esperado:** `count:13` (devueltos), `total:24`.

**Real:** `count` se calcula antes del recorte genérico
(`src/adapters/gui/ventanas.rs:270`).

**Evidencia:** `{"count":24,"controls_total":24,"truncado":true,
"controls":[13]}`.

**Regresión:** **preexistente** (§8.2).

### BUG-17 · P3 · `vista` dice que no hay ventana conocida con `THomeForm` abierta (re-verificado)

**Reproducción:** `{que:"vista", ventana:"THomeForm"}` con `THomeForm` visible
→ `entidad:null`, `aviso:"ninguna ventana conocida abierta; esquemas con
gui_observar que:esquema"`.

**Esperado:** «THomeForm no tiene esquema» o leerla por posición.

**Real:** `THomeForm` no está registrada en `cobertura` y `vista` no distingue
«no registrada» de «no abierta».

**Evidencia:** `{"entidad":null,"aviso":"ninguna ventana conocida abierta; …"}`.

**Archivo/línea:** `src/vista_agente/vista.rs:134-160`.

**Regresión:** **preexistente** (§8.2).

### BUG-18 · P3 · `menus` falla a medias (NO reproducido)

**Reproducción intentada:** `{que:"menus", ventana:"THomeForm"}` en
compartido → `count:8`, `completo:true`, `error:null`, 10,7 s.

**Resultado:** **no reproducido** en esta sesión; el informe previo lo vio con
`completo:false` y «El menú 7 de THomeForm no se abrió…». Se mantiene como
riesgo conocido, no como bug confirmado hoy. En aislado `menus` se rechaza por
diseño.

### BUG-23 · P3 · `teclas` envía con `foco:false` en el escritorio compartido (re-verificado)

**Reproducción:** `{accion:"teclas", ventana:"TToolsForm", teclas:"ESC"}` →
`sent:"ESC"`, `foco:false`, `via:"keybd_event"`, `vista.estado:"ok"`; la
ventana **no** se cerró.

**Esperado (instrucciones del servidor):** en el escritorio del usuario se
niega a enviar si no puede dar el foco.

**Real:** envía igualmente con `foco:false`.

**Evidencia:** `{"sent":"ESC","foco":false,"via":"keybd_event",
"vista":{"estado":"ok"}}`; `ventanas` seguía mostrando `TToolsForm`.

**Archivo/línea:** `src/adapters/gui/teclado.rs:37-49` (`resolver_foco`).

**Regresión:** **introducido durante el PR** según §8.3 del informe previo
(`68154ff`).

### BUG-24 · P2 · `clic_fisico` ignora `objetivo` y pulsa la ventana (re-verificado; elevado de P3 a P2 por el efecto)

**Reproducción:** `{accion:"clic_fisico", ventana:"THomeForm",
objetivo:{x:919,y:565}}` (el punto es el botón «Herramientas»).

**Esperado:** clic en el punto pedido (o error si `objetivo` no aplica).

**Real:** `invoked:"THomeForm"`, `hwnd:"0xB09AE"`; se abrió **`TProjectForm`
«New.PRJ»** (el centro de `THomeForm` es el botón «Conectado a la red»). El
`objetivo` no se lee; `clic_fisico_en_control` usa el centro del control o
`dx/dy`.

**Evidencia:** `{"action":"clic_fisico","invoked":"THomeForm",
"hwnd":"0xB09AE"}` y `ventanas` → `TProjectForm` «Proyecto: New.PRJ».

**Archivo/línea:** `src/adapters/gui/acciones.rs:137-160` y `:191-210`.

**Regresión:** **preexistente** según §8.3 del informe previo; el efecto
observado hoy (abrir un proyecto nuevo no deseado) justifica subirlo de P3 a
**P2**.

---

## 4. Cobertura

### 4.1 Tabla de cobertura por acción

| Acción | Feliz | Error/borde | Estado | Notas |
|---|---|---|---|---|
| `gui_observar ventanas` | 3 | 2 (+1 sin error) | ✅ | — |
| `gui_observar controles` | 4 | 2 | ✅ | BUG-16 |
| `gui_observar leer` | 2 | 2 | ✅ | GUI-10 |
| `gui_observar capturar` | 5 | 3 | ✅ | PNG verificado |
| `gui_observar uia` | 1 (+1) | 2 | ✅ | — |
| `gui_observar arbol` | 1 | 2 | ✅ | — |
| `gui_observar formulario` | 2 | 1 | ✅ | — |
| `gui_observar vista` | 7 | 3 | ✅ | BUG-17 |
| `gui_observar esquema` | 2 | 1 | ✅ | — |
| `gui_observar mensajes` | 2 | 2 | ✅ | Aviso real de PVsyst |
| `gui_observar funciones` | 3 | 1 lógico | ✅ | Sin ruta de error real |
| `gui_observar menus` | 1 | 3 | ✅ | BUG-18 no reproducido |
| `gui_actuar click` | 4 | 4 | ✅ | GUI-06/07/08 |
| `gui_actuar clic_fisico` | 1 | 3 | ✅ | BUG-24 |
| `gui_actuar mensaje` | 1 | 1 | ✅ | — |
| `gui_actuar cerrar` | 3 | 2 | ✅ | GUI-14 |
| `gui_actuar activar` | 1 | 1 | ✅ | — |
| `gui_actuar teclas` | 2 | 3 | ✅ | BUG-23 |
| `gui_actuar fila` | 2 | 6 | ✅ | Aislado rechaza por diseño |
| `gui_actuar escribir` | 2 | 3 | ✅ | GUI-01 |
| `gui_actuar seleccionar` | 1 | 3 | ✅ | GUI-01 |
| `gui_actuar pagina` | 2 | 4 | ✅ | — |
| `gui_actuar esperar_cierre` | 2 | 2 | ✅ | BUG-03 |
| `gui_actuar lanzar` | 2 | 2 | ✅ | GUI-15 |
| `gui_actuar uia_invocar` | 6 | 4 | ✅ | GUI-02 (cuelgue) |
| `gui_actuar esperar` | 1 | 2 | ✅ | — |
| `gui_actuar paso` | 1 | 3 | ✅ | — |
| `gui_actuar funcion` | 8 | 2 | ✅ | GUI-13 |
| `gui_actuar menu` | 2 | 5 | ✅ | GUI-11 |
| `gui_actuar campo` | 2 | 3 | ✅ | — |
| `gui_actuar rellenar` | 5 | 3 | ✅ | — |
| `gui_actuar clic_texto` | 2 | 3 | ✅ | GUI-09 |
| `gui_actuar reiniciar` | 1 (guarda) | 0 | ⚠️ | Fallo real no reproducible |
| `gui_actuar cerrar_todo` | 2 | 0 | ✅ | Sin ruta de error; idempotente por código |
| `gui_actuar secuencia` | 2 | 4 | ✅ | — |
| `gui_actuar escena` | 1 (simular) | 2 | ⚠️ | Feliz real bloqueada (GUI-03/04) |
| `gui_actuar escena_leer` | 0 | 1 | ⚠️ | Feliz bloqueada (GUI-03) |
| `gui_actuar escena_importar` | 0 | 3 | ⚠️ | Feliz bloqueada (GUI-03/05) |

### 4.2 Rutas no probadas o bloqueadas (honesto)

| Ruta | Motivo | ¿Se puede desbloquear? |
|---|---|---|
| `escena` feliz (crear + exportar) | GUI-03/GUI-04: PVsyst no escribe el `.SHD`; la lista de objetos queda vacía tras `escena`. | Sí, corrigiendo la exportación/verificación; la creación manual demuestra que la UI acepta los objetos. |
| `escena_leer` feliz | GUI-03: mismo fallo de exportación. | Sí, con GUI-03. |
| `escena_importar` feliz | GUI-03: la importación ocurre (`Objeto 57`), pero la exportación/lectura posterior falla. | Sí, con GUI-03. |
| `reiniciar` con fallo real | Provocar un crash de PVsyst (madExcept) no es reproducible de forma segura y podría dañar la sesión. | No sin un entorno desechable y una copia del workspace; se probó la guarda. |
| `menus`/`menu` en aislado | Rechazo por diseño (los menús se abren con teclado y el aislado ignora teclas). | Se probaron en compartido. |
| `fila` en aislado | Rechazo por diseño (clic físico no medido en aislado). | Se probó en compartido. |
| `mensajes` con error duro | Se capturó un aviso real de PVsyst (eV/KT) al rellenar un módulo; provocar un error duro exigiría corromper un proyecto. | Se puede con un `.PRJ` corrupto desechable, pero no se hizo para no tocar datos. |
| `cerrar` sobre `TMessageForm` pregunta | La rama específica «Message … is a question» no se ejecutó; sí la confirmación de `TPerspectiveForm` (misma familia). | Reproducible abriendo el editor 3D y cerrando con cambios. |
| `controles`/`vista` con `ventana` ambigua | `resolve` es first-match por subcadena; no hay error de ambigüedad. | Es comportamiento, no ruta de error. |

**Objetivo 0 rutas sin probar:** se alcanzó en 186 de 189 rutas; las 3
restantes son las felices de `escena*`, bloqueadas por GUI-03/GUI-04 con causa
técnica documentada. `reiniciar` con fallo real queda como riesgo no
reproducible, no como ruta olvidada.

---

## 5. Recomendaciones

### P0 (antes de publicar)

Ninguna: no hay pérdida de datos del usuario ni corrupción del workspace. Las
dos P1 afectan a la precisión de la automatización, no a los archivos del
usuario (el proyecto de prueba era desechable).

### P1

1. **GUI-01 — unificar el sistema de coordenadas.** Que `campos_vivos` reste
   `origen_cliente` (no `rect_of`) o que `Formulario::control` sume el borde de
   la ventana. Añadir un test de contrato: para cada entidad, escribir con la
   clave de `vista` y comprobar que cambia **ese** campo y no otro. Mientras
   tanto, documentar que las claves de `vista` son relativas a la ventana y que
   `campo`/`seleccionar` esperan coordenadas de cliente (o aceptar ambas).
2. **GUI-02 — timeout en UIA.** `Invoke()` no debe bloquear el servidor: usar
   un hilo con timeout, `IUIAutomationElement` con `SetFocus`/`Invoke` en
   segundo plano, o rechazar `invocar` en controles que abren formularios y
   recomendar `click`/`clic_texto`. Como mínimo, un timeout de 10 s que
   devuelva error y no deje el proceso colgado.

### P2

3. **GUI-03 — diagnosticar la exportación de escena.** El diálogo
   `TSaveObjPVForm` se cierra sin escribir; comprobar si PVsyst exige que la
   escena esté «definida» en `TShadingsForm` antes de exportar, si el nombre
   debe ir sin `_mcp_`, o si el `Directorio` debe seleccionarse explícitamente.
   Mientras tanto, `escena*` debe fallar rápido con la causa real, no con «no
   other way is attempted» tras 22–41 s.
4. **GUI-04 — verificar el objeto creado.** Tras `objeto::crear`, abrir
   `TShdObjectSelectionForm` y comprobar que el número de filas aumentó; si no,
   error con el nombre del objeto.
5. **GUI-05 — resolver `escena_importar` contra el workspace.** Usar
   `resolver_ruta(&paths.workspace, …)` como el resto de la tool, no
   `std::path::absolute`.
6. **GUI-06 — validar `texto`/`id` en `click`.** O implementarlos (delegando en
   `clic_texto`/`resolve`), o rechazarlos con un error que nombre `clic_texto`
   y `ventana`. Hoy se ignoran y la respuesta dice `ok`.
7. **GUI-15 — abrir el proyecto al lanzar.** Verificar el argumento que acepta
   PVsyst 8.1.8 (¿`/PROJECT:`? ¿`.`? ¿ruta con `\`?), o no prometer
   `proyecto` en `lanzar` y documentar `proyecto.cargar` + `fila` + `OK`.
8. **BUG-03 — que `vista` no diga `ok` con `cerrado:false`/`coincidencias:0`.**
   Añadir `ok: cerrado` (o `errores:["no cerró"]`) en `esperar_cierre`, y
   `ok: coincidencias > 0` en `clic_texto`; el decorador ya sabe leer `ok`.
9. **BUG-24 — honrar o rechazar `objetivo` en `clic_fisico`.** Si se pasa
   `objetivo`, usarlo como punto; si no, rechazarlo por nombre. Hoy un clic
   «al botón Herramientas» abrió un proyecto nuevo.

### P3

10. **BUG-16** — `count` debe ser la lista devuelta; `total` el total.
11. **BUG-17** — distinguir «ventana no registrada» de «no abierta».
12. **BUG-23** — en compartido, negarse a enviar teclas sin foco (como dice la
    descripción) o marcar `vista.estado:"parcial"`.
13. **GUI-07** — `click` sobre control `enabled:false` debe ser error.
14. **GUI-08** — resolver primero los hijos del modal y exigir coincidencia
    exacta de título para controles; «No» no debe casar con «cercanos».
15. **GUI-09** — si `indice` no es válido, error; no caer al fallback UIA.
16. **GUI-10** — `leer` debe usar `WM_GETTEXT` para `TEdit`/`TMemo` (o
    documentar que solo lee títulos).
17. **GUI-11/GUI-12** — alinear `menus`/`menu` (7 vs 0..6) y devolver textos.
18. **GUI-13** — elegir como bloqueante el primer modal de PVsyst (clase
    `*Form`) y no una ventana UAC.
19. **GUI-14** — nombrar el `TMessageForm` y sus botones en el error de
    `cerrar`.

---

## 6. Anexo

### 6.1 Entorno

| Elemento | Valor |
|---|---|
| SO | Windows 11 (26H2, build 26300.9818) |
| PVsyst | 8.1.8 (`C:\Program Files\PVsyst8.1.8\PVsyst.exe`, `PVsystCLI.exe`) |
| Workspace real | `C:\Users\Lenovo\PVsyst8.1_Data` (385 MB) |
| Workspace de pruebas | el real, solo con `QA_GUI_*`; no se usó la copia `_MCP_TEST` |
| MCP | `C:\pvsyst-mcp\pvsyst-mcp.exe`, 5 015 552 B, 2026-11-07 21:31 |
| SHA-256 | `501ec8ffabc020fb720bb58b16f9f3ff362dd97fb6a8268b2977e68bffa8abc0` |
| Código | worktree `.claude/worktrees/tarea-d-fila`, HEAD `5dbfdd0` |
| `PVSYST_MCP_ESCRITORIO` | `aislado` (fase 1) y `compartido` (fase 2) |
| `PVSYST_WORKSPACE` | `C:\Users\Lenovo\PVsyst8.1_Data` |
| `PVSYST_GUI` / `PVSYST_CLI` | `C:\Program Files\PVsyst8.1.8\PVsyst.exe` / `…\PVsystCLI.exe` |
| TRIAL | **0 ejecuciones**; no se llamó a `pvsyst_cli` ni a `simular` |
| Proyecto desechable | `QA_GUI_20261108_0351` (creado 03:51, borrado 05:02) |
| Malla de prueba | `QA_GUI_malla.pvc` (copia de `SHD_PVC_Fixed_Example.pvc`, borrada) |

### 6.2 Arnés

El arnés vive fuera del repositorio, en
`C:\Users\Lenovo\AppData\Local\Temp\qa_gui_2026\`:

- `qa_server.py`: lanza **un** `pvsyst-mcp.exe` por stdio y lo mantiene vivo;
  acepta peticiones JSON por TCP (`127.0.0.1:9411`–`9417`). Así el escritorio
  aislado y PVsyst sobreviven entre lotes.
- `qa.py`: cliente de una llamada.
- `batch.py`: ejecuta listas de pasos y guarda `results/<id>.json`.
- `calls.jsonl`: 459 llamadas con petición, respuesta y duración.
- `escena_concurrente.py`: `escena` en un MCP mientras otro cierra el
  `TDEMOForm` periódico.

Comandos representativos:

```bash
# servidor aislado
QA_ESCRITORIO=aislado PVSYST_WORKSPACE="C:\Users\Lenovo\PVsyst8.1_Data" \
  python qa_server.py 9411

# servidor compartido (solo fase 2)
QA_ESCRITORIO=compartido python qa_server.py 9414

# una llamada
python qa.py 9411 '{"tool":"gui_observar","args":{"que":"ventanas"}}'
```

No se usó el cliente `mcp__pvsyst__*` para estas pruebas: todo pasó por el
mismo binario y el mismo entorno, vía stdio, para poder registrar cada
respuesta.

### 6.3 Artefactos y limpieza

Creados y **eliminados**:

| Artefacto | Ruta | Estado |
|---|---|---|
| Proyecto QA | `…\Projects\QA_GUI_20261108_0351.PRJ` | borrado |
| Variante QA | `…\Projects\QA_GUI_20261108_0351.VC0` | borrado |
| Captura recorte | `…\QA_GUI_captura_recorte.png` | borrado |
| Captura botón | `…\QA_GUI_captura_boton.png` | borrado |
| Malla | `…\QA_GUI_malla.pvc` | borrado |
| Escenas | `…\Shadings\_mcp_*.SHD` | no se llegaron a escribir (GUI-03) |

Verificación final:

```text
find C:\Users\Lenovo\PVsyst8.1_Data -iname "*QA_GUI*" -o -iname "_mcp_*"
  → (sin resultados)
tasklist | findstr PVsyst.exe
  → (sin resultados)
```

`PVsyst.ini` no contiene `QA_GUI`; PVsyst quedó cerrado y no se dejó ninguna
ventana abierta. El escritorio compartido se usó solo en la fase 2 y se cerró
todo con `cerrar_todo` (4 ventanas, `quedan:[]`).

### 6.4 Diferencias con el informe previo

| Punto | Informe 2026-11-07 | Esta sesión |
|---|---|---|
| `fila` feliz | bloqueada por fixture | **probada** en compartido (2 controles) |
| `escribir`/`seleccionar` feliz | solo validación | **probadas**; se descubrió GUI-01 |
| `pagina` feliz | solo validación | **probada** en `TPVModuleForm` y `TPreferencesForm` |
| `campo`/`rellenar` orientation/system | bloqueadas | **probadas** con `_DEMO_Commercial` |
| `escena*` feliz | bloqueada por «no project» | **bloqueada por exportación** (GUI-03/04); importación sí ocurre |
| `click` con `texto` | se creyó «clic enviado» | **GUI-06**: `texto` se ignora |
| `clic_fisico` | BUG-24 P3 | **P2**: abrió un proyecto nuevo no deseado |
| `menus` | BUG-18 parcial | **no reproducido**; completo en 10,7 s |
| `uia_invocar` | sin incidencias | **GUI-02**: cuelgue del servidor |
| `lanzar proyecto` | no probado | **GUI-15**: no abre el proyecto |

---

## 7. Anexo: errores de la sesión completa (27 bugs del informe previo)

> **Procedencia:** estos 27 hallazgos **no son de esta sesión**. Los encontró la
> sesión del hilo
> `mcp:launch:75e27b4341bb67b8928259f41bbbfac7f207b6316ef7d3a98e9e1a209bc11317`
> y están documentados en `docs/INFORME_PRUEBAS_MCP_2026-11-07.md` (1 528
> líneas; 2 × P1, 11 × P2, 14 × P3). Se reproducen aquí solo como **índice de
> procedencia y estado**, no como bugs nuevos de la campaña GUI. No se modificó
> el informe de la otra sesión ni el código.
>
> **Cómo leer la columna «Esta sesión»:** «re-verificado» significa que repetí
> la reproducción con el binario de esta campaña y volví a ver el defecto;
> «ampliado» que además encontré una variante o un efecto nuevo; «no
> reproducido» que la reproducción ya no falló; «no re-verificado» que la ruta
> queda fuera del alcance GUI de esta sesión (datos, CLI, workspace,
> componentes, resultados) y su estado es el que fija el informe previo. Los
> que ya están en la matriz GUI no se duplican: se referencian por ID a §2.1,
> §2.2 y §3.

| ID | Sev. | Tool/acción | Reproducción (1 línea) | ¿GUI? | Esta sesión |
|---|---|---|---|---|---|
| BUG-01 | P1 | `resultados perdidas/mensual/entregable` | `resultados que:perdidas` sobre `QA_Sim` da `kwp:400.0` donde el array tiene 506,88 kWp; el entregable publica 2 178,1 kWh/kWp en vez de 1 719,0 | No | No re-verificado (fuera de alcance GUI) |
| BUG-02 | P1 | `simular ejecutar` (y `disenar_sistema`) | `simular ejecutar` sobre un proyecto con módulo sin dimensiones → `clasificacion:"ambiguo"`, `exit_code:0`, aunque `logs_cli` tiene `Instruction privilégiée` y `nb_crash:2` | No | No re-verificado (fuera de alcance GUI) |
| BUG-03 | P2 | `gui_actuar esperar_cierre` / `clic_texto` | `esperar_cierre {ventana:"THomeForm",timeout_ms:1200}` → `cerrado:false` con `vista.estado:"ok"`; `clic_texto` con 0 coincidencias también decía `ok` | **Sí** | **Re-verificado** en `esperar_cierre` (§2.2, §3): `cerrado:false` + `vista.estado:"ok"`. La variante `clic_texto` con `coincidencias:0` no se re-ejecutó en el mismo caso; mi prueba de texto no dibujado devolvió error (no `ok`), y se amplió con **GUI-09** (índice fuera de rango) |
| BUG-04 | P2 | `disenar_sistema` | `disenar_sistema` con un módulo inválido → `pasos[4]` (simulación) `ok:false` con `detail:null`; el error de PVsyst se pierde | No | No re-verificado (fuera de alcance GUI) |
| BUG-05 | P2 | `proyecto p50_p90` (ruta GUI) | `proyecto p50_p90 {proyecto:"QA_Sim",variante:"VC0"}` → no abre `TProjectForm` o no guarda; la ventana queda en modo «PRUEBA» | **Sí** | No re-verificado: la ruta `p50_p90` no se ejercitó en esta campaña (solo GUI de ventanas/controles) |
| BUG-06 | P3 | `disenar_sistema` | Repetir la misma llamada: el estado documentado `reutilizado:true` + `comparable:false` no se alcanza; siempre da `true`/`null` | No | No re-verificado (fuera de alcance GUI) |
| BUG-07 | P2 | `disenar_sistema` / `componente` | `disenar_sistema {modulo:"MCP_Test_400"}` falla por falta de `.PAN`, mientras `componente editar` acepta el nombre sin extensión | No | No re-verificado (fuera de alcance GUI) |
| BUG-08 | P2 | `disenar_sistema` | El error recomienda `via:"aproximacion"`, pero `tools/list` solo declara `via:{"enum":["pvsyst"]}` | No | No re-verificado (fuera de alcance GUI) |
| BUG-09 | P2 | `entorno exportar_logs`, `sitio convertir_meteo`, `entorno licencia_sync` | `exportar_logs` a una carpeta inexistente imprime `… does not exist. Please set another target folder.` y se clasifica como `ambiguo` | No | No re-verificado (fuera de alcance GUI) |
| BUG-10 | P2 | `workspace exportar/importar` | `exportar {destino:"QA_tmp/small.zip"}` escribe dentro del workspace exportado; `importar {zip:"QA_tmp/small.zip"}` lo busca en otra base y da `Zip not found` | No | No re-verificado (fuera de alcance GUI) |
| BUG-11 | P2 | `editar accion:campos` | `editar {accion:"campos", campos:{NoExiste:1}}` inserta el campo desconocido con `action:"inserted"` y `estado:"ok"` | No | No re-verificado (fuera de alcance GUI) |
| BUG-12 | P2 | `entorno licencia`, `costo_estimado.restantes` | El contador TRIAL oscila entre 133/134 y 249/250 sin activación de licencia | No | No re-verificado (fuera de alcance GUI) |
| BUG-13 | P2 | `leer` | `leer {archivo:"Projects/_DEMO_Commercial.VC0"}` sugiere `completo:true`, parámetro que `leer` no declara (el suyo es `detalle`) | No | No re-verificado (fuera de alcance GUI) |
| BUG-14 | P2 | `escribir_archivo` | Escribir dos veces el mismo archivo sin `sobrescribir` lo sobrescribe; `sobrescribir:true` no cambia nada | No | No re-verificado (fuera de alcance GUI) |
| BUG-15 | P3 | `componente accion:crear` | `componente crear` sin `cambios` → `Missing: cambios`, aunque la descripción dice «create from a template» | No | No re-verificado (fuera de alcance GUI) |
| BUG-16 | P3 | `gui_observar que:controles` | `controles` sin `detalle` → `count:31`, `controls_total:31`, `truncado:true` y 13 elementos | **Sí** | **Re-verificado** (§2.1, §3): `count:24`, `controls_total:24`, 13 controles |
| BUG-17 | P3 | `gui_observar que:vista` | Con `THomeForm` visible, `vista` → `entidad:null`, `aviso:"ninguna ventana conocida abierta…"` | **Sí** | **Re-verificado** (§2.1, §3): mismo aviso con `THomeForm` abierta |
| BUG-18 | P3 | `gui_observar que:menus` | `menus {ventana:"THomeForm"}` → `completo:false`, error «El menú 7… no se abrió», 11,7 s | **Sí** | **No reproducido** (§2.1): `count:8`, `completo:true`, `error:null`, 10,7 s. Se mantiene como riesgo, no como bug confirmado hoy |
| BUG-19 | P3 | `sitio escribir_horizonte` | `escribir_horizonte {archivo:"Sites/QA_Horizonte.HOR"}` crea `Shadings/Sites/QA_Horizonte.HOR` | No | No re-verificado (fuera de alcance GUI) |
| BUG-20 | P3 | `sitio crear`, `disenar_sistema` | `sitio crear {dry_run:true}` devuelve `command_line` sin `costo_estimado`; `disenar_sistema` en frío da `restantes:null` | No | No re-verificado (fuera de alcance GUI) |
| BUG-21 | P3 | `leer`, `proyecto crear`, `resultados comparar`, `editar insertar_objeto` | `leer {archivo:"Projects/NO_EXISTE.PRJ"}` dice «no hay ninguno en …\Projects» aunque hay 16 proyectos | No | No re-verificado (fuera de alcance GUI) |
| BUG-22 | P3 | `simular accion:preparar` | `preparar` devuelve `batch_params_file`/`batch_rvt_file:null` y un `siguiente` que lanza una simulación simple, perdiendo el batch | No | No re-verificado (fuera de alcance GUI) |
| BUG-23 | P3 | `gui_actuar accion:teclas` | `teclas {ventana:"TWorkspaceForm",teclas:"ESC"}` → `foco:false`, `via:"keybd_event"`, `vista.estado:"ok"`; el diálogo no se cierra | **Sí** | **Re-verificado** (§2.2, §3): `teclas ESC` sobre `TToolsForm` → `foco:false`, `vista.estado:"ok"`, `TToolsForm` sigue abierta |
| BUG-24 | P3 → **P2** | `gui_actuar accion:clic_fisico` | `clic_fisico {ventana:"TWorkspaceForm",objetivo:{x:1253,y:1146}}` → `invoked:"TWorkspaceForm"`; el botón de esa posición no recibe el clic | **Sí** | **Re-verificado y ampliado** (§2.2, §3): con `objetivo:{x:919,y:565}` (botón «Herramientas») pulsó el centro de `THomeForm` y abrió `TProjectForm` «New.PRJ». Se eleva de P3 a **P2** por el efecto no deseado |
| BUG-25 | P3 | `componente accion:editar` | `componente editar {campos:{PNom:999}}` aplica el cambio aunque `campos` no está declarado en `properties` | No | No re-verificado (fuera de alcance GUI) |
| BUG-26 | P3 | `sitio accion:crear` (`origen:"csv"`) | `sitio crear {origen:"csv"}` devuelve `expected_sit:null` (con `origen:"coordenadas"` sí trae la ruta) | No | No re-verificado (fuera de alcance GUI) |
| BUG-27 | P3 | `entorno accion:preferencias` | `entorno {accion:"preferencias"}` devuelve `secciones` con 24 entradas pero `vista.campos:[]` | No | No re-verificado (fuera de alcance GUI) |

### 7.1 Balance de los 27

| Estado en esta sesión | Bugs | Detalle |
|---|---|---|
| **GUI re-verificados** | **5** | BUG-03 (parcial), BUG-16, BUG-17, BUG-23, BUG-24 (ampliado y elevado a P2) |
| **GUI no reproducido** | **1** | BUG-18: `menus` terminó completo (`completo:true`) en 10,7 s |
| **GUI no re-verificado** | **1** | BUG-05: la ruta `proyecto p50_p90` no se ejercitó en esta campaña |
| **No GUI (fuera de alcance)** | **20** | BUG-01, 02, 04, 06–15, 19–22, 25–27: datos, CLI, workspace, componentes, resultados, `entorno`; su estado es el del informe previo |
| **Total** | **27** | 2 P1, 11 P2, 14 P3 en el informe de origen |

**Relación con los bugs de esta campaña.** Los **15 bugs nuevos** de §3
(GUI-01…GUI-15) no están en esta lista: son hallazgos de esta sesión. De los 27
de la lista, esta campaña **re-verificó 5** (BUG-03, 16, 17, 23, 24),
**no reprodujo 1** (BUG-18) y dejó **21 sin re-verificar** (20 no GUI +
BUG-05). Los 27 siguen siendo de la otra sesión y quedan como están en
`docs/INFORME_PRUEBAS_MCP_2026-11-07.md`. No se copian sus reproducciones
completas ni su análisis de regresión (§8 de aquel informe); esta sección es
solo el índice de procedencia y el estado en la campaña GUI.

---

*Fin del informe. No se modificó `src/`, no se hizo ningún commit y el PR #2
quedó intacto.*
