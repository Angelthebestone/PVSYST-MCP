# Servidor MCP de PVsyst

[English](README.md) | **Español**

[![Licencia: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![Plataforma: Windows](https://img.shields.io/badge/platform-Windows-0078D4.svg)](#requisitos)
[![PVsyst 8.1.7](https://img.shields.io/badge/PVsyst-8.1.7-2E7D32.svg)](#requisitos)
[![Rust](https://img.shields.io/badge/Rust-stable-DEA584?logo=rust&logoColor=black)](#compilar-desde-el-código-fuente)
[![MCP](https://img.shields.io/badge/MCP-2025--06--18%20%7C%202025--03--26%20%7C%202024--11--05-6E56CF.svg)](https://modelcontextprotocol.io)
[![Tests: herméticos](https://img.shields.io/badge/tests-herm%C3%A9ticos%20(sin%20PVsyst)-3FB950.svg)](#comprobaciones-de-desarrollo)

**Servidor del [Model Context Protocol](https://modelcontextprotocol.io) (MCP) que permite a un
agente de IA automatizar PVsyst 8.1.7** — simulaciones por lotes, creación de proyectos y
variantes, gestión de componentes y meteorología, y extracción de resultados — leyendo y
escribiendo **solo los archivos de datos de PVsyst** y ejecutando los **comandos documentados
de `PVsystCLI`** como caja negra.

Es la API que PVsyst no tiene: no existe una interfaz pública de scripting que un agente pueda
llamar, así que este servidor le da una a través de los artefactos que PVsyst ya lee y escribe.

> **No oficial.** Sin afiliación, respaldo ni patrocinio de PVsyst SA. "PVsyst" es una marca
> registrada de su respectivo titular y aquí se usa solo para describir interoperabilidad. Este
> proyecto no contiene código de PVsyst ni redistribuye su software: necesitas tu propia
> instalación con licencia. Ver [uso responsable](#uso-responsable-licencia-y-marca).

> **Reglas duras que sigue este proyecto**
> * Nada de código fuente de PVsyst ni descompilación. `PVsyst.exe` nunca se abre en
>   Ghidra/IDA; `PVsystCLI.exe` solo se *ejecuta* como caja negra.
> * Aquí no se implementa ningún motor de simulación. Cuando hace falta simular, el MCP
>   llama por línea de comandos al comando **documentado** `PVsystCLI run-simulation`.
> * El MCP prepara, duplica e inspecciona proyectos; la física la hace PVsyst.

**Índice**

- [Por qué existe esto](#por-qué-existe-esto)
- [Qué expone](#qué-expone-15-tools-12-por-defecto)
- [Cobertura](#cobertura)
- [Requisitos](#requisitos)
- [Compilar desde el código fuente](#compilar-desde-el-código-fuente)
- [Instalarlo en un cliente MCP](#instalarlo-en-un-cliente-mcp)
- [Ejemplos de uso](#ejemplos-de-uso)
- [Formatos de archivo](#formatos-de-archivo)
- [Verificación (evidencia)](#verificación-evidencia)
- [Estructura del proyecto](#estructura-del-proyecto)
- [Preguntas frecuentes](#preguntas-frecuentes)
- [Uso responsable, licencia y marca](#uso-responsable-licencia-y-marca)
- [Contribuir](#contribuir)
- [Comprobaciones de desarrollo](#comprobaciones-de-desarrollo)

## Por qué existe esto

PVsyst es la herramienta de referencia para el diseño de sistemas fotovoltaicos y la simulación
bancable de producción, y **no trae API pública**: automatizarlo significa o escribir a mano los
archivos batch de `PVsystCLI`, o dar clics en la interfaz de escritorio. Esa es exactamente la
brecha que un agente no puede cruzar solo, y este servidor la cierra:

* **Ejecución headless.** El agente prepara el proyecto, lanza la simulación por el CLI
  documentado y lee los resultados, sin ninguna ventana por medio.
* **Ediciones que preservan bytes.** Los archivos de proyecto son registros de texto UTF-8
  `Clave=Valor` ("PVObject"); el servidor edita campos, inserta y elimina objetos sin tocar un
  byte que no se le haya pedido cambiar.
* **Todo se respalda antes.** Cualquier escritura va precedida de un snapshot, así que un
  experimento se puede deshacer.
* **La superficie que solo existe en la GUI también es alcanzable**, vía UI Automation, mensajes
  Win32 y conducción de campos de formulario, para los diálogos que no tienen equivalente en
  archivo ni en CLI.

## Qué expone (15 tools)

Cada tool toma un discriminador (`accion` o `que`); si se omite, se usa el valor por defecto
indicado. Las 15 tools están disponibles por defecto; la única superficie opcional son los
cálculos propios del MCP.

**Todo resultado sale de PVsyst.** Por defecto el MCP solo devuelve lo que calcula PVsyst
(`PVsystCLI`, la variante que simula y su informe PDF). Los cálculos propios del MCP que duplican
una función de PVsyst están detrás de `PVSYST_MCP_APROXIMACIONES=1`: sin ella, las acciones que
solo existen como cálculo del MCP no aparecen en `tools/list` y, si se llaman, responden con la
vía de PVsyst que las sustituye; con ella, su resultado lleva `origen: "aproximacion_mcp"`. Son
`simular preestimar`/`barrido`, `sitio geometria_solar`, `resultados medido_vs_simulado` y el
dimensionado automático de `disenar_sistema`. Las acciones que mantienen PVsyst como ruta por
defecto siguen visibles: `componente crear_desde_ficha`, `sitio horizonte_pvgis`, `resultados
p50_p90` y `proyecto dimensionar` a partir de `kwp` usan las ventanas de PVsyst y solo ejecutan el
cálculo del MCP con `via: "aproximacion"` y la variable activada.

| Tool | Discriminador (por defecto) | Para qué sirve |
|---|---|---|
| `entorno` | `accion`: `diagnostico` (por defecto), `licencia`, `licencia_sync`, `licencia_activar`, `licencia_desactivar`, `exportar_logs`, `logs_cli`, `preferencias`, `preferencias_editar` | Entorno, licencia, logs y preferencias de PVsyst. `licencia_activar`/`licencia_desactivar` nunca ejecutan: devuelven la plantilla del comando. |
| `disenar_sistema` | — | Coordenadas + kWp → sitio, proyecto, dimensionado, simulación y KPIs en una sola llamada. |
| `listar` | `que`: `proyectos` (por defecto), `componentes`, `modulos`, `archivos`, `objetos` | Lista proyectos, componentes, módulos (con filtros `pnom_min`/`pnom_max`/`texto`), archivos y objetos. |
| `leer` | tipo detectado desde `archivo` | Lee un proyecto, un módulo (`.PAN`), un sitio, meteo, un componente, un archivo PVObject o bytes crudos. |
| `editar` | `accion`: `campos` (por defecto), `insertar_objeto`, `eliminar_objeto` | Edita campos de cualquier objeto (preservando bytes, con snapshot previo) o inserta/elimina objetos. |
| `escribir_archivo` | — | Escribe un archivo del workspace (snapshot previo; ruta acotada). |
| `proyecto` | `accion`: `crear`, `eliminar`, `renombrar`, `crear_variante`, `dimensionar`, `doctor`, `anadir`, `seguidor`, `limite_inyeccion`, `almacenamiento`, `perdidas`, `bombeo`, `economia`, `p50_p90`, `limpiar_temporales` | Ciclo de vida de proyectos y variantes, comprobaciones y reparaciones, conversión a seguidor, límite de inyección, almacenamiento, pérdidas, bombeo, economía de PVsyst, P50/P90 y limpieza de temporales. `dimensionar` usa la ventana de pre-dimensionado de PVsyst; pasa `n_mod_serie`/`n_strings` para escribirlos tal cual. |
| `componente` | `accion`: `crear`, `borrar`, `importar`, `exportar`, `crear_desde_ficha`* | Ciclo de vida de componentes; para módulos e inversores reales, importa el `.PAN`/`.OND` del fabricante. |
| `sitio` | `accion`: `crear` (`origen`: `coordenadas` (por defecto) o `csv`), `convertir_meteo`, `meteo_pvgis`, `geometria_solar`*, `horizonte_pvgis`* | Sitios (`PVsystCLI create-site`), conversión de meteo (`convert-meteo`) y TMY horario de PVGIS convertido por PVsyst. |
| `simular` | `accion`: `ejecutar` (por defecto), `preparar`, `estado`, `preestimar`*, `barrido`* | Ejecuta simulaciones (el CSV y el informe PDF van por defecto a `Results/<proyecto>_<variante>.csv` y `.pdf`; caché, ledger de trial y trabajos en segundo plano), prepara archivos batch y devuelve `avisos_pvsyst`, los avisos que PVsyst escribió en su log. Los barridos de orientación se hacen con `filas` (modo lote `-bpf`). |
| `resultados` | `que`: `kpis` (por defecto), `csv`, `perdidas`, `mensual`, `comparar`, `carbono`, `economia`, `entregable`, `p50_p90`*, `medido_vs_simulado`* | KPIs, pérdidas, tabla mensual, balance de carbono y evaluación económica leídos del informe PDF de PVsyst; CSV horario; entregable en Markdown. |
| `workspace` | `accion`: `crear`, `exportar`, `importar`, `exportar_proyecto`, `historial`, `restaurar` | Administración del workspace y restauración del historial (snapshots). |
| `pvsyst_cli` | — | Ejecuta cualquier comando de PVsystCLI con opciones crudas. |
| `gui_observar` | `que`: `ventanas` (por defecto), `controles`, `leer`, `capturar`, `uia`, `arbol`, `mensajes`, `funciones`, `formulario`, `menus` | Observa ventanas, controles, el árbol de UI Automation y los mensajes del log de PVsyst; captura PNG; `funciones` lista el catálogo de la GUI; `formulario` empareja cada campo de un diálogo con su rótulo (OCR de Windows). |
| `gui_actuar` | `accion`: `click` (por defecto), `clic_fisico`, `clic_texto`, `mensaje`, `cerrar`, `activar`, `teclas`, `escribir`, `seleccionar`, `pagina`, `esperar_cierre`, `lanzar`, `reiniciar`, `uia_invocar`, `esperar`, `paso`, `funcion`, `menu`, `campo` | Maneja ventanas y controles de PVsyst, incluido escribir en campos medidos a 96 DPI, elegir combos y listas y cambiar pestañas; `funcion` abre una función catalogada de PVsyst (`gui_observar que:funciones`), `menu` envía un comando de menú (por `id`, o por posición `menu`+`item` leída del popup abierto), `campo` rellena un campo por su rótulo, `clic_texto` pulsa texto visible (controles sin handle), `reiniciar` recupera PVsyst tras un diálogo de fallo. |

**Recursos** (`resources/list` / `resources/read`): `pvsyst://recetas` (recetario tarea → tool), `pvsyst://cobertura` (mapa de cobertura), `pvsyst://formato` (formatos de archivo), `pvsyst://cli` (referencia del CLI), `pvsyst://gui` (catálogo de funciones de la GUI y mapa de ventanas). Una URI desconocida devuelve JSON-RPC -32602 `Recurso desconocido`.

\* La acción tiene una rama de cálculo del MCP. Su ruta por defecto es PVsyst; `via: "aproximacion"` (y, en las acciones que solo existen como aproximación, la acción misma) requiere `PVSYST_MCP_APROXIMACIONES=1`. Sin la variable, esas acciones solo-aproximación no aparecen en `tools/list`.

Todas las tools leen y escriben solo archivos dentro del workspace; nada se envía a ninguna parte, salvo las peticiones PVGIS de `meteo_pvgis` y, con aproximaciones activadas, `horizonte_pvgis` y la consulta PVGIS opcional de `p50_p90` (PVGIS `MRcalc`, `printhorizon` y `tmy` verificados en vivo). No se usan datos de Meteonorm: incluso el bloque de horizonte que `proyecto anadir` copia de un proyecto demo se vacía antes — ver [uso responsable](#uso-responsable-licencia-y-marca).

**Alcance de rutas.** Toda ruta que viene del cliente (lectura, listado, edición, import/export, salida de capturas, destinos de CSV e informes, restauración del historial) queda confinada al workspace: `..` y las rutas de fuera se rechazan. Pasa `permitir_fuera_del_workspace: true` para saltarte ese límite de forma explícita. La restauración del historial valida siempre el manifiesto completo y nunca escribe fuera del workspace. Las ediciones rechazan archivos cuya estructura PVObject está desbalanceada, y un selector con segmentos o índices mal formados (`pvSubArray[abc]`) es un error en vez de caer a `[0]`.

Los snapshots de los archivos editados viven en `UserData/mcp_history/<id>/` (con `manifest.json`); la caché de simulación y el ledger del presupuesto de trial, en `UserData/mcp_cache/`.


### Rendimiento / latencia

La ejecución de una tool es rápida (milisegundos); la lentitud percibida es **latencia del
modelo por turno más el tamaño de la salida de la tool**. Hay dos mitigaciones incorporadas:

- **`elapsed_ms`** — cada respuesta lleva la duración del lado del servidor, para medir la
  latencia en vez de suponerla.
- **`detalle`** en todas las tools: `resumen` (por defecto) recorta los arrays a 10 elementos y
  las cadenas largas; `normal` y `completo` devuelven más. Usa `completo` solo cuando haga falta.
- **`siguiente`** en los resultados correctos sugiere la siguiente llamada del flujo.


Otras palancas: agrupar llamadas independientes en un mismo turno, evitar `esperar_segundos`
largos (mejor poll), y capturar pantallas solo cuando se necesiten.

### Cobertura

Como las primitivas actúan sobre toda la superficie *datos + CLI + GUI*, el MCP cubre **cada
estado, cálculo y diálogo** de PVsyst — proyectos, variantes, todos los tipos de componente,
sitio/meteo, sombras, modelos, ejecuciones batch, sobrescritura de parámetros, licencias, logs
y las herramientas de GUI que se manejan con el ratón (editor 3D, gráficas informativas,
asistentes), gobernadas por UI Automation y Win32 (`gui_observar` / `gui_actuar`).
Mapa completo: [`docs/cobertura.md`](docs/cobertura.md) y el inventario página a página de
[`docs/pvsyst_features.md`](docs/pvsyst_features.md); el recurso `pvsyst://cobertura` devuelve la
versión legible por máquina.


### "X kWp en X coordenadas" — un flujo real

1. `sitio {accion:"crear", origen:"coordenadas", latitud, longitud, nombre}` → `.SIT` (reutiliza un sitio existente si las coordenadas coinciden).
2. `proyecto {accion:"crear", plantilla, nombre_nuevo, modulo, inversor, n_mod_serie, n_strings}`.
3. `proyecto {accion:"dimensionar", proyecto, n_mod_serie, n_strings}` → escribe `NModSerie`/`NStringCh`/`PNomSizing`.
4. `editar {accion:"campos", proyecto, ediciones:[…]}` → inclinación/azimut, pérdidas, etc.
5. `simular {proyecto, site:"…SIT", synthetic_weather:true}` (con licencia trial, pide confirmación si va a gastar dos o más ejecuciones). Revisa `avisos_pvsyst`: ahí PVsyst avisa, por ejemplo, de un Vmpp fuera de la ventana del inversor.
6. `resultados {que:"perdidas"}`, y luego `que:"mensual"`, `que:"carbono"`, `que:"entregable"`.


---

## Requisitos

* **PVsyst 8.1.7** instalado (por defecto `C:\Program Files\PVsyst8.1.7\`).
* **Windows** — el puente de GUI usa la API Win32.
* **Una licencia de PVsystCLI** para simular headless sin límite. Sin
  `C:\ProgramData\PVsystCLI\PVsystCLI.lic`, el CLI funciona en **modo TRIAL** (días y
  ejecuciones limitados; `entorno accion:licencia` muestra los contadores y el servidor lleva un
  ledger, pide confirmación antes de ejecuciones múltiples y cachea simulaciones repetidas). Para
  activarla, ejecuta tú mismo `"C:\Program Files\PVsyst8.1.7\PVsystCLI.exe" lic-activate -k:<TU_CLAVE>`
  (`entorno accion:licencia_activar` imprime el comando; el servidor nunca lo ejecuta). Todo lo
  demás (archivos, sitios, dimensionado, GUI) funciona sin licencia.
* **Rust** (cualquier stable reciente).
* Un **enlazador**: MSVC (`link.exe` de VS Build Tools) — o, como en la máquina donde se
  construyó esto, MinGW-w64 con la toolchain GNU:

  ```powershell
  winget install BrechtSanders.WinLibs.POSIX.UCRT      # gcc/ld para el target GNU
  rustup toolchain install stable-x86_64-pc-windows-gnu
  rustup override set stable-x86_64-pc-windows-gnu     # por proyecto (este directorio)
  # asegúrate de que <WinLibs>\mingw64\bin está en el PATH para que cargo encuentre gcc.exe
  ```

---

## Compilar desde el código fuente

```powershell
cargo build --release
# → target\release\pvsyst-mcp.exe
```

Ejecutar los tests:

```powershell
cargo test
```

---

## Instalarlo en un cliente MCP

### Claude Desktop (bundle de un clic)

Descarga el archivo `.mcpb` de la
[última release](https://github.com/Angelthebestone/PVSYST-MCP/releases/latest) y
ábrelo (o arrástralo a *Settings → Extensions*). Claude Desktop pide el workspace de
PVsyst, `PVsystCLI.exe` y `PVsyst.exe`; deja las rutas vacías para que el
servidor las autodetecte. El bundle se genera a partir de [`mcpb/manifest.json`](mcpb/manifest.json).
El bundle no se actualiza solo: instala el `.mcpb` nuevo de cada release.

### npm / npx (se actualiza solo)

```json
{
  "mcpServers": {
    "pvsyst": { "command": "npx", "args": ["-y", "pvsyst-mcp"] }
  }
}
```

El lanzador de npm mantiene el servidor al día por sí mismo. Cada arranque usa el binario más
nuevo que tiene (el empaquetado o uno descargado antes en `%LOCALAPPDATA%\pvsyst-mcp`) y, con
el servidor ya en marcha, consulta npm desde un proceso en segundo plano desacoplado. Una
versión más nueva se descarga para el **siguiente** arranque; la sesión en curso nunca cambia
de binario. El registro está en `%LOCALAPPDATA%\pvsyst-mcp\actualizacion.log`. Pon
`PVSYST_MCP_AUTO_ACTUALIZAR=0` para desactivarlo. Solo se actualizan así las instalaciones por
npm/npx; el zip y el `.mcpb` no.

### Command Code (Windows)

> **Importante — la trampa del espacio en la ruta.** Command Code no entrecomilla la ruta del
> ejecutable cuando lanza un servidor MCP por stdio, así que un `command` con un espacio (por
> ejemplo `...\PVSYST MCP\...`) **falla al arrancar en silencio**: el servidor aparece listado
> pero nunca conecta y sus tools no llegan a verse. Instala el binario en una ruta **sin
> espacios**.

```powershell
# 1. compila release y cópialo a C:\pvsyst-mcp\pvsyst-mcp.exe
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1

# 2. regístralo (el binario es cmd / cmdc / command-code; en Windows usa "cmdc")
cmdc mcp add --scope user --env PVSYST_WORKSPACE=C:\Users\Lenovo\PVsyst8.1_Data pvsyst -- "C:\pvsyst-mcp\pvsyst-mcp.exe"
cmdc mcp list
```

`mcp.example.json` muestra una configuración lista para copiar para ese binario en ruta sin espacios. `.mcp.json` está ignorado, así que copia el ejemplo y ajusta las rutas:

```json
{
  "mcpServers": {
    "pvsyst": {
      "transport": "stdio",
      "enabled": true,
      "command": "C:\\pvsyst-mcp\\pvsyst-mcp.exe",
      "env": { "PVSYST_WORKSPACE": "C:\\Users\\Lenovo\\PVsyst8.1_Data" }
    }
  }
}
```

Las tools aparecen entonces como `mcp__pvsyst__listar_proyectos`, etc. Vuelve a ejecutar
`scripts\install.ps1` después de cada recompilación.

### Otros clientes MCP

Cualquier cliente que hable MCP por **stdio** puede usarlo (Claude Desktop, Cursor, VS Code y
cualquier otro que lea el mapa habitual `mcpServers`). Apunta el cliente al binario en ruta sin
espacios y dale la carpeta del workspace:

```json
{
  "mcpServers": {
    "pvsyst": {
      "command": "C:\\pvsyst-mcp\\pvsyst-mcp.exe",
      "env": { "PVSYST_WORKSPACE": "C:\\Users\\Lenovo\\PVsyst8.1_Data" }
    }
  }
}
```

Mantenlo **local**: el servidor está pensado para la máquina que tiene tu licencia de PVsyst, no
para un endpoint compartido — ver [uso responsable](#uso-responsable-licencia-y-marca).

**Notas de compatibilidad.** El servidor devuelve el `protocolVersion` que pide el cliente cuando
lo soporta (`2024-11-05`, `2025-03-26`, `2025-06-18`; Command Code pide `2025-03-26`) y, si no,
responde con su versión soportada más reciente, como exige la especificación MCP. Expone cinco
recursos (`pvsyst://recetas`, `pvsyst://cobertura`, `pvsyst://formato`, `pvsyst://cli`, `pvsyst://gui`) y siete
prompts construidos a partir de esas recetas. Con `2025-06-18`, los resultados de las tools
también llevan `structuredContent`; las simulaciones largas reportan progreso y aceptan
`notifications/cancelled`. Los argumentos de cada tool se validan contra su esquema antes de
despachar. Comprueba la conexión con `/mcp` dentro de una sesión, o headless:
`cmdc -p "usa la tool pvsyst entorno" --tools-all -t --yolo`.


### Variables de entorno (todas opcionales)

Un valor vacío cuenta como no definido, así que se aplica el valor por defecto o la autodetección.

| Variable | Significado | Por defecto |
|---|---|---|
| `PVSYST_WORKSPACE` | Carpeta del workspace del usuario | `%USERPROFILE%\PVsyst8.1_Data`, luego `Documents\…` |
| `PVSYST_CLI` | Ruta a `PVsystCLI.exe` | `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe` |
| `PVSYST_GUI` | Ruta a `PVsyst.exe` | `C:\Program Files\PVsyst8.1.7\PVsyst.exe` |
| `PVSYST_MCP_APROXIMACIONES` | `1` activa los cálculos propios del MCP (marcados `origen: "aproximacion_mcp"`) | sin definir (solo resultados de PVsyst) |
| `PVSYST_MCP_ESCRITORIO` | `aislado` ejecuta PVsyst y las tools de GUI en un escritorio Win32 oculto y propio (el agente nunca toma tu foco, cursor ni teclado); `compartido` usa tu escritorio. Cualquier otro valor es un error | `compartido` |
| `PVSYST_MCP_AUTO_ACTUALIZAR` | Solo lanzador npm: `0` desactiva la auto-actualización | activada |
| `PVSYST_MCP_ALMACEN` | Solo lanzador npm: dónde se descargan las actualizaciones | `%LOCALAPPDATA%\pvsyst-mcp` |


> **Nota sobre la ruta del workspace.** PVsyst 8 guarda los datos de usuario en
> `%USERPROFILE%\PVsyst8.1_Data` (se crea en el primer arranque), no en `Documents\PVsyst8.1`.
> El MCP detecta el workspace real automáticamente y también acepta un argumento `workspace`
> explícito en todas las tools. Ese argumento mueve la raíz del confinamiento (todas las
> comprobaciones de ruta son relativas a él), así que es una capacidad deliberada de este
> servidor local: conecta solo clientes MCP en los que confíes.

---

## Ejemplos de uso

```
listar                 { "que": "modulos", "texto": "CX-M500" }
leer                   { "archivo": "CX-M500_2450Wp_1_00_086.PAN" }
→ { "format": "legacy-real48", "vendor": "Soitec", "pnom": 2450.0,
    "voc": 695.7, "isc": 3.861, ... }

disenar_sistema         { "latitud": 7.07, "longitud": -73.84, "kwp": 10,
                          "n_mod_serie": 10, "n_strings": 3 }
→ { "kpis": { "energia_producida_kwh": 15517, "specific_yield_kwh_per_kwp": 1567, "pr": 83.0 },
    "site": { "weather_source": "NASA SSE 6.0 grid cell 7/-74 ..." }, "pasos": [ ... ] }

proyecto                { "accion": "crear", "plantilla": "_DEMO_Commercial",
                          "nombre_nuevo": "MY_VARIANT",
                          "modulo": "CX-M500_2450Wp_1_00_086.PAN" }

simular                 { "proyecto": "MY_VARIANT", "variante": "VC0",
                          "output_csv": "C:\\...\\Results\\out.csv",
                          "report_pdf": "C:\\...\\Results\\out.pdf" }
→ { "success": true, "stdout_tail": "... Simulation done in 0min 3sec ...", "costo": { ... } }

resultados              { "que": "kpis" }

```

---

## Formatos de archivo

`docs/pvsyst_format.md` documenta cada tipo de archivo observado desde fuera, incluido el
formato recuperado del `.PAN` binario legacy (Delphi `Real48`, 50 ranuras de 6 bytes) con su
tabla de campos decodificada. Hay un decodificador de referencia independiente en Python en
`tools/pan_decode.py`.

`PVSYST_CLI_REFERENCE.md` es la referencia exhaustiva y verificada de `PVsystCLI.exe` (comandos,
gramática exacta de opciones, modo batch y detección de éxito).

---

## Verificación (evidencia)

* Tests: `cargo test` ejecuta los unitarios y los smoke tests end-to-end de `tests/mcp_smoke.rs`, que manejan el servidor compilado por stdio igual que un cliente. Los tests que necesitan PVsyst real se saltan cuando PVsyst no está.
* `leer` decodifica el módulo legacy `CX-M500` incluido y da **PNom = 2450 W** (exacto), Voc = 695.7 V, Isc = 3.861 A.
* **Flujo completo coordenadas → potencia → simulación → resultados** (todo con tools MCP, registrado en la superficie de 15 tools antes de la consolidación): se creó el sitio desde coordenadas (lat 46.2044, lon 6.1432); `proyecto crear` construyó una variante de ~507 kWp; `simular` la ejecutó en el sitio nuevo con meteo sintética → **`Simulation done in 0min 2sec`** + PDF generado; `resultados que:csv` parseó el CSV de 330 KB (8 760 filas horarias, `date;GlobInc;GlobEff;EArray;E_Grid;PR`).
* `editar` (campos), `proyecto dimensionar` y `workspace restaurar` están cubiertos por smoke tests; los snapshots del historial se comprueban vía `UserData/mcp_history/<id>/manifest.json`.
* Los errores de validación de PVsyst se propagan (`success:false`), por ejemplo un array sobredimensionado rechazado por PVsyst con "array Voc … greater than the inverter absolute maximum input voltage".
* **Automatización de GUI verificada contra la aplicación real**: `gui_observar que:ventanas|controles` enumeró las ventanas y controles VCL reales de PVsyst; `gui_actuar accion:click` sobre el botón *Herramientas* abrió `TToolsForm`; `gui_observar que:capturar` produjo una captura; `gui_actuar accion:cerrar` la cerró. Cubierto por el test con puerta `gui_invokes_button_and_opens_window`.

Todas las tablas, figuras e informes citados arriba se produjeron con **PVsyst 8.1.7
(PVsystCLI)**; ver [cómo citar](#cómo-citar-este-trabajo).


## Estructura del proyecto

Una carpeta por área de negocio de PVsyst; las dependencias apuntan hacia dentro, a `format/`
(núcleo puro, no importa nada del crate). `server/` (entrada) y `adapters/` (salida) son los
bordes. `tools/check_deps.py` hace fallar el CI ante cualquier ciclo; la estratificación, de
abajo arriba, es:

`error` → `format`, `state` → `config` → `args` → `history` → `workspace` →
`adapters` → módulos de capacidad (`components`, `objects`, `projects`, `sites`,
`simulation`, `admin`, `prefs`, `easy_mode`) → `server`

Ningún archivo de producción en `src/` pasa de 300 líneas; las áreas grandes son módulos de
directorio divididos por concepto. Las dos suites end-to-end antiguas (`tests/mcp_smoke.rs` y
`tests/mcp_regresiones.rs`) son anteriores a ese límite.

```
src/
  main.rs              punto de entrada por stdio
  config.rs            resolución de rutas de workspace / CLI / GUI
  state/               registros por servidor (trabajos en segundo plano, caché UIA) y sus registros
  args.rs              extracción de argumentos de las tools (alias, errores tempranos)
  server/              adaptador de entrada: MCP por stdio
    mod.rs             protocolo JSON-RPC / MCP (delimitado por saltos de línea, negociación de versión)
    tools/             una declaración por tool: esquema + validación + handler (por familia)
  format/              núcleo: formatos de archivo de PVsyst (sin I/O)
    object.rs          parser genérico de texto PVObject (seguro con BOM) + diagnóstico estructural
    selector.rs        parser estricto de selectores de objeto (`pvVCalcul/pvSubArray[1]`)
    span.rs            escáner de spans de objeto que preserva bytes
    edit.rs / insert.rs / delete.rs   edición de campos, inserción/extracción y borrado de objetos
  adapters/            adaptadores de salida hacia los ejecutables de PVsyst
    cli/               ejecutor de PVsystCLI: captura acotada, clasificación tipada, trabajos en segundo plano
    gui/               automatización de UI Win32 (ventanas, acciones, esperas, captura GDI con RAII, PNG)
    uia/               árbol de UI Automation y operaciones sobre elementos
  history/             snapshots, restauración validada, escrituras atómicas (temporal único, bloqueo por ruta)
  workspace/           alcance de rutas (`resolver_ruta`), lecturas acotadas, listado con guardas, escrituras
  objects/             leer_objeto / listar_objetos / editar_objeto
  projects/            listado, lifecycle/, doctor/, anadir/, temporales
  components/          todos los tipos de componente; pan/ (Real48, decodificador legacy estricto), ficha/
  sites/               sitios, lectores, rejillas NASA, solar/, pvgis/ (por etapas, transaccional)
  simulation/          preparación, ejecución, estado, batch, cache/, results/, report_pdf/, engineering/
  admin/               import/export de workspace/proyecto/componente, licencia, logs
  prefs/               preferencias de PVsyst.ini y diagnóstico de logs del CLI
  easy_mode/           disenar_sistema: pipeline tipado por pasos, diagnostico, leer_kpis
tests/mcp_smoke.rs     tests end-to-end del protocolo
docs/pvsyst_format.md    formatos de archivo (observados desde fuera)
docs/cobertura.md        mapa de cobertura (área -> tool, frontera solo-GUI)
docs/pvsyst_features.md  inventario completo de las 623 páginas de funcionalidad
tools/pan_decode.py      decodificador de referencia
```

---

## Preguntas frecuentes

### ¿Se puede simular con PVsyst sin abrir la GUI?

Sí. `simular` prepara la variante, llama por línea de comandos al comando documentado
`PVsystCLI run-simulation`, interpreta los marcadores de éxito y devuelve los KPIs: no se abre
ninguna ventana. Las tools de GUI (`gui_observar`, `gui_actuar`) existen para los diálogos
que no tienen equivalente en archivo ni en CLI; están disponibles por defecto.

### ¿Hace falta licencia de PVsyst?

Para simular headless, sí. `PVsystCLI` funciona en modo TRIAL sin licencia (con los límites de
evaluación, ver [uso responsable](#uso-responsable-licencia-y-marca)); el servidor lleva el
presupuesto restante en un ledger, pide confirmación antes de una ejecución que gastaría varias
y cachea simulaciones repetidas. El trabajo a nivel de archivo — listar, leer, editar, sitios,
dimensionado, creación de componentes — no necesita licencia alguna.

### ¿Con qué clientes MCP funciona?

Con cualquiera que hable MCP por stdio y lea el mapa habitual `mcpServers`: Command Code, Claude
Desktop, Cursor, VS Code y otros. Ver
[instalarlo en un cliente MCP](#instalarlo-en-un-cliente-mcp). El servidor no abre ningún puerto
de red.

### ¿Está afiliado a PVsyst SA?

No. Es un proyecto de interoperabilidad no oficial: sin código de PVsyst, sin descompilación, sin
redistribuir software de PVsyst y sin gestión de licencias. `PVsyst.exe` nunca se abre en un
desensamblador; `PVsystCLI.exe` solo se ejecuta.

### ¿Puede corromper mis proyectos?

Solo edita lo que le pides, a nivel de byte, y rechaza archivos cuya estructura PVObject está
desbalanceada. Toda escritura va precedida de un snapshot en `UserData/mcp_history/<id>/`, y
`workspace accion:restaurar` valida el manifiesto completo antes de escribir nada de vuelta.
Ten en cuenta que PVsyst documenta su formato como interno y sin soporte, así que la red de
seguridad son los snapshots, no el soporte de PVsyst.

### ¿Envía mis datos a alguna parte?

No. Las únicas peticiones salientes son las llamadas a PVGIS que tú provoques: `meteo_pvgis` y,
con `PVSYST_MCP_APROXIMACIONES=1`, `horizonte_pvgis` y la consulta opcional de
`resultados que:p50_p90`. No se sube nada y no se recoge telemetría.

### ¿Sirve para lotes y barridos de parámetros?

Sí: `simular accion:ejecutar` con `filas` lanza un lote de PVsyst (`-bpf`, una fila por
orientación o configuración, cada una cuenta como ejecución), `simular accion:preparar` escribe
archivos batch y de parámetros, y las ejecuciones largas pueden
lanzarse como trabajos en segundo plano que se consultan con `simular accion:estado`.

### ¿Puede leer los módulos `.PAN` binarios antiguos?

Sí. El formato legacy de Delphi `Real48` se recuperó desde fuera y lo decodifica
`components/pan/`; `tools/pan_decode.py` es un decodificador de referencia independiente que
puedes ejecutar sin el servidor.

### ¿Qué versiones de PVsyst y qué tipos de archivo soporta?

Por defecto apunta a PVsyst 8.1.7, con las rutas del ejecutable y del workspace configurables por
`PVSYST_CLI`, `PVSYST_GUI` y `PVSYST_WORKSPACE`. Los artefactos soportados incluyen proyectos y
variantes `.PRJ` / `.VCi`, componentes `.PAN` (texto y binario legacy) y `.OND`, sitios `.SIT`,
archivos meteorológicos `.MET` y texto PVObject crudo.

### ¿Por qué no usar una API de PVsyst?

Porque no existe una API pública que usar. Este proyecto toma a propósito las dos vías
soportadas — los comandos documentados de `PVsystCLI` y los archivos de datos — en lugar de
reimplementar un motor de simulación, que te daría números distintos a los de PVsyst.

---

## Uso responsable, licencia y marca

Esta es una herramienta no oficial que maneja **tu** instalación de PVsyst, así que las
condiciones que aplican al programa son las que aceptaste al instalarlo: las
[*Condiciones Generales de Uso*](https://www.pvsyst.com/pdf/common/general-conditions-of-use-english.pdf)
de PVsyst SA y tu contrato de suscripción. Así se posiciona este proyecto frente a ellas:

* **Sin descompilación ni modificación de los programas.** Las CGU prohíben reproducir,
  modificar, traducir, descompilar o recompilar el Software. Este proyecto nunca toca
  `PVsyst.exe`/`PVsystCLI.exe`: lee y escribe *tus* archivos de datos y ejecuta el CLI
  documentado.
* **Mantenlo local.** Las CGU prohíben hacer accesible el Software a terceros y obligan a
  impedir el acceso no autorizado, con una licencia por puesto de trabajo. No expongas este
  servidor como endpoint compartido o remoto: está pensado para la máquina que tiene la licencia,
  y su argumento `workspace` más `permitir_fuera_del_workspace` le dan a cualquier cliente
  conectado un alcance real sobre el sistema de archivos.
* **La automatización de GUI se queda en el puesto de trabajo.** PVsyst 8 no debe usarse en un
  servidor ni operarse desde un cliente de Escritorio Remoto de Windows; `PVsystCLI` sí puede
  explícitamente. Eso se traduce en esta superficie: las tools basadas en el CLI son válidas en
  un servidor, las de GUI `gui_observar` / `gui_actuar` no.
* **Nunca toques el reloj.** Las licencias dependen de que la fecha y la hora de la máquina no
  se hayan modificado nunca; nada en este proyecto las lee ni las cambia.
* **Ningún dato de Meteonorm sale de PVsyst.** Las CGU prohíben exportar datos de Meteonorm a
  otro software, así que el MCP no los usa: los sitios vienen de coordenadas (NASA SSE) o del TMY
  de PVGIS, y el bloque de horizonte derivado de Meteonorm de un proyecto demo se vacía antes de
  copiarlo.
* **Los límites de evaluación se respetan, no se saltan.** El servidor solo *cuenta* las
  ejecuciones de trial que quedan para avisarte antes de gastarlas, y
  `entorno accion:licencia_activar` se limita a imprimir el comando de activación para que lo
  ejecutes tú con tu clave. El archivo de licencia, el registro y el reloj del sistema nunca se
  escriben.
* **Editar archivos es decisión tuya.** PVsyst documenta su formato como de uso interno y sin
  soporte; editarlo es legal — son tus archivos de proyecto — pero PVsyst no dará soporte a un
  archivo que este proyecto (o tú) haya roto. Valida con `proyecto accion:doctor` y conserva los
  snapshots.

### Cómo citar este trabajo

La cláusula 8.4 de las CGU exige citar a PVsyst siempre que publiques imágenes o tablas
generadas por él. Todos los resultados citados en este repositorio — incluido el informe de
ejemplo en `tests/fixtures/results/` — se generaron con:

> PVsyst 8.1.7 (PVsystCLI), PVsyst SA, Satigny, Suiza — https://www.pvsyst.com

### Licencia de este programa

`pvsyst-mcp` está licenciado bajo la **Apache License 2.0** ([`LICENSE`](LICENSE)), con el aviso
de atribución en [`NOTICE`](NOTICE). Si haces un fork o redistribuyes una versión modificada,
debes conservar los avisos de atribución y marcar los archivos que cambiaste (secciones 4(b)–4(d)),
y la sección 6 no concede ningún derecho sobre el nombre del proyecto. Las contribuciones se
aceptan bajo la misma licencia — ver [CONTRIBUTING.md](CONTRIBUTING.md) (en inglés).

## Contribuir

Los issues y pull requests son bienvenidos. Empieza por [CONTRIBUTING.md](CONTRIBUTING.md) para
las reglas básicas (nada de datos del fabricante en el repositorio, nada de descompilación, tests
basados en comportamiento), el [Código de Conducta](CODE_OF_CONDUCT.md) y
[SECURITY.md](SECURITY.md) para reportar una vulnerabilidad en privado. Los cambios se registran
en [CHANGELOG.md](CHANGELOG.md).

## Comprobaciones de desarrollo

```powershell
cargo fmt --check
cargo clippy --all-targets -- -D warnings   # los lints (unwrap/panic/unsafe documentado/complejidad/...) viven en Cargo.toml + clippy.toml
cargo test                                  # herméticos: workspace sintético en tests/fixtures/workspace, no hace falta PVsyst
cargo test -- --ignored                     # además los tests que necesitan los datos demo de PVsyst 8.1.7 o la GUI abierta
cargo audit; cargo deny check               # avisos, licencias, vetos y orígenes (deny.toml)
```

El crate es una librería (`pvsyst_mcp`: `Paths`, `Server`, `serve`) más un binario fino
`pvsyst-mcp`. Los errores son tipados (`src/error.rs`); los logs van a stderr y se filtran con
`PVSYST_MCP_LOG` (por defecto `info`) porque stdout transporta el protocolo.
