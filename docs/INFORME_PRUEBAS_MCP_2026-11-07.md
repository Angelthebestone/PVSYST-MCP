# Informe de pruebas del servidor MCP `pvsyst` (PVsyst 8.1.8)

> **Fecha real de la sesión:** 2026-11-08, 01:38–03:10 (hora local).
> El nombre del archivo conserva la fecha pedida en el encargo (`2026-11-07`);
> el trabajo se ejecutó en la madrugada del 8.
>
> **Alcance:** QA de caja negra del binario instalado. No se modificó `src/`,
> no se hizo ningún commit y el PR #2 no se tocó.
>
> **Binario probado:** `C:\pvsyst-mcp\pvsyst-mcp.exe`
> `sha256 = 501ec8ffabc020fb720bb58b16f9f3ff362dd97fb6a8268b2977e68bffa8abc0`
> (5 015 552 bytes, 2026-11-07 21:31). Código correspondiente: worktree
> `.claude/worktrees/tarea-d-fila`, rama `worktree-tarea-d-fila`, HEAD `5dbfdd0`.
>
> **Verificación de binario nuevo:** la descripción de `disenar_sistema` que
> devuelve `tools/list` contiene `proyecto_comparable` (y la frase «A project an
> earlier call left behind with the same parameters is reused instead of
> rebuilt»). ✔ Es el binario del PR #2.

---

## 1. Resumen ejecutivo

| Métrica | Valor |
|---|---|
| Tools ejercitadas | **15 / 15** |
| Acciones canónicas probadas | **108 / 108** (105 acciones de discriminador + 3 tools de acción única; `pvsyst_cli` excluida) |
| Llamadas registradas | **351** (273 por arnés stdio + 78 por el cliente MCP) |
| Respuestas con `isError` | **153** (122 + 31), casi todas casos de error/validación deliberados |
| Respuestas sin `isError` | **198** |
| Resultado `ok` (ruta feliz o validación correcta) | **185** |
| Resultado `bug` | **13 llamadas** sin `isError` que devolvieron un resultado incorrecto |
| Resultado `bloqueada` | **9** (5 por fixture, 3 por gate de aproximaciones, 1 por presupuesto) |
| Bugs encontrados | **27** (2 × P1, 11 × P2, 14 × P3) — 0 × P0 |
| Antigüedad | **22 existían antes del PR #2 (en `main`), 3 los introduce el PR, 2 son de funcionalidad nueva** (§8.3) |
| Ejecuciones TRIAL gastadas | **≈11** (por encima del límite blando de 10; se reporta en §7) |
| Duración | ≈ 3 h 30 min (01:38–03:10), incluidas preparación, prueba diferencial (§8) y limpieza |
| Workspace de pruebas | `C:\Users\Lenovo\PVsyst8.1_Data_MCP_TEST` (copia de 395 MB, borrada al final) |
| Estado final | Workspace real sin artefactos `QA_*`; `PVsyst.ini` restaurado byte a byte; PVsyst cerrado |

**Lo que funciona bien.** El núcleo de datos es sólido: `leer` detecta y resume
correctamente `.PRJ`, `.VC0`, `.PAN`, `.OND`, `.SIT`, `.MET` (incluida la serie
horaria), `.RVT`, objetos sueltos y bytes crudos; `listar` con filtros y
truncado informado; `editar` preserva bytes, versiona en historial y verifica
tras escribir; `proyecto` cubre el ciclo completo (crear, variantes, doctor,
10 elementos de `anadir`, seguidor, límite de inyección, pérdidas, configurar
por pasos); `resultados` extrae KPIs, mensual, pérdidas, P50/P90, carbono y
economía del PDF de PVsyst; `workspace` exporta/importa/restaura; y la GUI es
automatizable de verdad (se abrieron y cerraron formularios, se invocó un
botón por UI Automation, se capturó PNG, se leyó OCR y se ejecutó una función
del catálogo).

**Lo que no funciona.** Tres áreas concentran los fallos:

1. **Exactitud de un KPI** (P1): `resultados perdidas|mensual|entregable`
   calculan el kWp con `PNomSizing` en vez de con la potencia real del array.
   En el proyecto de prueba eso dio **2 178,1 kWh/kWp** donde PVsyst dice
   **1 719,0**.
2. **Clasificación de fallos del CLI** (P1/P2): un *crash* de PVsystCLI
   (`Instruction privilégiée`) y errores de texto claro («Invalid date Line 6.»,
   «does not exist. Please set another target folder.») se devuelven como
   `ambiguo` con `exit_code:0`.
3. **Contrato `vista`** (P2): `esperar_cierre` y `clic_texto` devuelven
   `vista.estado:"ok"` cuando el campo real dice `cerrado:false` y
   `coincidencias:0`. Es exactamente el fallo que el contrato `vista` existía
   para evitar.

---

## 2. Matriz tool × acción

Convenciones de la columna **Resultado**:

- `ok` — la acción hizo lo que dice y el efecto se verificó.
- `ok (validación)` — se ejercitó la ruta real de validación/error (argumento
  faltante, valor inválido, ruta fuera del workspace, id inexistente) y el
  rechazo fue correcto; la ruta feliz no era segura o necesitaba un fixture que
  no existe.
- `bug` — comportamiento incorrecto (ver §3).
- `bloqueada` — no se pudo ejecutar; el motivo se indica.

Las llamadas se abrevian; el JSON íntegro de las 351 llamadas está en
`C:\Users\Lenovo\Desktop\PVSYST MCP\.qa\out*.jsonl` (formato JSONL: `label`,
`tool`, `args`, `wall_ms`, `response`).

### 2.1 `entorno` (9/9)

| Acción | Llamada | Resultado | Evidencia (recortada) | Notas |
|---|---|---|---|---|
| `diagnostico` | `{accion:"diagnostico"}` | ok | `entorno{cli_exists:true,gui_exists:true,workspace:"…PVsyst8.1_Data",herramientas:15}`, `preflight.checks[6]` | 23 ms. `cli_license` sale `ok:false` (modo TRIAL), correcto. |
| `licencia` | `{accion:"licencia"}` | ok | `license:"Status: TRIAL"`, `stdout_tail:"Remaining days: 30\nRemaining executions: 134"` | 6,4 s (llama a `PVsystCLI lic-info`). Ver BUG-12. |
| `licencia_sync` | `{accion:"licencia_sync"}` | ok (validación) | `vista.estado:"error"`, `error.codigo:"fallo_cli"`, `motivo:"ambiguo: Salida sin confirmación de éxito ni marcador de error concluyente"` | 4,4 s. La sincronización no confirma nada; el MCP lo dice. Aceptable, pero cae en el mismo saco «ambiguo» (BUG-09). |
| `licencia_activar` | `{accion:"licencia_activar"}` y `{…,clave:"XXXX-0000"}` | ok | `{ejecutado:false, comando:"PVsystCLI.exe lic-activate -k:<TU_CLAVE>"}` | **No ejecuta nada**: devuelve el comando para que lo corra el humano. Seguro. `clave` se ignora en silencio (no aparece en `ignorados`). |
| `licencia_desactivar` | `{accion:"licencia_desactivar"}` | ok | `{ejecutado:false, comando:"PVsystCLI.exe lic-deactivate -c:<TU_CUSTOMER_ID>"}` | Igual: no toca la licencia real. |
| `exportar_logs` | `{accion:"exportar_logs",carpeta_destino:"…/QA_tmp_logs"}` | ok | `clasificacion:"exito"`, `stdout_tail:"…logs.zip"` → `2026_11_08-01_44_56-PVsystCLI818-logs.zip` | 4,5 s. **Requiere que la carpeta exista**: si no, PVsyst dice «does not exist» y el MCP lo clasifica `ambiguo` (BUG-09). Ruta fuera del workspace → rechazo correcto. |
| `logs_cli` | `{accion:"logs_cli",n:5}` / `{n:3,solo_errores:true}` | ok | `logs[].lineas_no_norm` con `EPrivilege\|run-simulation\|Instruction privilégiée`; `sesion.comandos{run-simulation:{count:4,nb_crash:2}}` | 17 ms. Excelente fuente de diagnóstico; el MCP **no** la usa al clasificar (BUG-02). `n:0` → `Value 0 out of range (minimum 1)`. |
| `preferencias` | `{accion:"preferencias"}` / `{…,seccion:"Preferences"}` | ok | `archivo:"…\AppData\Local\PVsyst\8.1\Admin\PVsyst.ini"`, 24 secciones; con `seccion` devuelve el bloque entero | 7 ms. El INI es **UTF-16 LE con BOM**. `vista.campos` sale vacío (BUG-27). Sección inexistente → `Sección 'NoExiste' no existe en PVsyst.ini`. |
| `preferencias_editar` | `{accion:"preferencias_editar",seccion:"Preferences",clave:"Dialog BeepSound",valor:"0"}` y luego `"1"` | ok | `{ok:true, historial_id:"20261108_064436_786_0000"}`; INI sigue en UTF-16 (`bytes 12878 bom b'\xff\xfe'`) y `Dialog BeepSound=0` → `=1` | Con backup previo y restauración posterior. Clave inexistente y sección inexistente → error claro. |

### 2.2 `disenar_sistema` (1/1, 8 escenarios)

| Escenario | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `dry_run` red | `{lat:46.2044,lon:6.1432,kwp:10,modulo:"QA_PAN_400.PAN",inversor:"MCP_Inv_10k.OND",n_mod_serie:22,n_strings:1,dry_run:true}` | ok | `costo_estimado{ejecuciones:1,restantes:249}`, `plan:["simular Sistema_10kWp VC0 (PVsystCLI run-simulation)"]`, `sitio_reutilizado:true` | 13 ms. En frío (`restantes:null`) ver BUG-20. |
| `dry_run` aislado | `{…,tipo_sistema:"aislado",n_mod_serie:22,n_strings:1,dry_run:true}` | ok | `plan:["simular Sistema_10kWp VC3 …"]` | Elige VC3 (variante stand-alone de la plantilla). |
| Real red | `{…,modulo:"QA_PAN_400.PAN",inversor:"MCP_Inv_10k.OND",n_mod_serie:22,n_strings:1,nombre:"QA_Diseno4"}` | **bloqueada por fixture** | `pasos[]`: sitio ok, proyecto ok, orientación ok, dimensionado ok (`kwp_realizado:8.8`); `pasos[4].ok:false`, `errores:["paso 'simulacion' fallido: PVsyst … Meteonorm … failed to open encrypted station db: DataMN9/station.db.enc", "The sizes of this module are not defined"]` | Sitio+proyecto+dimensionado funcionan. La simulación no puede correr en esta máquina: (a) Meteonorm 9 no encuentra `DataMN9/station.db.enc`, (b) los `.PAN` del workspace no tienen dimensiones válidas para PVsyst. Es la misma limitación que ya documenta `docs/vista_agente_salidas.md`. |
| Real aislado | `{…,tipo_sistema:"aislado",n_mod_serie:22,n_strings:1,n_baterias_serie:1,n_baterias_paralelo:2,nombre:"QA_Diseno6",simular:false}` | ok | `ok:true`, `tipo_sistema:"aislado"`, `kwp_realizado:8.4`, `avisos:["kWp realizado 8.4 se desvía -16.0 % del objetivo 10.0"]` | Sin `simular:false` habría fallado por Meteonorm. |
| Reutilización | la misma llamada dos veces con `simular:false` | ok | 1.ª y 2.ª: `proyecto_reutilizado:true`, `proyecto_comparable:true`, `proyecto_creado:"QA_Diseno4"` | La reutilización se observa. |
| Sin `n_mod_serie`/`n_strings` | `{lat,lon,kwp:500}` | ok (validación) | `Sin 'n_mod_serie' y 'n_strings' el dimensionado no es fiable (red: lo haría la GUI de PVsyst; aislado: no hay pre-dimensionado): pasa ambos o usa via:'aproximacion'` | Rechazo **antes** de gastar. Pero `via:'aproximacion'` no está en el enum del esquema (BUG-08) y la descripción de la tool no menciona que n's son obligatorios (BUG-08). |
| `tipo_sistema:aislado` sin n's | `{lat,lon,kwp:50,tipo_sistema:"aislado"}` | ok (validación) | mismo mensaje | Igual. |
| Sin módulo/inversor | `{lat,lon,kwp:10,n_mod_serie:22,n_strings:1,nombre:"QA_Diseno5",simular:false}` | ok | `proyecto_reutilizado:false`, `proyecto_comparable:null`; repetida → `reutilizado:true`, `comparable:true` | `null` en proyecto nuevo es deliberado; `comparable:false` es inalcanzable (BUG-06, P3). |

### 2.3 `listar` (5/5)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `proyectos` | `{que:"proyectos"}` / `{…,detalle:"normal"}` / `{…,texto:"DEMO"}` | ok | `count:16`, 13 elementos + `vista.recortado[{clave:"projects",mostrados:13,total:16,pedir_resto:"detalle:completo"}]`; con `detalle:normal` → 16 | El truncado se informa. `pedir_resto` sugiere `completo` aunque `normal` (50) bastaba (P3, §4). |
| `componentes` | `{que:"componentes",tipo:"PAN"}` / `OND` / `BTR` / `inversor` (alias) | ok | PAN: 2 archivos; OND: `Generic_50kW.OND`, `MCP_Inv_10k.OND`; BTR: `count:0` con `aviso` de carpeta ausente | Alias en español/inglés funcionan. `tipo:"ZZZ"` → `Unknown component kind 'ZZZ'. Known: PAN, OND, BTR, PMP, RLT, RLP, GEN, REV, CLI`. Sin `tipo` → `Missing: tipo`. |
| `modulos` | `{que:"modulos"}` / `{…,texto:"400",pnom_min:300,pnom_max:500}` | ok | `count:2` → `[{model:"MCP Test 400",pnom_w:400.0},{model:"Z_E2E_Ficha",pnom_w:410.0}]`; con filtros `count:1` | 9 ms. |
| `archivos` | `{que:"archivos",carpeta:"Projects",extension:".PRJ"}` / `{…,carpeta:"Sites",recursivo:true}` | ok | `count:16`, `files[].relative/size_bytes/modified_unix` | `carpeta:"C:/Windows"` → `Ruta fuera del workspace`. |
| `objetos` | `{que:"objetos",archivo:"Projects/_DEMO_Commercial.PRJ"}` / `.VC0` | ok | PRJ: `count:2`; VC0: `count:25` | Sin `archivo` → `Missing: archivo`. `file` sale con separadores mixtos (`Projects/_DEMO_Commercial.PRJ`), cosmético. |

### 2.4 `leer` (1/1, 12 escenarios)

| Tipo | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `.PRJ` | `{archivo:"Projects/_DEMO_Commercial.PRJ"}` | ok | `summary{type:"pvProject",version:"8.1.0",site:"Oakland CA",latitude:"37.8029",meteo_file:"Oakland CA_MN90_SYN.MET"}` | 1 ms, 1 746 B. |
| `.VC0` | `{archivo:"Projects/_DEMO_Commercial.VC0"}` | ok | `size_bytes:1499760`, `summary{pv_module:"Generic_Mono_440W_Half.PAN",inverter:"Generic_60kW.OND"}` | 23 ms para 1,5 MB (solo cabecera). |
| `.PAN` | `{archivo:"ComposPV/PVmodules/MCP_Test_400.PAN"}` | ok | `pnom:400.0`, `parameters{isc:10.5,voc:49.0,vmpp:40.4,…}`, `tree{…}` | Detecta el tipo por contenido. |
| `.OND` | `{archivo:"ComposPV/Inverters/MCP_Inv_10k.OND"}` | ok | `is_text:true`, árbol PVObject | |
| `.SIT` | `{archivo:"Sites/Barrancabermeja.SIT"}` | ok | `Latitude:7.0708`, `monthly{GlobalH:[…,1881.6],TAmb:[…]}` | Aplana el objeto y añade la tabla mensual. |
| `.MET` | `{archivo:"Meteo/Barrancabermeja__SYN.MET"}` | ok | `Comment:"…NASA SSE 6.0…"`, `DateFormat`, `HourShift` | |
| `.MET` + `serie` | `{…,serie:true,limite:3}` | ok | `horas[{dia:"0101",hora:0,valores:{GlobHor:0.9978,DiffHor:0.0,T_Amb:19.72}}]`, `total_horas:8760`, `unidades` | Decodifica el cuerpo horario. |
| `.RVT` | `{archivo:"UserBatch/Z_Plantilla.RVT"}` | ok | `{is_text:true,text:"  ResultVars=GlobHor\n"}` | |
| objeto | `{archivo:"…VC0",objeto:"pvSystem"}` | ok | `objects:[{_type:"pvSystem",SystemType:"Grid",Flags:"$00"}]` | |
| bytes | `{archivo:"…MCP_Test_400.PAN",base64:true,max_bytes:64}` | ok | `base64:"77u/UFZPYmplY3Rf…"`, `range:[0,64]`, `next_offset:64`, `truncated:true` | |
| inexistente | `{archivo:"Projects/NO_EXISTE.PRJ"}` | ok (validación) | `Proyecto 'Projects/NO_EXISTE.PRJ' no encontrado; no hay ninguno en …\Projects` | Mensaje engañoso: sí hay 16 (BUG-21). |
| fuera del workspace | `{archivo:"C:/Windows/win.ini"}` | ok (validación) | `Ruta fuera del workspace: 'C:/Windows/win.ini'` | |

### 2.5 `editar` (5/5)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `campos` (archivo) | `{accion:"campos",archivo:"Sites/Barrancabermeja.SIT",objeto:"pvSite",campos:{Comment:"QA editado"}}` | ok | `changes:[{action:"replaced",key:"Comment",old:"Barrancabermeja;Colombia;South_America;",new:"QA editado"}]` | 34 ms. |
| `campos` (proyecto+variante) | `{accion:"campos",proyecto:"QA_Test",variante:"VC1",ediciones:[{objeto:"pvOrient",campos:{FieldTilt:25}}]}` | ok | `edits:[{objeto:"pvOrient",changes:[{action:"replaced",key:"FieldTilt",old:"15.0",new:"25"}]}]` | Verificado con `leer` e historial. |
| `campos` campo inexistente | `{…,campos:{NoExiste:1}}` | **bug** | `changes:[{action:"inserted",key:"NoExiste",new:"1",old:null}]` | Inserta el campo en vez de rechazarlo (BUG-11). |
| `insertar_objeto` | `{accion:"insertar_objeto",archivo:"Projects/QA_Test.VC1",padre:"pvSubArray",desde:{archivo:"Projects/QA_Test.VC0",objeto:"pvShading"}}` | ok | `{insertado:"pvShading",padre:"pvSubArray",historial_id:"…"}` | `desde` es `{archivo,objeto}`, no un objeto en línea. Si falta `desde.archivo` el error dice `Missing required argument 'archivo'` sin aclarar que es `desde.archivo` (P3). |
| `eliminar_objeto` | `{accion:"eliminar_objeto",archivo:"Projects/QA_Test.VC1",objeto:"pvShading"}` | ok | `{eliminado:"pvShading"}`; raíz → `No se puede eliminar el objeto raíz del archivo` | Objeto inexistente → `Object 'pvNoExiste' not found`. |
| `rvt` | `{accion:"rvt",operacion:"listar",archivo:"UserBatch/Z_Plantilla.RVT"}` | ok | `{ok:true,operacion:"listar",variables:[…],total:1}` | Operaciones: `listar, anadir, eliminar, reordenar, crear, catalogo`. |
| `plantilla_rvt` | `{accion:"plantilla_rvt",operacion:"crear",archivo:"UserBatch/QA_Plantilla.RVT",variables:["GlobHor","E_Grid"]}` | ok | `{ok:true,archivo:"…QA_Plantilla.RVT",historial_id:"…"}` | `rvt` y `plantilla_rvt` son **el mismo handler** (redundancia, §4). Operación inválida → lista de válidas. |

### 2.6 `escribir_archivo` (1/1, 8 escenarios)

| Escenario | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| texto | `{archivo:"QA_tmp/nota.txt",contenido:"hola QA\n"}` | ok | `{bytes_written:8,historial_id:"20261108_064730_142_0000"}` | |
| existente sin `sobrescribir` | `{…,contenido:"otra vez\n"}` | **bug** | `bytes_written:9`, nuevo `historial_id`; el archivo quedó con `otra vez` | Sobrescribe por defecto (BUG-14). |
| `sobrescribir:true` | `{…,sobrescribir:true}` | ok | idéntico al anterior | El flag no cambia nada. |
| `sobrescribir:false` | `{…,sobrescribir:false}` | ok (validación) | `'…nota.txt' already exists and 'sobrescribir': false was requested: nothing was written` | El veto sí funciona. |
| base64 | `{archivo:"QA_tmp/bin.dat",contenido:"UUEgYmFzZTY0…",base64:true}` | ok | `bytes_written:18`; el archivo contiene `QA base64 payload` | |
| vacío | `{archivo:"QA_tmp/vacio.txt",contenido:""}` | ok (validación) | `'contenido' must not be empty: a 0-byte file has nothing to verify. Pass 'base64': true…` | |
| traversal | `{archivo:"../fuera.txt",…}` | ok (validación) | `Ruta fuera del workspace: '../fuera.txt' contiene '..'` | |
| extensión protegida | `{archivo:"QA_tmp/escena.SHD",…}` | ok (validación) | `escena 3D: solo lectura; el formato de las mallas no está demostrado…` | |
| historial | `{accion:"historial",n:5}` | ok | `snapshots[{id,archivos,fecha}]` | Crece con cada escritura. |

### 2.7 `proyecto` (16/16)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `crear` | `{accion:"crear",plantilla:"_DEMO_Commercial",proyecto:"QA_Test"}` | ok | `{project:"QA_Test",based_on:"_DEMO_Commercial",applied:[…],historial_id:"…"}` | 102 ms. Duplicado → `Project exists: …` (en inglés, BUG-22). Plantilla inexistente y falta `plantilla` → error claro. |
| `eliminar` | `{accion:"eliminar",proyecto:"QA_Renombrado"}` | ok | `{count:2,recycled:["…QA_Renombrado.PRJ","…QA_Renombrado.VC0"]}` | Mueve a `UserRecycleBin` (borrado suave). |
| `renombrar` | `{accion:"renombrar",proyecto:"QA_Test",nombre_nuevo:"QA_Renombrado"}` | ok | `{renamed:["…QA_Renombrado.PRJ","…QA_Renombrado.VC0"]}` | **Colisión probada**: `No se puede renombrar: ya existe …QA_RenB.PRJ`. Correcto. |
| `crear_variante` | `{accion:"crear_variante",proyecto:"QA_Test",variante:"VC0",variante_nueva:"VC1"}` | ok | `{all_variants:["QA_Test.VC0","QA_Test.VC1"],created:"…VC1"}` | |
| `dimensionar` (con n's) | `{accion:"dimensionar",proyecto:"QA_Test",variante:"VC1",n_mod_serie:22,n_strings:40}` | ok | `applied:[{key:"NModSerie",old:"24",new:"22"},{key:"NStringCh",…}]` | 174 ms, sin GUI. |
| `dimensionar` (sin n's, GUI) | `{accion:"dimensionar",proyecto:"QA_Sim",variante:"VC0",kwp:450}` (workspace real) | ok | `{metodo:"pre-dimensionado de PVsyst (ventana Sistema, potencia planeada)",propuesta_pvsyst:{n_mod_serie:"27",n_strings:"38"},guardado:{NInverter:"6",NModSerie:"27",NStringCh:"38",PNomSizing:"450.0"}}` | **38,9 s**, abre PVsyst y guarda. Con `workspace` de prueba → `The pre-sizing runs on PVsyst in its own workspace: 'workspace' is not supported`. |
| `doctor` | `{accion:"doctor",proyecto:"QA_Test"}` | ok | `{ok:true,avisos:["…Generic_Mono_440W_Half.PAN no está como archivo (¿componente interno de PVsyst?)…"],errores:[]}` | 205 ms. |
| `anadir` × 10 | `{accion:"anadir",proyecto:"QA_Test",variante:"VC1",elemento:<X>}` | ok | `baterias`→`pvGridStorage`; `carga`→`pvLoad`; `horizonte`→`pvHorizon` (`neutralizado:true`); `envejecimiento`, `bifacial`, `carbono`, `domo`, `zona_campo` (`avisos:["The inserted geometry i…"]`), `bifacial_seguidor` | `circuito_hidraulico` → `El elemento 'circuito_hidraulico' solo aplica a proyectos de Bombeo: La variante es de tipo Red` (validación de dominio correcta). Elemento desconocido → lista de válidos. |
| `seguidor` | `{accion:"seguidor",proyecto:"QA_Test",variante:"VC0",azimut_eje:0,angulo_max:60,backtracking:true}` | ok | `cambios:[{action:"replaced",key:"FieldType",…}]` | Sobre VC1 (con bifacial) → `Quita el modelo bifacial de mesas antes de convertir (o usa la plantilla _DEMO_Utility.VC6)`. |
| `limite_inyeccion` | `{…,potencia_kw:300,estrategia:"recorte_picos"}` | ok | `cambios:[{action:"inserted",key:"…"}]` | |
| `almacenamiento` | `{…,bateria:"Generic_LI_Module_180Ah.BTR",n_serie:1,n_paralelo:2,estrategia:"autoconsumo"}` | ok (validación) | `Componente 'Generic_LI_Module_180Ah.BTR' no encontrado; no hay ninguno en …ComposPV\Batteries` | Sin `estrategia` → `Missing: estrategia`. No hay baterías en el workspace: **bloqueada por fixture** para la ruta feliz. |
| `perdidas` | `{…,suciedad_pct:3.0,mismatch_pct:2.0,lid_pct:1.5}` | ok | `cambios:[{action:"replaced",key:"…"}]` | 225 ms. |
| `bombeo` | `{…,caudal_m3_dia:100,profundidad_estatica_m:30}` | ok (validación) | `La variante es de tipo Red; esta acción requiere Bombeo` | Validación de dominio correcta; ruta feliz **bloqueada por fixture** (no hay proyecto de bombeo). |
| `economia` | `{…,tarifa_venta_kwh:0.12,moneda:"EUR",vida_anios:25,tasa_descuento:0.04,inflacion:0.02}` | **bloqueada por fixture** | `La variante no tiene evaluación económica; crÉala una vez en la GUI de PVsyst (Economic evaluation) y luego aj…` | Requiere el bloque `EconomicEval`, que solo se crea en la GUI. `tasa_descuento:4` → `'tasa_descuento' debe estar entre 0 y 1: 4` (el esquema no dice que sea fracción, P3). |
| `configurar` | `{accion:"configurar",proyecto:"QA_Test",variante:"VC0",pasos:[{accion:"perdidas",suciedad_pct:2.0},{accion:"seguidor",azimut_eje:0,angulo_max:60},{accion:"limite_inyeccion",potencia_kw:250}]}` | ok | `{pasos_aplicados:3,resultados:[…]}` | 680 ms. Paso inválido → lista de acciones válidas. `pasos:[]` → `'pasos' must be a non-empty list of {accion:…}`. |
| `p50_p90` | `{accion:"p50_p90",proyecto:"QA_Test",variante:"VC1",dry_run:true}` y GUI sobre `QA_Sim` | **bug** | dry_run: `The P50/P90 configuration runs on PVsyst in its own workspace: 'workspace' is not supported`. GUI: `PVsyst did not open 'TProjectForm' in 15 s` y luego `PVsyst no guardó …\QA_Sim.VC0: ¿usa PVsyst otro espacio de trabajo?`; la ventana quedó titulada **«PVsyst 8.1.8 - PRUEBA»** | BUG-05. La lectura de P50/P90 sí funciona (`resultados que:p50_p90`). |
| `limpiar_temporales` | `{accion:"limpiar_temporales"}` | ok | `{ok:true,movidos:[],fallidos:[]}` | 31 ms. |

### 2.8 `componente` (6/6)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `crear` | `{accion:"crear",plantilla:"MCP_Test_400",nombre_nuevo:"QA_PAN_450",tipo:"PAN",cambios:{PNom:450}}` | ok | `{created:"…\PVModules\QA_PAN_450.PAN",changes_applied:[{key:"PNom",old:"400.0",new:"450"}],historial_id:"…"}` | **Exige `cambios`** aunque la descripción diga «crear desde una plantilla» (BUG-15). Duplicado → `ya existe: pasa 'sobrescribir': true`. |
| `editar` | `{accion:"editar",componente:"QA_PAN_450",cambios:{PNom:455}}` | ok | `changes_applied:[{key:"PNom",old:"450",new:"455"}]` | Campo inexistente → `Campos que el componente no tiene: ["NoExiste"]. componente editar cambia valores existentes; para añadir un campo usa editar accion:campos…`. |
| `borrar` | `{accion:"borrar",componente:"QA_PAN_Import"}` | ok | `{recycled:"…\PVModules\QA_PAN_Import.PAN",moved_to:"…\UserRecycleBin\QA_PAN_Import.PAN"}` | Borrado suave. Inexistente → error claro. |
| `importar` | `{accion:"importar",origen:"C:/Users/Lenovo/Desktop/QA_PAN_450.PAN",nombre_nuevo:"QA_PAN_Import",tipo:"PAN"}` | ok | `{imported:"…",to:"…\PVModules\QA_PAN_Import.PAN",aviso:"'nombre_nuevo' had no extension: saved as 'QA_PAN_Import.PAN'…"}` | Añade la extensión y avisa. |
| `exportar` | `{accion:"exportar",componente:"QA_PAN_450",destino:"C:/Users/Lenovo/Desktop/QA_PAN_450.PAN",permitir_fuera_del_workspace:true}` | ok | `{exported:"…\PVModules\QA_PAN_450.PAN",to:"C:/Users/Lenovo/Desktop/QA_PAN_450.PAN"}` | Sin `permitir_fuera_del_workspace` → `Ruta fuera del workspace`. |
| `crear_desde_ficha` (GUI) | `{accion:"crear_desde_ficha",tipo:"PAN",nombre_nuevo:"QA_Ficha",datos:{Model:"QA Ficha 400",Manufacturer:"QA",PNom:400,Isc:10.5,Voc:49,Imp:9.9,Vmp:40.4,NCelS:72,Technol:"mtSiMono"}}` | ok | **43,8 s**; `{archivo:"…\PVmodules\QA_Ficha.PAN",origen:"pvsyst",valores_pvsyst:{PNom:"400.0",Voc:"49.00",…},avisos_pvsyst:["El exponente eV/KT es demasiado grande (76.333), reducido a -0.001",…]}` | PVsyst ajusta el modelo en su ventana y devuelve sus valores. Ficha incompleta → `faltan ["Technol"]; no admitidos ["NCelP"]`. |

### 2.9 `sitio` (6/6)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `crear` (coordenadas) | `{accion:"crear",lat:-33.45,lon:-70.66,nombre_sitio:"QA_Site_Santiago"}` | ok | `{annual_ghi_kwh_m2:1991.3,monthly_ghi_kwh_m2:[…],costo:{ejecuciones:1,restantes:249},license:"…remaining executions: 250",create_site.clasificacion:"exito"}` | 7,3 s. **1 ejecución TRIAL**. `dry_run:true` previo devolvió `command_line` pero **sin `costo_estimado`** (BUG-20). `lat:999` → `'latitud' fuera de rango (-90 a 90)`. Nombre con `/` → `Nombre inválido`. |
| `crear` (CSV) | `{accion:"crear",origen:"csv",nombre_sitio:"QA_SiteCSV",lat:7.07,lon:-73.84,weather_data_file:"UserData/Barrancabermeja_Monthly.csv"}` | ok | `{clasificacion:"exito",costo:{ejecuciones:1,restantes:248},stdout_tail:"…File …QA_SiteCSV.SIT created with success"}` | 8,2 s, **1 TRIAL**. El parámetro es `weather_data_file`, no `input_csv_file` (el nombre del esquema despista, P3). `expected_sit:null` (BUG-26). |
| `convertir_meteo` | `{accion:"convertir_meteo",input_csv_file:"Meteo/Sig2sept.csv",input_mef_file:"Meteo/SIG_Geneva.MEF",input_sit_file:"Sites/QA_Site.SIT",output_met_file:"Meteo/QA_Geneva.MET"}` | **bug** | `clasificacion:"ambiguo"`, `success:false`, `stdout_tail:"…Running weather data conversion ...\nInvalid date Line 6."`, `detected_error:"ambiguo: …"` | El CSV fixture no vale para PVsyst, pero el MCP **sí** tenía el error en la salida y lo llamó «ambiguo» (BUG-09). `dry_run` previo ok. Falta `output_met_file` → error claro; MEF inexistente → `'input_mef_file' no existe: …`. |
| `horizonte_pvgis` (GUI) | `{accion:"horizonte_pvgis",proyecto:"QA_Sim",variante:"VC0"}` | ok | `{puntos:"42",fuente:"PVGIS (importación de PVsyst)",FHHDiffus:"0.998",FHPAlbedo:"0.917",comentario:"Horizon from PVGIS website API, Lat=37°48'10\"…"}` | 15,6 s. Con `workspace` → `The PVGIS horizon runs on PVsyst in its own workspace: 'workspace' is not supported`. |
| `meteo_pvgis` | `{accion:"meteo_pvgis",lat:46.2044,lon:6.1432,nombre_sitio:"QA_PVGIS",confirmar:true}` | ok | `{horas:8760,costo:{ejecuciones:1,restantes:249},cli.stdout_tail:"…Conversion ended successfully…"}`; `.MET` de 193 912 B | 19,3 s. Sin `confirmar` → `{ejecuciones_estimadas:2,mensaje:"Esta acción consume 2 ejecuciones de prueba; repítela con confirmar:true"}`. `dry_run` → `'dry_run' no existe en sitio meteo_pvgis: esa acción no tiene ensayo y se ejecutaría de verdad`. |
| `escribir_horizonte` | `{accion:"escribir_horizonte",archivo:"Sites/QA_Horizonte.HOR",puntos:[{azimut:-90,altura:3},{azimut:0,altura:10},{azimut:90,altura:5},{azimut:180,altura:0}]}` | ok | `{archivo:"…\Shadings/Sites/QA_Horizonte.HOR",bytes_escritos:171,puntos:4}` | Escribe bajo `Shadings/` aunque se pida `Sites/` (BUG-19). Puntos desordenados → `Puntos no ordenados por azimut: 180.00 seguido de -90.00`; 1 punto → `se requieren al menos 2 puntos`; azimut 270 → `fuera de rango [-180, 180]`. |
| `escribir_serie` | `{accion:"escribir_serie",archivo:"Meteo/QA_Serie.MET",variables:["GlobHor"],valores:{GlobHor:[8760 valores]}}` | ok | `{variables_actualizadas:["GlobHor"],total_horas:8760,bytes_escritos:110729}`; `leer serie:true` confirma `GlobHor≈0.9979` | Exige 8760/8784 valores: `Variable 'GlobHor' tiene 3 horas; esperado 8760 o 8784`. Archivo inexistente → `no encontrado`. |

### 2.10 `simular` (3 expuestas + 2 ocultas)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `ejecutar` (dry_run) | `{accion:"ejecutar",proyecto:"QA_Test",dry_run:true}` | ok | `cli_args:["run-simulation","-w:…","-p:QA_Test.PRJ","-v:VC0","-ocf:…","-rpf:…"]`, `costo_estimado:{ejecuciones:1,restantes:249}`, `variant_file_exists:true` | 8 ms. |
| `ejecutar` (real) | `{accion:"ejecutar",proyecto:"QA_Sim"}` (workspace real) | ok | `success:true`, `confirmed_by_output:true`, `costo:{ejecuciones:1,restantes:247}`, `stdout_tail:"…Simulation done in 0min 3sec\n…Report PDF … generated"` | **25,3 s**, 1 TRIAL. Genera CSV (336 253 B) y PDF. |
| `ejecutar` (crash) | `{accion:"ejecutar",proyecto:"QA_Diseno3"}` | **bug** | `clasificacion:"ambiguo"`, `exit_code:0`, `success:false`, `avisos_pvsyst:["Instruction privilégiée","Error: Invalid command syntax","Unknown command line option : --bogus"]`; `logs_cli` confirma `EPrivilege\|run-simulation\|Instruction privilégiée` y `nb_crash:2` | BUG-02. Consume 1 TRIAL. |
| `preparar` | `{accion:"preparar",proyecto:"QA_Test",filas:[{tilt:20},{tilt:30}]}` y `{…,parametros:{FieldTilt:25}}` | ok | `{batch_params_file:"…\UserBatch\MCP_BatchParams.csv",batch_rvt_file:null}`; con `parametros` → `param_file:"…\Other\Param_Modif.dat"` | `filas` acepta solo `csv, tilt, azim, pan, rserie, rshunt, rsh0, n_mod_serie, n_…`: `Row 1: unsupported keys ["FieldTilt"]`. `filas:[]` → `'filas' must not be empty`. El `siguiente` no arrastra los archivos de batch (BUG-23). |
| `estado` | `{accion:"estado",trabajo:"NO_EXISTE"}` | ok (validación) | `No existe el trabajo 'NO_EXISTE'; no hay trabajos en segundo plano en esta sesión; lanza uno con simular {segu…` | |
| `preestimar` | `{accion:"preestimar",lat:46.2044,lon:6.1432,kwp:500}` | **bloqueada por gate** | `'simular preestimar' is an MCP in-house calculation, not a PVsyst one. To get it from PVsyst: simular accion:e…` | **No está en el enum del esquema** (no expuesta por defecto). Con `PVSYST_MCP_APROXIMACIONES=1` funciona y se etiqueta `origen:"aproximacion_mcp"` + `aviso_origen`. |
| `barrido` | `{accion:"barrido",proyecto:"QA_Sim",lat:46.2044,lon:6.1432,kwp:500,filas:[{tilt:20},{tilt:30}]}` | **bloqueada por gate** | Igual; con el gate activo: `{requiere_confirmacion:true,ejecuciones_estimadas:3,restantes:249,plan:[…]}` | Tampoco está en el enum. Pide `proyecto` **y** `lat/lon/kwp` (contradictorio, P3). |

### 2.11 `resultados` (11/11)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `kpis` | `{que:"kpis",proyecto:"QA_Sim"}` | ok | `{energia_anual_kwh:871243.0,pr_pct:82.68,produccion_especifica_kwh_kwp:1719.0,weather_file:"Oakland CA_MN90_SYN.MET"}` | 242 ms, lee el PDF. |
| `csv` | `{que:"csv",proyecto:"QA_Sim",count:3}` | ok | `results[{path:"…QA_Sim_VC0.csv",size_bytes:336253,summary:{row_count:8760,totals:{E_Grid:{sum:871243167.3,unit:"Wh"}}}}]` | `count` limita **archivos**, no filas. `archivo` directo también funciona; inexistente → `CSV file not found: …`. |
| `perdidas` | `{que:"perdidas",proyecto:"QA_Sim"}` / `detalle:"completo"` | **bug** | `{energia_kwh:871243.0,**kwp:400.0**,nodos:[{concepto:"Global horizontal irradiation",…}]}` | BUG-01: el kWp real es 506,88. |
| `mensual` | `{que:"mensual",proyecto:"QA_Sim"}` | **bug** | `{anual:{E_Grid:871243.0,PR:0.827,…},**kwp:400.0**,meses[12]}` | BUG-01. |
| `comparar` | `{que:"comparar",proyecto:"Planta_Solar_50kW",variantes:["VC0","VC1"]}` | ok | `deltas_respecto_primera:[{delta:{energia_kwh:-41438.0,energia_pct:-58.5,…}}]` | 10 s. Con 1 variante → `Se necesitan al menos 2 variantes con resultados en 'X' (hay 1)` (BUG-25). |
| `medido_vs_simulado` | `{que:"medido_vs_simulado",proyecto:"QA_Sim",medido_csv:"Results/QA_Medido_Mensual.csv"}` | **bloqueada por gate** | Sin gate: `'resultados medido_vs_simulado' is an MCP in-house calculation, not a PVsyst one…`. Con gate: `{anual:{medido_kwh:862012.0,simulado_kwh:871243.0,desviacion_pct:-1.06},origen:"aproximacion_mcp"}` | CSV horario → `Fila mensual no numérica: ';File;File date;Description'` (espera 12 filas `mes;energia_kwh`). |
| `p50_p90` | `{que:"p50_p90",proyecto:"QA_Eco"}` | ok | `{energia_kwh:773994.0,desviacion_media_pct:0.0,fuente_meteo:"Synthetic - Multi-year ave…"}` | Sin la página en el PDF → `El informe PDF no contiene la página 'P50 - P90 evaluation'…`. |
| `carbono` | `{que:"carbono",proyecto:"QA_Eco"}` | ok | `{balance_neto_total:10187.0,balance_neto_unidad:"tCO2",degradacion_anual_pct:1.0,detalles_lce:[…]}` | Sin la página → mensaje que dice qué activar. |
| `economia` | `{que:"economia",proyecto:"QA_Eco"}` | ok | `{anualidades:35966.59,anualidades_unidad:"USD/yr",costes_operacion:57780.46,…}` | Idem. |
| `entregable` | `{que:"entregable",proyecto:"QA_Sim"}` | **bug** | `{archivo:"…QA_Sim_VC0_entregable.md",**kwp:400.0**,**produccion_especifica_kwh_kwp:2178.1**,secciones_omitidas:[{seccion:"carbono"},{seccion:"economia"}]}` | Genera `.md`, no PDF (el PDF de PVsyst va aparte en `pdf`). BUG-01: 2 178,1 contradice los 1 719,0 de `kpis`. |
| `resumen` | `{que:"resumen",proyecto:"QA_Sim"}` | ok | `{kpis:{…},mensual:{…},perdidas:{…}}` en **una** llamada: 708 ms / 1,7 KB frente a `kpis`+`mensual`+`perdidas` = 763 ms / 3,1 KB | Ahorra 2 llamadas y ~45 % de bytes. Ver §4. |

### 2.12 `workspace` (6/6)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `crear` | `{accion:"crear",ruta:"…/QA_WS_Small"}` | ok | `{folders:["ComposPV","DataMeas","Meteo","Models","Projects","Shadings","Other","Sites","User…"]}` | Acepta ruta absoluta fuera del workspace (es su propósito). Existente → `Workspace already exists: … (pass sobrescribir=true to reuse)`. |
| `exportar` | `{accion:"exportar",destino:"QA_tmp/small.zip",workspace:"…/QA_WS_Small"}` | **bug** | `{zip:"…\QA_WS_Small\QA_tmp/small.zip",size_bytes:6430}`; el zip contiene `QA_tmp/small.zip.mcp_tmp_20960.zip` (0 B) | El `destino` relativo se resuelve contra el workspace **exportado**, no contra `workspace` (BUG-10), y el zip se incluye a sí mismo. |
| `importar` | `{accion:"importar",zip:"…/QA_WS_Small/QA_tmp/small.zip",destino:"…/QA_WS_Import"}` | ok | `{archivos:5,extracted_to:"…QA_WS_Import"}`; `QA_a.txt` = `alpha` | Ida y vuelta correcta **con rutas absolutas**. Con ruta relativa falla: `Zip not found: …\QA_tmp/small.zip` (BUG-10). Destino no vacío → error claro; `..` → rechazo. |
| `exportar_proyecto` | `{accion:"exportar_proyecto",proyecto:"QA_Test",destino:"QA_tmp/QA_Test.zip"}` | ok | `{count:3,files:["QA_Test.PRJ","QA_Test.VC0","QA_Test.VC1"]}` | 765 ms. |
| `historial` | `{accion:"historial",n:5}` / `detalle:"completo"` | ok | `{snapshots:[{id:"20261108_064730_142_0000",archivos:1,fecha:"…"}],corruptos:[]}` | `n:0` → `Value 0 out of range (minimum 1)`. |
| `restaurar` | `{accion:"restaurar",id:"20261108_064730_142_0000",dry_run:true}` y real | ok | dry_run: `{simulado:true,cambiarian:1,acciones:[{accion:"borrar",archivo:"…/nota.txt"}]}`; real: borra el archivo | Id inexistente → `Snapshot 'NO_EXISTE' no encontrado`. |

### 2.13 `gui_observar` (12/12)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `ventanas` | `{que:"ventanas"}` | ok | `{count:2,windows:[{class:"THomeForm",title:"PVsyst 8.1.8 - DEMO"},{class:"TApplication"}]}` | 15 ms. |
| `controles` | `{que:"controles",ventana:"THomeForm",detalle:"completo"}` | **bug** | `count:24`, 24 controles con `hwnd`/`class`/`title` | Sin `ventana` usa la ventana activa; con `detalle` por defecto devuelve 13 y `count:31` (BUG-16). |
| `leer` | `{que:"leer",ventana:"THomeForm"}` | ok | `{text:"PVsyst 8.1.8 - DEMO"}` | 18 ms. Sin `ventana` → `Missing required argument 'ventana' (hwnd, class or title)`. |
| `capturar` | `{que:"capturar",ventana:"THomeForm"}` | ok | `{path:"…\Temp\pvsyst_gui_THomeForm.png",bytes:77063,width:801,height:701}` | 233 ms. |
| `uia` | `{que:"uia",ventana:"TWorkspaceForm"}` | ok | `arbol{…hijos:[{clase:"TPngBitBtn",nombre:"Cerrar",patrones:["invoke"],rect:[1148,1118,1358,1175]}]}` | 501 ms. |
| `arbol` | `{que:"arbol",ventana:"THomeForm"}` | ok | árbol de controles Win32 + UIA | 21 ms. |
| `formulario` | `{que:"formulario"}` (OCR) | ok | `botones[{texto:"Herramientas",rect:{…}},{texto:"Datos medidos"},{texto:"Bases de datos"},{texto:"Cambiar"}…]` | 1,5 s. |
| `vista` | `{que:"vista",ventana:"TDimensForm"}` | ok | `{entidad:"ventana.TDimensForm",titulo:"Pre-dimensionamiento sistema de red",botones:[{texto:"Horizonte",habilitado:false},{texto:"Sistema"},{texto:"Resultados"},{texto:"Sitio y datos meteo"}]}` | Sobre `THomeForm`: `aviso:"ninguna ventana conocida abierta"` aunque está abierta (BUG-17). |
| `esquema` | `{que:"esquema"}` | ok | `count:5` entidades (`components.panels`, `components.inverters`, `components.batteries`, `variante.orientation`, `variante.system`, `proyecto.ventana`), `cobertura{total:89,por_estado:{2:71,3:14,4:4}}` | 33,9 KB, el payload más grande de la GUI. |
| `mensajes` | `{que:"mensajes"}` | ok | `{count:0,mensajes:[],ruta:"…\PVsyst\8.1\Admin\20261108_0200Err.log"}` | Lee el log propio de PVsyst. |
| `funciones` | `{que:"funciones"}` | ok | `count:209`, p. ej. `{clave:"diseno_preliminar.conectado_a_la_red",etiqueta:"Diseño preliminar > Conectado a la red",abre:"TDimensForm",via:"menu"}` | 3,7 KB. |
| `menus` | `{que:"menus",ventana:"THomeForm"}` | **bug** | `vista.estado:"parcial"`, `error:"El menú 7 de THomeForm no se abrió o la entrada pedida no tiene submenú"`, `menus[0].items[14]` | 11,7 s para fallar a medias (BUG-18). |

### 2.14 `gui_actuar` (26/26)

| Acción | Llamada | Resultado | Evidencia | Notas |
|---|---|---|---|---|
| `lanzar` | `{accion:"lanzar"}` | ok | `{pid:26508,ready:true,windows.count:2}` | 17,2 s. |
| `activar` | `{accion:"activar",ventana:"THomeForm"}` | ok | `{action:"activar"}` | 32 ms. |
| `cerrar` | `{accion:"cerrar",ventana:"TDEMOForm"}` | ok | `{invoked:"TDEMOForm",window.closed:true,window.visible:false}` | 635 ms. Cerró el modal «Modo DEMO». |
| `click` | `{accion:"click",ventana:"TDimensForm",texto:"Sistema"}` y `{…,id:0x110766}` | ok | `{action:"click"}` | Clic enviado; no se observó apertura de ventana (el botón de pre-dimensionado no abrió nada sin proyecto). |
| `clic_fisico` | `{accion:"clic_fisico",ventana:"TWorkspaceForm",objetivo:{x:1253,y:1146}}` | ok | `{invoked:"TWorkspaceForm",window.visible:true}` | Invoca la **ventana**, no el control de las coordenadas (BUG-29). |
| `mensaje` | `{accion:"mensaje",ventana:"TWorkspaceForm",mensaje:16}` | ok | `{action:"mensaje"}`; la ventana desapareció después | WM_CLOSE funciona (asíncrono). |
| `teclas` | `{accion:"teclas",ventana:"TWorkspaceForm",teclas:"ESC"}` | ok | `{sent:"ESC",via:"keybd_event",**foco:false**,aviso:"The target is the window: the keys reach the control that already has the focus inside it"}` | Envió sin foco en escritorio compartido (BUG-24). ESC no cerró el diálogo. |
| `fila` | `{accion:"fila",ventana:"TWorkspaceForm",objetivo:{clase:"TVirtualStringTree"},indice:0}` | ok (validación) | `TWorkspaceForm of TWorkspaceForm has no measured row geometry; measured on: TVirtualStringTree of TProjectLoadForm, TListBox of TChoixObjetPVForm, …` | Mensaje accionable. Ruta feliz **bloqueada por fixture** (hace falta un formulario con geometría medida y filas). |
| `escribir` | `{accion:"escribir",ventana:"TWorkspaceForm",objetivo:{clase:"TVirtualStringTree"},texto:"x"}` | ok (validación) | `'escribir' on class TWorkspaceForm has no effect: pass 'campo' (clase, x, y) of the control or a window of class TEdit/TMemo/TLabeledEdit/TMaskEdit/TSpinEdit` | |
| `seleccionar` | `{accion:"seleccionar",ventana:"TWorkspaceForm",objetivo:{clase:"TVirtualStringTree"},texto:"x"}` | ok (validación) | `'seleccionar' on class TWorkspaceForm has no effect: pass 'campo' … or a window of class TComboBox/TComboBoxEx/TListBox` | |
| `pagina` | `{accion:"pagina",ventana:"TWorkspaceForm",indice:1}` | ok (validación) | `'Espacio de trabajo de usuario' has no tab pages. Form texts: ["Espado de trabajo de usuario","Espacio de trabajo de usuario",…]` | Sin `indice` → `Missing: indice`. |
| `esperar_cierre` | `{accion:"esperar_cierre",ventana:"TWorkspaceForm",timeout_ms:3000}` y sobre `THomeForm` | **bug** | `{cerrado:false,espera_ms:3000}` pero `vista.estado:"ok"`, `resumen:"Cierre de la ventana esperado."` | BUG-03: la vista miente. |
| `uia_invocar` | `{accion:"uia_invocar",ventana:"TWorkspaceForm",nombre:"Cerrar",operacion:"invocar"}` | ok | `{action:"uia_invocar"}`; la ventana se cerró | Sin `operacion` → `Missing required argument 'operacion' (invocar\|fijar_valor\|alternar\|seleccionar\|expandir\|contraer)`. |
| `esperar` | `{accion:"esperar",clase:"TWorkspaceForm",timeout_ms:5000}` | ok | `{action:"esperar"}` | Sin `clase`/`titulo` → `Pass 'clase' or 'titulo' to wait for a window`. |
| `paso` | `{accion:"paso",objetivo:{nombre:"Cerrar"},pasos:[{accion:"activar",ventana:"THomeForm"}]}` | ok | `{action:"paso"}` | Sin `objetivo` → `Missing required argument 'objetivo' (nombre, rol, clase o titulo)`. |
| `secuencia` | `{accion:"secuencia",pasos:[{accion:"activar",ventana:"THomeForm"},{accion:"esperar",clase:"THomeForm",timeout_ms:2000}]}` | ok | `{action:"secuencia"}` | 52 ms. |
| `funcion` | `{accion:"funcion",funcion:"diseno_preliminar.conectado_a_la_red"}` | ok | `{action:"funcion"}`; abrió `TDimensForm` | Con un modal encima → `THomeForm is disabled: a modal dialog is open on top of it (TDEMOForm). Close it (gui_actuar accion:cerrar) before sending menu commands`. |
| `menu` | `{accion:"menu",ventana:"THomeForm",menu:0,item:0}` | ok | `{action:"menu"}`; abrió `TWorkspaceForm` | **12,5 s**. |
| `campo` | `{accion:"campo",ventana:"TWorkspaceForm",etiqueta:"NoExiste",valor:"1"}` | ok (validación) | `No field is called 'noexiste'. Labels seen: . Use gui_actuar accion:rellenar with the keys of gui_observar que:vista (que:esquema lists the entities)` | Sin `etiqueta`/`valor` → `Missing: etiqueta, valor`. |
| `rellenar` | `{accion:"rellenar",ventana:"TWorkspaceForm",entidad:"no.existe",valores:{x:1}}` | ok (validación) | `Entidad 'no.existe' desconocida: usa ["components.panels","components.inverters","components.batteries","variante.orientation","variante.system","proyecto.ventana"]` | `simular:true` (ensayo) no se llegó a probar: requiere una entidad con campos. |
| `clic_texto` | `{accion:"clic_texto",ventana:"TWorkspaceForm",texto:"Cerrar"}` | **bug** | `{coincidencias:0,punto:{x:1253,y:1146}}` con `vista.estado:"ok"` | BUG-03: cero coincidencias y aun así «ok». |
| `reiniciar` | `{accion:"reiniciar"}` | ok (validación) | `PVsyst shows no failure dialog (madExcept or the instability warning): there is nothing to restart. Use cerrar or lanzar` | Guarda de seguridad correcta. |
| `cerrar_todo` | `{accion:"cerrar_todo"}` | ok | `{action:"cerrar_todo"}`; `gui_observar ventanas` → `count:0` | 318 ms. |
| `escena` | `{accion:"escena",proyecto:"QA_Diseno4",variante:"VC0",escena:"QA_Escena",objetos:[{tipo:"paralelepipedo",…}]}` | ok (validación) | Primero: `objetos[0]: 'ancho' no existe (claves de paralelepipedo: ["tipo","nombre","x","y","z","rotacion_azimut_deg","anchura_dx","longitud_dy","altura_dz"])`. Con las claves bien: `No 3D editor is open: open it with gui_actuar accion:funcion funcion:variante.sombreados_cercanos.construccion_perspectiva and retry` | **Bloqueada por fixture**: hace falta un proyecto abierto en la GUI para abrir el editor 3D. |
| `escena_leer` | `{accion:"escena_leer",escena:"QA_Escena"}` | bloqueada | `No 3D editor is open: …` | Mismo motivo. |
| `escena_importar` | `{accion:"escena_importar",escena:"QA_Escena",archivo:"UserRecycleBin/Z_malla.pvc"}` | bloqueada | `No existe UserRecycleBin/Z_malla.pvc` (el archivo existe en disco; `UserRecycleBin` no se resuelve) y luego `No 3D editor is open` | Doble bloqueo: resolución de rutas + editor. |

---

## 3. Bugs

> **Regresión o preexistente:** se resolvió con una **prueba diferencial** contra
> el build instalado antes (2026-11-06), con los mismos fixtures y el mismo
> arnés. El resultado completo, con el binario y los comandos, está en **§8**.
> Resumen: **22 existían antes del PR #2** (su commit de introducción es
> ancestro del merge-base, así que están en `main`), **3 los introduce el PR
> durante su desarrollo** (BUG-18, 23, 25) y **2 son de funcionalidad que estrena
> el PR** (BUG-06, 20). **Ninguna regresión** de comportamiento: no se encontró
> nada que funcionara antes y se rompiera ahora.

---

### BUG-01 · P1 · `resultados perdidas`/`mensual`/`entregable` calculan mal el kWp

**Tool/acción:** `resultados` `que:perdidas`, `que:mensual`, `que:entregable`.

**Reproducción**

1. `proyecto crear {plantilla:"_DEMO_Commercial", proyecto:"QA_Sim"}`.
2. `simular ejecutar {proyecto:"QA_Sim"}` → `success:true`, `energia_anual_kwh:871243.0`.
3. `resultados que:kpis {proyecto:"QA_Sim"}` → `pr_pct:82.68`,
   `produccion_especifica_kwh_kwp:1719.0`.
4. `resultados que:perdidas {proyecto:"QA_Sim"}` → `kwp:400.0`.
5. `resultados que:entregable {proyecto:"QA_Sim"}` → `kwp:400.0`,
   `produccion_especifica_kwh_kwp:2178.1`.

**Esperado:** el kWp del array. La variante tiene `NModSerie=24`,
`NStringCh=48` y módulo de 440 W → **506,88 kWp**; 871 243 / 506,88 = **1 718,9**,
que es exactamente lo que dice el informe de PVsyst (`kpis` = 1 719,0).

**Real:** 400,0, que es el campo `PNomSizing` de `pvSubArray` (potencia de
dimensionado, no la potencia del array). El entregable publica **2 178,1
kWh/kWp**, un 26,7 % por encima del valor de PVsyst.

**Evidencia**

```json
{"energia_anual_kwh":871243.0,"pr_pct":82.68,"produccion_especifica_kwh_kwp":1719.0}   // kpis
{"energia_kwh":871243.0,"kwp":400.0,"nodos":[…] }                                     // perdidas
{"kwp":400.0,"produccion_especifica_kwh_kwp":2178.1,"archivo":"…QA_Sim_VC0_entregable.md"} // entregable
```

```
# Projects/QA_Sim.VC0 (leer objeto pvSubArray)
NModSerie=24  NStringCh=48  NInverter=6  PVModule=Generic_Mono_440W_Half.PAN  PNomSizing=400.0
```

**Archivo/línea sospechosa:** `src/simulation/results/lookup.rs:88-101`
(`variant_kwp()`: `root.find_deep("PNomSizing")`). Se usa desde
`resultados perdidas/mensual/entregable`.

**Regresión:** **preexistente** — §8.2: el build del 6-nov devuelve exactamente `kwp:400.0` y `produccion_especifica_kwh_kwp:2178.1` sobre el mismo PDF.

---

### BUG-02 · P1 · Un *crash* de PVsystCLI se reporta como `ambiguo` con `exit_code:0`

**Tool/acción:** `simular ejecutar` (y, por elevación, `disenar_sistema`).

**Reproducción**

1. `proyecto crear {plantilla:"_DEMO_Commercial", proyecto:"QA_Diseno3"}` y
   dimensionar con un módulo sin dimensiones válidas.
2. `simular ejecutar {proyecto:"QA_Diseno3"}`.
3. Observar `clasificacion:"ambiguo"`, `exit_code:0`, `success:false`.
4. `entorno logs_cli {n:2, solo_errores:true}` → `EPrivilege | run-simulation |
   Instruction privilégiée` y `sesion.comandos["run-simulation"].nb_crash:2`.

**Esperado:** `estado:"error"` con `error.codigo:"crash"` (o similar) y el
mensaje de PVsyst; el MCP ya tiene el dato en `logs_cli`.

**Real:** `ambiguo: Salida sin confirmación de éxito ni marcador de error
concluyente`, `exit_code:0`. Un agente que reintente «porque fue ambiguo»
quema ejecuciones TRIAL en cada intento (cada uno costó 1).

**Evidencia**

```json
{"clasificacion":"ambiguo","exit_code":0,"success":false,
 "avisos_pvsyst":["Instruction privilégiée","Error: Invalid command syntax",
                  "Unknown command line option : --bogus"],
 "costo":{"ejecuciones":1,"restantes":null},
 "detected_error":"ambiguo: Salida sin confirmación de éxito ni marcador de error concluyente"}
```

**Archivo/línea sospechosa:** `src/adapters/cli/clasificacion.rs:170`
(`Clasificacion::Ambiguo(m) => (false, Some(format!("ambiguo: {m}")))`), que no
mira `logs_cli` ni el código de salida del proceso.

**Regresión:** **preexistente** — §8.3: el clasificador (`eeede5e`, 19-oct) nunca lee `nb_crash`; el PR no toca `clasificacion.rs` (0 commits en el rango).

---

### BUG-03 · P2 · `vista.estado:"ok"` cuando la acción no hizo nada

**Tools/acciones:** `gui_actuar esperar_cierre`, `gui_actuar clic_texto`.

**Reproducción**

1. `gui_actuar {accion:"esperar_cierre", ventana:"THomeForm", timeout_ms:1200}`
   (THomeForm **no** se cierra nunca).
2. `gui_actuar {accion:"clic_texto", ventana:"TWorkspaceForm", texto:"Cerrar"}`
   sobre una ventana donde el OCR no encuentra el texto.

**Esperado:** `vista.estado:"parcial"` o `"error"` con
`error.codigo:"no_cerro"` / `"sin_coincidencias"`.

**Real:** `vista.estado:"ok"`, `resumen:"Cierre de la ventana esperado."` /
`"Clic en el texto visible de la ventana."`, mientras los campos reales dicen
`cerrado:false` y `coincidencias:0`.

**Evidencia**

```json
{"accion":"esperar_cierre","cerrado":false,"espera_ms":1200,
 "vista":{"estado":"ok","resumen":"Cierre de la ventana esperado.","campos":[{"clave":"cerrado","valor":false}]}}

{"accion":"clic_texto","coincidencias":0,"punto":{"x":1253,"y":1146},
 "vista":{"estado":"ok","resumen":"Clic en el texto visible de la ventana."}}
```

**Archivo/línea sospechosa:** `src/adapters/gui/campos.rs:183-205`
(`esperar_cierre_con` devuelve `cerrado` pero el catálogo de `vista` fija
`estado:ok`); `src/adapters/gui/clic_por_texto.rs:27-41` (idem con
`coincidencias`).

**Regresión:** **preexistente** — §8.2: el build del 6-nov devuelve `cerrado:false` con `vista.estado:"ok"` y el mismo resumen.

---

### BUG-04 · P2 · `disenar_sistema` pierde el error de la simulación

**Tool/acción:** `disenar_sistema`.

**Reproducción**

1. `disenar_sistema {lat:46.2044, lon:6.1432, kwp:500, modulo:"MCP_Test_400.PAN",
   inversor:"MCP_Inv_10k.OND", n_mod_serie:22, n_strings:60, nombre:"QA_Diseno"}`.
2. Ver `pasos[4]` (simulación): `ok:false`, **`detail:null`**,
   `simulacion:"Checking license..."`.

**Esperado:** el `stdout_tail`/`avisos_pvsyst` de PVsyst en `pasos[].detail`
(como sí hace `simular ejecutar`).

**Real:** el error se pierde. En otra corrida (mismo fixture) `simular ejecutar`
devolvió `avisos_pvsyst:["The sizes of this module are not defined", …]` y
`"Simulation cannot be run."`, información que `disenar_sistema` no muestra.

**Evidencia**

```json
{"paso":"simulacion","ok":false,"detail":null,"csv":"…QA_Diseno.csv","pdf":"…QA_Diseno.pdf"}
{"errores":["paso 'simulacion' fallido","leer_kpis: Informe PDF no encontrado en '…QA_Diseno.pdf'"],
 "simulacion":"Checking license..."}
```

**Archivo/línea sospechosa:** `src/easy_mode/simulacion.rs:50,76`
(`leer_kpis` solo informa del PDF ausente) y el serializador de pasos en
`src/easy_mode/diseno.rs`.

**Regresión:** **preexistente** — §8.3: `a_pasos_json` (que deja `detail` nulo) viene de `eeede5e` (19-oct), anterior al merge-base. No se replicó en el 6-nov por depender del fixture, pero el código es anterior al PR.

---

### BUG-05 · P2 · `proyecto p50_p90` abre PVsyst en modo PRUEBA y no puede guardar

**Tool/acción:** `proyecto p50_p90` (ruta GUI).

**Reproducción**

1. `proyecto p50_p90 {proyecto:"QA_Sim", variante:"VC0"}` (workspace real).
2. Primera vez: `PVsyst did not open 'TProjectForm' in 15 s`.
3. Segunda vez: `PVsyst no guardó …\QA_Sim.VC0: ¿usa PVsyst otro espacio de trabajo?`
4. `gui_observar que:ventanas` → `title:"PVsyst 8.1.8 - **PRUEBA**"` (la sesión
   anterior, lanzada con `gui_actuar lanzar`, decía `- DEMO`).

**Esperado:** abrir PVsyst con el workspace real (como hace `proyecto
dimensionar`, que sí guardó) y escribir `P50/P90` en la variante.

**Real:** la instancia queda en modo PRUEBA, sin workspace, y la acción no
puede guardar. La lectura (`resultados que:p50_p90`) sí funciona.

**Evidencia**

```
proyecto.p50_p90.gui       15495ms  PVsyst did not open 'TProjectForm' in 15 s
proyecto.p50_p90.gui.retry 13280ms  PVsyst no guardó C:\Users\Lenovo\PVsyst8.1_Data\Projects\QA_Sim.VC0: ¿usa PVsyst otro espacio de trabajo?
gui_observar ventanas      {"class":"THomeForm","title":"PVsyst 8.1.8 - PRUEBA"}
```

**Archivo/línea sospechosa:** el lanzador de GUI de `proyecto p50_p90`
(`src/projects/gui/…`, ruta de `p50_p90`) frente al de `dimensionar`.

**Regresión:** **preexistente** — §8.3: la acción `p50_p90` entra en `0fca119` (21-oct), antes del merge-base.

---

### BUG-06 · P3 · El estado documentado `reutilizado:true` + `comparable:false` es inalcanzable

**Tool/acción:** `disenar_sistema`.

**Reproducción**

1. `disenar_sistema {lat:46.2044, lon:6.1432, kwp:10, n_mod_serie:22,
   n_strings:1, nombre:"QA_D6", simular:false}` → proyecto nuevo:
   `proyecto_reutilizado:false`, `proyecto_comparable:null`.
2. La misma llamada otra vez → `proyecto_reutilizado:true`,
   `proyecto_comparable:true`.
3. Intentar la combinación documentada (reutilizado + sin módulo/inversor/n's):
   sin n's la llamada se rechaza antes de empezar.

**Esperado (descripción de la tool):** «`proyecto_reutilizado:true` with
`proyecto_comparable:false` means the call asked for no module, inverter or
string layout, so there was nothing to compare».

**Real:** `comparable:false` no se puede observar: cualquier llamada que pasa la
validación lleva `n_mod_serie`/`n_strings` (obligatorios, BUG-08), y eso ya es
«algo que comparar», así que devuelve `true`. El `null` del proyecto nuevo sí es
deliberado (`src/easy_mode/resultado.rs:32-41`: «`comparable` is `None` on a
fresh project»).

**Evidencia**

```json
{"proyecto_reutilizado":false,"proyecto_comparable":null,"ok":true}   // proyecto nuevo
{"proyecto_reutilizado":true, "proyecto_comparable":true,"ok":true}   // reutilizado con n's
```

**Archivo/línea sospechosa:** `src/easy_mode/resultado.rs:32-41` y la
descripción en `src/server/tools/entorno.rs` (afirma un estado que la validación
hace inalcanzable).

**Regresión:** **nuevo en el PR** — §8.3: el build del 6-nov **no reutiliza**
(la segunda llamada devuelve `Project exists: …\QA_D7.PRJ`) y no emite
`proyecto_comparable`. La función de reutilización es nueva; el defecto es de su
documentación, no del dato.

---

### BUG-07 · P2 · `modulo` exige extensión en `disenar_sistema` pero no en `componente`

**Tool/acción:** `disenar_sistema` (`modulo`), `leer`/`componente`.

**Reproducción**

1. `disenar_sistema {…, modulo:"MCP_Test_400", …}` → `Error al leer el módulo
   'MCP_Test_400': Módulo 'MCP_Test_400' no encontrado`.
2. `disenar_sistema {…, modulo:"MCP_Test_400.PAN", …}` → funciona.
3. `componente editar {componente:"QA_PAN_450"}` (sin extensión) → funciona.

**Esperado:** aceptar el nombre con o sin `.PAN`, como `componente` y como
`listar que:modulos` (que devuelve `modulo:"PVmodules\\MCP_Test_400.PAN"` con
extensión).

**Real:** sin extensión, `NotFound`. Además `dry_run` **no** valida el módulo:
`dry_run` con `modulo:"MCP_Test_400"` devuelve `ok` y el fallo aparece solo en
la llamada real (después de crear sitio y proyecto).

**Evidencia**

```
disenar.dry_run.modulo_sin_ext  5ms  {"costo_estimado":{"ejecuciones":1},"dry_run":true,"plan":[…]}
disenar.real.red              562ms  Error al leer el módulo 'MCP_Test_400': Módulo 'MCP_Test_400' no encontrado
```

**Archivo/línea sospechosa:** `src/components/pan/mod.rs:124-142`
(`resolve_module_file` usa `file_name()` sin añadir `.PAN`);
`src/easy_mode/modulo_de_plantilla.rs:36-45`.

**Regresión:** **preexistente** — §8.2: el build del 6-nov falla con el mismo `Módulo 'MCP_Test_400' no encontrado`.

---

### BUG-08 · P2 · `via:"aproximacion"` no está en el esquema aunque el error lo recomienda

**Tool/acción:** `disenar_sistema`.

**Reproducción**

1. `disenar_sistema {lat, lon, kwp:500}` → error que termina en
   «…pasa ambos o usa `via:'aproximacion'`».
2. `disenar_sistema {…, via:"aproximacion"}` → **el servidor lo acepta** y
   ejecuta un diseño real (6,2 s, creó `Sistema_500kWp`).
3. Pero `tools/list` declara `via: {enum:["pvsyst"]}` y el cliente MCP rechaza
   el valor antes de enviarlo.

**Esperado:** o `via` admite `aproximacion` en el esquema, o el mensaje no lo
recomienda.

**Real:** el cliente que valide el esquema no puede usar la salida que el
propio servidor sugiere.

**Evidencia**

```json
// tools/list
"via": {"type":"string","enum":["pvsyst"],"description":"pvsyst (default): the PVsyst route."}
// llamada directa por stdio (sin validación de cliente)
{"avisos":["El módulo de la plantilla no existe: se usa 'MCP_Test_400.PAN'…"],
 "proyecto_creado":"Sistema_500kWp","proyecto_comparable":null,"ok":false}
```

**Archivo/línea sospechosa:** `src/server/tools/entorno.rs` (declaración de
`disenar_sistema`, `via` con `TipoDato::String` pero enum cerrado en el
generador de esquema) y `src/easy_mode/diseno.rs:75-84` (mensaje).

**Regresión:** **preexistente** — §8.3: la declaración de `via` viene de `0fca119` (21-oct), antes del merge-base; el enum es idéntico en los dos builds.

---

### BUG-09 · P2 · Errores de texto claro del CLI se clasifican como `ambiguo`

**Tools/acciones:** `entorno exportar_logs`, `sitio convertir_meteo`,
`entorno licencia_sync`.

**Reproducción**

1. `entorno {accion:"exportar_logs", carpeta_destino:"…/QA_tmp_logs"}` **sin
   crear la carpeta** → PVsyst imprime
   `C:/…/QA_tmp_logs does not exist. Please set another target folder.`
2. `sitio convertir_meteo {…}` con un CSV que PVsyst no entiende → imprime
   `Invalid date Line 6.`

**Esperado:** `estado:"error"` con el mensaje de PVsyst como `motivo`
(`not_found` / `format`).

**Real:** `clasificacion:"ambiguo"`, `success:false`,
`detected_error:"ambiguo: Salida sin confirmación de éxito ni marcador de error
concluyente"`, y el texto real solo aparece dentro de `stdout_tail`.

**Evidencia**

```json
{"clasificacion":"ambiguo","success":false,
 "stdout_tail":"C:/Users/Lenovo/PVsyst8.1_Data/QA_tmp_logs does not exist. Please set another target folder."}
{"clasificacion":"ambiguo","success":false,
 "stdout_tail":"…Running weather data conversion ...\nInvalid date Line 6."}
```

**Archivo/línea sospechosa:** `src/adapters/cli/clasificacion.rs` (tabla de
marcadores de error incompleta).

**Regresión:** **preexistente** — §8.2: `exportar_logs` a una carpeta inexistente da `clasificacion:"ambiguo"` en los dos builds.

---

### BUG-10 · P2 · `workspace exportar`/`importar` resuelven rutas relativas contra bases distintas

**Tool/acción:** `workspace exportar`, `workspace importar`.

**Reproducción**

1. `workspace {accion:"exportar", destino:"QA_tmp/small.zip",
   workspace:"…/QA_WS_Small"}` → escribe en
   `…/QA_WS_Small/QA_tmp/small.zip` (dentro del workspace exportado).
2. `workspace {accion:"importar", zip:"QA_tmp/small.zip",
   destino:"QA_WS_Import", workspace:"…/PVsyst8.1_Data_MCP_TEST"}` →
   `Zip not found: …\PVsyst8.1_Data_MCP_TEST\QA_tmp/small.zip`.

**Esperado:** la misma base para las dos (la del argumento `workspace`), o un
error que diga contra qué base se resolvió.

**Real:** la ida y la vuelta solo funcionan con rutas absolutas. Además el zip
exportado **contiene su propio temporal** (`QA_tmp/small.zip.mcp_tmp_20960.zip`),
porque se escribe dentro del árbol que recorre.

**Evidencia**

```
exportar → {"zip":"…/QA_WS_Small\\QA_tmp/small.zip","size_bytes":6430}
importar → Zip not found: …\PVsyst8.1_Data_MCP_TEST\QA_tmp/small.zip
zip listing → QA_tmp/small.zip.mcp_tmp_20960.zip (0 bytes)
```

**Archivo/línea sospechosa:** `src/admin/workspace.rs:112-140` (`destino_zip`
usa el workspace exportado) frente a `importar_workspace` (usa el `workspace`
de la llamada).

**Regresión:** **preexistente** — §8.2: los dos builds resuelven `destino` contra el workspace exportado.

---

### BUG-11 · P2 · `editar campos` inserta campos desconocidos sin avisar

**Tool/acción:** `editar accion:campos`.

**Reproducción**

1. `editar {accion:"campos", archivo:"Sites/Barrancabermeja.SIT",
   objeto:"pvSite", campos:{NoExiste:1}}`.
2. `leer` el archivo: `NoExiste=1` está escrito.

**Esperado:** rechazo (o al menos `avisos` + `estado:"parcial"`), porque un
typo (`FieldTitl` por `FieldTilt`) produce un archivo que PVsyst puede
rechazar.

**Real:** `action:"inserted"` y `estado:"ok"`. En cambio `componente editar`
**sí** rechaza campos que el componente no tiene: la política es incoherente
entre las dos tools.

**Evidencia**

```json
{"changes":[{"action":"inserted","key":"NoExiste","new":"1","old":null}],
 "vista":{"estado":"ok","resumen":"Campos editados con copia en el historial."}}
// contraste:
componente editar {campos:{NoExiste:1}} → "Campos que el componente no tiene: [\"NoExiste\"]…"
```

**Archivo/línea sospechosa:** `src/format/edit.rs` (aplicación de cambios sin
verificar que la clave exista) frente a `src/components/…` (que sí verifica).

**Regresión:** **preexistente** — §8.2: los dos builds insertan el campo desconocido (`action:"inserted"`).

---

### BUG-12 · P2 · El contador de ejecuciones TRIAL no es fiable

**Tool/acción:** `entorno licencia`, `costo_estimado.restantes`.

**Reproducción**

1. 01:43 `entorno licencia` → `Remaining days: 30`, `Remaining executions: 134`.
2. 01:51 `sitio crear` (real) → `license:"…remaining days: 60, remaining
   executions: 250"`, `costo.restantes:249`.
3. 01:52 `simular ejecutar dry_run` → `costo_estimado.restantes:133`.
4. 01:53 `simular ejecutar` (real) → `costo.restantes:247`.
5. 02:10 `entorno licencia` → `Remaining days: 60`, `Remaining executions: 250`
   tras ≈11 ejecuciones.

**Esperado:** un contador monótono decreciente.

**Real:** oscila entre dos estados (133/134 y 249/250) sin activación de
licencia. Un agente no puede presupuestar con `restantes`.

**Evidencia**

```
{"license":"Status: TRIAL","stdout_tail":"Remaining days: 30\nRemaining executions: 134"}   // 01:43
{"costo":{"ejecuciones":1,"restantes":249},"license":"…remaining executions: 250"}          // 01:51
{"costo_estimado":{"ejecuciones":1,"restantes":133}}                                        // 01:52
{"costo":{"ejecuciones":1,"restantes":247}}                                                 // 01:53
{"stdout_tail":"Remaining days: 60\nRemaining executions: 250"}                             // 02:10
```

**Archivo/línea sospechosa:** `src/adapters/ledger.rs` (lee el contador de dos
fuentes: `lic-info` y la cabecera de cada ejecución).

**Regresión:** **preexistente** — §8.3: el ledger de costo entra en `7e46870` (25-oct), antes del merge-base.

---

### BUG-13 · P2 · `leer` sugiere un parámetro que no existe (`completo`)

**Tool/acción:** `leer`.

**Reproducción:** `leer {archivo:"Projects/_DEMO_Commercial.VC0"}` →
`hint:"Pass completo:true for the full tree, or leer with 'objeto' for one
object."`. `leer` no declara `completo`; el parámetro es `detalle`.

**Esperado:** `hint` con `detalle:"completo"`.

**Real:** un agente que siga el `hint` recibe `ignorados:["completo"]` (o nada).

**Evidencia**

```json
{"hint":"Pass completo:true for the full tree, or leer with 'objeto' for one object.",
 "vista":{"estado":"ok","resumen":"Archivo de PVsyst leído."}}
```

**Archivo/línea sospechosa:** `src/projects/mod.rs:216`.

**Regresión:** **preexistente** — §8.2: el `hint` «Pass completo:true» es idéntico en los dos builds.

---

### BUG-14 · P2 · `escribir_archivo` sobrescribe por defecto; `sobrescribir:true` no hace nada

**Tool/acción:** `escribir_archivo`.

**Reproducción**

1. `escribir_archivo {archivo:"QA_tmp/nota.txt", contenido:"hola QA\n"}` → ok.
2. `escribir_archivo {archivo:"QA_tmp/nota.txt", contenido:"otra vez\n"}` (sin
   `sobrescribir`) → **ok**, el archivo pasa a `otra vez`.
3. `escribir_archivo {…, sobrescribir:true}` → idéntico.
4. `escribir_archivo {…, sobrescribir:false}` → rechazo correcto.

**Esperado:** que el nombre del parámetro signifique lo que dice (opt-in), o
que la descripción de la tool lo advierta.

**Real:** el valor por defecto es sobrescribir. El comentario del código lo
asume (`«Overwriting is this tool's default»`), pero ni el esquema ni la
descripción lo dicen; el historial guarda copia, así que la pérdida es
recuperable.

**Evidencia**

```json
{"bytes_written":9,"historial_id":"20261108_064730_188_0001","path":"…/QA_tmp/nota.txt"}
// el archivo quedó con "otra vez"
```

**Archivo/línea sospechosa:** `src/workspace/escritura.rs:20-36`
(`exigir_sobrescritura`: `if pide != Some(false) … return Ok(())`).

**Regresión:** **preexistente en el comportamiento** — §8.3: `8940569` (6-nov, dentro del PR) añade el veto `sobrescribir:false`, pero el default de sobrescribir ya existía antes; el PR mejora el caso `false` y deja `true` como no-op.

---

### BUG-15 · P3 · `componente crear` exige `cambios` aunque la descripción no lo diga

**Tool/acción:** `componente accion:crear`.

**Reproducción:** `componente {accion:"crear", plantilla:"MCP_Test_400",
nombre_nuevo:"QA_PAN_450", tipo:"PAN"}` → `Missing: cambios. Example:
{"accion":"crear","cambios":"<cambios>","nombre_nuevo":"<nombre_nuevo>","plantilla"…}`.

**Esperado:** copiar la plantilla tal cual (la descripción dice «create … from a
template»).

**Real:** obliga a inventar un cambio. `requeridos_por_accion` lo declara
(`src/server/tools/componentes.rs:47-53`), pero la descripción pública no.

**Evidencia:** `{"error":"Missing: cambios. Example: {…}"}`.

**Regresión:** **preexistente** — §8.2: los dos builds responden `Missing: cambios`.

---

### BUG-16 · P3 · `gui_observar controles` devuelve `count` = total con la lista truncada

**Tool/acción:** `gui_observar que:controles`.

**Reproducción:** `gui_observar {que:"controles"}` (sin `detalle`) →
`count:31`, `controls_total:31`, `truncado:true`, y **13** elementos en
`controls`.

**Esperado:** `count:13` (devueltos), `total:31`.

**Evidencia:** `{"controls":[13 elementos],"count":31,"controls_total":31,"truncado":true}`.

**Archivo/línea sospechosa:** `src/adapters/gui/ventanas.rs:270`
(`json!({"count": controls.len(), …})` se calcula antes del recorte).

**Regresión:** **preexistente** — §8.2: el build del 6-nov también da `count:24` con 13 controles y `truncado:true`.

---

### BUG-17 · P3 · `gui_observar vista` dice que no hay ventana conocida con THomeForm abierta

**Tool/acción:** `gui_observar que:vista`.

**Reproducción:** con `THomeForm` visible, `gui_observar {que:"vista"}` →
`aviso:"ninguna ventana conocida abierta; esquemas con gui_observar
que:esquema"`, `entidad:null`.

**Esperado:** o leer THomeForm (no tiene esquema) o decir «THomeForm no tiene
esquema».

**Evidencia:** `{"aviso":"ninguna ventana conocida abierta; …","entidad":null}`.

**Regresión:** **preexistente** — §8.2: el build del 6-nov también dice «ninguna ventana conocida abierta» con THomeForm visible.

---

### BUG-18 · P3 · `gui_observar menus` falla a medias tras 11,7 s

**Tool/acción:** `gui_observar que:menus`.

**Reproducción:** `gui_observar {que:"menus", ventana:"THomeForm"}` →
`vista.estado:"parcial"`, `error:"El menú 7 de THomeForm no se abrió o la
entrada pedida no tiene submenú"`, 11,7 s.

**Esperado:** enumerar los menús que sí se abren y no tardar 12 s en fallar uno.

**Evidencia:** `{"completo":false,"elapsed_ms":11679,"error":"El menú 7 de THomeForm no se abrió…","menus":[…],"vista":{"estado":"parcial"}}`.

**Regresión:** **introducido durante el PR** — §8.3: el mensaje «no se abrió o la entrada pedida no tiene submenú» entra en `79f621b` (29-oct), dentro del rango del PR.

---

### BUG-19 · P3 · `escribir_horizonte` mete la ruta pedida dentro de `Shadings/`

**Tool/acción:** `sitio escribir_horizonte`.

**Reproducción:** `{accion:"escribir_horizonte", archivo:"Sites/QA_Horizonte.HOR",
puntos:[…]}` → `archivo:"…\Shadings/Sites/QA_Horizonte.HOR"`.

**Esperado:** `…/Shadings/QA_Horizonte.HOR` (nombre) o rechazo de una ruta con
carpeta.

**Real:** crea una carpeta `Sites` dentro de `Shadings`. `leer` la encuentra
(usa la misma lista de candidatos), así que no rompe, pero el archivo queda en
un sitio que nadie espera.

**Evidencia:** `{"archivo":"C:/Users/Lenovo/PVsyst8.1_Data_MCP_TEST\\Shadings/Sites/QA_Horizonte.HOR","bytes_escritos":171}`.

**Archivo/línea sospechosa:** `src/horizonte_archivo/aplicacion.rs:84-98`
(`resolver_destino_escritura`).

**Regresión:** **preexistente** — §8.2: los dos builds escriben en `Shadings/Sites/QA_D_HOR.HOR`.

---

### BUG-20 · P3 · `dry_run` incompleto para presupuestar

**Tools/acciones:** `sitio crear`, `disenar_sistema`.

**Reproducción**

1. `sitio {accion:"crear", … , dry_run:true}` → `{command_line, dry_run:true}`,
   **sin `costo_estimado`**.
2. `disenar_sistema {…, dry_run:true}` en frío (sin ledger previo) →
   `costo_estimado:{ejecuciones:1, restantes:null}`.

**Esperado:** `costo_estimado{ejecuciones, restantes}` en todas las acciones que
gastan TRIAL, como ya hacen `simular ejecutar` y `disenar_sistema` en caliente.

**Evidencia**

```
sitio.crear.dry_run → {"command_line":"…create-site…","dry_run":true,"vista":{…}}   // sin costo
disenar.dry_run.modulo_ext → {"costo_estimado":{"ejecuciones":1,"restantes":null}}
```

**Regresión:** **nuevo en el PR** — §8.3: en el build del 6-nov `dry_run` **se ignoraba y creaba el sitio de verdad**; el `dry_run` es nuevo y le falta el costo.

---

### BUG-21 · P3 · Mensajes de «no encontrado» que dicen algo falso

| Tool | Mensaje real | Problema |
|---|---|---|
| `leer` | `Proyecto 'Projects/NO_EXISTE.PRJ' no encontrado; no hay ninguno en …\Projects` | Sí hay 16 proyectos; el que falta es el archivo pedido. |
| `proyecto crear` (duplicado) | `Project exists: …\QA_Test.PRJ` | En inglés, en una tool con el resto en español. |
| `resultados comparar` (1 variante) | `Se necesitan al menos 2 variantes con resultados en 'Planta_Solar_50kW' (hay 1)` | Se pidió **una** variante; el mensaje sugiere que el proyecto solo tiene una. |
| `editar insertar_objeto` (falta `desde.archivo`) | `Missing required argument 'archivo'` | No dice `desde.archivo`; el llamador busca el `archivo` de nivel superior, que sí está. |

**Regresión:** **preexistente** — §8.2: los mensajes son idénticos en los dos builds.

---

### BUG-22 · P3 · `simular preparar` pierde los archivos de batch en `siguiente`

**Tool/acción:** `simular accion:preparar`.

**Reproducción:** `{accion:"preparar", proyecto:"QA_Test", filas:[{tilt:20},
{tilt:30}]}` → `{batch_params_file:"…MCP_BatchParams.csv", batch_rvt_file:null}`;
`vista.siguiente[0] = {tool:"simular", argumentos:{accion:"ejecutar",
proyecto:"QA_Test", workspace:…}}`.

**Esperado:** `siguiente` con `batch_params_file`/`batch_rvt_file` para que el
batch se ejecute de verdad.

**Real:** seguir `siguiente` lanza una simulación simple y el batch preparado se
pierde.

**Evidencia:** ver §2.10.

**Regresión:** **preexistente** — §8.2: el `siguiente` de `preparar` es idéntico en los dos builds.

---

### BUG-23 · P3 · `teclas` envía con `foco:false` en el escritorio compartido

**Tool/acción:** `gui_actuar accion:teclas`.

**Reproducción:** `{accion:"teclas", ventana:"TWorkspaceForm", teclas:"ESC"}` →
`{sent:"ESC", via:"keybd_event", foco:false, aviso:"The target is the window:
the keys reach the control that already has the focus inside it"}`; el diálogo
**no** se cerró.

**Esperado (instrucciones del servidor):** «On your own desktop it refuses to
send keys when it cannot give them the focus; on the isolated desktop it reports
`foco:false` and sends them anyway».

**Real:** en escritorio compartido envía igualmente con `foco:false` y
`vista.estado:"ok"`.

**Regresión:** **introducido durante el PR** — §8.3: `68154ff` (28-oct) añade el campo `foco` y el envío pese a `foco:false`; dentro del rango del PR.

---

### BUG-24 · P3 · `clic_fisico` ignora las coordenadas y invoca la ventana

**Tool/acción:** `gui_actuar accion:clic_fisico`.

**Reproducción:** `{accion:"clic_fisico", ventana:"TWorkspaceForm",
objetivo:{x:1253,y:1146}}` → `{invoked:"TWorkspaceForm", window.visible:true}`;
el botón «Cerrar» de esa posición no recibe el clic.

**Esperado:** clic en el punto pedido (o error si el objetivo no es un control).

**Evidencia:** `{"action":"clic_fisico","hwnd":"0x3E08B6","invoked":"TWorkspaceForm",…}`.

**Regresión:** **preexistente** — §8.3: `clic_fisico` viene de `eeede5e` (19-oct), antes del merge-base.

---

### BUG-25 · P3 · `componente editar` acepta `campos`, que no declara

**Tool/acción:** `componente accion:editar`.

**Reproducción:** `{accion:"editar", componente:"QA_PAN_450",
campos:{PNom:999}}` → aplica el cambio (`PNom` 455 → 999).

**Esperado:** o `campos` es alias declarado de `cambios` (y aparece en el
esquema), o se rechaza.

**Real:** funciona pero no está en `properties`, así que un cliente que valide
el esquema lo bloquea; y `consumidos_por_accion` declara `campos` (que no es
propiedad) en vez de `cambios`.

**Archivo/línea sospechosa:** `src/server/tools/componentes.rs:47-53`.

**Regresión:** **introducido durante el PR** — §8.3: `consumidos_por_accion` con la clave `campos` entra en `9043410` (7-nov). El comportamiento no se pudo replicar en la comparación (faltaba el componente), pero el commit es del PR.

---

### BUG-26 · P3 · `sitio crear origen:csv` no informa del `.SIT` esperado

**Tool/acción:** `sitio accion:crear` (`origen:"csv"`).

**Reproducción:** `{accion:"crear", origen:"csv", …, weather_data_file:"…"}` →
`expected_sit:null` (con `origen:"coordenadas"` sí trae la ruta).

**Evidencia:** `{"expected_sit":null,"site_name":"QA_SiteCSV","costo":{…}}`.

**Regresión:** **preexistente** — §8.3: `expected_sit` viene de `eeede5e` (19-oct), antes del merge-base.

---

### BUG-27 · P3 · `entorno preferencias` devuelve `vista.campos` vacío

**Tool/acción:** `entorno accion:preferencias`.

**Reproducción:** `{accion:"preferencias"}` → `secciones:{…24…}` pero
`vista.campos:[]`.

**Esperado:** al menos `archivo` y el número de secciones como campos.

**Evidencia:** `{"archivo":"…PVsyst.ini","secciones":{…},"vista":{"campos":[]}}`.

**Regresión:** **preexistente** — §8.2: los dos builds devuelven `vista.campos:[]` en `preferencias`.

---

## 4. Optimización para reducir llamadas

Medido sobre las mismas tareas de este informe.

### 4.1 Flujos actuales vs propuestos

| Flujo | Llamadas hoy | Llamadas propuestas | Ahorro |
|---|---:|---:|---:|
| «Dame los resultados de un proyecto» | `resultados kpis` + `mensual` + `perdidas` = **3** (763 ms, 3,1 KB) | `resultados resumen` = **1** (708 ms, 1,7 KB) | **2 llamadas (66 %)**, −45 % bytes |
| «Diseña y dime qué salió» | `disenar_sistema` + `resultados kpis` + `resultados perdidas` = **3** | `disenar_sistema` (con `kpis` ya incluido) + `resultados resumen` = **2** | 1 llamada |
| «Crea el proyecto y simúlalo» | `proyecto crear` → `proyecto doctor` → `simular ejecutar` → `resultados kpis` = **4** | `disenar_sistema`/`proyecto configurar` con simulación + `resultados resumen` = **2** | 2 llamadas |
| «Mira qué hay en el workspace» | `listar proyectos` + `listar archivos` + `listar modulos` = **3** | `listar que:proyectos` con `include_variants:true` + `que:archivos` (los módulos ya vienen en `templates`/`modules`) = **2** | 1 llamada |
| «Edita un campo y compruébalo» | `editar campos` + `leer` + `workspace historial` = **3** | `editar campos` (ya devuelve `verified_after_write` y `historial_id`) = **1** | **2 llamadas** |
| «Prepara un batch y ejecútalo» | `simular preparar` + `simular ejecutar` = **2** | `simular ejecutar` con `filas` (prepara y ejecuta) = **1** | 1 llamada |
| «Exporta el workspace y reimpórtalo» | `workspace exportar` + `workspace importar` + `listar archivos` ×2 = **4** | `workspace exportar` (con `verificar:true`) + `importar` (con `comparar:true`) = **2** | 2 llamadas |
| «Estado del sistema antes de gastar» | `entorno diagnostico` + `entorno licencia` = **2** (6,4 s) | `entorno diagnostico` con `licencia` incluida = **1** | 1 llamada y ~6 s |
| «Abre PVsyst y mira la pantalla» | `gui_actuar lanzar` + `gui_observar ventanas` + `controles` + `capturar` = **4** | `gui_actuar lanzar` (ya devuelve `windows`) + `gui_observar que:vista` = **2** | 2 llamadas |
| «Confirma que un diálogo se cerró» | `cerrar` + `esperar_cierre` + `ventanas` = **3** | `cerrar` con `verificar:true` = **1** | 2 llamadas |

**Ahorro agregado en los 10 flujos:** de 31 a 16 llamadas (**−48 %**).

### 4.2 Acciones que se pueden fusionar

1. **`editar rvt` y `editar plantilla_rvt`** son el mismo handler
   (`src/server/tools/workspace_ejecucion.rs:196`: `"rvt" | "plantilla_rvt" =>
   gestionar_rvt`). Una sola acción `rvt` con `operacion` basta.
2. **`entorno licencia_activar` / `licencia_desactivar`** no ejecutan nada:
   devuelven una cadena. Se pueden colapsar en `licencia` con
   `accion:"comando_activar"|"comando_desactivar"`, o mejor, en un campo
   `comando_sugerido` de `entorno licencia`.
3. **`listar que:modulos`** es un caso particular de `que:componentes` con
   `tipo:"PAN"` (y `que:componentes` ya devuelve `pnom_w` si se pide). Fusionar
   ahorra una acción y un modo de fallo (`tipo` obligatorio en uno y no en el
   otro).
4. **`sitio escribir_horizonte` + `sitio escribir_serie`** comparten
   `resolver_ruta` y validación de puntos/valores; podrían ser
   `sitio escribir {que:"horizonte"|"serie"}`.
5. **`gui_observar que:vista` y `que:formulario`**: `vista` ya cae a OCR cuando
   no hay esquema; `formulario` podría ser `vista` con `ocr:true`.

### 4.3 Defaults que ahorran una llamada

| Tool | Hoy | Propuesta |
|---|---|---|
| `resultados` | `que:kpis` | `que:resumen` (kpis+mensual+pérdidas; ya existe y es más barato que las tres) |
| `listar` | 13 elementos | 25 (la mitad de los `count` reales son 16-20; hoy obliga a un segundo `detalle:normal`) |
| `gui_observar` | 13 controles | 25, o `detalle:normal` por defecto cuando `count ≤ 50` |
| `leer` | resumen | `detalle:"normal"` para `.SIT`/`.MET` (hoy el resumen ya trae la tabla mensual, así que no cambia) |
| `entorno` | `diagnostico` sin licencia | incluir `licencia` (6,4 s) solo si `detalle:"completo"` |

### 4.4 `vista.siguiente`: calidad y coste

- **Bien:** la mayoría de los `siguiente` son ejecutables tal cual (se validaron
  antes de enviarse) y arrastran `workspace`.
- **Mal:** `simular preparar` pierde los archivos de batch (BUG-22);
  `proyecto crear` sugiere `dimensionar {proyecto,n_mod_serie,n_strings}` sin
  decir que sin los n's se abre la GUI; `workspace exportar` sugiere
  `listar que:proyectos` con el workspace exportado (útil, pero no el paso
  natural: importar).
- **Coste:** el bloque `vista` añade 0,5–1,3 KB por llamada (medido: 49 % más
  que sin él, según `docs/vista_agente_salidas.md`). En `resultados resumen`
  son 1,7 KB de los cuales ~0,6 KB son `vista`.

### 4.5 Batch, caché y plan

- **Batch:** `proyecto configurar` ya aplica N pasos en 1 llamada (probado con
  3). Faltaría `resultados que:resumen` para N variantes y `listar` para N
  tipos de componente.
- **Caché:** `entorno licencia` tarda 6,4 s y se llama en cada flujo; el
  resultado podría cachearse 60 s (el contador ya es inconsistente, BUG-12).
  `gui_observar que:esquema` devuelve 33,9 KB y es estático: cacheable por
  versión de PVsyst.
- **`dry_run`/plan:** `disenar_sistema` y `simular` ya lo tienen; faltaría en
  `sitio crear` (BUG-20) y `proyecto anadir` (hoy escribe sin ensayo).
- **Sondas del CLI:** la sesión registró **24 `help` + 7 `version` + 13 comandos
  vacíos** (`cli-session-2026-11-08.json`). Cada uno es un proceso (~0,2 s).
  Cachear la firma de `help`/`version` por binario ahorraría ~30 procesos por
  sesión.

---

## 5. Eficiencia e instrucciones redundantes

### 5.1 Texto duplicado

| Texto | Dónde aparece | Repeticiones | Comentario |
|---|---|---|---|
| «Los resultados deben venir de PVsyst; no inventes cifras» | `instructions` del servidor, `SKILL.md`, `docs/cobertura.md`, `README.md`, descripciones de `simular`/`resultados` | 5 | Correcto como principio, pero se puede dejar en **un** sitio (`instructions`) y que las tools lo referencien. |
| «Componentes por NOMBRE, no por ruta» | `instructions`, descripción de `componente`, `SKILL.md` | 3 | La descripción de `componente` lo repite entero. |
| «`dry_run` solo lo aceptan estas acciones…» | `instructions` (lista de 10) y `docs/cobertura.md` | 2 | La lista se desincroniza: `sitio horizonte_pvgis` rechaza `dry_run` pero está en la lista de `instructions`. |
| «listas truncadas a 13; usa `detalle:normal`/`completo`» | `instructions`, `hint` de `listar`, `vista.recortado[].pedir_resto` | 3 | El `hint` y `pedir_resto` dicen `completo` cuando `normal` basta. |
| «`que:resumen` devuelve KPIs+mensual+pérdidas en una llamada» | `instructions`, descripción de `resultados`, `docs/cobertura.md`, `SKILL.md` | 4 | Útil, pero es la misma frase cuatro veces. |
| «no modifiques `C:\Program Files\PVsyst8.1.8`» | `SKILL.md`, `README.md`, `instructions` | 3 | — |
| «Meteonorm/MN9…» | `instructions`, error de `disenar_sistema`, `docs/cobertura.md` | 3 | El error ya lo explica; sobra en `instructions`. |

**Medición:** `instructions` = 2 670 caracteres; descripciones de las 15 tools =
2 628; esquemas = 31 134. El `tools/list` completo pesa ~36 KB y se envía en
cada sesión.

### 5.2 Claves declaradas pero ignoradas

El servidor ya anota `ignorados` en algunos casos (`entorno` con `workspace` →
`ignorados:["workspace"]`), pero no en todos:

| Tool | Clave declarada | Se ignora en | Evidencia |
|---|---|---|---|
| `entorno` | `workspace` | todas las acciones | `ignorados:["workspace"]` ✔ (bien) |
| `entorno` | `clave` | `licencia_activar`/`licencia_desactivar` | se pasa `clave:"XXXX-0000"` y la respuesta es idéntica; **sin** `ignorados` |
| `entorno` | `n`, `solo_errores` | todas menos `logs_cli` | sin anotar |
| `componente` | `campos` (no declarada) | `editar` | funciona como alias implícito (BUG-25) |
| `leer` | `tipo`, `max_bytes` | rama `serie:true` | **sí** se anota (`rama(... &["tipo","max_bytes"])`) ✔ |
| `sitio` | `input_csv_file` | `crear origen:csv` (usa `weather_data_file`) | el nombre del esquema no es el que se lee (P3) |
| `simular` | `abrir_gui`, `segundo_plano`, `idioma_reporte`, `paginas_reporte` | `ejecutar` por CLI | sin anotar |
| `proyecto` | `comentario`, `sitio`, `variante_origen` | varias acciones | sin anotar |
| `gui_actuar` | `verificar`, `cancelar`, `cerrar_demo` | la mayoría | sin anotar |

### 5.3 Payloads grandes

| Llamada | Bytes | Por qué |
|---|---:|---|
| `gui_observar que:esquema` | 33 886 | 5 entidades × ~20 campos, con `clave_archivo`, `unidad`, `presente` |
| `proyecto anadir domo` | 6 650 | `insertados[]` con todos los campos del objeto |
| `disenar_sistema` (real) | 6 125 | `pasos[]` con `detail` de cada paso + `vista` |
| `listar proyectos detalle:normal` | 5 147 | 16 proyectos × 7 campos |
| `gui_observar que:uia` | 5 499 | árbol completo |
| `gui_observar que:arbol` | 5 035 | idem |

`vista` añade 0,5–1,3 KB **por copia** (va en `content` y en
`structuredContent`, así que se duplica en el cable).

### 5.4 Latencias (medidas)

| Tool | n | media | máx | Nota |
|---|---:|---:|---:|---|
| `listar` | 21 | 13 ms | 54 ms | |
| `leer` | 16 | 12 ms | 38 ms | |
| `editar` | 16 | 48 ms | 183 ms | |
| `escribir_archivo` | 10 | 21 ms | 50 ms | |
| `workspace` | 19 | 106 ms | 765 ms | |
| `resultados` | 24 | 668 ms | 10 044 ms | `comparar` es el caro |
| `entorno` | 20 | 987 ms | 6 193 ms | `licencia` |
| `proyecto` | 57 | 1 283 ms | 38 868 ms | `dimensionar` GUI |
| `disenar_sistema` | 23 | 1 507 ms | 13 624 ms | |
| `componente` | 26 | 1 693 ms | 43 770 ms | `crear_desde_ficha` GUI |
| `sitio` | 33 | 1 759 ms | 19 312 ms | `meteo_pvgis` |
| `simular` | 19 | 4 153 ms | 48 473 ms | simulación real |
| `gui_actuar` | 52 | 1 349 ms | 17 200 ms | `lanzar`, `menu` |

p50 global 10 ms; p95 5 172 ms. Nada supera el `timeout` de 600 s del cliente.

### 5.5 Validaciones repetidas

- `workspace` se valida contra la raíz en **cada** tool (13 tools aceptan
  `workspace`), y `entorno`/`pvsyst_cli` no lo aceptan: la regla no es
  uniforme y obliga a recordar cuáles sí.
- La comprobación «¿existe el proyecto?» se hace en `proyecto`, `simular`,
  `resultados`, `editar` y `disenar_sistema`, cada una con su mensaje
  (`Proyecto 'X' no encontrado; no hay ninguno en …` vs `Project exists: …`).
- `permitir_fuera_del_workspace` se exige en `componente exportar/importar` pero
  `workspace crear` acepta rutas absolutas sin él (decisión razonable, pero
  inconsistente desde fuera).

---

## 6. Recomendaciones priorizadas

### P0 (antes de publicar el PR)

Ninguna. No se encontró pérdida de datos irreversible ni fuga de seguridad.

### P1

| # | Recomendación | Impacto | Esfuerzo |
|---|---|---|---|
| 1 | **Arreglar `variant_kwp`** (BUG-01): calcular el kWp del array con `NModSerie × NStringCh × PNom(módulo)` (o leer el «Pnom» del informe) y usar el mismo valor en `kpis`, `perdidas`, `mensual` y `entregable`. | Elimina un KPI falso en el entregable (−26,7 % de error en producción específica). | S (1 archivo, 1 test) |
| 2 | **Clasificar el crash del CLI** (BUG-02): leer `logs_cli`/`nb_crash` y devolver `error.codigo:"crash"` con el mensaje; no consumir reintentos automáticos. | Evita que un agente queme TRIAL en bucle y oculte un fallo duro. | S |

### P2

| # | Recomendación | Impacto | Esfuerzo |
|---|---|---|---|
| 3 | **Hacer que `vista` no mienta** (BUG-03): `esperar_cierre` → `parcial` si `cerrado:false`; `clic_texto` → `parcial` si `coincidencias:0`; `teclas` → `parcial` si `foco:false`. | Restaura la confianza en el contrato que el PR introduce. | S |
| 4 | **Propagar el error de simulación en `disenar_sistema`** (BUG-04): `pasos[].detail` con `stdout_tail`/`avisos_pvsyst`. | Un fallo deja de ser «Checking license…». | S |
| 5 | **Arreglar `proyecto p50_p90`** (BUG-05): lanzar PVsyst con el workspace real (como `dimensionar`) o rechazar la acción si la instancia está en PRUEBA. | Desbloquea una acción hoy inservible. | M |
| 6 | **(P3)** **Alinear la descripción de `proyecto_comparable`** (BUG-06): documentar `null` en proyecto nuevo y quitar el caso `false` inalcanzable (o permitir un diseño sin n's con `via:"aproximacion"`). | La descripción deja de prometer un estado imposible. | XS |
| 7 | **Aceptar `modulo` con y sin extensión** (BUG-07) y **validarlo en `dry_run`**. | Evita un fallo tardío tras crear sitio y proyecto. | S |
| 8 | **Alinear `via` con el esquema** (BUG-08): añadir `aproximacion` al enum o cambiar el mensaje. | Quita una salida imposible. | XS |
| 9 | **Completar la tabla de errores del CLI** (BUG-09): `does not exist`, `Invalid date`, `cannot be run`, `not valid` → `error` con el texto. | Convierte «ambiguo» en diagnóstico accionable. | S |
| 10 | **Unificar la base de las rutas de `workspace exportar/importar`** (BUG-10) y excluir el zip de sí mismo. | Ida y vuelta con rutas relativas. | S |
| 11 | **`editar campos` no debe inventar campos** (BUG-11): rechazar claves ausentes salvo `reemplazar:true`/`nuevo:true`. | Evita corromper archivos por un typo. | S |
| 12 | **Contador TRIAL único y monótono** (BUG-12): una sola fuente (`lic-info`) y `restantes` siempre presente. | Permite presupuestar. | M |
| 13 | **Corregir el `hint` de `leer`** (BUG-13): `detalle:"completo"`. | Quita una llamada fallida. | XS |
| 14 | **Documentar el default de `escribir_archivo`** (BUG-14) o invertirlo a opt-in. | Evita sobrescrituras sorpresa. | XS |

### P3

| # | Recomendación | Impacto | Esfuerzo |
|---|---|---|---|
| 15 | `componente crear` sin `cambios` (BUG-15). | Copiar plantillas sin inventar cambios. | XS |
| 16 | `count` = devueltos en `gui_observar controles` (BUG-16). | Métrica honesta. | XS |
| 17 | `gui_observar vista` con THomeForm (BUG-17) y `menus` sin 12 s de espera (BUG-18). | Menos confusión y menos latencia. | S |
| 18 | `escribir_horizonte` sin prefijo duplicado (BUG-19). | Ruta predecible. | XS |
| 19 | `costo_estimado` en `sitio crear dry_run` y `restantes` no nulo (BUG-20). | Presupuesto antes de gastar. | S |
| 20 | Mensajes de «no encontrado» precisos (BUG-21). | Menos llamadas de sondeo. | XS |
| 21 | `siguiente` de `simular preparar` con los archivos de batch (BUG-22). | El batch se ejecuta siguiendo la pista. | XS |
| 22 | `teclas` coherente con las instrucciones (BUG-23) y `clic_fisico` con `objetivo` (BUG-24). | GUI predecible. | S |
| 23 | Declarar `campos` como alias de `cambios` (BUG-25) y `weather_data_file` en `crear csv` (BUG-26). | Esquema = realidad. | XS |
| 24 | `vista.campos` en `entorno preferencias` (BUG-27). | Consistencia del contrato. | XS |

### Recomendaciones de producto (no bugs)

1. **`resultados que:resumen` como default** de `resultados` (ahorra 2 de cada
   3 llamadas del flujo más común).
2. **Cachear `entorno licencia` 60 s** y `gui_observar que:esquema` por versión.
3. **Cachear la firma de `help`/`version` del CLI** (24+7 procesos por sesión).
4. **Fusionar `rvt`/`plantilla_rvt`** y `licencia_activar`/`desactivar`.
5. **`detalle:normal` como default de `listar`** (13 → 50): la mitad de los
   listados reales tienen 16-20 elementos y hoy exigen una segunda llamada.
6. **Un `pvsyst://estado`** que devuelva diagnóstico + licencia + contador en un
   recurso (los recursos no cuestan una llamada de tool).

---

## 7. Anexo

### 7.1 Entorno

| Elemento | Valor |
|---|---|
| SO | Windows 11 (26H2, build 26300.9818) |
| PVsyst | 8.1.8 (`C:\Program Files\PVsyst8.1.8\PVsyst.exe`, `PVsystCLI.exe`) |
| Workspace real | `C:\Users\Lenovo\PVsyst8.1_Data` (395 MB) |
| Workspace de pruebas | `C:\Users\Lenovo\PVsyst8.1_Data_MCP_TEST` (copia, borrada) |
| `PVsyst.ini` | `C:\Users\Lenovo\AppData\Local\PVsyst\8.1\Admin\PVsyst.ini` (UTF-16 LE, 12 878 B) — backup en `.qa\PVsyst.ini.bak`, restaurado byte a byte |
| MCP | `C:\pvsyst-mcp\pvsyst-mcp.exe`, 5 015 552 B, 2026-11-07 21:31 |
| SHA-256 | `501ec8ffabc020fb720bb58b16f9f3ff362dd97fb6a8268b2977e68bffa8abc0` |
| Código | worktree `.claude/worktrees/tarea-d-fila`, rama `worktree-tarea-d-fila`, HEAD `5dbfdd0` |
| Cliente | `mcp__pvsyst__*` (78 llamadas) + arnés stdio propio (273 llamadas), mismo binario y mismo entorno |
| `PVSYST_MCP_APROXIMACIONES` | desactivado salvo en 8 llamadas explícitas (marcadas «aprox.») |
| `PVSYST_MCP_ESCRITORIO` | `compartido` (no se usó el escritorio aislado) |

### 7.2 Presupuesto TRIAL

| Momento | Valor reportado | Fuente |
|---|---|---|
| Inicio (01:43) | 30 días / **134** ejecuciones | `entorno licencia` |
| Tras `sitio crear` (01:51) | 60 días / **250** | salida de PVsystCLI |
| `costo.restantes` mínimo observado | **133** | `simular ejecutar dry_run` |
| `costo.restantes` tras simulaciones | 247 / 249 | `simular ejecutar` |
| Final (02:10) | 60 días / **250** | `entorno licencia` |

**Ejecuciones realmente gastadas (según `costo.ejecuciones` y la cabecera de
licencia de cada corrida): ≈11.**

1. `sitio crear` por coordenadas (QA_Site_Santiago)
2. `sitio convertir_meteo` (falló, pero cobró)
3. `simular ejecutar` (QA_Sim)
4. `simular ejecutar` (QA_Eco)
5. `disenar_sistema` real (Sistema_500kWp)
6. `disenar_sistema` real (QA_Diseno)
7. `disenar_sistema` real (QA_Diseno3)
8. `simular ejecutar` (QA_Diseno3, crash)
9. `disenar_sistema` real (QA_Diseno4)
10. `sitio meteo_pvgis` (QA_PVGIS)
11. `sitio crear` por CSV (QA_SiteCSV)

**Se reporta el exceso sobre el límite blando de 10.** No se pudo cuadrar con el
contador porque es inconsistente (BUG-12).

### 7.3 Log resumido de llamadas

Ficheros: `.qa/out1.jsonl` … `.qa/out64.jsonl` (arnés) y `.qa/tools.json`
(`tools/list` completo). Cada línea:

```json
{"label":"…","tool":"…","args":{…},"wall_ms":123,"response":{"jsonrpc":"2.0","id":7,
 "result":{"content":[{"type":"text","text":"{…}"}],"structuredContent":{…},"isError":false}}}
```

| Tool | Llamadas | Ficheros |
|---|---:|---|
| `proyecto` | 57 | `out8`, `out10`, `out25`, `out30`–`out33`, `out42`–`out43`, `out52`–`out53`, `out58`, `out60` |
| `sitio` | 33 | `out17`–`out23`, `out63`–`out64` |
| `componente` | 26 | `out15`–`out16`, `out42`–`out43`, `out58`, `out61`–`out62` |
| `resultados` | 24 | `out28`, `out34`–`out35`, `out54`–`out57` |
| `disenar_sistema` | 23 | `out36`–`out51` |
| `listar` | 21 | `out5`–`out6`, `out10`, `out44` |
| `entorno` | 20 | `out1`–`out4`, `out20`, `out47`, `out59` |
| `workspace` | 19 | `out12`–`out14` |
| `simular` | 19 | `out24`–`out27`, `out33`, `out46`, `out54`–`out57` |
| `editar` | 16 | `out9`–`out10`, `out43` |
| `leer` | 16 | `out7`, `out23`–`out24`, `out29` |
| `escribir_archivo` | 10 | `out11`, `out13` |
| **Total** | **273** | |

### 7.4 Limpieza

| Elemento | Acción | Estado |
|---|---|---|
| `C:\Users\Lenovo\PVsyst8.1_Data_MCP_TEST` | borrado | ✔ |
| Proyectos `QA_*` y `Sistema_500kWp` (8) | `proyecto eliminar` + purga de `UserRecycleBin` | ✔ |
| Sitios `QA_*_Site.SIT` (7) | borrados | ✔ |
| `QA_PAN_400.PAN`, `QA_PAN_450.PAN`, `QA_Ficha.PAN`, `QA_PAN_Import.PAN` | `componente borrar` + purga | ✔ |
| `Results/QA_*` (6), `Meteo/QA_*` (3), `QA_tmp_logs/` | borrados | ✔ |
| `C:\Windows\Temp\QA_WS` | borrado | ✔ |
| `C:\Users\Lenovo\Desktop\QA_PAN_450.PAN` | borrado | ✔ |
| `PVsyst.ini` | restaurado desde backup (idéntico byte a byte) | ✔ |
| PVsyst | `cerrar_todo` + `Stop-Process` (pids 26508, 14036, 7160) | ✔ cerrado |
| `Meteo/.staging` | borrado | ✔ |

**No se pudo limpiar:** nada pendiente. Los `.bak` que PVsyst crea al guardar
variantes (`*.VC0.bak`) son de proyectos del usuario y no se tocaron.

### 7.5 Cobertura no ejercitada (honesta)

| Acción | Motivo |
|---|---|
| `gui_actuar fila` (ruta feliz) | requiere un formulario con geometría de fila medida y filas cargadas (`TProjectLoadForm`); no se abrió un proyecto del usuario en la GUI. |
| `gui_actuar escribir`/`seleccionar`/`pagina`/`campo`/`rellenar` (ruta feliz) | requieren un formulario con `TEdit`/`TComboBox`/pestañas; los formularios accesibles sin cargar un proyecto no tienen ninguno. Se ejercitó la validación, que es la ruta segura. |
| `gui_actuar escena`/`escena_leer`/`escena_importar` (ruta feliz) | requieren el editor 3D abierto, que exige un proyecto cargado en la GUI. Se ejercitó la validación y el mensaje de guía. |
| `proyecto economia` (ruta feliz) | requiere el bloque `EconomicEval`, que solo se crea en la GUI. |
| `proyecto bombeo` (ruta feliz) | no hay proyecto de bombeo en el workspace; la validación de tipo sí se probó. |
| `proyecto almacenamiento` (ruta feliz) | no hay `.BTR` en el workspace. |
| `disenar_sistema` real con simulación completa | Meteonorm 9 sin `DataMN9/station.db.enc` y módulos sin dimensiones válidas. |
| `resultados medido_vs_simulado` como dato PVsyst | es una aproximación del MCP (gate); se probó con el gate activo y se etiqueta como tal. |
| `pvsyst_cli` | excluida a propósito por el encargo. |

---

## 8. ¿Regresión o preexistente? (prueba diferencial)

El encargo pedía distinguir lo nuevo de lo viejo. Se hizo una **prueba
diferencial** contra el build anterior instalado
(`pvsyst-mcp.exe.old-20261107213119.exe`, 2026-11-06 01:47), con el mismo
arnés y los mismos fixtures.

### 8.1 Procedencia del binario probado

| Hecho | Dato |
|---|---|
| Binario | 2026-11-07 **21:31** |
| Último commit del PR | `5dbfdd0` — 2026-11-07 **21:34** |
| Commit anterior | `40c212f` — 2026-11-07 **21:27** (`fix(revision): apply the ultrareview #2 findings`) |
| Qué toca `5dbfdd0` | **solo `tools/generar_contrato.py`** (Python, 49+/24−) |
| ¿Está `40c212f` en el binario? | Sí: `proyecto_comparable` (introducido en ese commit) aparece en la respuesta |
| Worktree | `worktree-tarea-d-fila` limpio, HEAD `5dbfdd0` |

**Conclusión:** el binario se construyó entre `40c212f` (21:27) y `5dbfdd0`
(21:34). Como `5dbfdd0` solo cambia un generador de contrato en Python, el
binario es **equivalente en comportamiento al HEAD `5dbfdd0`**. Sí se probó la
versión con todos los commits del PR #2.

**Matiz importante:** el PR #2 tiene **257 commits** desde su merge-base
(`81b4f59`, 2026-10-27). Ninguno de los binarios de respaldo
(`pvsyst-mcp-0.3.0-anterior.old.exe` del 20-oct, `.old-20261024235842` del
24-oct, `.old-20261101212903` del 1-nov, `.old-20261107213119` del 6-nov) es un
build de `main`: **los cuatro exponen las mismas 15 tools con los mismos
esquemas**. Es decir, no hay en disco un binario «antes del PR #2»; la línea de
desarrollo del PR viene de antes del 20-oct.

### 8.2 Resultado de la comparación (6-nov vs 7-nov)

| Bug | Comportamiento 6-nov | Comportamiento probado | Veredicto |
|---|---|---|---|
| BUG-01 kWp = `PNomSizing` | `kwp:400.0`, `esp:2178.1` | idéntico | **preexistente** |
| BUG-03 `esperar_cierre` ok con `cerrado:false` | `cerrado:false` + `vista.estado:"ok"` | idéntico | **preexistente** |
| BUG-07 `modulo` sin extensión | `Módulo 'MCP_Test_400' no encontrado` | idéntico | **preexistente** |
| BUG-09 «does not exist» → `ambiguo` | `clasificacion:"ambiguo"` | idéntico | **preexistente** |
| BUG-10 `exportar` resuelve `destino` contra el workspace exportado | `zip: …/QA_DIFF\QA_DIFF.zip` | idéntico | **preexistente** |
| BUG-11 `editar` inserta campo desconocido | `action:"inserted"` | idéntico | **preexistente** |
| BUG-13 `hint` «completo:true» | mismo texto | idéntico | **preexistente** |
| BUG-14 `escribir_archivo` sobrescribe por defecto | `bytes_written:7` | idéntico | **preexistente** |
| BUG-15 `componente crear` exige `cambios` | `Missing: cambios` | idéntico | **preexistente** |
| BUG-16 `controles` `count` = total | `count:24`, 13 ítems, `truncado:true` | `count:31`, 13 ítems | **preexistente** |
| BUG-17 `vista` sin ventana conocida | mismo aviso | idéntico | **preexistente** |
| BUG-19 `escribir_horizonte` → `Shadings/Sites/…` | misma ruta | idéntico | **preexistente** |
| BUG-21 mensajes («no hay ninguno», «Project exists» en inglés) | idénticos | idénticos | **preexistente** |
| BUG-22 `preparar` sin archivos de batch en `siguiente` | mismo `siguiente` | idéntico | **preexistente** |
| BUG-27 `preferencias` con `vista.campos:[]` | idéntico | idéntico | **preexistente** |
| BUG-20 `sitio crear dry_run` sin `costo_estimado` | **`dry_run` se ignoraba y creaba el sitio de verdad** | `dry_run` respetado, sin costo | **nuevo** (la función `dry_run` es nueva; le falta el costo) |
| BUG-06 reutilización / `proyecto_comparable` | **no reutiliza**: `Project exists: …\QA_D7.PRJ`; no emite `proyecto_comparable` | reutiliza y emite `true`/`null` | **nuevo** (función nueva; el defecto es de documentación) |

**No comparables en el 6-nov (10):** BUG-02 (crash), 04, 05 (`p50_p90`), 08
(`via`), 12 (contador TRIAL), 18 (`menus`), 23 (`teclas`), 24 (`clic_fisico`),
25 (`componente campos`), 26 (`expected_sit`). Reproducirlos en el build del
6-nov exige fixtures que ya no existen (proyecto que hace *crash* a PVsystCLI),
GUI en un estado concreto o gastar TRIAL. **Se resolvieron por antigüedad de
commit en §8.3**, no por ejecución.

### 8.3 Antigüedad de cada defecto (commit que lo introdujo)

Método: `git log -S'<cadena del defecto>' --reverse` para encontrar el commit que
introdujo el código, y `git merge-base --is-ancestor <commit> 81b4f59` para saber
si es anterior al merge-base del PR (es decir, si **ya estaba en `main`**).
`81b4f59` es ancestro de `main`, así que todo lo anterior a él está en `main`.

| Bug | Commit | Fecha | ¿Antes del PR? |
|---|---|---|---|
| BUG-01 `PNomSizing` como kWp | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-02 crash no detectado (`nb_crash` sin leer) | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-03 `esperar_cierre` con vista ok | `4856acd` | 2026-10-27 | **sí (en `main`)** |
| BUG-04 `pasos[].detail` nulo | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-05 `p50_p90` GUI | `0fca119` | 2026-10-21 | **sí (en `main`)** |
| BUG-06 reutilización / `comparable` | — | 2026-11-07 | **no: función nueva del PR** |
| BUG-07 `modulo` sin extensión | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-08 `via` fuera del enum | `0fca119` | 2026-10-21 | **sí (en `main`)** |
| BUG-09 errores del CLI como «ambiguo» | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-10 `destino_zip` | `3dc546d` | 2026-10-25 | **sí (en `main`)** |
| BUG-11 `editar_objetos` inserta | `a71b715` | 2026-10-20 | **sí (en `main`)** |
| BUG-12 ledger de costo | `7e46870` | 2026-10-25 | **sí (en `main`)** |
| BUG-13 `hint` «completo:true» | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-14 sobrescribir por defecto | `8940569` | 2026-11-06 | función nueva, **comportamiento anterior** (siempre sobrescribía; el PR añadió el veto `false`) |
| BUG-15 `componente crear` exige `cambios` | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-16 `count` antes del recorte | `c9a3bf7` | 2026-10-25 | **sí (en `main`)** |
| BUG-17 `vista` sin ventana | `6644abc` | 2026-10-21 | **sí (en `main`)** |
| BUG-18 `menus` a medias | `79f621b` | 2026-10-29 | **no: dentro del PR** |
| BUG-19 `Shadings/{ruta}` | `0fca119` | 2026-10-21 | **sí (en `main`)** |
| BUG-20 `dry_run` sin costo | — | 2026-11-07 | **no: función nueva del PR** |
| BUG-21 mensajes de «no encontrado» | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-22 `preparar` sin batch en `siguiente` | `4856acd` | 2026-10-27 | **sí (en `main`)** |
| BUG-23 `teclas` con `foco:false` | `68154ff` | 2026-10-28 | **no: dentro del PR** |
| BUG-24 `clic_fisico` invoca la ventana | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-25 `consumidos_por_accion` | `9043410` | 2026-11-07 | **no: dentro del PR** |
| BUG-26 `expected_sit` nulo | `eeede5e` | 2026-10-19 | **sí (en `main`)** |
| BUG-27 `preferencias` sin campos | `4856acd` | 2026-10-27 | **sí (en `main`)** |

**Recuento:** **22 existían antes del PR #2** (21 probados + BUG-14 por
comportamiento), **3 los introduce el PR durante su desarrollo** (18, 23, 25) y
**2 son de funcionalidad que estrena el PR** (06, 20).

Dato revelador: `src/adapters/cli/clasificacion.rs` tiene **0 commits** en el
rango del PR (`81b4f59..HEAD`). El PR nunca tocó el clasificador que convierte
un *crash* en «ambiguo» (BUG-02/09). `src/simulation/results/lookup.rs` tiene
**1** commit en el rango y el defecto del kWp sobrevivió al refactor intacto
(BUG-01). En cambio `src/workspace/escritura.rs` tiene **5** commits del PR y
aun así conserva el default de sobrescritura (BUG-14).

### 8.4 Qué significa para el PR #2

- **22 de los 27 defectos ya estaban en `main`** antes del PR #2: el PR los
  hereda y, en varios casos, los toca sin corregirlos. El caso más claro es
  `src/adapters/cli/clasificacion.rs`: **0 commits** del PR y el *crash* sigue
  siendo «ambiguo».
- **3 los introduce el PR durante su desarrollo**: `menus` a medias (29-oct),
  `teclas` con `foco:false` (28-oct) y `consumidos_por_accion` declarando la
  clave equivocada (7-nov).
- **2 son de funcionalidad nueva**: `dry_run` en `sitio crear` (sin
  `costo_estimado`) y la reutilización de proyectos en `disenar_sistema` (con
  una descripción que promete un estado inalcanzable).
- **Ninguno es una regresión de comportamiento**: no se encontró nada que
  funcionara el 6-nov y se rompiera el 7-nov. Al contrario, el build probado
  **mejora** dos cosas: `sitio crear dry_run` ya no crea el sitio, y
  `disenar_sistema` reutiliza el proyecto en vez de fallar con `Project exists`.
- **Respuesta a «¿existen después de todos los cambios del PR #2?»: sí.** Los 27
  están presentes en el binario que corresponde al HEAD del PR. El PR no cierra
  ninguno de los 22 heredados y añade 5 propios (3 en código existente, 2 en
  funciones nuevas).

### 8.5 Reproducir la comparación

```bash
cd .qa
# binario instalado
python qa_call.py diff.json  outD_new.jsonl
python qa_call.py diff2.json outD2_new.jsonl
# build anterior (6-nov)
PVSYST_MCP_EXE="C:/pvsyst-mcp/pvsyst-mcp.exe.old-20261107213119.exe"   python qa_call.py diff.json outD_old.jsonl
```

Los pares están en `.qa/outD_new.jsonl` / `.qa/outD_old.jsonl`,
`.qa/outD2_new.jsonl` / `.qa/outD2_old.jsonl`,
`.qa/outB6_new.jsonl` / `.qa/outB6o.jsonl`,
`.qa/outK2_new.jsonl` / `.qa/outK2_old.jsonl`,
`.qa/outG_old.jsonl`.

---

*Fin del informe. Todas las cifras provienen de las respuestas del MCP o de los
archivos de PVsyst citados; el JSON íntegro está en `.qa/`.*
