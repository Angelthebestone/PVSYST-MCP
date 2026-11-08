# Informe de prueba en vivo — servidor MCP `pvsyst` (binario pr/2)

Fecha: 2026-11-06 · Binario: `C:\pvsyst-mcp\pvsyst-mcp.exe` (punta de `origin/pr/2`, commit `5132627`, MD5 `16f4e08a…`, construido con `scripts/install.ps1` / toolchain `stable-x86_64-pc-windows-gnu`) · PVsyst 8.1.8 (CLI rev. 49527) · Licencia TRIAL: 32 días, 138 ejecuciones al inicio, 136 al final (2 consumidas: 1 `create-site`, 1 `run-simulation`).

## 1. Método

Prueba en vivo contra el servidor real de esta sesión (stdio del cliente MCP), sin bancos de pruebas:

- Lecturas libres sobre el workspace real `C:\Users\Lenovo\PVsyst8.1_Data`.
- Escrituras estructurales en workspace de prueba `C:\Users\Lenovo\pvsyst_mcp_viva\ws_prueba` (borrado al final).
- Ciclo de vida de proyecto/componentes/sitio con nombres `E2E_VIVA_*` en el workspace real; limpieza al final (las eliminaciones van a `UserRecycleBin`, recuperables por diseño).
- GUI real: observación completa + `lanzar`/`cerrar_todo`; ninguna pulsación de Guardar/Eliminar/OK.
- Verificación cruzada de cifras: KPIs ↔ mensual ↔ árbol de pérdidas ↔ metadatos del CSV.

## 2. Cobertura ejecutada (50 llamadas en vivo)

| Tool | Llamadas | Resultado |
|---|---|---|
| `entorno` | diagnostico (resumen), licencia | OK. Detecta 8.1.8 solo; `herramientas:15` |
| `listar` | proyectos (×3), archivos, modulos (×4), objetos | OK. `texto` filtra de verdad; 736 archivos, 16 proyectos |
| `leer` | PRJ, VC0 (resumen), PAN (árbol), MET+serie, objeto (error esperado), base64 ida/vuelta, texto, vacío | OK. Roundtrip base64 exacto (`BINARIO-PRUEBA-123`) |
| `escribir_archivo` | texto, base64, sobrescribir, `contenido:""`, fuera-de-workspace vía `workspace:` | OK salvo N5 |
| `workspace` | crear, historial (×2), restaurar, exportar_proyecto (×3), crear con `..` | OK salvo N3 |
| `editar` | rvt sin operación, campos con `A=B` | Validaciones correctas (F-03/F-28 cerrados) |
| `proyecto` | crear, crear_variante, dimensionar (kwp / kwp+módulos malos), doctor, configurar (×3), eliminar (con y sin variante) | OK salvo N1/N6 |
| `componente` | importar (×3), editar (×2), exportar, borrar (×2), listar modulos (×4) | OK salvo N2 |
| `sitio` | crear (×2: reutiliza y nuevo) | OK. `reutilizado:true` honesto; nombre respetado |
| `pvsyst_cli` | version, help, help+opciones | OK. F-27 cerrado |
| `simular` | preparar (error guía), ejecutar dry_run, ejecutar real | OK. Simulación real exitosa |
| `resultados` | kpis, mensual, csv, perdidas, resumen, carbono, p50_p90, economia | OK. Errores accionables |
| `gui_observar` | ventanas (×2), controles, vista, esquema, funciones, mensajes | OK. Cobertura del esquema: 89 ventanas en 4 estados |
| `gui_actuar` | lanzar, cerrar_todo | OK. F-12 cerrado (`ya_en_ejecucion:true`) |
| Protocolo | initialize, tools/list (2 modos) | OK. 15/15 herramientas |

No ejecutadas en vivo (razón): `editar insertar_objeto/eliminar_objeto/rvt` operativos (cubiertos por la suite del repo, 1007 tests), `simular preparar` con datos reales y `barrido` (requieren series de entrada; el costo en ejecuciones no justifica la cobertura repetida), 16 acciones de GUI con efecto destructivo (regla de seguridad), `componente crear_desde_ficha` y `disenar_sistema` completos (abren la GUI de PVsyst y escriben en su espacio; verificados sus errores de validación en la suite).

## 3. Estado de los hallazgos del informe E2E del 25-10

| Hallazgo | Estado |
|---|---|
| F-01 (A) `proyecto eliminar` con `variante` borraba el proyecto | **CORREGIDO**: recicla solo `E2E_VIVA_1.VC1`; VC0 sobrevive |
| F-02 (A) `escribir_serie` corrompía MET con variables inexistentes | No reproducido en vivo (exige MET real); la suite lo cubre |
| F-03 claves con `=` | **CORREGIDO**: «Nombre de campo inválido 'A=B': no puede estar vacío ni contener '='» |
| F-05 `dimensionar` ignoraba `modulo`/`inversor` | **RESUELTO como contrato**: «'modulo' no admite el pre-dimensionado de PVsyst: pase n_mod_serie y n_strings, o via 'aproximacion'» |
| F-06 `proyecto crear` ignoraba `comentario` | **CORREGIDO**: `applied` lo aplica en PRJ y VC0; llega a los metadatos del CSV de la simulación |
| F-07 `sitio crear` ignoraba `nombre_sitio` | **CORREGIDO**: `create-site -sn:E2E_VIVA_Site2` respeta el nombre; reutilización honesta (`reutilizado:true`, 0 ejecuciones) |
| F-08/F-09 `workspace` ignorado/engañoso en rutas GUI | **CORREGIDO**: las rutas puros-PVsyst rechazan `workspace` con mensaje claro (suite: 5 regresiones verdes tras el ajuste de textos) |
| F-11 `contenido:""` | **PERSISTE** en el servidor: crea archivo vacío con `bytes_written:0` y `estado:ok` (N5) |
| F-12 `lanzar` lanzaba 2.ª instancia | **CORREGIDO**: `ya_en_ejecucion:true`, `ready:true`, sin segunda copia |
| F-18 `controles`/`funciones` truncados | **PERSISTE**: 13 de ~25/203 sin `detalle:"completo"` |
| F-21 `listar texto:` ignorado | **CORREGIDO**: filtra de verdad (0 con `E2E_VIVA` vacío, 1 con coincidencia) |
| F-23 `workspace crear` aceptaba `..`; exportar pisaba sin aviso | **CORREGIDO**: «'ruta' no admite '..'»; duplicado exige `sobrescribir:true` |
| F-24 `componente` sobrescribía sin aviso | **CORREGIDO**: «ya existe: pasa 'sobrescribir': true» |
| F-27 `pvsyst_cli help` clasificado error | **CORREGIDO**: `clasificacion:"exito"`; `opciones:['run-simulation']` llega a la CLI |
| F-28 `editar rvt` sin `operacion` ejecutaba `listar` | **CORREGIDO**: error con ejemplo |
| F-30 `pagina indice:9` mensaje engañoso | No verificado en vivo (requiere pestaña activa); conversar en el PR |

Los 7 falles de test que traía la punta de pr/2 eran de esta familia (textos ES esperados tras la migración a EN); corregidos en el worktree de compilación (`mcpb/manifest.json` + 3 ficheros de test). Suite completa: **1007+76+… verde, 0 fallos**.

## 4. Hallazgos nuevos (esta sesión)

**N1 (M) `proyecto dimensionar` solo-`kwp` falla y deja PVsyst.exe huérfano.**
Repro: `proyecto {accion:"dimensionar", proyecto:"E2E_VIVA_1", variante:"VC0", kwp:5}` con la GUI cerrada → `'variante_activa' está deshabilitado en este momento. PVsyst dijo: Empty project name detected in fPrjList`; el log `AppData\Local\PVsyst\8.1\Admin\*_Err.log` confirma `Empty project name detected in fPrjList`; el proceso 7488 quedó vivo (antes de la llamada `gui_running:false`). Dos defectos: el mensaje no describe el fallo real (arranque de PVsyst sin proyecto que leer) y el proceso queda sin recoger. Sugerencia: si la ruta kwp-only requiere la GUI, validarlo antes (como ya hace `dimensionar` con `workspace`) y recoger el proceso al fallar; documentar que kwp solo funciona con PVsyst arrancado o convertirlo en error de contrato como `modulo`.

**N2 (M) `componente importar` sin extensión crea un componente invisible.**
`nombre_nuevo:"E2E_VIVA_PAN"` crea `ComposPV\PVModules\E2E_VIVA_PAN` (sin `.PAN`); `listar modulos` (filtra `*.PAN`) no lo lista nunca. Con `nombre_nuevo:"E2E_VIVA_PAN2.PAN"` todo funciona. Sugerencia: normalizar el nombre añadiendo la extensión del tipo (PAN/OND/…) o rechazar nombres sin extensión con mensaje que lo diga.

**N3 (M) `workspace restaurar` ignora `dry_run` y ejecuta de verdad.**
`workspace {accion:"restaurar", id:…, dry_run:true}` restauró de verdad (`restaurado`, archivos tocados) y contestó `"ignorados":["dry_run"]`. Una operación que reescribe ficheros no debe ejecutarse bajo un `dry_run` pedido; o lo implementa, o rechaza el parámetro con error.

**N4 (B) El campo `ignorados` contradictorio.**
`escribir_archivo` con `sobrescribir:true` que sí sobrescribió responde `"ignorados":["sobrescribir"]`; `componente editar` con `campos` aplicados (ver `changes_applied`) responde `"ignorados":["campos"]`. Si «ignorados» significa «parámetros sin efecto en esta acción», choca con el efecto observado; si significa otra cosa, el nombre induce a error. Sugerencia: «no_aplicables» o quitar los que sí se aplicaron.

**N5 (B) `contenido:""` crea un archivo vacío sin entrada de historial.**
`estado:ok`, `bytes_written:0`, `historial_id:null` — incumple la promesa «with history» y contradice la política del PR de no aceptar work no verificado en silencio. Sugerencia: rechazar contenido vacío en `escribir_archivo` (ya se rechaza clave vacía en `campos`).

**N6 (B) `configurar` con paso inválido aplica los anteriores y reporta «replaced» para valores idénticos.**
`pasos:[{dimensionar 12/1},{dimensionar -1}]` → «Paso 2 fallado tras aplicar 1 paso(s)»: honesto, pero sin rollback; y en el camino feliz reporta `{"action":"replaced","key":"NModSerie","new":"12","old":"12"}` (ruido). Sugerencia: validar valores de todos los pasos antes de aplicar (G8 cubre la estructura, no los valores) y usar «sin_cambio» cuando old==new.

**N7 (B) Mensajes a medio traducir.**
La capa de servidor mezcla «Missing: … Example: …» con «Tipo incorrecto … esperado boolean» y vistas con claves en español (`herramientas`). El test `should_name_every_missing_required_arg_of_the_action` exigía aún `Ejemplo:`. Sugerencia: una sola pasada final de i18n sobre `src/server` y sus tests, con los textos de error como constantes compartidas.

**N8 (B) `leer` de un objeto inexistente en .VC0.**
`leer {archivo:*.VC0, objeto:"pvProject"}` → «Object 'pvProject' not found; see listar que:objetos» — correcto pero no dice qué objetos HAY; la llamada de seguimiento obligatoria encarece el flujo (ver §5).

## 5. Eficiencia: cómo reducir llamadas

Medido en vivo: lecturas de ficheros 0–35 ms; listados 6–550 ms; `resultados` 240–800 ms; CLI 3,3–6,6 s por llamada; simulación real ~1 ejecución + ~10 s. Con ese coste por llamada de CLI/GUI, cada ida-y-vuelta que se ahorra cuenta.

1. **Usar `resultados que:"resumen"`** (nuevo): KPIs + mensual (+perdidas) en una llamada (804 ms) en vez de 3 (≈1.400 ms y 3 llamadas). Es el patrón a promocionar en la documentación y en los `siguiente` sugeridos tras `simular`.
2. **`costo_estimado` en el dry_run** (nuevo): anuncia `ejecuciones:1, restantes:137` antes de gastar. Promocionarlo también en `disenar_sistema` y en las rutas que lanzan GUI.
3. **Errores accionables evitan re-llamadas**: `carbono/p50_p90/economia` dicen qué activar y cómo re-simular. Aplicar el mismo patrón a N1 y N8 (incluir en el error los objetos disponibles o el primer paso del arreglo).
4. **`proyecto configurar` ya agrupa los cambios de variante** — Posee la semántica adecuada para sustituir secuencias editar+dimensionar; falta solo la validación de valores previa (N6) para que sea todo-o-nada.
5. **Corrección de los textos de test** (hecha en el worktree) reduce cycles de CI, no del runtime.
6. Pendiente de diseño (no implementar sin decidir producto): `listar` multi-`que` en una llamada, y que `leer` acepte varios `archivo` para lotes de lectura pequeña.

## 6. Limpieza y residuos

- Eliminados: `C:\Users\Lenovo\pvsyst_mcp_viva\` (completo), `Results\E2E_VIVA_1_VC0.{csv,pdf}`, `Sites\E2E_VIVA_Site*.SIT`, `UserData\E2E_VIVA_Site2_Monthly.csv`, proceso PVsyst.exe huérfano.
- En `UserRecycleBin` (mecanismo de recuperación del propio servidor, se dejan): `E2E_VIVA_1.PRJ/.VC0/.VC1`, `E2E_VIVA_PAN`, `E2E_VIVA_PAN2.PAN`.
- Residuos de sesiones anteriores (no míos, sin tocar): `E2E_Demo.PRJ`, `MCP_Test_400.PAN`, `Z_E2E_Importado.PAN`, `MCP_Inv_10k.OND`, `Z_E2E_Ficha`.
- Consumo TRIAL: 138 → 136 (1 `create-site`, 1 `run-simulation` reales).

## 8. Segunda pasada — búsqueda de hallazgos nuevos (~35 llamadas más)

Superficies nuevas ejercitadas en vivo: escenas 3D (validación), `editar rvt`/`plantilla_rvt` ciclo completo, `insertar_objeto`/`eliminar_objeto` sobre copia de un VC0 real, el resto de `proyecto` (10 elementos de `anadir`, `seguidor`, `limite_inyeccion`, `perdidas`, `almacenamiento`, `bombeo`, `economia`, `p50_p90`, `renombrar`, `limpiar_temporales`), `sitio` completo (geometría, horizontes, `meteo_pvgis` con red real), `disenar_sistema` extremo a extremo, y lote adversarial (unicode/espacios, nombres de 180 caracteres, rutas profundas, inyección en `pvsyst_cli`, idempotencia, valores-objeto).

**Verificaciones positivas nuevas**: F-04 (`anadir envejecimiento`) **corregido**; cadena RVT completa con `historial_id` en cada operación explícita; `insertar/eliminar_objeto` con historial y vuelta a 26 objetos; `meteo_pvgis` real descarga PVGIS (GHI 1.655 kWh/m², `confirmed_by_output:true`);`horizonte_pvgis` rechaza `dry_run` con el mejor mensaje del día («esa acción no tiene ensayo y se ejecutaría de verdad»); `disenar_sistema` valida el kWp resultando contra el objetivo con aritmética explícita («+92 % del objetivo, tolerancia 20 %»); sin inyección en `pvsyst_cli` (un solo argumento citado); unicode/espacios/nombres largos/rutas profundas OK; integridad del `.VC0` editado probada con simulación real exitosa.

### Hallazgos nuevos

**N9 (M) `componente editar` acepta un objeto como valor y escribe su JSON como texto.**
`campos:{Pnom:{anidado:true}}` → `changes_applied: Pnom new "{\"anidado\":true}"` sobre el `.PAN` real: el campo numérico queda con un string JSON (corrupto para PVsyst). Se reparó en vivo vía `workspace restaurar`. Debería rechazar el tipo como ya hace con booleanos mal usados. (Comprobado que el historial salva: `Pnom` de vuelta a `400.0`.)

**N10 (M) El historial solo respalda sobrescrituras; las creaciones se van sin copia.**
`escribir_archivo` a archivo **nuevo** → `historial_id:null` aunque el resumen dice «con copia en el historial» (falso en ese caso). Sobre archivo existente sí registra. La primera versión de todo archivo nuevo es irrecuperable pese al mensaje. Verificado con prueba controlada en el workspace real (creación `null`, resobreescritura `0020`). Afecta también a `escribir_horizonte` y a las escrituras del workspace alternativo. Sugerencia: registrar también la creación (snapshot «vacío→contenido») o corregir el texto del resumen.

**N11 (M) Las tres rutas que lanzan la GUI de PVsyst comparten una raíz rota en este entorno.**
`dimensionar` solo-`kwp`, `proyecto p50_p90` y `sitio horizonte_pvgis` fallan con `'variante_activa' está deshabilitado... Empty project name detected in fPrjList` (confirmado en el log de PVsyst y con proceso huérfano). `horizonte_pvgis` añade la guarda de foco del PR (correcta: «PVsyst perdió el primer plano (TProjectForm): no se envían teclas a otra aplicación») y mezcla en el mensaje un aviso de parseo ajeno (`TEnergyMixSet`/`EndTags`) que también emite — benigno — la simulación normal. Sugerencia: detectar el arranque sin proyecto y fallar temprano con la causa; recoger el proceso; filtrar avisos de parseo conocidos del mensaje de error.

**N12 (M) `disenar_sistema` no puede simular sin Meteonorm y `synthetic_weather` no llega al paso de simulación.**
Sitio nuevo por CLI (NASA SSE) + simulación → `Failed to initialize Meteonorm... DataMN9/station.db.enc... Simulation cannot be run`, tras 11 s, con `ok:false` honesto pero sin destilar la causa raíz ni sugerir la alternativa. `synthetic_weather:true` se acepta y no tiene efecto en ese paso. La cadena sitio→proyecto→dimensionado sí funciona (4,8 kWp verificado). Sugerencia: detectar Meteonorm ausente antes de lanzar y sugerir `synthetic_weather`/meteo sintética; pasar la señal al `simular` interno.

**N13 (M) `disenar_sistema` no es idempotente ante reintentos.**
Reejecutar con el mismo `nombre` tras un fallo parcial → `Project exists` en el paso 2, obligando a borrar y repetir TODO (sitio incluido) para reintentar solo la simulación. Sugerencia: si el proyecto existe y los parámetros coinciden, continuar desde el paso fallido (o `sobrescribir`/`reanudar` explícito).

**N14 (B) `almacenamiento` aplica tamaños por defecto enormes sin explicar el criterio.**
`estrategia:"autoconsumo"` sin parámetros de dimensión → `NBattSerie 1→10`, `NBattParall 2→20` (200 celdas de 180 Ah) sin explicación en la respuesta de de dónde salen ni su criterio, y con el aviso críptico «PVsyst debe conocer la batería…». Sugerencia: devolver el criterio de dimensionado o exigir los parámetros.

**N15 (B) Descubribilidad de acciones gated.**
`geometria_solar` existe solo con `PVSYST_MCP_APROXIMACIONES=1`; desde un cliente conectado sin la variable, el rechazo lista candidatos sin mencionar que la acción existe tras esa variable. Sugerencia: «acción disponible con PVSYST_MCP_APROXIMACIONES=1».

**N16 (B) Detalles menores**: `rvt anadir` sobre archivo inexistente crea el archivo sin `historial_id` (las operaciones explícitas sí registran); `configurar`/`componente editar` repiten `{"action":"replaced","old":X,"new":X}` para valores idénticos (ruido); `pvsyst_cli` usa `opciones` (el inexistente `argumentos` se ignora silenciosamente); `mcp.example.json` no documenta `PVSYST_DATARO` (sin él, leer DataRO da «Ruta fuera del workspace»); `escribir_horizonte` escribió el `.HOR` bajo `Shadings/Meteo/…` (ruta a validar contra lo que PVsyst lee realmente).

## 9. Conclusiones de la primera pasada

- El binario de pr/2 está vivo y operativo de punta a punta: instalación → lectura → escritura con historial → ciclo de proyecto → simulación real → resultados verificados por dos caminos independientes (delta 0,1 kWh por redondeo).
- De los 16 hallazgos accionables del informe del 25-10 verificados en vivo, 13 están corregidos; quedan F-02 (pendiente de repro en vivo), F-11 (N5), F-18 (N8) y el matiz de F-30.
- 8 hallazgos nuevos (2 M de comportamiento: N1, N2; 1 M de seguridad operativa: N3; 5 B de mensajes/contratos).
- Prioridad sugerida para el PR: N1 (mensaje + proceso huérfano), N3 (dry_run que ejecuta), N2 (extensión del componente), y la pasada final de i18n (N7) que cierra también los 7 falles de test ya corregidos.

## 10. Tercera pasada — GUI en vivo completa (~45 llamadas)

Cobertura GUI lograda: **gui_observar** — ventanas, controles (normal 13 / completo 24), leer, capturar (PNG 77 KB), formulario (OCR con rects), vista (claves con coordenadas y valores reales), esquema (89 ventanas; `components.inverters` ahora específico — punto de F-18 corregido), funciones (13 normal / **209 completo**), mensajes. **gui_actuar** — lanzar, activar, esperar (es espera-de-ventana, no sleep), mensaje (WM_n), cerrar, cerrar_todo, secuencia (2 pasos OK), funcion (4 formularios), rellenar (sin validar: impecable), seleccionar (valida contra la lista real), uia_invocar (invocar/fijar_valor/seleccionar probados), teclas, fila, campo, click, clic_texto, clic_fisico (vía errores/validaciones), esperar_cierre, reiniciar (rechazo correcto), pagina (con `ventana`).

**Hallazgos nuevos de GUI**:

**N17 (M) Mapa de funciones cruzado**: `herramientas.factor_transposicion` abre `TTranspositionForm` (el propio tool esperaba `TArrayCharactForm` y su nota interna dice lo contrario de lo observado); termina `completo:false` tras 10,5 s de espera inútil. Además la ventana queda abierta como modal y bloquea la siguiente llamada.

**N18 (M, el de mayor impacto) No existe selección de filas en `TVirtualStringTree`**: `fila` dice tener geometría medida ("TVirtualStringTree de TProjectLoadForm") pero no acepta ninguna forma de dirigir el control (`clase` se ignora); `click` solo acepta TEdit/TComboBox/+2 (ni árboles ni botones); el árbol UIA del `TVirtualStringTree` no expone patrones (control dibujado a mano). **Consecuencia: no se puede cargar un proyecto por GUI** (TProjectLoadForm) y todo el bloque `variante.*` (12 funciones) + escena 3D + `escena_leer/importar` queda inalcanzable en flujos no atendidos. Verificado con 4 intentos independientes. Es la carencia nº 1 de la automatización GUI.

**N19 (M) Reseña de padres: cerrar un editor reabre su selector.** Tras `cerrar` de `TPVModuleForm`/editores, el `TChoixObjetPVForm`/`TDatabaseForm` padre reaparece como modal (dos veces observado); el `cerrar` reporta `closed:true` antes de que el padre reaparezca. Los flujos necesitan cierre en orden inverso o soporte de "cerrar cadena".

**N20 (A, con cautela) `teclas` sin gestión de foco terminó la aplicación.** Tras `teclas RETURN` con `foco:false` (la respuesta apuntaba a THomeForm, no al diálogo activo), PVsyst terminó por completo sin orden de salida (`ventanas:[]`, proceso ausente). No se pudo aislar si el detonante fue el Return a ciegas o la X del diálogo de demo; en la réplica, cerrar `TDEMOForm` por su botón vía UIA fue seguro y la app sobrevivió. Recomendación: `teclas` debe exigir/establecer foco (o abortar con error claro cuando `foco:false`).

**N21 (M) `gui_observar ventanas` no lista el modal bloqueante**: con `TDEMOForm` modal abierto, `ventanas` devolvió solo `THomeForm` mientras el guarda-modal lo delataba. Ceguera del observador ante el bloqueo exacto.

**N22 (B) `esperar_cierre` sobre ventana ya cerrada da error** en vez de éxito trivial (uso natural tras `cerrar`, que es síncrono): carrera garantizada.

**N23 (B) `campo` por etiqueta falla con rótulos no-texto**: "Rótulos vistos: " (vacío) aunque `vista` sí ve el campo (`modelo`). La vía funcional es `rellenar`. El error debería sugerirlo.

**Re-verificaciones del informe del 25-10**: **F-14 CONFIRMADO que persiste** (`uia_invocar seleccionar` sobre pestaña vetada "Datos adicionales" devuelve OK y la vista sigue en "Datos básicos"); F-17 persiste (`clic_texto 'Cancelar'` → "no está dibujado"); F-18 parcialmente corregido (controles/funciones responden a `detalle:"completo"`; esquema por entidad ya es específico); **F-12 confirmado corregido**; F-13 parcialmente: `seleccionar` ahora valida contra la lista real (bien), `escribir` sobre formulario sigue sin efecto (mensaje correcto que sugiere `campo`/controls).

**Patrón de arquitectura sugirido**: `uia_invocar` (con los patrones UIA reales) es la única vía fiable para botones/menus; `click`/`clic_texto`/`fila` quedan como superficies rotas o cojo soporte. Consolidar GUI alrededor de UIA + `rellenar` + `vista` y deprecar/rediseñar el resto evitaría la mitad de los hallazgos N.

## 12. Cuarta pasada — las 12 funciones `variante.*` ejecutadas (con proyecto cargado)

Cómo se logró cargar un proyecto pese a N18: `proyecto.nuevo_conectado_a_la_red` abre `TProjectForm` (pero con proyecto vacío los diálogos de variante están deshabilitados, como ya vio el E2E); la vía que funcionó fue el diálogo de carga con teclado: `funcion proyecto.cargar` → `activar` → `teclas DOWN` → `teclas RETURN` — **la carga tardó MÁS que los 12 s del `esperar`** (falso negativo: el proyecto apareció después; N26). Con `TProjectForm` mostrando `Proyecto: _DEMO_Commercial.PRJ`:

| Función | Ventana abierta | Resultado |
|---|---|---|
| `variante.orientacion` | TOrientationForm | OK (datos reales: Tracking, Tilt 10°, Azim 10°) |
| `variante.sistema` | TSystemegridForm | OK (Reparto PNom, MPPT, 50/60 Hz) |
| `variante.perdidas_detalladas` | TDetailedLossesForm | OK |
| `variante.autoconsumo` | TUserNeedsForm | OK |
| `variante.almacenamiento` | TGridStorageForm | OK |
| `variante.horizonte` | THorizonForm | OK (8 filas de edición) |
| `variante.gestion_energia` | TSystemeAddForm | OK |
| `variante.evaluacion_economica` | TEcoEvaluationForm | OK (2 pestañas) |
| `variante.simulacion_avanzada` | TAdvancedSimForm | OK |
| `variante.resultados_detallados` | TResultForm | OK |
| `variante.resumen_sistema` | TSummaryForm | OK |
| `variante.sombreados_cercanos` | TShadingsForm | OK |

**12/12 abiertas y cerradas limpiamente** (patrón adaptativo: la clase exacta sale en `ventanas_nuevas` de cada respuesta — cerrar esa). Los nombres de clase no son adivinables (`TSystemegridForm`, no `TSystemsForm`): el cerrar a ciegas falla y deja el modal bloqueando; recomendación: que `funcion` ofrezca cerrado emparejado (`cerrar: true`) o que el error de `cerrar` sugiera la clase observada.

**Escena 3D de punta a punta (lograda)**: `variante.sombreados_cercanos.construccion_perspectiva` → **TPerspectiveForm** (editor 3D) → `escena_leer` leyó `Shadings/principal.SHD` ("Ninguna escena definida", 0 objetos) → `escena` escribió 2 objetos (`paralelepipedo` + `prisma_triangular`) → **`verificado:true`, 17 campos comprobados, 0 diferencias** (24,7 s) → cierre descartando (sin guardar; la variante quedó intacta). La validación de `escena` es ejemplar: lista TODOS los campos que faltan de golpe y enseña el esquema por tipo (`prisma_triangular` usa `lado_1_2/lado_2_3/lado_3_1/altura`, no dx/dy/dz); los tipos válidos son 12+ primitivas (`triangulo…paralelepipedo, piramide_cuadrada, prisma_*, porcion_de_cilindro…`).

**N24 (M, agravado) `escena_leer`/`escena` se autobloquean con su propio `.SHD`:** cada lectura escribe `Shadings/<escena>.SHD` para leerla (aviso honesto) y la siguiente operación con el mismo nombre —lectura O escritura— falla con «Ya existe …: elige otro nombre». Tras UNA lectura, la escena `principal` queda inutilizable para todo hasta borrado manual (reproducido 3 veces). Debe reutilizar/sobrescribir su propio export o borrarlo al terminar.

**N25 (M) `reiniciar` con crash real (¡ocurrió!):** `madExceptWndClass` apareció de verdad al cerrar la escena; `reiniciar` detectó el diálogo y pulsó su botón (bien), pero el proceso no terminó en 20 s → error «no terminó en 20 s tras pulsar el botón» y el proceso quedó zombi (verificado con `tasklist`). Debe verificar la muerte y escalar a kill forzado.

**N26 (B) `esperar` con plazo corto da falso negativo en cargas lentas**: la carga del proyecto tardó >12 s (timeout vencido, proyecto cargado después). El `esperar` debería poder esperar más o re-verificar.

**Cierre observado**: el cierre de la escena disparó una confirmación (`TMessageForm`); `cerrar_todo` la canceló sin guardar (política del PR funcionando) y el reporte de `cerrar` fue honesto («puede haber abierto una confirmación»). Las listas de ventanas pueden servir estados stale (dos ventanas «vivas» que ya estaban cerradas) — conviene re-leer antes de decidir.

## 13. Conclusiones finales

- Cuatro pasadas en vivo: ~165 llamadas reales, 15/15 herramientas, **12/12 funciones `variante.*` ejecutadas sobre proyecto real**, escena 3D de punta a punta (leer + escribir + verificar), 2 simulaciones reales verificadas por rutas independientes.
- Hallazgos acumulados: 1 A prudente (N20), 9 M (N1, N2, N3, N9, N10, N11, N12, N13, N24, N25) y 8 B (N4–N8, N14–N16, N21–N23, N26), con F-14/F-17 reconfirmados.
- Prioridad sugerida global: **N18** (sin selección de filas no hay automatización GUI no atendida) → **N10** (historial que no respalda creaciones) → **N24** (escenas que se autobloquean con su .SHD) → **N9** (tipos arbitrarios escritos en componentes) → **N25** (reiniciar sin verificación de muerte) → **N20/N11** (teclas sin foco; rutas GUI con huérfanos) → **N12/N13** (`disenar_sistema`) → resto B.
- Consumo TRIAL total de la sesión: 138 → 134 (2 `create-site`, 2 `run-simulation` exitosas). Cero residuos fuera de `UserRecycleBin` (mecanismo de recuperación del propio servidor).
