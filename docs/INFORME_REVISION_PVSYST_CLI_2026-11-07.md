# Revisión de `pvsyst_cli` — MCP `pvsyst` (PVsyst 8.1.8)

**Fecha real de la revisión:** 2026-11-08, 01:47–02:10 (hora local HPS; 06:47–07:10 UTC).
El nombre del archivo conserva la fecha del encargo (2026-11-07).
**Rol:** revisor read-only. No se modificó el PR, ni el código, ni la configuración; no se ejecutó ninguna simulación real ni se activó/desactivó licencia.
**Código revisado:** worktree `C:\Users\Lenovo\Desktop\PVSYST MCP\.claude\worktrees\tarea-d-fila`, rama `worktree-tarea-d-fila`, HEAD `5dbfdd0d2973a5e4f2990930f3d7aef693d17873`.
**Binario revisado:** `C:\pvsyst-mcp\pvsyst-mcp.exe` (v0.3.0), idéntico byte a byte al `target/release/pvsyst-mcp.exe` del worktree.

---

## 1. Resumen ejecutivo y veredicto

`pvsyst_cli` es un passthrough deliberadamente crudo: el nombre del comando va como primer `argv` y `opciones` como tokens adicionales; no hay shell, no hay lista blanca y no hay confinamiento de rutas. Esa parte está bien resuelta (argv separado, sin inyección de shell, captura con tope de 1 MiB por flujo, timeout con kill, `stderr` separado, `vista` en éxito/error/dry-run).

**Veredicto: NO es seguro/consistente todavía como passthrough de producción**, por cuatro problemas de nivel P1 y doce P2. El PR **no introduce** los P1: los hereda de `main` (el P1 principal viene del commit `8a16235`, del 2026-10-25). El PR sí mejora el adaptador en dos puntos reales: rechaza elementos no-texto de `opciones` (antes se descartaban en silencio) y acota la espera de los hilos lectores a 5 s (antes un nieto con la tubería abierta podía bloquear la respuesta). Pero el binario instalado, construido desde este HEAD, contiene todos los defectos descritos.

Lo más grave, en una frase: **`pvsyst_cli help --bogus`, `help help`, `version --bogus` y `lic-info --bogus` devuelven `success:true` y `clasificacion:"exito"` aunque PVsystCLI imprima `Error: Invalid command syntax` / `Unknown command`**. El agente que usa la tool para diagnosticar recibe una confirmación falsa. En el sentido contrario, `VERSION` y `Help` (el CLI es case-insensitive) se clasifican como `ambiguo` y `success:false`.

Además, `dry_run:null` **ejecuta de verdad** (el `null` se interpreta como `false`), y la documentación afirma dos invariantes que `pvsyst_cli` rompe: "el servidor nunca activa ni desactiva la licencia" y "todas las tools leen/escriben solo dentro del workspace". El passthrough puede ejecutar `lic-activate`/`lic-deactivate`, escribir en cualquier ruta absoluta y resolver rutas relativas contra el CWD del servidor, no contra el workspace.

---

## 2. Matriz de pruebas read-only

Se hicieron **90 llamadas `tools/call` + 1 `tools/list`** contra el binario instalado, más **6 ejecuciones directas** de `PVsystCLI.exe` (`help`, `help run-simulation`, `help lic-activate`, `version`, `lic-info` ×2) para comparar la salida completa con la devuelta por la tool. Todas las llamadas fueron de solo lectura salvo una invocación `run-simulation --bogus` que se rechazó en el parseo de opciones (ver §7, limitaciones).

### 2.1 Casos representativos

| # | Llamada JSON | Resultado | Evidencia recortada |
|---|---|---|---|
| A | `{"comando":"version"}` | `success:true`, `exito`, 5,51 s | `PVsystCLI 8.1.8 (rev. 49527)` / `stdout_tail` 171 B |
| B | `{"comando":"help"}` | `success:true`, `exito`, 3,44 s | `stdout_tail` 813 B, **falta la línea `lic-activate`** (el CLI la imprime) |
| C | `{"comando":"help","opciones":["run-simulation"]}` | `success:true`, `exito`, 3,51 s | `stdout_tail` 4 333 B = 60 líneas; el CLI emite 65 líneas (4 478 B): se pierden las 5 primeras, incluida `Usage: pvsystcli run-simulation <options>` |
| D | `{"comando":"lic-info"}` | `success:true`, `exito`, 5,39 s | `Status: TRIAL` / `Remaining days: 30` / `Remaining executions: 134` (lectura de las 01:47) |
| E | `{"comando":"no-existe-xyz"}` | `success:false`, `error` | `detected_error:"Invalid command syntax"`; `exit_code:0` |
| F | `{"comando":"help","opciones":["--bogus"]}` | **`success:true`, `exito`** | `stdout_tail:"Error: Invalid command syntax\n\nUnknown command line option : --bogus..."` |
| G | `{"comando":"help","opciones":"run-simulation"}` | error de tipo | `Wrong type for 'opciones' (expected array\|object, got string)` |
| H | `{"comando":"help","opciones":null}` | `success:true` | `command_line:"...PVsystCLI.exe help"` |
| I | `{"comando":"help","opciones":{}}` | `success:true` | `command_line:"...PVsystCLI.exe help"` |
| J | `{"comando":"help","options":["run-simulation"]}` | `success:true` | alias `options` → `opciones`; `command_line:"... help run-simulation"` |
| K | `{"comando":"help","command":"version"}` | error de conflicto | `Argument conflict: both 'comando' and its alias 'command' were passed with different values` |
| L | `{"comando":"   "}` | `success:false`, `error` | `command_line:"... \"   \""`; el CLI responde `Invalid command syntax` (no se recorta el comando) |
| M | `{"comando":""}` | `missing_arg` | `Missing required argument 'comando' (e.g. "help", "version", "lic-info")` |
| N | `{"comando":"VERSION"}` | **`success:false`, `ambiguo`** | salida correcta `PVsystCLI 8.1.8 (rev. 49527)`; `motivo_ambiguedad:"Salida sin confirmación..."` |
| O | `{"comando":"Help"}` | **`success:false`, `ambiguo`** | salida correcta de ayuda; mismo motivo |
| P | `{"comando":" help"}` | `success:false`, `error` | `command_line:"... \" help\""`; `Invalid command syntax` |
| Q | `{"comando":"version","log_level":3}` | `success:true`, `ignorados:["log_level"]` | `command_line:"... version"` (no se pasa `-ll`) |
| R | `{"comando":"version","foo":1,"workspace":"C:/x"}` | `success:true`, `ignorados:["foo","workspace"]` | `command_line:"... version"` |
| S | `{"comando":"version","timeout_s":0}` | `success:false`, `timeout` | `detected_error:"timeout tras 1 s"`; 1,13 s |
| T | `{"comando":"version","timeout_s":1.5}` | error de tipo | `Wrong type for 'timeout_s' (expected integer, got number)` |
| U | `{"comando":"help","detalle":"completo"}` | `success:true` | `stdout_tail` 813 B, **idéntico** a B: `detalle` no cambia nada y no aparece en `ignorados` |
| V | `{"comando":"help","via":"aproximacion"}` | `success:true` | `ignorados` ausente: `via` se acepta y se ignora en silencio |
| W | `{"comando":"help","dry_run":null}` | **ejecuta** (`success:true`, sin `dry_run`) | `command_line:"... help"`; `null` se trata como `false` |
| X | `{"comando":"help","opciones":["help"]}` | **`success:true`, `exito`** | `stdout_tail:"Unknown command \"help\"..."` |
| Y | `{"comando":"version","opciones":["--bogus"]}` | **`success:true`, `exito`** | `stdout_tail:"Error: Invalid command syntax..."` |
| Z | `{"comando":"lic-info","opciones":["--bogus"]}` | **`success:true`, `exito`** | `stdout_tail:"Error: Invalid command syntax..."` |
| AI | `{"comando":"help","opciones":["version","help"]}` | **`success:true`, `exito`** | `stdout_tail:"Error: Invalid command syntax\n\nUnknown option : help..."` |
| AJ | `{"comando":"version","opciones":{"bogus":1}}` | **`success:true`, `exito`** | `command_line:"... version --bogus:1"`; `stdout_tail:"Error: Invalid command syntax..."` |

### 2.2 `dry_run` (ningún archivo, ningún proceso)

| # | Llamada JSON | Resultado | Evidencia |
|---|---|---|---|
| D1 | `{"comando":"run-simulation","dry_run":true,"opciones":["-w:...","-p:SYNTH_Plant.PRJ","-v:VC0","-ocf:<tmp>/out.csv","-rpf:<tmp>/out.pdf","-ll:3"]}` | `{"dry_run":true,"command_line":"... run-simulation -w:... -p:... -v:VC0 -ocf:... -rpf:... -ll:3"}` | `elapsed_ms:0`; el directorio temporal quedó **vacío**; `vista.estado:"ok"`, `resumen:"Ensayo, no se aplicó nada..."` |
| D2 | `{"comando":"run-simulation","dry_run":true,"opciones":{"workspace":"...","project":"SYNTH_Plant.PRJ","variant":"VC0","output-csv-file":"...","report-pdf-file":"...","log-level":3}}` | `command_line:"... run-simulation --log-level:3 --output-csv-file:... --project:... --report-pdf-file:... --variant:VC0 --workspace:..."` | claves ordenadas alfabéticamente; todas con `--` |
| D3 | `{"comando":"run-simulation","dry_run":true}` | `command_line:"... run-simulation"` | no valida `-w`/`-p`/`-v` |
| D4 | `{"comando":"lic-activate","dry_run":true,"opciones":["-k:XXXX"]}` | `command_line:"... lic-activate -k:XXXX"` | `vista.siguiente[0]` = la misma llamada **sin** `dry_run`, motivo `"Aplicar de verdad lo ensayado"` |
| D5 | `{"comando":"run-simulation","dry_run":true,"timeout_s":7}` | `{"dry_run":true,...}` | `timeout_s` no se usa ni se reporta |
| D6 | `{"comando":"run-simulation","dry_run":true,"opciones":5}` | error de tipo | `Wrong type for 'opciones' (expected array\|object, got integer)` |
| D7 | `{"comando":"run-simulation","dry_run":true,"log_level":9}` | `{"dry_run":true,...}` | `ignorados:["log_level"]`; no se valida el rango |
| D8 | `{"comando":"run-simulation","dry_run":true,"workspace":"C:/x"}` | `{"dry_run":true,...}` | `ignorados:["workspace"]` |
| D9 | `{"comando":"run-simulation","dry_run":true,"detalle":"completo"}` | `{"dry_run":true,...}` | `detalle` no se reporta ni cambia nada |
| D10 | `{"comando":"  run-simulation  ","dry_run":true}` | `command_line:"... \"  run-simulation  \""` | no se recorta |
| D11 | `{"comando":"run\"simulation","dry_run":true}` | `command_line:"... run\"simulation"` | comilla sin escapar |
| D12 | `{"comando":"run-simulation","dry_run":true,"opciones":["-rpf:C:/a\"b c.pdf"]}` | `command_line:"... \"-rpf:C:/a\"b c.pdf\""` | comilla interior sin escapar |
| D13 | `{"comando":"run-simulation","dry_run":true,"opciones":["-rpf:C:/a\tb.pdf"]}` | `command_line:"... -rpf:C:/a\tb.pdf"` | tabulador no se cita |
| D14 | `{"comando":"run-simulation","dry_run":true,"opciones":{"odc":true}}` | `command_line:"... --odc:true"` | flag booleano con valor (el CLI lo documenta sin valor) |
| D15 | `{"comando":"run-simulation","dry_run":true,"opciones":{"site":null}}` | `command_line:"... --site:null"` | `null` se convierte en el texto `null` |
| D16 | `{"comando":"run-simulation","dry_run":true,"opciones":{"rp":["cover","summary"]}}` | `command_line:"... --rp:[\"cover\",\"summary\"]"` | array JSON como valor |
| D17 | `{"comando":"run-simulation","dry_run":true,"opciones":["-ll:1","-ll:3"]}` | `command_line:"... -ll:1 -ll:3"` | duplicados conservados |
| D18 | `{"comando":"run-simulation","dry_run":true,"args":["-v:VC0"]}` | `command_line:"... run-simulation -v:VC0"` | alias `args` funciona |

**Prueba de que `dry_run` no lanza proceso:** con `PVSYST_CLI=C:\no-existe\PVsystCLI.exe`, un `dry_run` de `run-simulation` devolvió `{"dry_run":true,...}` (M1), mientras que la misma llamada sin `dry_run` devolvió `PVsystCLI.exe not found at 'C:\no-existe\PVsystCLI.exe'. Set PVSYST_CLI.` (M2). El `dry_run` retorna antes de `requerir_cli`/`run` (`src/adapters/cli/proceso.rs:206-212`).

### 2.3 Timeout, entorno y cancelación

| # | Llamada | Resultado | Evidencia |
|---|---|---|---|
| T1 | `{"comando":"version","timeout_s":-1}` | `success:true`, 4,67 s | **no** usa 1 s: `as_u64` falla y cae al defecto de 120 s |
| T2 | `{"comando":"version","timeout_s":1}` | `timeout`, 1,13 s | `detected_error:"timeout tras 1 s"` |
| T3 | `{"comando":"version","timeout_s":"60"}` | error de tipo | `Wrong type for 'timeout_s' (expected integer, got string)` |
| T4 | `{"comando":"version","timeout_s":3601}` | `success:true`, 4,54 s | se acota a 3600 s en silencio |
| T5 | `{"comando":"version","timeout_s":0}` | `timeout`, 1,13 s | se acota a 1 s |
| M1 | `PVSYST_CLI` inexistente + `dry_run` | `{"dry_run":true,...}` | no se valida la CLI en `dry_run` |
| M2 | `PVSYST_CLI` inexistente + `version` | `Error::Cli` | `PVsystCLI.exe not found at '...'. Set PVSYST_CLI.` |
| P1/P2 | `PVSYST_CLI=pvsystcli-qa` (nombre relativo) + PATH con el fake | `success:true` | el nombre relativo se resuelve por PATH y el spawn funciona |
| R1 | `PVSYST_CLI=<directorio>` + `version` | error `io` | `Failed to launch <dir>: Acceso denegado. (os error 5)`; `cli_available` aceptó un directorio |
| C1 | `pvsyst_cli help` (fake, 10 s) + `notifications/cancelled` + `pvsyst_cli version` | el hijo **sigue vivo** tras cancelar; la segunda llamada responde a los **20,57 s** | la petición cancelada no recibe respuesta; el worker único queda ocupado |
| R2 | `pvsyst_cli run-simulation -ocf:rel.csv -rpf:rel.pdf` (fake, CWD del servidor = temp) | `success:true` | `rel.csv`/`rel.pdf` aparecen en el **CWD del servidor**, no en `Results` del workspace |

### 2.4 Comparación con la salida real del CLI

| Comando directo | Salida real | Devuelto por `pvsyst_cli` | Diferencia |
|---|---|---|---|
| `PVsystCLI.exe help` | 16 líneas / 894 B, incluye `lic-activate` | 13 líneas / 813 B | se pierde `   lic-activate: Activate PVsystCLI with the given license key` |
| `PVsystCLI.exe help run-simulation` | 65 líneas / 4 478 B, empieza por `Help for "run-simulation" command.` y `Usage: pvsystcli run-simulation <options>` | 60 líneas / 4 333 B | se pierden las 5 primeras líneas (incluida la línea `Usage`) |
| `PVsystCLI.exe help lic-activate` | 457 B, incluye `Help for "lic-activate" command.` y `Usage: pvsystcli lic-activate <options>` | 379 B | se pierden las dos líneas con `lic-activate`; queda solo desde `Activate PVsystCLI with the given license key` |
| `PVsystCLI.exe version` | 176 B, `PVsystCLI 8.1.8 (rev. 49527)` | 171 B | correcto |
| `PVsystCLI.exe lic-info` | 446 B | 307 B | correcto (el `tail` recorta el `Host ID` largo) |

---

## 3. Bugs

Convención: **Confirmado** = reproducido contra el binario o verificado en código; **Plausible** = solo lectura de código. "Regresión" se refiere a este PR (`main..HEAD`); "preexistente" = ya está en `main`.

### P0

Ninguno. No se encontró pérdida de datos, corrupción de workspace ni ejecución no solicitada de una simulación real.

### P1

#### B1 — Falso éxito en `help`/`version`/`lic-info` cuando el CLI imprime un error explícito

- **Severidad:** P1 (confirmación falsa en la tool de diagnóstico).
- **Reproducción:** `{"comando":"help","opciones":["--bogus"]}` → `success:true`, `clasificacion:"exito"`, `stdout_tail` con `Error: Invalid command syntax` / `Unknown command line option : --bogus`. Igual con `help help`, `help no-existe`, `version --bogus`, `lic-info --bogus`, `version -ll:3`.
- **Esperado:** `success:false`, `clasificacion:"error"`, `detected_error:"Invalid command syntax"` (o `Unknown command`).
- **Real:** `success:true`, `clasificacion:"exito"`, sin `detected_error`; `vista.resumen:"Comando de PVsystCLI ejecutado."`.
- **Evidencia:** casos F, X, Y, Z, AI, AJ de §2.1; el CLI imprime literalmente `Error: Invalid command syntax` y sale con código 0 (el código de salida siempre es 0, ver `PVSYST_CLI_REFERENCE.md` §7).
- **Archivo:línea:** `src/adapters/cli/clasificacion.rs:144-165` (`clasificar_salida`), en particular `:150` (`Some(0) if es_comando_informativo(command) => clasificar_informativo(command, stdout)`) y `:158-164` (`clasificar_informativo` solo comprueba que la salida no esté vacía).
- **¿Regresión?** **Preexistente en `main`**, pero es una regresión respecto del commit inicial: la introdujo `8a16235` ("fix(cli): classify help/version with exit 0 by exit code, not help text", 2026-10-25). El PR no la toca. El CHANGELOG la anuncia en `CHANGELOG.md:252` como "help with exit code 0 is success"; la implementación generalizó de más y también declara éxito cualquier error de un comando informativo.
- **Causa raíz:** el atajo para comandos informativos se salta `buscar_marcador_error`. La intención era no marcar como error el texto de ayuda que documenta `Required option`; la solución correcta es detectar los prefijos de error explícitos (`Error: Invalid command syntax`, `Unknown command`, `Unknown command line option`) antes del atajo.

#### B2 — `dry_run:null` ejecuta el comando

- **Severidad:** P1 (un `null` en un campo opcional convierte un ensayo en una ejecución real; con `run-simulation`, `lic-activate` o `export-logs` es destructivo/costoso).
- **Reproducción:** `{"comando":"help","dry_run":null}` → se ejecuta `help` (no hay `dry_run` en la respuesta). `{"comando":"lic-info","dry_run":null}` → se ejecuta `lic-info`. `{"comando":"run-simulation","dry_run":null,"opciones":["--bogus"]}` → se ejecuta `PVsystCLI.exe run-simulation --bogus` (falla en el parseo, no simula).
- **Esperado:** `dry_run:null` debería rechazarse por tipo (o tratarse como `true`), nunca como `false`.
- **Real:** `flag()` usa `as_bool()` y `unwrap_or(false)`; `validar_tipo_propiedad` deja pasar `null` (`src/server/tools/tipos.rs:32`).
- **Evidencia:** casos W y §2.3; `src/args.rs:31-36` (`flag`), `src/server/tools/tipos.rs:23-33` (null permitido).
- **Archivo:línea:** `src/args.rs:31-36`, `src/server/tools/tipos.rs:32`, `src/adapters/cli/proceso.rs:205`.
- **¿Regresión?** Preexistente.

#### B3 — `pvsyst_cli` puede activar/desactivar la licencia, contra la política documentada

- **Severidad:** P1 (invariante de seguridad documentado que la tool rompe; además permite que la clave pase por el chat, justo lo que `entorno` evita).
- **Reproducción:** `{"comando":"lic-activate","dry_run":true,"opciones":["-k:XXXX"]}` devuelve la línea de comando con la clave y un `vista.siguiente` que invita a ejecutarla. Sin `dry_run`, el passthrough ejecuta `lic-activate`/`lic-deactivate`/`lic-sync`.
- **Esperado:** o se rechazan esos comandos en `pvsyst_cli` (como hace `entorno licencia_activar`, que devuelve un placeholder y `"ejecutado": false`), o la documentación dice explícitamente que `pvsyst_cli` es la excepción.
- **Real:** `SECURITY.md:40` afirma "The server never activates or deactivates the licence"; `docs/gui_hibrida.md:176` afirma "Licence activation out of the model's reach"; `README.md:128`/`README.es.md:129` afirman que nada sale del workspace. `pvsyst_cli` contradice las tres.
- **Evidencia:** D4; `src/server/tools/entorno.rs:164-168` (política de `entorno`), `src/server/tools/nativos.rs:9-40` (passthrough sin lista blanca).
- **Archivo:línea:** `src/adapters/cli/proceso.rs:196-217`, `src/server/tools/nativos.rs:9-40`, `SECURITY.md:40`, `docs/gui_hibrida.md:176`, `README.md:128`, `README.es.md:129`.
- **¿Regresión?** Preexistente (documentación y comportamiento).

#### B4 — La afirmación "solo workspace / nada sale" es falsa para `pvsyst_cli`

- **Severidad:** P1 (documentación de seguridad que un usuario puede tomar como garantía).
- **Reproducción:** `pvsyst_cli run-simulation -ocf:C:\Windows\Temp\x.csv` escribe fuera del workspace; `pvsyst_cli export-logs` (sin `-t`) escribe el ZIP en el home del usuario; `pvsyst_cli version` consulta actualizaciones por red; `pvsyst_cli lic-sync` usa la red; una ruta relativa (`-ocf:rel.csv`) se resuelve contra el CWD del servidor (R2).
- **Esperado:** que el README diga "salvo `pvsyst_cli` (passthrough sin confinamiento, con red y TRIAL)" o que la tool confine las rutas.
- **Real:** `README.md:128` y `README.es.md:129` dicen "All tools read/write only files under the workspace; nothing is sent anywhere, except the PVGIS requests...".
- **Evidencia:** R2 (archivos en el CWD del servidor), D1-D4, `PVSYST_CLI_REFERENCE.md` §6 (`export-logs` sin `-t` va al home), §1 (`version` comprueba actualizaciones).
- **Archivo:línea:** `README.md:128`, `README.es.md:129`, `src/adapters/cli/proceso.rs:196-217`.
- **¿Regresión?** Preexistente.

### P2

#### B5 — `pvsyst_cli` ignora la cancelación y bloquea el worker único

- **Severidad:** P2.
- **Reproducción:** con el CLI falso durmiendo 10 s, `pvsyst_cli help` + `notifications/cancelled` + `pvsyst_cli version`: el proceso hijo sigue vivo tras cancelar, la petición cancelada no recibe respuesta y la siguiente llamada responde a los 20,57 s.
- **Esperado:** matar el hijo y liberar el worker (como hace `simular`, que usa `run_seguido` y `SeguimientoTarea` real).
- **Real:** `ejecutar` usa `run()` con `SeguimientoTarea::inerte()` (`cancelado` siempre `false`); el bucle `atender_trabajos` es de un solo worker (`src/server/bucle.rs:55-80`).
- **Evidencia:** C1 de §2.3; `tests/cli_fake/ejecucion.rs:57-95` demuestra que `simular` sí cancela, pero no hay prueba equivalente para `pvsyst_cli`.
- **Archivo:línea:** `src/adapters/cli/proceso.rs:42-50` (`run` → `inerte`), `src/adapters/cli/proceso.rs:196-217`, `src/server/bucle.rs:55-80`.
- **¿Regresión?** Preexistente.

#### B6 — `pvsyst_cli run-simulation` gasta TRIAL sin `costo_estimado`, sin `confirmar` y sin ledger

- **Severidad:** P2.
- **Reproducción:** `pvsyst_cli run-simulation` (real) no devuelve `costo_estimado` ni `requiere_confirmacion`; no registra el gasto en `UserData/mcp_cache/ledger.json`; no usa la caché de resultados de `simular`.
- **Esperado:** o el passthrough documenta que bypassa el ledger/confirmación, o `run-simulation` se enruta por la lógica de `simular`.
- **Real:** `output_json` solo añade `{"command": comando}`; `ledger::registrar` solo se llama desde `src/simulation/ejecucion.rs:29`.
- **Evidencia:** D1-D4 (sin `costo_estimado`), `src/simulation/mod.rs:33-52` (dry_run de `simular` sí lo reporta), `src/adapters/ledger.rs:83` (`costo_estimado`), `src/simulation/preparacion.rs:186` (`requiere_confirmacion`).
- **Archivo:línea:** `src/adapters/cli/proceso.rs:212-217`, `src/adapters/cli/salida_json.rs:85-107`, `src/simulation/ejecucion.rs:29`.
- **¿Regresión?** Preexistente.

#### B7 — La salida de `help` se mutila: se pierde `lic-activate` y la línea `Usage`

- **Severidad:** P2 (contenido perdido en un passthrough, sin aviso).
- **Reproducción:** `{"comando":"help"}` → falta la línea `lic-activate`; `{"comando":"help","opciones":["run-simulation"]}` → faltan las 5 primeras líneas, incluida `Usage: pvsystcli run-simulation <options>`; `{"comando":"help","opciones":["lic-activate"]}` → faltan `Help for "lic-activate" command.` y `Usage: pvsystcli lic-activate <options>`.
- **Esperado:** devolver la salida completa (o marcar el recorte).
- **Real:** `filtrar_cuerpo_stdout` aplica `is_license_banner` a **toda** salida exitosa; `is_license_banner` descarta cualquier línea que contenga `lic-activate` (`src/adapters/cli/clasificacion.rs:38-44`). Además `LINEAS_TAIL_STDOUT = 60` recorta a las últimas 60 líneas sin marcar `stdout_truncado` (ese flag solo refleja el tope de 1 MiB).
- **Evidencia:** §2.4; B, C, `lichelp` de §2.1.
- **Archivo:línea:** `src/adapters/cli/salida_json.rs:8`, `:47-55`, `:95`; `src/adapters/cli/clasificacion.rs:38-44`.
- **¿Regresión?** Preexistente.

#### B8 — Los comandos case-insensitive se clasifican como fallo

- **Severidad:** P2.
- **Reproducción:** `{"comando":"VERSION"}` y `{"comando":"Help"}` → `success:false`, `clasificacion:"ambiguo"`, aunque el CLI responde correctamente (`PVsystCLI 8.1.8 (rev. 49527)` y la ayuda).
- **Esperado:** `success:true`, `exito` (el CLI documenta que los comandos son case-insensitive, `PVSYST_CLI_REFERENCE.md` §1).
- **Real:** `es_comando_informativo` compara con `matches!` sensible a mayúsculas; `confirmed` también.
- **Evidencia:** N, O de §2.1.
- **Archivo:línea:** `src/adapters/cli/clasificacion.rs:94-98`, `:46-58`.
- **¿Regresión?** Preexistente.

#### B9 — La forma objeto de `opciones` no puede expresar alias cortos ni flags booleanos, y convierte valores raros en texto

- **Severidad:** P2.
- **Reproducción:** `{"opciones":{"w":"C:/ws","p":"P","v":"VC0"}}` genera `--w:... --p:... --v:...`; `{"opciones":{"odc":true}}` genera `--odc:true`; `{"opciones":{"site":null}}` genera `--site:null`; `{"opciones":{"rp":["cover","summary"]}}` genera `--rp:["cover","summary"]`; las claves se ordenan alfabéticamente (serde_json sin `preserve_order`).
- **Esperado:** o la forma objeto acepta los alias documentados (`-w`, `-p`, `-v`, `-ll`, `-ocf`, `-rpf`) y los flags sin valor, o rechaza valores `null`/array/objeto y documenta "solo nombres largos".
- **Real:** `format!("--{}:{}", k.trim_start_matches('-'), value_text(v))` fuerza `--` y `value_text` convierte cualquier JSON a texto (`src/format/mod.rs:73-78`). La referencia documenta `--workspace|-w` y "boolean flags take no value" (`PVSYST_CLI_REFERENCE.md` §1, §2).
- **Evidencia:** D2, D14-D16, E4-E6, E10; no se ejecutó `--w` contra `run-simulation` para no gastar TRIAL (la referencia es la evidencia).
- **Archivo:línea:** `src/adapters/cli/proceso.rs:185-188`, `src/format/mod.rs:73-78`, `src/server/tools/tipos.rs:65-70`.
- **¿Regresión?** Preexistente (la forma objeto ya estaba en `main`; el PR solo añadió el rechazo de elementos no-texto del array).

#### B10 — `timeout_s` por defecto de 120 s para `run-simulation`, acotado en silencio

- **Severidad:** P2 (una simulación real puede tardar más de 120 s; el kill deja archivos a medias y un `timeout` sin diagnóstico).
- **Reproducción:** `{"comando":"run-simulation",...}` sin `timeout_s` usa 120 s; `timeout_s:0` → 1 s; `timeout_s:-1` → 120 s (no 1 s); `timeout_s:3601` → 3600 s.
- **Esperado:** defecto por comando (900 s para `run-simulation`, como `TIMEOUT_SIMULACION_S`), rechazo de valores ≤ 0 y límites declarados en el esquema.
- **Real:** `timeout_de(args, TIMEOUT_DEFECTO_S)` con `TIMEOUT_DEFECTO_S = 120`; `as_u64().unwrap_or(defecto).clamp(1, 3600)`.
- **Evidencia:** T1-T5, S; `src/adapters/cli/linea_comando.rs:13`, `:65-73`.
- **Archivo:línea:** `src/adapters/cli/linea_comando.rs:13`, `:22`, `:65-73`; `src/adapters/cli/proceso.rs:212`.
- **¿Regresión?** Preexistente.

#### B11 — `command_line` no es reproducible

- **Severidad:** P2 (es la única salida del `dry_run` y el texto que un humano copiaría).
- **Reproducción:** el ejecutable va sin comillas (`C:\Program Files\...\PVsystCLI.exe ...`); una comilla interior no se escapa (`-p:a"b`); un tabulador no se cita; una ruta con espacio y barra final produce `"-w:C:/my ws\"` (la barra escapa la comilla de cierre).
- **Esperado:** comillas alrededor del ejecutable y escape de comillas/backslashes/tabs, o una representación explícitamente "no copiable".
- **Real:** `display_command_line` solo cita si el argumento contiene un espacio (`' '`), no cita el ejecutable y no escapa nada.
- **Evidencia:** D1, D11-D13, prueba de barra final; `tests/unit/adapters/cli/linea_comando.rs` solo prueba el caso feliz.
- **Archivo:línea:** `src/adapters/cli/linea_comando.rs:25-38`.
- **¿Regresión?** Preexistente.

#### B12 — El `siguiente` de un `dry_run` invita a ejecutar de verdad

- **Severidad:** P2 (el servidor instruye que "cada paso de siguiente es una llamada que puede enviarse tal cual").
- **Reproducción:** `{"comando":"lic-activate","dry_run":true,"opciones":["-k:XXXX"]}` → `vista.siguiente[0]` = `{"comando":"lic-activate","opciones":["-k:XXXX"]}`, motivo `"Aplicar de verdad lo ensayado"`. Igual para `run-simulation` (gasta TRIAL).
- **Esperado:** que el `siguiente` de un passthrough no sea la ejecución real, o que exija `confirmar:true`/una confirmación explícita.
- **Real:** `Pendiente::Ensayo::siguiente` elimina la clave de ensayo y devuelve la misma llamada.
- **Evidencia:** D1, D4; `src/vista_agente/contrato/pendiente.rs:84-100`.
- **Archivo:línea:** `src/vista_agente/contrato/pendiente.rs:84-100`, `src/server/mod.rs:218` (instrucción 12).
- **¿Regresión?** Preexistente.

#### B13 — `stdout_tail` limitado a 60 líneas, sin salida completa ni aviso

- **Severidad:** P2.
- **Reproducción:** `help run-simulation` devuelve 60 de 65 líneas; no hay campo `stdout` completo; `stdout_truncado` es `false` porque no se alcanzó el 1 MiB; `detalle:completo` no cambia nada.
- **Esperado:** un campo `stdout` completo (o `lineas`/`detalle` que lo controle) y un flag de recorte por líneas.
- **Real:** `LINEAS_TAIL_STDOUT = 60` fijo; `stdout_truncado` solo refleja `LIMITE_BYTES_STREAM`.
- **Evidencia:** C, AH; `src/adapters/cli/salida_json.rs:8`, `:95`.
- **Archivo:línea:** `src/adapters/cli/salida_json.rs:8`, `:95`; `src/adapters/cli/captura.rs:17`.
- **¿Regresión?** Preexistente.

#### B14 — Huecos de esquema: `timeout_s` sin límites, `opciones` objeto sin restricciones, `dry_run`/`timeout_s` sin descripción

- **Severidad:** P2 (el cliente no puede validar lo que el servidor sí acota o rechaza).
- **Reproducción:** `tools/list` devuelve `timeout_s:{"type":"integer"}` sin `minimum`/`maximum`; `opciones.anyOf[1]` es `{"type":"object"}` sin `additionalProperties`; `dry_run` y `timeout_s` no tienen `description`; `opciones` solo dice `"opciones (alias: options, args)"`.
- **Esperado:** `minimum:1`, `maximum:3600`, descripciones de `dry_run`/`timeout_s`/`opciones` (array vs objeto, `--` largo, flags sin valor) y valores de objeto restringidos.
- **Real:** `LIMITES_ENTEROS` no incluye `timeout_s` (`src/server/tools/restricciones_esquema.rs:9-15`); `TipoDato::ArrayOrObject` genera un objeto libre (`src/server/tools/tipos.rs:65-70`); `Anotador::propiedad` solo añade alias (`src/server/tools/declaracion/anotador.rs:10-27`).
- **Evidencia:** `tools/list` de §2; D14-D16.
- **Archivo:línea:** `src/server/tools/tipos.rs:65-70`, `src/server/tools/restricciones_esquema.rs:9-15`, `src/server/tools/declaracion.rs:165-200`.
- **¿Regresión?** Preexistente.

#### B15 — Las rutas relativas de salida se resuelven contra el CWD del servidor, no contra el workspace

- **Severidad:** P2 (el archivo aparece en un lugar inesperado; contradice el confinamiento que el resto de tools sí aplica).
- **Reproducción:** con el CWD del servidor en un temporal, `pvsyst_cli run-simulation -ocf:rel.csv -rpf:rel.pdf` deja `rel.csv`/`rel.pdf` en ese temporal, no en `Results` del workspace.
- **Esperado:** resolver `-ocf`/`-rpf`/`-osf`/`-omf`/`-t` relativos contra el workspace, o rechazarlos con un error que lo nombre.
- **Real:** `Command::new` no fija `current_dir`; el hijo hereda el CWD del servidor.
- **Evidencia:** R2 de §2.3; `src/adapters/cli/proceso.rs:72-80`.
- **Archivo:línea:** `src/adapters/cli/proceso.rs:72-80`.
- **¿Regresión?** Preexistente.

#### B16 — El `dry_run` de `lic-activate` devuelve la clave en claro

- **Severidad:** P2 (la clave pasa por el chat y por `command_line`, justo lo que `entorno` evita con `<TU_CLAVE>`).
- **Reproducción:** `{"comando":"lic-activate","dry_run":true,"opciones":["-k:XXXX"]}` → `command_line:"... lic-activate -k:XXXX"`.
- **Esperado:** redactar `-k:`/`-c:` en `command_line` (o rechazar `lic-*` en el passthrough).
- **Real:** `display_command_line` no redacta nada; `output_json` tampoco.
- **Evidencia:** D4; `src/server/tools/entorno.rs:164-168` (política de placeholder).
- **Archivo:línea:** `src/adapters/cli/linea_comando.rs:25-38`, `src/adapters/cli/proceso.rs:206-209`.
- **¿Regresión?** Preexistente.

### P3

#### B17 — `log_level` no se soporta en `pvsyst_cli` (va a `ignorados`)

- **Reproducción:** `{"comando":"version","log_level":3}` → `ignorados:["log_level"]`; `{"comando":"run-simulation","dry_run":true,"log_level":9}` → `ignorados:["log_level"]`, sin validar el rango.
- **Esperado:** o se traduce a `-ll:<n>` (como `simular`/`sitio`), o la descripción dice que `-ll` debe ir en `opciones`.
- **Real:** `log_level` no está en `propiedades`; `validar_log_level` existe pero solo lo usan `simulation/opciones.rs:56` y `sites/mod.rs:56`.
- **Archivo:línea:** `src/server/tools/nativos.rs:25-30`, `src/adapters/cli/linea_comando.rs:90-101`.
- **¿Regresión?** Preexistente.

#### B18 — `detalle` y `via` se aceptan y se ignoran sin aparecer en `ignorados`

- **Reproducción:** `{"comando":"help","detalle":"completo"}` y `{"comando":"help","via":"aproximacion"}` → sin `ignorados`, sin efecto.
- **Esperado:** reportarlos como ignorados (la instrucción 11 del servidor dice que una clave en `ignorados` no se aplicó; aquí no se aplican y no se listan).
- **Real:** `declara()` los considera declarados (`src/server/tools/requeridos.rs:132-140`) y `NO_LEIDOS` no tiene entradas para `pvsyst_cli`.
- **Archivo:línea:** `src/server/tools/requeridos.rs:85-104`, `:132-140`, `:157-233`.
- **¿Regresión?** Preexistente.

#### B19 — `comando` no se recorta ni valida (espacios, saltos de línea, controles)

- **Reproducción:** `{"comando":"   "}` → `Invalid command syntax`; `{"comando":" help"}` → error; `{"comando":"help\nextra"}` → error con salto de línea en `command_line`.
- **Esperado:** `trim()` y rechazo de vacío/control, con un error que nombre `comando`.
- **Real:** `req_str`/`opt_str` solo descartan la cadena vacía.
- **Archivo:línea:** `src/args.rs:23-29`, `:74-76`; `src/adapters/cli/proceso.rs:197-201`.
- **¿Regresión?** Preexistente.

#### B20 — `opciones` acepta cadenas vacías, claves vacías, duplicados y orden alfabético

- **Reproducción:** `[""]` → `help ` (argumento vacío); `{"":"x"}` → `--:x`; `["-ll:1","-ll:3"]` → ambos; `{"variant":"VC0","workspace":"W","project":"P"}` → `--project --variant --workspace`.
- **Esperado:** rechazar cadenas/claves vacías, deduplicar o avisar, y preservar el orden (o documentar que el objeto se ordena).
- **Archivo:línea:** `src/adapters/cli/proceso.rs:177-188`.
- **¿Regresión?** Preexistente.

#### B21 — `openWorldHint:false` con comandos que usan red

- **Reproducción:** `version` ("check for available updates") y `lic-sync` usan red; la anotación declara `openWorldHint:false`.
- **Esperado:** `openWorldHint:true` (o separar comandos locales de remotos).
- **Archivo:línea:** `src/server/tools/nativos.rs:23`, `src/server/tools/declaracion.rs:197`.
- **¿Regresión?** Preexistente.

#### B22 — `cli_available` acepta directorios y el error de spawn cambia de tipo

- **Reproducción:** `PVSYST_CLI=<directorio>` + `version` → `Failed to launch <dir>: Acceso denegado. (os error 5)` (kind `io`, no `cli`).
- **Esperado:** `is_file()` y un `Error::Cli` que nombre `PVSYST_CLI`.
- **Real:** `paths.cli_exe.exists()` acepta directorios; `which("")` podría devolver el primer directorio del PATH (riesgo de código, no reproducido en producción).
- **Archivo:línea:** `src/adapters/cli/linea_comando.rs:41-62`, `src/adapters/cli/proceso.rs:72-80`.
- **¿Regresión?** Preexistente.

#### B23 — Ineficiencias del adaptador

- **Doble `requerir_cli`:** `ejecutar` (`proceso.rs:211`) y `run_inner` (`:71`) comprueban la CLI dos veces.
- **Doble `display_command_line`:** se construye en `ejecutar` (`:208`) y otra vez en `run_inner` (`:104`).
- **Copia O(n²) del stdout:** `esperar_proceso` copia el buffer completo con `texto_sink` cada vez que el lector añade bytes, incluso con `SeguimientoTarea::inerte()` (pvsyst_cli). Medición con el fake: `help` 140 ms, `huge-output` (2 MiB) 255 ms; parte del delta es la copia/escaneo.
- **Jobs que no se borran:** `paths.estado.trabajos` nunca elimina trabajos terminados; cada uno retiene hasta 2 MiB (`src/adapters/cli/trabajo.rs:31-53`); `estado_trabajo` clona el `CliOutput` completo (`:67-100`).
- **Archivo:línea:** `src/adapters/cli/proceso.rs:71`, `:104`, `:154-165`, `:211`; `src/adapters/cli/trabajo.rs:31-53`, `:67-100`.
- **¿Regresión?** Preexistente.

#### B24 — Codificación: `from_utf8_lossy` puede romper marcadores acentuados

- **Reproducción:** no reproducido (toda la salida observada de 8.1.8 fue ASCII). Riesgo de código: si PVsystCLI emite CP1252 (p. ej. `Acceso denegado` con acento), `from_utf8_lossy` produce `U+FFFD` y el marcador `"Acceso denegado"` no coincide.
- **Esperado:** decodificar según la página de códigos del proceso (o comparar en bytes).
- **Archivo:línea:** `src/adapters/cli/captura.rs:112-118`, `src/adapters/cli/clasificacion.rs:12-35`.
- **¿Regresión?** Preexistente.

#### B25 — El filtro de éxito descarta cualquier línea que termine en `%`

- **Reproducción:** `filtrar_cuerpo_stdout` elimina las líneas que terminan en `%` cuando `success:true`; pensado para el progreso de `run-simulation`, se aplica a cualquier comando (p. ej. una salida `PR: 80 %` desaparecería del `stdout_tail`).
- **Esperado:** aplicar el filtro solo a comandos de simulación, o no filtrar en un passthrough.
- **Archivo:línea:** `src/adapters/cli/salida_json.rs:47-55`.
- **¿Regresión?** Preexistente.

#### B26 — Documentación desalineada

- `PVSYST_CLI_REFERENCE.md:1` es la referencia de **8.1.7**; el binario instalado es **8.1.8 (rev. 49527)**. El recurso `pvsyst://cli` sirve ese texto (`src/server/resources.rs:8`).
- `SKILL.md:36` dice "Las simulaciones son lentas: informan progreso y aceptan cancelación"; `pvsyst_cli run-simulation` no informa progreso ni acepta cancelación (B5).
- `docs/cobertura.md:80` y `src/server/resources.rs:143` listan `pvsyst_cli run-simulation` como ruta de simulación sin advertir que no usa caché, ledger, `costo_estimado` ni confirmación (B6).
- `docs/vista_agente_salidas.md:126` dice "en vivo: ok" para `pvsyst_cli`; la verificación en vivo probablemente usó `help`/`version`, que son justo los comandos afectados por B1.
- **¿Regresión?** Preexistente.

---

## 4. Ineficiencias y oportunidades de latencia

| # | Ineficiencia | Evidencia | Impacto | Arreglo |
|---|---|---|---|---|
| I1 | `requerir_cli` dos veces por ejecución | `proceso.rs:71`, `:211` | 2 `exists()`/`which` por llamada | una sola comprobación en `ejecutar` |
| I2 | `display_command_line` dos veces | `proceso.rs:208`, `:104` | 2 allocaciones + recorrido | pasar la cadena ya construida a `run_inner` |
| I3 | Copia completa del stdout en cada chunk de progreso, incluso sin tracker | `proceso.rs:154-165` | O(n²) en salidas grandes; 255 ms vs 140 ms para 2 MiB | saltar el escaneo si `seguimiento` no tiene reporter, o escanear solo los bytes nuevos |
| I4 | `which` recorre el PATH en cada `requerir_cli` si la ruta no existe | `linea_comando.rs:41-57` | coste pequeño pero evitable | cachear la resolución (OnceLock) |
| I5 | `tail()` materializa todas las líneas para quedarse con 60 | `salida_json.rs:16-20` | allocación O(n) por respuesta | iterar desde el final o usar `rsplit` |
| I6 | `estado_trabajo` clona `CliOutput` (hasta 2 MiB) en cada consulta | `trabajo.rs:67-100` | latencia/memoria en `simular estado` | devolver referencias/`Arc` |
| I7 | Trabajos terminados nunca se eliminan | `trabajo.rs:31-53` | fuga de memoria por sesión | limpiar al leer o por TTL |
| I8 | Sondeo de 50 ms del proceso y 20 ms de los lectores | `proceso.rs:28`, `captura.rs:24` | ~440 despertares en 22 s | `wait_timeout`/evento, o 100–200 ms |
| I9 | `opciones_de` clona cada string y `value_text` clona los strings | `proceso.rs:177-188`, `format/mod.rs:73-78` | allocaciones evitables | `Vec<Cow<str>>` o construir el `Vec<String>` una vez |
| I10 | `preparar_args` clona todo el mapa de args; `vista::solo_ejecutables` vuelve a clonar para validar el paso | `validacion.rs:43-53`, `vista.rs:28-39` | 2–3 clones por llamada | tomar prestado o validar sin clonar |

Ninguna de estas ineficiencias es crítica para una llamada de 3–5 s, pero I3 es la que más crece con la salida y la que más afecta a `run-simulation` (logs grandes).

---

## 5. Instrucciones redundantes y problemas de contrato/esquema

### 5.1 Contradicciones entre fuentes

| Afirmación | Dónde | Realidad |
|---|---|---|
| "The server never activates or deactivates the licence" | `SECURITY.md:40` | `pvsyst_cli lic-activate`/`lic-deactivate` lo hacen (B3) |
| "Licence activation out of the model's reach" | `docs/gui_hibrida.md:176` | falso para `pvsyst_cli` (B3) |
| "All tools read/write only files under the workspace; nothing is sent anywhere" | `README.md:128`, `README.es.md:129` | `pvsyst_cli` escribe fuera y usa red (B4) |
| "Las simulaciones... informan progreso y aceptan cancelación" | `SKILL.md:36` | `pvsyst_cli run-simulation` no (B5) |
| "Simulation | engine | `simular`, `pvsyst_cli run-simulation`" | `docs/cobertura.md:80`, `resources.rs:143` | no son equivalentes: sin caché, ledger, `costo_estimado` ni confirmación (B6) |
| "help with exit code 0 is success" | `CHANGELOG.md:252` | se generalizó a cualquier error de `help`/`version`/`lic-info` (B1) |
| "cualquier otra clave se lista en ignorados y no se pasa" | `nativos.rs:11-14` | `detalle` y `via` no se listan (B18); `timeout_s` se ignora en `dry_run` (D5) |
| `PVSYST_CLI_REFERENCE.md` "8.1.7 ... verificada en esta máquina" | `PVSYST_CLI_REFERENCE.md:1`, `pvsyst://cli` | el binario es 8.1.8 (rev. 49527) (B26) |

### 5.2 Claves declaradas pero no usadas (o usadas a medias)

- `detalle`: declarada para todas las tools (`declaracion.rs:181-184`), sin efecto en `pvsyst_cli` y sin aparecer en `ignorados`.
- `via`: declarada y validada (`validacion.rs:126-151`), sin efecto en `pvsyst_cli` y sin aparecer en `ignorados`.
- `timeout_s`: declarada, usada solo fuera de `dry_run`; en `dry_run` se ignora sin aviso.
- `log_level`: no declarada; va a `ignorados` aunque `simular`/`sitio` la soportan (B17).
- `workspace`: no declarada; va a `ignorados` (correcto para un passthrough, pero el README sugiere que todas las tools lo aceptan).
- `confirmar`: no declarada; no hay puerta de confirmación para TRIAL (B6).

### 5.3 Esquema (`tools/list`)

- `comando`: `{"type":"string"}`, sin `minLength`, sin `pattern`, sin descripción de "solo el nombre del comando".
- `opciones`: `anyOf [array<string>, object]`; el objeto no tiene `additionalProperties` ni restricción de valores; no se explica que la forma objeto fuerza `--nombre:valor`.
- `dry_run`: `{"type":"boolean"}`, sin descripción; no dice que la respuesta es solo `{"dry_run":true,"command_line":...}`.
- `timeout_s`: `{"type":"integer"}`, sin `minimum`/`maximum` ni descripción del defecto (120 s) y del acotado [1, 3600].
- `detalle`: enum global, sin efecto en esta tool.
- `via`: enum global, sin efecto en esta tool.
- `annotations`: `readOnlyHint:false`, `destructiveHint:true`, `idempotentHint:false` son correctos; `openWorldHint:false` no (B21).
- `required`: `[]` + `allOf anyOf comando|command`; correcto, pero algunos clientes no soportan `allOf` (bajo riesgo).

### 5.4 Redundancia de instrucciones

- La descripción de `pvsyst_cli` se repite literalmente en `mcpb/manifest.json:434` y en `nativos.rs:11-14`; cualquier cambio debe hacerse en dos sitios (hoy coinciden).
- La tabla de alias se repite en `README.md:119`, `README.es.md:120`, `mcpb/manifest.json:221-227` y `SINONIMOS_PVSYST_CLI`; coinciden.
- `PVSYST_CLI_REFERENCE.md` se sirve como recurso `pvsyst://cli` (`resources.rs:8`), pero también se referencia desde `README.md:376` y `SKILL.md:52`; una actualización debe tocar el archivo, no el recurso.
- Las instrucciones del servidor (`src/server/mod.rs:218`, punto 6) anuncian `dry_run` para `pvsyst_cli`, pero la descripción de la tool no lo menciona; el agente que solo lee `tools/list` no sabe que existe.
- `docs/vista_agente_salidas.md:126` documenta el `resumen` de `pvsyst_cli` como "Comando de PVsystCLI ejecutado."; para `dry_run` el resumen real es "Ensayo, no se aplicó nada. Al aplicarlo: ...", que no está documentado.

---

## 6. Recomendaciones priorizadas

| Prioridad | Recomendación | Impacto | Esfuerzo |
|---|---|---|---|
| **P1.1** | Corregir B1: en `clasificar_salida`, para comandos informativos, buscar primero los prefijos de error explícitos (`Error: Invalid command syntax`, `Unknown command`, `Unknown command line option`) y solo después aplicar el atajo. Añadir tests de `help --bogus`, `help help`, `version --bogus`, `lic-info --bogus`, `VERSION`, `Help`. | Alto: elimina la confirmación falsa en la tool de diagnóstico | S (1 archivo + tests) |
| **P1.2** | Corregir B2: rechazar `dry_run:null` con un error de tipo (o tratarlo como `true`). Añadir test. | Alto: evita ejecuciones reales por un `null` | S (1 línea + test) |
| **P1.3** | Resolver B3/B4: o se rechazan `lic-activate`/`lic-deactivate`/`lic-sync` en `pvsyst_cli` (como `entorno`), o se documenta la excepción en `SECURITY.md`, `README.md`, `README.es.md` y `docs/gui_hibrida.md`. Documentar que las rutas no están confinadas, que las relativas van al CWD y que `version`/`lic-sync` usan red. | Alto: la documentación deja de prometer garantías falsas | S/M (docs + quizá lista negra) |
| **P2.1** | Usar `run_seguido` también en `pvsyst_cli` (cancelación + progreso) y, para `run-simulation`, añadir `costo_estimado`, `confirmar` y registro en el ledger (o documentar explícitamente que el passthrough los bypassa). | Alto: cancelación real y control del gasto TRIAL | M |
| **P2.2** | No aplicar `is_license_banner` ni el filtro `%` a comandos que no son de simulación; añadir un campo `stdout` completo (o `lineas`/`detalle`) y un flag `stdout_tail_truncado`. | Medio: `help` deja de perder `lic-activate` y la línea `Usage` | S/M |
| **P2.3** | Hacer `es_comando_informativo`/`confirmed` case-insensitive (`eq_ignore_ascii_case`) y probar `VERSION`/`Help`. | Medio: elimina falsos `ambiguo` | S |
| **P2.4** | Endurecer la forma objeto de `opciones`: aceptar alias cortos documentados o rechazar el objeto; rechazar `null`/array/objeto como valor; documentar el orden alfabético. Añadir `minimum`/`maximum` y descripciones a `timeout_s`/`dry_run`/`opciones`. | Medio: el esquema y el runtime dicen lo mismo | M |
| **P2.5** | `timeout_s` por comando (900 s para `run-simulation`), rechazo de ≤ 0 y límites en el esquema. | Medio: menos timeouts inesperados | S |
| **P2.6** | `display_command_line` con comillas en el ejecutable y escape de comillas/backslashes/tabs; tests de rutas con espacios, comillas y barra final. | Medio: `dry_run` reproducible | S |
| **P2.7** | Para un `dry_run` de `pvsyst_cli`, no devolver la ejecución real como `siguiente` (o exigir `confirmar:true`); redactar `-k:`/`-c:` en `command_line`. | Medio: evita un salto accidental a `lic-activate`/TRIAL | S |
| **P2.8** | Resolver `-ocf`/`-rpf`/`-osf`/`-omf`/`-t` relativos contra el workspace, o rechazarlos con un error que lo nombre. | Medio: los archivos aparecen donde el usuario espera | M |
| **P2.9** | Soportar `log_level` (añadir `-ll:`) o documentar que debe ir en `opciones`; reportar `detalle`/`via` en `ignorados` cuando no se usan. | Bajo/Medio: menos claves silenciosas | S |
| **P3.1** | `cli_available` con `is_file()`, cachear la resolución, unificar el error de spawn como `Error::Cli`. | Bajo | S |
| **P3.2** | Evitar la copia O(n²) del stdout cuando el tracker es inerte; limpiar trabajos terminados; no clonar `CliOutput` en `estado_trabajo`. | Bajo/Medio (latencia y memoria) | M |
| **P3.3** | Documentar la codificación (UTF-8/CP1252) y comparar marcadores en bytes si hace falta; no filtrar líneas `%` en el passthrough. | Bajo | S |
| **P3.4** | Actualizar `PVSYST_CLI_REFERENCE.md` a 8.1.8 y corregir `SKILL.md`/`docs/cobertura.md`/`resources.rs` para que `pvsyst_cli run-simulation` no se presente como equivalente a `simular`. | Bajo/Medio: documentación fiable | S |

**Orden sugerido:** P1.1 → P1.2 → P1.3 → P2.1 → P2.2 → P2.3 → P2.4/P2.5 → P2.6/P2.7 → P2.8 → P3.x.

---

## 7. Anexo

### 7.1 Binario y versiones

| Dato | Valor |
|---|---|
| Binario instalado | `C:\pvsyst-mcp\pvsyst-mcp.exe` |
| SHA-256 | `501ec8ffabc020fb720bb58b16f9f3ff362dd97fb6a8268b2977e68bffa8abc0` |
| Tamaño | 5 015 552 bytes |
| Fecha | 2026-11-07 21:31 (local) |
| `serverInfo` | `{"name":"pvsyst-mcp","version":"0.3.0"}` |
| `target/release/pvsyst-mcp.exe` del worktree | mismo SHA-256 → el instalado es el build de HEAD |
| Worktree | `C:\Users\Lenovo\Desktop\PVSYST MCP\.claude\worktrees\tarea-d-fila` |
| Rama / HEAD | `worktree-tarea-d-fila` / `5dbfdd0d2973a5e4f2990930f3d7aef693d17873` |
| PVsystCLI | `C:\Program Files\PVsyst8.1.8\PVsystCLI.exe` |
| `PVsystCLI.exe version` | `PVsystCLI 8.1.8 (rev. 49527)` |
| Subsistema PE de PVsystCLI | 3 (CONSOLE) |
| `PVSYST_WORKSPACE` | `C:\Users\Lenovo\PVsyst8.1_Data` |

### 7.2 Comandos ejecutados

**Directos (solo lectura):**
```
PVsystCLI.exe help
PVsystCLI.exe help run-simulation
PVsystCLI.exe help lic-activate
PVsystCLI.exe version
PVsystCLI.exe lic-info   (×2)
```
**MCP (90 `tools/call` + 1 `tools/list`):** scripts Python propios en `%TEMP%\qa-cli` (`matrix.py`, `dry.py`, `clasif.py`, `timeout.py`, `edge.py`, `envtest.py`, `cancel.py`, `perf.py`, `relpath.py`, `schema.py`, `lichelp.py`, `errshape.py`), contra el binario instalado por stdio. No se invocó la tool MCP `mcp__pvsyst__pvsyst_cli`: todas las llamadas fueron contra el mismo binario instalado, lanzado por stdio, que es el proceso que registra `.pi/mcp.json`.

**Tests de Rust:**
```
cargo test --lib adapters::cli        → 6 passed, 0 failed
cargo test --test cli_fake            → 10 passed, 0 failed
cargo fmt --check                     → OK (sin cambios)
```
No se ejecutó `cargo test` completo (los tests restantes no tocan el adaptador CLI y algunos escriben en workspaces temporales).

**Tests existentes relevantes (cobertura):**
- `tests/unit/adapters/cli/clasificacion.rs`: 1 test (`should_not_mask_error_marker_with_license_banner`). **No cubre** el atajo de comandos informativos (B1).
- `tests/unit/adapters/cli/proceso.rs`: 2 tests (timeout en `output_json`, rechazo de `opciones` no-texto). **No cubre** la forma objeto, `dry_run`, `ignorados`, `timeout_s`.
- `tests/unit/adapters/cli/linea_comando.rs`: 1 test (comillas con espacios). **No cubre** el ejecutable con espacios, comillas interiores, tabs ni barra final (B11).
- `tests/unit/adapters/cli/captura.rs`: 1 test (tope de 1 MiB).
- `tests/unit/adapters/cli/trabajo.rs`: 1 test (pánico en el hilo).
- `tests/cli_fake/ejecucion.rs`: 3 tests de `pvsyst_cli` (timeout, salida masiva, ambiguo). **No cubre** `help`/`version`/`lic-info`, `dry_run`, `opciones` objeto, cancelación de `pvsyst_cli` ni rutas relativas.
- `tests/mcp_regresiones/`: **ningún** test de `pvsyst_cli`.

### 7.3 Limitaciones y actividad concurrente (importante)

1. **Una invocación `run-simulation` no prevista.** Para probar `dry_run:null` ejecuté `{"comando":"run-simulation","dry_run":null,"opciones":["--bogus"]}`. El CLI la rechazó en el parseo (`Invalid command syntax` / `Unknown command line option : --bogus`), no cargó proyecto, no creó archivos y no gastó TRIAL. Aun así, era una invocación de `run-simulation` y no debí hacerla; la dejo registrada para que no se repita. No se ejecutó ninguna simulación real.
2. **Actividad concurrente en el workspace real.** Durante la sesión aparecieron archivos que no son de esta revisión: `Results/QA_Eco_VC0.csv/.pdf` (01:55–01:56), `Projects/QA_Diseno*`, `Projects/Sistema_500kWp.*`, `Meteo/MCP_Meteo__SYN.*`, `UserData/mcp_history/20261108_065727_*`, `UserData/mcp_cache/ledger.json` (01:59, `restantes:249`) y `.qa/calls*.json`. Otro proceso/agente estaba ejecutando QA en el mismo workspace.
3. **Contador TRIAL.** La primera lectura de `lic-info` (01:47) dio `Remaining days: 30`, `Remaining executions: 134`; la última (01:59) dio `60`/`250`. El contador **subió**, no bajó, y ninguna llamada de esta revisión ejecutó una simulación; el cambio coincide con la actividad concurrente del punto 2. No lo atribuyo a esta revisión, pero conviene verificarlo antes de dar por bueno el estado de la licencia.
4. **No se probó en vivo** `--w`/`-w` contra `run-simulation` (habría que ejecutar el comando); la evidencia de B9 es la referencia del CLI (`--workspace|-w`, flags booleanos sin valor) más la línea de comando generada en `dry_run`.
5. **No se probó** `lic-activate`/`lic-deactivate`/`lic-sync`/`export-logs`/`convert-meteo`/`create-site` reales (prohibido por el encargo). B3/B6/B15 se apoyan en código, referencia y `dry_run`.

### 7.4 Archivos del worktree consultados

`src/server/tools/nativos.rs`, `declaracion.rs`, `declaracion/anotador.rs`, `tipos.rs`, `requeridos.rs`, `mod.rs`, `vista.rs`, `restricciones_esquema.rs`, `validacion.rs`, `alias.rs`, `recorte.rs`; `src/adapters/cli/{mod,proceso,linea_comando,captura,clasificacion,salida_json,trabajo}.rs`; `src/args.rs`, `src/config.rs`, `src/error.rs`, `src/state/{mod,registros}.rs`, `src/format/mod.rs`, `src/progreso/mod.rs`, `src/server/{mod,bucle,resources}.rs`, `src/vista_agente/contrato/{pendiente,ensayo}.rs`, `src/vista_agente/contrato/catalogo/gui.rs`, `src/simulation/{mod,ejecucion,opciones,preparacion}.rs`, `src/adapters/ledger.rs`, `src/admin/{licencia,logs}.rs`; `tests/unit/adapters/cli/*`, `tests/cli_fake/*`, `tests/mcp_regresiones/*`; `README.md`, `README.es.md`, `SECURITY.md`, `CHANGELOG.md`, `PVSYST_CLI_REFERENCE.md`, `docs/{cobertura,gui_hibrida,vista_agente_salidas}.md`, `mcpb/manifest.json`, `.agents/skills/pvsyst-mcp/SKILL.md`.
