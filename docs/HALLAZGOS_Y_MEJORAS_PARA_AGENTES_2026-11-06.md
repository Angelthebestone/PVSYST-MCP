# Hallazgos y mejoras para agentes — MCP `pvsyst` (binario pr/2)

Fecha: 2026-11-06 · Binario: `origin/pr/2` @ `5132627` ( instalado en `C:\pvsyst-mcp\pvsyst-mcp.exe` ) · PVsyst 8.1.8 TRIAL · ~165 llamadas en vivo · 4 pasadas.

Metodología: prueba en vivo contra el servidor real (todas las mutaciones con nombres `E2E_*VIVA*` y limpieza posterior; el workspace de pruebas se borró; ver `INFORME_PRUEBA_VIVA_2026-11-06.md` para la bitácora completa).

---

## 1. Resumen ejecutivo

- **27 hallazgos nuevos** (N1–N26 + el lote de textos de test), además de re-verificar 30 hallazgos (F-01…F-30) del E2E del 25-10: 13 corregidos, 4 persisten, el resto sin repro en vivo.
- El producto funciona de punta a punta: instalación → lectura → escritura con historial → ciclo de proyecto → **2 simulaciones reales** → resultados verificados por 2 caminos independientes (delta 0,1 kWh) → **12/12 funciones `variante.*`** → **escena 3D con verificación de 17 campos**.
- Las 3 mayores palancas de calidad percibida por un agente: **N18** (selección de filas imposible → bloquea 12+ funciones sin operador), **N10** (el historial no respalda creaciones aunque lo promete) y la **consistencia de mensajes** (mezcla ES/EN, campo `ignorados` contradictorio).

---

## 2. Hallazgos nuevos (N), con reproducción

Leyenda: **A** = riesgo de datos/proceso · **M** = comportamiento incorrecto u engañoso · **B** = menor/UX.

### Bloque A —Persistencia y datos

**N9 (M) `componente editar` acepta un objeto como valor y escribe su JSON como texto.**
Repro: `componente {accion:"editar", componente:"X", campos:{Pnom:{anidado:true}}}` → `changes_applied: Pnom new "{\"anidado\":true}"` — el campo numérico del `.PAN` real queda con texto JSON (corrupto para PVsyst). Se reparó en vivo con `workspace restaurar`. Debe rechazar tipos no escalares como ya hace con booleanos.

**N10 (M) El historial no respalda creaciones aunque el mensaje lo promete.**
Repro: `escribir_archivo` a archivo **nuevo** → `historial_id:null` con resumen «con copia en el historial» (falso). Sobre archivo existente sí registra (verificado: creación `null`, sobreescritura `0020`). La primera versión de todo archivo es irrecuperable. Afecta también a `escribir_horizonte` y escrituras en workspace alternativo.

**N24 (M) `escena_leer`/`escena` se autobloquean con su propio `.SHD`.**
Cada `escena_leer` escribe `Shadings/<escena>.SHD` para leer (aviso honesto) y la operación siguiente con el mismo nombre —lectura **o** escritura— falla «Ya existe …: elige otro nombre». Tras UNA lectura, la escena `principal` queda inservible hasta borrado manual (reproducido 3 veces). Debe reutilizar su export, sobrescribirlo o borrarlo al terminar.

**N3 (M) `workspace restaurar` ignora `dry_run` y ejecuta de verdad.**
Repro: `workspace {accion:"restaurar", id:…, dry_run:true}` → restauró de verdad y respondió `"ignorados":["dry_run"]`. Una operación que reescribe ficheros no debe ejecutar un ensayo pedido.

**N20 (A, con causa no aislada) `teclas` sin gestión de foco terminó la aplicación.**
Tras `teclas RETURN` con `foco:false` (la respuesta apuntaba a THomeForm, no al diálogo activo), PVsyst terminó por completo sin orden de salida. En la réplica, cerrar `TDEMOForm` por su botón (UIA) fue seguro → el detonante más probable es el envío a ciegas. `teclas` debe exigir/establecer foco o abortar con error cuando no puede garantizarlo.

### Bloque B — Rutas que lanzan la GUI de PVsyst

**N1/N11 (M) Las tres rutas GUI comparten una raíz rota + proceso huérfano.**
`dimensionar` solo-`kwp`, `proyecto p50_p90` y `sitio horizonte_pvgis` fallan con `'variante_activa' está deshabilitado… Empty project name detected in fPrjList` (confirmado en `AppData\Local\PVsyst\8.1\Admin\*_Err.log`); dejan `PVsyst.exe` abierto (huérfano). `horizonte_pvgis` añade la guarda de foco (correcta) y mezcla en el mensaje un aviso de parseo ajeno (`TEnergyMixSet`/`EndTags`, benigno — también lo emite la simulación normal). Deben: fallar temprano con la causa, recoger el proceso, filtrar avisos conocidos.

**N17 (M) Mapa de funciones cruzado.**
`herramientas.factor_transposicion` abre `TTranspositionForm`, pero la herramienta espera `TArrayCharactForm` (su nota interna dice lo contrario de lo observado) → `completo:false` tras 10,5 s de espera inútil + ventana modal colgando que bloquea la siguiente llamada.

**N18 (M — la carencia nº 1) No existe selección de filas en `TVirtualStringTree`.**
`fila` dice tener la geometría medida («TVirtualStringTree de TProjectLoadForm») pero no acepta dirigir el control (`clase` se ignora silenciosamente); `click` solo acepta `TEdit/TComboBox/+2` (ni árboles ni `TPngBitBtn`); el árbol UIA del control no expone patrones (dibujado a mano). **Consecuencia: no se puede cargar un proyecto por GUI sin operador** → todo `variante.*` (12 funciones) + escena 3D quedan inalcanzables. 4 intentos independientes documentados. (Se logró cargar por teclado `DOWN`+`RETURN`, ver §5.)

**N19 (M) Cerrar un editor reabre su selector padre.**
Tras `cerrar` de `TPVModuleForm`, el `TChoixObjetPVForm`/`TDatabaseForm` padre reaparece como modal (2 veces observado); `cerrar` reporta `closed:true` antes de que el padre reaparezca. Flujos: cierre en orden inverso o soporte de "cerrar cadena".

**N21 (M) `gui_observar ventanas` no lista el modal que bloquea todo.**
Con `TDEMOForm` modal, `ventanas` devolvió solo `THomeForm` mientras el guarda-modal lo delataba. El observador ciego ante el bloqueo exacto.

**N25 (M) `reiniciar` con crash real no verifica la muerte.**
Con `madExceptWndClass` real en pantalla (apareció al cerrar la escena), `reiniciar` detectó el diálogo y pulsó (la acción que el E2E nunca pudo probar) pero el proceso no terminó en 20 s → error y **zombi verificado** (`tasklist`). Debe verificar muerte y escalar a kill forzado.

**N12 (M) `disenar_sistema` no puede simular sin Meteonorm y `synthetic_weather` no llega al paso.**
Sitio nuevo (NASA SSE) + simulación → `Failed to initialize Meteonorm… DataMN9/station.db.enc… Simulation cannot be run` tras 11 s, con `ok:false` honesto pero sin destilar la causa ni sugerir alternativa; `synthetic_weather:true` se acepta y no tiene efecto. La cadena sitio→proyecto→dimensionado sí funciona (4,8 kWp).

**N13 (M) `disenar_sistema` no es idempotente.**
Reintento con el mismo `nombre` tras fallo parcial → `Project exists` en el paso 2; para reintentar hay que borrar TODO y repetir el sitio. Debe continuar desde el paso fallido u ofrecer `reanudar`.

### Bloque C — Menores (B)

- **N4** El campo `ignorados` contradice lo hecho: `sobrescribir:true` que sí sobrescribió y `campos` que sí se aplicaron aparecen en `ignorados`. Renombrar a `no_aplicables` o excluir los aplicados.
- **N5** `contenido:""` crea archivo vacío con `ok` y **sin** entrada de historial (F-11 familiar); la clave vacía en `campos` sí se rechaza — inconsistente.
- **N6** `configurar` aplica pasos previos antes de fallar por valor inválido (lo reporta, sin rollback); y reporta `replaced old:X new:X` para valores idénticos (ruido).
- **N7** Mensajes a medio traducir: `Missing:/Example:` conviven con `Tipo incorrecto…esperado boolean` y claves `herramientas`. Una pasada de i18n con textos como constantes compartidas.
- **N8** `leer {objeto:'X'}` no encontrada → «see listar que:objetos» pero no lista lo disponible en línea (obliga a ida y vuelta).
- **N14** `almacenamiento {estrategia:"autoconsumo"}` aplica tamaños enormes (10s×20p de 180 Ah) sin explicar el criterio ni de dónde salen.
- **N15** Acciones gated (`geometria_solar` requiere `PVSYST_MCP_APROXIMACIONES=1`): el rechazo no menciona que la acción existe tras esa variable.
- **N16** Detalles: `rvt anadir` implícito sin historial; `pvsyst_cli` usa `opciones` (el inexistente `argumentos` se ignora); `mcp.example.json` no documenta `PVSYST_DATARO` (sin él, leer DataRO falla); `escribir_horizonte` escribió el `.HOR` bajo `Shadings/…` (verificar contra lo que PVsyst lee).
- **N22** `esperar_cierre` sobre ventana ya cerrada da error en vez de éxito trivial.
- **N23** `campo` por etiqueta falla con rótulos no-texto (`Rótulos vistos: ` vacío) aunque `vista` ve el campo; la vía útil es `rellenar` — el error debería sugerirla.
- **N26** `esperar` con plazo corto da falso negativo: la carga del proyecto tardó >12 s (timeout vencido, proyecto cargado después).
- **Extras**: listas de ventanas con estados stale (dos «vivas» ya cerradas — conviene re-leer antes de decidir); la selección por texto de `seleccionar` es por prefijo exacto, no subcadena.

### Re-verificación del E2E del 25-10 (F-01…F-30)

| Estado | Hallazgos |
|---|---|
| **Corregidos y verificados en vivo** | F-01, F-03, F-05 (como contrato), F-06, F-07, F-08/F-09 (como rechazo explícito), F-12, F-21, F-23 (parcial), F-24, F-27, F-28 + F-04 (envejecimiento) |
| **Persisten (verificado en vivo)** | F-11 (N5), F-14 (falso positivo de pestaña vía `uia_invocar seleccionar`), F-17 (`clic_texto`), F-18 (parcial: `detalle:completo` ya funciona; esquema por entidad ya es específico) |
| **Cubiertos por la suite, sin repro en vivo** | F-02, F-10, F-15/F-16 (parcial), resto |

Los 7 falles de test que traía la punta de pr/2 eran del grupo de textos (aserciones ES tras la migración a EN) — corregidos en el worktree de compilación junto con `mcpb/manifest.json` (15 descripciones); suite completa verde.

---

## 3. Cómo hacerlo más amigable para un agente

### 3.1 Contrato de error uniforme (el mayor multiplicador)

Hoy los errores son textos con calidad desigual (a veces excelentes, a veces crudos). Proponer UNA forma para todos:

```json
{
  "error": "Variable 'Ta' desconocida",
  "causa": "no es una variable de resultado de PVsyst",
  "alternativas": ["T_Amb", "T_amb_eff"],
  "siguiente": { "tool": "editar", "argumentos": {"accion":"rvt","operacion":"catalogo"} }
}
```

Reglas: (1) siempre nombrar el parámetro exacto y su tipo esperado; (2) siempre listar alternativas válidas cuando existe enumeración (catálogos, enums, clases de ventana observadas); (3) siempre un `siguiente` ejecutable. Con eso, un agente se autocorrige sin re-levantar el esquema. Casos que ya lo hacen bien y son el patrón a copiar: `crear_variante` (missing con ejemplo), `escena` (todos los campos faltantes de golpe + esquema por tipo), `carbono/p50_p90/economia` (qué activar y cómo re-simular), la guarda de modales («Ciérralo con… antes de…»), `horizonte_pvgis` rechazando `dry_run` («esa acción no tiene ensayo»).

### 3.2 Estado de sesión consultable en una llamada

Un agente pierde llamadas re-conociendo el mundo: qué PVsyst corre, qué proyecto está cargado, qué ventanas/modales hay, qué workspace, licencia restante. Propuesta: `entorno {accion:"estado"}` (o `pvsyst://estado`) que devuelva TODO eso de golpe. Las respuestas ya traen piezas (`vista`, `costo_estimado`, `avisos`) — unificarlas en un snapshot canónico evita 3-5 llamadas por sesión.

### 3.3 Promesas verificables y honestidad estructural

- El campo `ignorados` debe ser `no_aplicable` y excluir lo aplicado (N4).
- `historial_id: null` debería venir con `historial: "no_aplica_creacion"` o similar — el agente no debe deducir cuándo la promesa de historial es falsa (N10).
- Cuando algo queda abierto (editor, modal), decir QUÉ quedó abierto con suclase exacta (el cierre de la escena lo hizo bien: «puede haber abierto una confirmación»).
- `esperar` debería distinguir «timeout sin aparecer» de «apareció después» (N26) — o reintentar la comprobación una vez más antes de fallar.

### 3.4 Nombres y descubrimiento

- Normalizar extensiones al crear componentes (N2) o rechazar sin ellas diciendo la regla.
- Las acciones gated deben anunciarse: «disponible con PVSYST_MCP_APROXIMACIONES=1» (N15).
- Publicar el mapa de funciones (209) y el mapa de clases de ventana como recursos (`pvsyst://funciones`, `pvsyst://ventanas`) — hojeables sin gastar llamadas lentas (la lista de funciones tarda; la de menús, 10 s).
- Documentar en el README del servidor las claves de `vista` (patrón `tipo_x_y` coordenadas cliente) y que sirven para `campo`/`seleccionar`.

### 3.5 Idempotencia y reintentos (AGENTS.md: las operaciones reintentables DEBEN serlo)

- `disenar_sistema` con proyecto existente y parámetros iguales → continuar desde el paso fallido (N13).
- `escena_leer/escena` con su propio `.SHD` previo → sobrescribirlo (N24).
- `reiniciar` con proceso que no muere → escalar a kill (N25).
- `workspace restaurar` con `dry_run` → implementarlo o rechazarlo (N3).

---

## 4. Cómo reducir la cantidad de tool calls (medido)

Costes medidos en vivo: lectura de ficheros 0–35 ms · listados 6–550 ms · `resultados` 240–800 ms · CLI 3,3–6,6 s por llamada · simulación real ~10 s + 1 ejecución · apertura de formularios GUI 0,5–15 s. Con ese coste, cada ida y vuelta que se ahorra es tiempo real del agente.

1. **Promocionar `resultados que:"resumen"`** (ya existe): KPIs + mensual (+pérdidas) en 1 llamada (804 ms) vs 3 (≈1.400 ms + 2 llamadas). Hacerlo el `siguiente` por defecto tras `simular`.
2. **`costo_estimado` en TODO dry_run** (ya existe en `simular`): extenderlo a las rutas que lanzan GUI (`p50_p90`, `horizonte_pvgis`, `disenar_sistema`) — el agente decide sin ensayo.
3. **`listar` multi-`que`**: `listar {que:["proyectos","componentes"]}` — hoy 2 llamadas; el listado ya itera internamente, es un cambio de firma barato.
4. **`leer` multi-archivo** para lotes de lectura pequeña (el caso frecuente: leer N ficheros de una carpeta).
5. **Errores con `alternativas` inline** (3.1): elimina la llamada de descubrimiento que hoy sigue a cada error de enum/catálogo (N8/N15/N23).
6. **`funcion` con cierre emparejado**: `gui_actuar {funcion, cerrar:true}` abre-observa-cierra en 1 llamada (hoy: abrir + cerrar = 2, y el cierre falla si no se adivina la clase — ver N17/N19). El `secuencia` ya existe y funcionó: documentarlo como la vía para flujos de 2-3 pasos.
7. **Estado de sesión en 1 llamada** (3.2): sustituye 3-5 llamadas de reconocimiento por sesión.
8. **`esperar`/`esperar_cierre` más listos**: aceptar `esperar_y_verificar` (N26/N22) para no gastar una llamada extra de verificación.
9. **Recursos MCP para lo estático**: funciones (209), esquemas de `vista`, catálogo de variables (122), cobertura de ventanas (89) — hoy cuestan llamadas de 0,5–10 s; como recursos se leen una vez y el agente los cachea.
10. **Evitar rechazos por forma cuando se puede enriquecer**: aceptar `valor` numérico O string numérico ya se hace en varios sitios; extenderlo a fechas (`2026-06-21` vs `21/06/2026`) y listas (string único vs array).

Impacto estimado con 1-8 aplicados: una sesión típica de trabajo (explorar → leer → editar → simular → resultados) pasa de ~14-18 llamadas a ~7-9.

---

## 5. Eficiencia del runtime

- **Agrupar los 3 avisos de arranque**: el arranque en frío de la GUI (~15 s) es inevitable, pero `lanzar` podría devolver en su primera respuesta el árbol de la home + estado de licencia + workspace (hoy son llamadas separadas).
- **Cache de simulación**: existe (`sin_cache` para saltarlo); documentar cuándo aplica y exponer en la respuesta si el resultado vino de cache.
- **No re-compactar/leer PDF dos veces**: `resultados` lee el PDF por llamada; un cache por (proyecto, variante, mtime) evitaría re-lecturas en ráfaga (kpis→mensual→pérdidas leen el mismo PDF 3 veces: tres llamadas × ~300 ms).
- **CLI en batch**: `pvsyst_cli` con `opciones` funciona; un `lote:[{comando,...}]` para 2-3 comandos de reconocimiento (version + lic-info) ahorraría 3-6 s.
- **Fallas tempranas**: N1/N11/N12 (validar entorno antes de lanzar GUI de 10-15 s), N17 (no esperar 10,5 s una ventana que no vendrá; reportar la clase observada y salir).
- **Paralelismo documentado**: qué herramientas son seguras en paralelo (lecturas) y cuáles no (TODO lo que escribe o toca la GUI) — un agente paraleliza solo lo seguro.

---

## 6. Matriz de cobertura de las 4 pasadas

| Superficie | Cubierto en vivo |
|---|---|
| Protocolo | initialize, tools/list (2 modos), herramientas 15/15 |
| `entorno` | diagnostico ×3, licencia ×2 |
| `listar` | proyectos ×5, archivos, modulos ×5, objetos ×4 |
| `leer` | PRJ, VC0, PAN, MET+serie, objeto, base64 ida/vuelta, texto/unicode/empty, rutas raras |
| `escribir_archivo` | texto, base64, sobrescribir, vacío, unicode, 180 chars, profundas |
| `editar` | campos (variantes), rvt (crear/listar/añadir/reordenar/eliminar/catálogo), plantilla_rvt (validación), insertar_objeto, eliminar_objeto |
| `workspace` | crear (×3), historial ×4, restaurar (×3), exportar_proyecto (×3) |
| `proyecto` | crear ×4, crear_variante ×2, dimensionar ×3, doctor, anadir ×3, seguidor, limite_inyeccion, perdidas, almacenamiento ×2, bombeo, economía ×2, configurar ×3, p50_p90, renombrar, eliminar ×4, limpiar_temporales |
| `componente` | importar ×3, editar ×3, exportar, borrar ×2 |
| `sitio` | crear ×2 (reuso + nuevo), escribir_horizonte, horizonte_pvgis, meteo_pvgis (red real), convertir_meteo/geometria_solar (validación) |
| `simular` | preparar (validación), ejecutar dry_run + real ×2 |
| `resultados` | kpis ×3, mensual ×2, csv, perdidas, resumen, carbono (ciclo completo), p50_p90, economia |
| `pvsyst_cli` | version, help, help+opciones, inyección |
| `gui_observar` | 11/11 modalidades |
| `gui_actuar` | 17/26 acciones (las 9 restantes: variantes EN de las probadas o la X de `cerrar_demo`) |
| `variante.*` | 12/12 con proyecto real cargado |
| Escena 3D | leer + escribir + verificar (17 campos), ciclo completo |
| Consumo TRIAL | 138 → 134 ejecuciones (2 create-site, 2 run-simulation) |

Pendiente de ejercicio en vivo (motivo): `simular preparar` con datos reales y `barrido` (usan series de entrada; cubiertos por la suite), 9 acciones GUI con efecto destructivo (regla de seguridad), `componente crear_desde_ficha`/`disenar_sistema` con GUI de por medio (abren y guardan en la GUI; probados sus errores y validaciones).

---

## 7. Instrucciones y descripciones débiles (auditoría con coste en llamadas)

Cada ítem indica el coste medido en llamadas fallidas de esta sesión por culpa de la debilidad.

### 7.1 El campo `instructions` (initialize)

Lo bueno (conservar): el punto (2) dice qué acciones gastan ejecuciones TRIAL (úselo y funcionó); el (3) documenta el truncado y `detalle:normal/completo` (úselo y funcionó); el (6) promete `vista` y `siguiente` ejecutable (casi siempre cierto).

Lo débil:

- **(1) «the workspace argument is a per-call override and some actions ignore it»** — no dice CUÁLES. Lo observado es peor y mejor a la vez: las rutas puros-PVsyst ya no lo ignoran: lo **rechazan** con error. El texto se quedó en el comportamiento antiguo. Coste: 1 llamada de sondeo.
- **(5) «gui_actuar accion:funcion resolves its prerequisites itself»** — **falso** en los tres prerequisitos que encontré: no resuelve «proyecto cargado» (`variante.*` falla con «TProjectForm no está abierta»), no resuelve la selección de filas (N18) y no resuelve el modal `TDEMOForm` (el guarda bloquea y exige cerrarlo a mano). Coste: 3 llamadas.
- **(4) bendice el campo `ignorados`** («ignored parameters are listed in ignorados») — instruir al agente a confiar en un campo que contradice lo ejecutado (N4: `sobrescribir`/`campos` aplicados y listados como ignorados) propaga el error.
- **Faltan por completo** (coste entre paréntesis, medido): las operaciones de `componente` van por **nombre** (`componente`), no por ruta — 3 llamadas; `teclas` solo acepta **nombres de teclas**, no texto — 1; `esperar` es espera-de-**ventana**, no sleep — 1; `campo` casa por **rótulo visible**, no por clave de `vista` — 1; `escena_leer` se autobloquea con su `.SHD` — 3; los argumentos de `pvsyst_cli` van en `opciones` — 2; `disenar_sistema` elige «el primer .PAN disponible», exige Meteonorm para sitios nuevos y no es idempotente — 4; las claves de `vista` (`tipo_x_y`) son coordenadas cliente reutilizables en `campo`/`seleccionar` — descubrimiento por observación.

### 7.2 Descripciones de tools: prometen lo que no funciona, callan lo que sí

La raíz estructural: la descripción corta (la que el agente ve en el esquema) y la verdad operativa (README línea 100, larga y buena) **divergen**; el agente solo ve la corta.

| Tool | La descripción dice | Lo que pasa (medido) |
|---|---|---|
| `gui_actuar` | «click (also by on-screen text)… list or tree row…» | El clic por texto es `clic_texto` (OCR, F-17: falla con botones existentes); `fila` no puede seleccionar en `TVirtualStringTree` (N18); `click` no acepta botones ni árboles |
| `teclas` (README) | «gives it the focus first and fails when it cannot» | Devolvió `foco:false` con `estado:ok` y sin efecto — el contrato «falla cuando no puede» no se cumple (N20) |
| `escribir_archivo` | «(with history)» | Falso para creaciones nuevas (N10): `historial_id:null` |
| `componente` | «import: origen (file), optional nombre_nuevo…» | No dice que el resto de operaciones van por `componente` (nombre) — 3 fallos míos |
| `pvsyst_cli` | «Passthrough to any PVsystCLI command» | No dice que los argumentos van en `opciones`; un parámetro inventado (`argumentos`) se ignora en silencio |
| `resultados` | «KPIs, CSV, losses, monthly…» | No menciona `resumen` (KPIs+mensual+pérdidas en 1 llamada): la función de ahorro más valiosa es invisible |
| `simular` | «run (optional rows/parameters), prepare input and status» | No menciona `dry_run`+`costo_estimado` (lo mejor para agentes) ni que `preestimar`/`barrido` están tras `PVSYST_MCP_APROXIMACIONES` |
| `disenar_sistema` | «End-to-end in one call…» | No documenta elección de módulo, requisito Meteonorm, no-idempotencia ni `tolerancia_kwp_pct` |
| `esperar` | «esperar (alias: wait)» | Ambiguo: es espera-de-ventana (exige `clase`/`titulo`), no una pausa |
| `fila` (README) | «selects row N of a list or tree… (project tree TVirtualStringTree…)» | El runtime no puede hacerlo (N18): promesa y código divergen |

### 7.3 Parámetros

- **`workspace`**: «Requires a trusted MCP client» — intangible: el agente no puede saber si lo es (es política del cliente, no del agente). Mejor: decir qué pasa si no lo es («el puente lo rechazará») o quitarlo del texto.
- **`via`**: «pvsyst (default): the PVsyst route.» — circular y, sin aproximaciones, es el único valor: ruido de esquema en cada llamada.
- **Enums dobles ES/EN** (`gui_observar` 22 valores = 11 + 11 alias): duplican tokens del esquema en cada llamada. Alternativa: enum canónico + una nota «se aceptan alias EN/ES» en la description.
- **Errores de esquema del puente** («"zzz" is not one of "proyectos", "componentes" or 8 other candidates»): no lista los 8 — el agente debe abrir el esquema. Listarlos todos cuesta nada.

### 7.4 El contrato `vista.error` está documentado y no llega

El README promete errores con estructura `{codigo, motivo, causa_probable, que_hacer, claves_validas}`. Medido: los errores del servidor llegan como **texto plano** («Archivo 'No/Existe.XYZ' no encontrado; no hay ninguno en …» — 114 caracteres, sin JSON; los tests del propio repo buscan subcadenas, lo que corrobora que es texto). O se implementa el bloque en los errores (recomendado: mi §3.1) o se corrige el README. Hoy la documentación promete lo que el runtime no entrega — la definición exacta de instrucción débil.

### 7.5 Las pistas `siguiente`: buenas y malas

- **Buena** (patrón a generalizar): tras `proyecto crear` → «proyecto dimensionar {proyecto,…}» — contextual y ejecutable tal cual.
- **Malas**: tras `leer` de un `.txt` sugiere «listar objetos» de ese archivo (un texto no tiene objetos — llamada perdida segura); tras `listar` sugiere más `listar` (bucle). Regla propuesta: el `siguiente` debe ser válido para el ESTADO resultante (tipo de archivo, modo, permisos), no genérico por herramienta.

### 7.6 Cómo evitar que vuelvan a aparecer

Un test de coherencia documentación↔comportamiento, al estilo del `should_match_manifest_tools_and_declarations` que ya existe para el manifiesto: (a) cada afirmación de capacidad de una description («list or tree row», «with history», «gives focus») debe tener un test que la ejecute y falle si el runtime no cumple; (b) generar la tabla de acciones/alias del README desde el registro (hoy es manual y diverge); (c) un test que recorra los `siguiente` emitidos en una sesión grabada y valide que cada uno es ejecutable y útil para el estado en que se emite.

## 8. Prioridades propuestas

1. **N10** — historial que respalde creaciones (promesa central del producto; los usuarios crean más de lo que sobrescriben).
2. **N18** — selección de filas (UIA-valor, clic por coordenadas de control o captura de foco): desbloquea `variante.*` + escena 3D sin operador.
3. **N9** — rechazar valores no escalares en ediciones (corrupción silenciosa de componentes).
4. **N24** — escena: reutilizar/sobrescribir su propio `.SHD`.
5. **N25/N3** — `reiniciar` con verificación de muerte + `dry_run` real en restaurar (seguridad operativa).
6. **N1/N11/N12/N13** — rutas GUI: fallar temprano con causa, recoger procesos, idempotencia de `disenar_sistema`.
7. **Sección 7 (sincronizar documentación y descripciones con el runtime)** + **§3.1/§4.5** — el lote de mayor relación impacto/esfuerzo: implementar el `vista.error` ya documentado, corregir las 3 promesas falsas (`funcion` auto-resuelve, `teclas` enfoca, `fila` en árbol), y añadir a `instructions` los 8 hechos operativos que costaron llamadas.
8. **N7** — pasada final de i18n (cierra también los 7 falles de test ya corregidos).

---

## 9. Propuesta lista para pegar: nuevo texto de `instructions`

Reescritura de `INSTRUCCIONES_BASE` con los hechos que faltaban (cada frase corrige una llamada perdida medida). Inglés, como el resto de la capa de servidor:

```text
Operate PVsyst 8.1 (measured on 8.1.7 and 8.1.8) through its data files, its CLI
and its GUI. Every figure comes from PVsyst. Facts that save round-trips:
(1) components are addressed by NAME (componente:"Name"), never by path; only
    importar takes a path (origen).
(2) teclas accepts key names only (RETURN, TAB, DOWN); to type text use escribir
    with a control (campo:{clase,x,y} from the vista keys). It cannot guarantee
    focus: verify the effect instead of trusting the ok.
(3) esperar waits for a WINDOW (clase/titulo required); it is not a sleep. After a
    slow load, re-check instead of trusting one timeout (loads can exceed 12 s).
(4) campo matches the VISIBLE label; rellenar matches schema keys. vista keys like
    combobox_100_138 are client coordinates you can pass to campo/seleccionar.
(5) resultados que:resumen returns KPIs + monthly (+losses) in one call: prefer it
    over three separate reads of the same PDF.
(6) every mutating route accepts dry_run and reports costo_estimado (executions
    the call will spend): run it first. sitio crear, convertir_meteo, meteo_pvgis
    and simular spend TRIAL executions (meteo_pvgis is free when cached).
(7) lists truncate at 13 items: detalle:normal (50) or detalle:completo (all).
(8) the workspace argument is a per-call override: pure-PVsyst routes REJECT it
    with a named error. If rejected, do NOT simply retry without it: that targets
    the configured workspace, which is almost never what you wanted.
(9) GUI: launch once (single instance). The TDEMOForm trial dialog blocks every
    command: dismiss it with uia_invocar {nombre:"Cerrar", operacion:"invocar"}.
    variante.* functions need a project loaded (TProjectForm open). Loading is
    keyboard-only today: cargar -> activar -> teclas DOWN -> teclas RETURN.
(10) escena_leer leaves Shadings/<escena>.SHD behind and the next same-name read
     or write fails until it is deleted: delete it between calls or use new names.
(11) errors list the valid alternatives when there is an enumeration; a parameter
     reported in ignorados was NOT applied - verify instead of assuming.
(12) every answer carries vista (estado, resumen, campos, siguiente); each
     siguiente step is a call you can send as written. Recipes and references:
     pvsyst:// resources.
```

(Reemplaza los puntos 1-6 actuales; los hechos 1-5 y 8-11 son nuevos y cada uno corrige una llamada perdida documentada en la §7.)

## 10. Tests de regresión sugeridos (Given-When-Then, estilo del repo)

| Hallazgo | Test propuesto |
|---|---|
| N10 | `should_record_history_when_creating_a_new_file` — Given un workspace vacío When `escribir_archivo` a nombre nuevo Then `historial_id` no es null y `historial` lista 1 snapshot |
| N9 | `should_reject_object_and_array_values_in_campos` — Given un componente When `campos:{Pnom:{...}}` Then error de tipo y el archivo sin cambios |
| N24 | `should_reuse_its_own_shd_export_on_repeated_escena_leer` — Given una `escena_leer` hecha When la misma `escena_leer` de nuevo Then éxito (re-utiliza/sobrescribe su export) |
| N24b | `should_not_block_escena_write_after_escena_leer` — Given una lectura When `escena` con el mismo nombre Then éxito |
| N3 | `should_not_touch_files_when_restaurar_asks_dry_run` — Given un snapshot When `restaurar` con `dry_run:true` Then los ficheros quedan iguales y la respuesta marca `simulado:true` |
| N25 | `should_verify_process_death_and_force_kill_on_reiniciar` — Given un diálogo madExcept simulado When `reiniciar` Then el proceso muere (o kill forzado) y la respuesta lo dice |
| N1/N11 | `should_fail_fast_when_pvsyst_route_cannot_open_its_window` — Given GUI cerrada When `dimensionar` kwp-only Then error inmediato con causa (sin lanzar PVsyst ni colgar procesos) |
| N12 | `should_pass_synthetic_weather_to_the_simulation_step` — Given `disenar_sistema` con `synthetic_weather:true` sin Meteonorm Then la simulación usa meteo sintética o falla temprano con esa causa |
| N13 | `should_resume_disenar_when_project_exists_with_same_params` — Given un `disenar_sistema` fallido a mitad When repetir con los mismos parámetros Then continúa desde el paso fallido |
| N2 | `should_append_extension_when_naming_components` — Given `importar` con `nombre_nuevo` sin extensión Then el archivo lleva la extensión del tipo y `listar` lo ve |
| N4 | `should_not_list_applied_params_in_ignorados` — Given cualquier mutación con parámetros aplicados Then `ignorados` no los contiene |
| N6 | `should_validate_all_step_values_before_applying_any` — Given `configurar` con paso 2 inválido por VALOR Then nada se aplica |
| N17 | `should_map_factor_transposicion_to_the_window_it_opens` — Given la función When abrir Then la ventana esperada coincide con la observada |
| N18 | `should_select_a_row_in_the_project_load_tree` — Given TProjectLoadForm When `fila` sobre TVirtualStringTree Then la fila queda seleccionada (habilita la carga sin operador) |
| N21 | `should_list_modal_windows_in_gui_observar` — Given un modal bloqueante When `ventanas` Then el modal aparece en la lista |
| N26 | `should_recheck_once_when_a_slow_load_exceeds_the_timeout` — Given una carga >timeout When `esperar` expira Then reintenta una vez antes de fallar |

## 11. Checklist para cerrar el PR

- [ ] **Commit y push de los 7 fixes de test + `mcpb/manifest.json`** (están sin commit en el worktree `C:\Users\Lenovo\Desktop\pvsyst-pr2-build`; la suite queda verde con ellos).
- [ ] Top-3 de código: **N10** (historial en creaciones), **N9** (rechazo de tipos no escalares), **N24** (reutilizar el `.SHD`) — cada uno con su test de la §10.
- [ ] Sincronizar las 3 promesas falsas con el runtime (N18/N20/N17): o el código cumple lo que README/descripciones dicen, o el texto dice lo que el código hace.
- [ ] Implementar el `vista.error` ya documentado (`codigo/causa_probable/que_hacer/claves_validas`) o corregir el README (§7.4).
- [ ] Sustituir `INSTRUCCIONES_BASE` por la propuesta de la §9 (12 hechos, cada uno con su costo medido).
- [ ] **Ejecutar los 23 tests `--ignored` en esta máquina** (`cargo test --release -- --ignored`): tiene PVsyst 8.1.8 + DataRO + workspace real, los requisitos exactos que piden; nadie los ha corrido y pueden revelar más hallazgos ( son los que cubren F-02 y la lectura DataRO real).
- [ ] `cargo fmt`, `clippy`, `test`, `audit` en verde (AGENTS.md §8) antes del merge.
- [ ] Volver a ejecutar este protocolo en vivo tras los fixes: este documento ES lasuite de regresión manual (las reproducciones de N1-N26 son los casos).
- [ ] Housekeeping: borrar el worktree (`git worktree remove`) tras subir los fixes; decidir si los 2 informes de `docs/` entran al PR o quedan locales.
