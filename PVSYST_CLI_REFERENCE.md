# PVsystCLI 8.1.7 — exact command-line reference (extracted)

Source of truth: the installed binary's own `help` output (black‑box, no decompilation), the shipped
docs under `C:\Program Files\PVsyst8.1.7\help-cli\`, and the shipped example scripts under
`C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\CLI\Ressources\Examples\`.
All commands below were executed and verified on this machine.

## 1. Binary and base syntax

* Executable: `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe`
* Also on PATH (User PATH) as `pvsystcli` → `where pvsystcli` = `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe`.
* Base syntax (from `help-cli\reference\index.html`):

```
PVsystCLI.exe <command> <options>
```

### Option grammar (verified empirically)
* The command must be the **first** token; options follow in any order.
* Option value is attached with a colon: `--workspace:"C:\path"`. A space separator or `=` is **rejected**
  (`Option [workspace] expected a following :<value> but none was found` / `Unknown command line option : --workspace=...`).
* Long names accept **either** `--name` or `-name` (`-input-csv-file:` worked). Short aliases are `-xx`.
* Boolean flags take no value (e.g. `-odc`, `-swg`).
* Command names are **case-insensitive** (`VERSION` works).
* Running with no arguments prints the command list (same as `help`).
* `help <command>` prints per-command help; `help help` is invalid.

### Available commands (from `PVsystCLI.exe help`)
```
convert-meteo    Convert an input file using a conversion format definition
create-site      Create a site file .SIT from location information and monthly weather data
export-logs      Export execution logs in a zip file
lic-activate     Activate PVsystCLI with the given license key
lic-deactivate   Deactivate PVsystCLI license from the machine
lic-info         Show license status and expiration date
lic-sync         Update the current license status and information from the server
run-simulation   Run a simulation for the given project and variant, and output results
version          Show PVsystCLI version and check for available updates
```

`version` output on this machine: `PVsystCLI 8.1.7 (rev. 49382)`.

## 2. `run-simulation` (the simulation entry point)

Usage: `PVsystCLI.exe run-simulation <options>`
"Run a simulation for the given project and variant, and output results".

**Required options**
| Option | Alias | Meaning |
|---|---|---|
| `--workspace` | `-w` | Full path to the workspace to use |
| `--project` | `-p` | Name of the project (e.g. `Geneva` or `Geneva.PRJ`) |
| `--variant` | `-v` | Variant id (e.g. `VC0`, `VCA`, `VD5`) |

**Optional options**
| Option | Alias | Value |
|---|---|---|
| `--time-step` | `-ts` | `hour` \| `subhour` (subhour needs sub-hourly weather) |
| `--site` | `-s` | Full path to the .SIT file |
| `--synthetic-weatherdata-generation` | `-swg` | flag (only with `--site`) |
| `--input-met-file` | `-imf` | Full path to the MET file |
| `--input-sfi-file` | `-isf` | Full path to the SFI file |
| `--input-rvt-file` | `-irf` | Full path to the RVT file |
| `--input-param-file` | `-ipf` | Full path to a `Param_Modif.dat` file |
| `--batch-params-file` | `-bpf` | Full path to the batch parameters CSV |
| `--batch-rvt-file` | `-brf` | Full path to the batch RVT |
| `--output-csv-file` | `-ocf` | Full path to the output CSV |
| `--overwrite-default-csv` | `-odc` | flag |
| `--report-pdf-file` | `-rpf` | Full path to the output PDF report |
| `--report-language` | `-rl` | e.g. `en` (installed help prints `Available language: []`, i.e. no list; docs list `en, fr, de, es, it, pt, tr, ko, zh, ja, pl, ru`) |
| `--report-pages` | `-rp` | comma list, e.g. `cover,summary,p50p90` |
| `--start-date` | `-sd` | installed help: `MM.DD` \| `YY.MM.DD` \| `YYYY.MM.DD`; HTML reference: `YYYY-MM-DD` \| `YY-MM-DD` \| `MM-DD` (docs examples use dashes) |
| `--end-date` | `-ed` | same formats as start date |
| `--recompute-shadingfactors-tables` | `-rst` | `0`/`1` (default TRUE); accepts bare `-rst` or `-rst:0` |
| `--log-level` | `-ll` | `0`[OFF] `<` `1`[ERROR] `<` `2`[WARNING] `3`[INFO] (default 3) |
| `--cmd-options-file` | `-cof` | Full path to an options file (see grammar below) |

Report pages available: `cover, summary, notes, params, horizon, shadings, usersneeds, results, econeval, circuit, losses, financialbalance, p50p90, carbonbalance, predefgraphs`.

### Verbatim examples (from the docs and the shipped scripts)
From `help-cli\reference\index.html`:
```
PVsystCLI.exe run-simulation --workspace:"C:\PVsyst8.0_Data" --project:"Geneva" --variant:"VC0"
```
```
PVsystCLI.exe run-simulation -w:"C:\PVsyst8.0_Data" -p:"Geneva.PRJ" -v:"VC0" -sd:2024-01-01 -ed:2024-01-31 -rpf:"C:\Downloads\PVsystReport.pdf"
```
```
PVsystCLI.exe run-simulation -w:"C:\PVsyst8.0_Data" -p:"Geneva.PRJ" -v:"VC0" -bpf:"C:\PVsyst8.0_Data\UserBatch\BatchsParams_0.csv" -brf:"C:\PVsyst8.0_Data\Params\BatchsVariables.rvt" -rl:"fr"
```
From `ScriptsBat\01_Demo_PVsystCLI_SimpleSimulation.bat` (full pipeline: MET + SFI + CSV + PDF):
```
pvsystcli run-simulation -w:"%Ws_path%" -p:"%Prj_filename%" -v:"%VarNo%" -imf:"%MET_filename%" -isf:"%SFI_filename%" -ocf:"%UniqueResName%" -rpf:"%UniqueReportName%"
```
From `ScriptsBat\07_Demo_PVsystCLI_CreateSiteAndSimulate.bat` (use a generated .SIT + synthetic weather):
```
pvsystcli run-simulation -w:"%Ws_path%" -p:"%Prj_filename%" -v:"%VarNo%" --site:"%OutputSIT%" -ocf:"%Res_name%" -rpf:"%Report_name%" -swg
```
From `ScriptsPython\Demo_PVsystCLI_fun.py` (`runSim`, adds log level + param file):
```
pvsystcli run-simulation -w:{ws_path} -p:{prj_filename} -v:{var_no} -imf:{met_filename} -isf:{sfi_filename} -ocf:{results_name} -ipf:{params_filename}
```
From `ScriptsR\Demo_PVsystCLI_fun.R`:
```
pvsystcli run-simulation -w:"..." -p:"..." -v:"..." -imf:"..." -isf:"..." -ocf:"..." -ipf:"..." -ll:3
```

## 3. `convert-meteo` (CSV/MEF/SIT → MET)

Usage: `PVsystCLI.exe convert-meteo <options>`

**Required**: `--input-csv-file|-icf`, `--input-mef-file|-imf`, `--input-sit-file|-isf`, `--output-met-file|-omf`
**Optional**: `--input-mef-timeshift|-imt` ([-30;+30]), `--log-level|-ll`, `--cmd-options-file|-cof`

Verbatim example (from `help-cli\reference\index.html`; note it uses single‑dash long names, which do work):
```
PVsystCLI.exe convert-meteo -input-csv-file:Input.CSV -input-mef-file:Format.MEF -input-sit-file:Geneva.SIT -output-met-file:OutputFile.MET
```
From `ScriptsBat\03_Demo_PVsystCLI_NextDayForecast.bat`:
```
pvsystcli convert-meteo -icf:"%MET_CSVfilename%" -imf:"%MEF_filename%" -isf:"%SIT_filename%" -omf:"%MET_filename%"
```
From the Python/R helpers (with time shift):
```
pvsystcli convert-meteo -icf:{csv_file} -imf:{mef_filename} -isf:{sit_filename} -omf:{met_filename} -imt:{met_timeshift}
```

## 4. `create-site` (→ .SIT)

Usage: `PVsystCLI.exe create-site <options>`

**Required**: `--site-name|-sn`, `--latitude|-lat`, `--longitude|-lon`, `--weather-data-file|-wdf`
**Optional**: `--altitude|-alt`, `--timezone|-tz`, `--country-code|-cc`, `--region|-r`
(`africa, antarctica, australia, asia, europe, north_america, south_america, pacific`), `--source|-s`,
`--albedo|-alb`, `--output-sit-file|-osf`, `--log-level|-ll`, `--cmd-options-file|-cof`.

`--weather-data-file` = CSV with 14 rows (2 header rows + 12 monthly rows). Column names (any order):
`GlobH` (kWh/m²/mth, required), `Temp` (°C, required), `DiffH`, `WindVel`, `Linke`, `RelHum`.

Verbatim examples:
```
PVsystCLI.exe create-site --site-name:"Geneva" --latitude:46.2044 --longitude:6.1432 --weather-data-file:"C:\Data\Geneva_Weather.csv"
```
```
PVsystCLI.exe create-site --site-name:"Geneva" --latitude:46.2044 --longitude:6.1432 --altitude:375 --timezone:1 --country-code:CH --region:europe --albedo:0.2 --source:"Meteonorm" --weather-data-file:"C:\Data\Geneva_Weather.csv" --output-sit-file:"C:\Sites\Geneva.SIT"
```
From `ScriptsBat\05_Demo_PVsystCLI_CreateSite.bat`:
```
pvsystcli create-site --site-name:"Geneva" --latitude:46.2044 --longitude:6.1432 --altitude:375 --timezone:1 --country-code:CH --region:europe --weather-data-file:"%WeatherFile%" --output-sit-file:"%OutputSIT%"
```
If `--output-sit-file` is omitted the file is saved as `<site-name>.SIT` in the workspace `Sites` folder.

## 5. Batch mode (multiple simulations in one call)

From `help-cli\use-cases\simulation.html` (Example 5, "Batch mode") and release notes v8.0.12
("a new command option has been added to enable batch simulation from an input file"):
* `--batch-params-file` (`-bpf`) — batch parameters CSV edited in PVsyst.
* `--batch-rvt-file` (`-brf`) — RVT listing the batch result variables.

Verbatim:
```
PVsystCLI.exe run-simulation -w:"C:\PVsyst8.0_Data" -p:"Geneva.PRJ" -v:"VC0" -bpf:"C:\PVsyst8.0_Data\UserBatch\BatchsParams_0.csv" -brf:"C:\PVsyst8.0_Data\Params\BatchsVariables.rvt" -rl:"fr"
```
```
pvsystcli run-simulation -w:"%Ws_path%" -p:"%Prj_filename%" -v:"%VarNo%" -bpf:"%BatchParams_filename%" -brf:"%BatchRVT_filename%"
```
There is **no** dedicated `--headless`/`--batch`/`--silent` toggle: the CLI is always a console tool.
"Batch" = the `-bpf`/`-brf` option pair (parametric study from one input file). It links graphical
libraries, so on Windows Server a UI session is required (docs, Setup → Technical requirements).

## 6. Other commands

* `--cmd-options-file|-cof` (global option for the multi-option commands): a text file with one
  `--name:value` per line, no other separator. **Verified**: lines must be dashed (`--workspace:C:\...`),
  plain `workspace:...` lines are rejected with `Unknown option : workspace:...`.
* `export-logs` — optional `--target-folder|-t` (default = user home); writes a zip.
* `lic-info`, `lic-sync` — no options.
* `lic-activate` — required `--key|-k`; documented output code `100` = activated.
* `lic-deactivate` — required `--customer-id|-c`.

## 7. Exit codes / success detection (IMPORTANT for automation)

* **The process exit code is always 0**, including for invalid syntax, unknown options, missing required
  options, bad values, and missing/invalid workspace. Verified via the .NET Process object and via
  `cmd /c ... & echo %ERRORLEVEL%` for `version`, `--help`, and `run-simulation` (all → `ERRORLEVEL=0`).
* All messages (success **and** errors) go to **stdout**; stderr is empty.
* Therefore success must be detected by **parsing stdout**:
  * `create-site`  → `File "<path>" created with success.`
  * `convert-meteo` → `Conversion ended successfully... <path>`
  * `run-simulation` → `Simulation done in <m>min <s>sec` and `Report PDF <path> generated`
    (the shipped Python/R helpers test for the substrings `"Conversion ended successfully..."` and `"Simulation done in "`).
* Error strings to match on failure, e.g.:
  * `Error: Invalid command syntax` + `Unknown command line option : <x>` / `Unknown option : <x>`
  * `Required option [<name>] was not specified`
  * `Option [<name>] expected a following :<value> but none was found`
  * `Invalid value for option "<name>", accepted values are: 0, 1, 2, 3`
  * `Loading workspace <p> failed` + `Invalid workspace path: <p>` (or `This directory does not contain a workspace for PVsyst (PVsyst8.1_Data\)`)
  * `Loading project <name> failed`

Only documented exit/output code: `lic-activate` → `100` (activated successfully). No numeric exit
codes are documented for simulations.

## 8. Output files

* `--output-csv-file|-ocf` (and the default CSV when neither `-ocf` nor `-isf` is given): semicolon‑separated
  CSV, `PVSYST 8.1.7` banner, `Project/…` metadata lines, then a `date;…` header row, a units row
  (`;W/m²;…`), a blank line, then hourly rows `dd/mm/yy HH:MM;…`. (Verified: 333 KB CSV produced.)
* `--report-pdf-file|-rpf`: the PDF report (verified: 600 KB PDF produced).
* `--input-sfi-file|-isf` / `--input-rvt-file|-irf` select which variables land in the CSV.
* Default-CSV gotcha (reference): the default CSV is only written if both `-ocf` and `-isf` are absent; if
  that file already exists and `--overwrite-default-csv|-odc` is not passed, **the simulation will not run**.

## 9. Workspace expectations

* `run-simulation` **requires** `--workspace|-w` (full path). `convert-meteo`/`create-site` do not need a workspace.
* The path must be a genuine PVsyst workspace — a folder whose name ends in `_Data` (this machine:
  `C:\Users\Lenovo\PVsyst8.1_Data`) containing the standard subfolders
  (`Projects, Sites, Meteo, Models, ComposPV, Shadings, DataMeas, Templates, UserBatch, UserData, UserHourly, UserHourlyParams, UserImages, UserOptimization, UserRecycleBin, Other`).
* **Verified on this machine**: only the real workspace `C:\Users\Lenovo\PVsyst8.1_Data` was accepted.
  Copies of the same structure (including a full copy of the template workspace with a real project,
  SIT and MET) placed at other paths — `C:\PVsystTestWS\...`, `C:\PVsystTestWS2\...`,
  `C:\Users\Lenovo\TestWS\...`, `C:\Users\Lenovo\Documents\TestWS\...`, and
  `…\Desktop\PVSYST MCP\.scratch\ws\PVsyst8.1_Data` — were **rejected** with
  `Invalid workspace path: <p>` or `This directory does not contain a workspace for PVsyst (PVsyst8.1_Data\)`.
  The exact rule is not documented; the practical rule is: **use the workspace PVsyst itself uses**
  (default `%USERPROFILE%\PVsyst8.1_Data`, or the value configured in the PVsyst UI).
* Projects/variants must already exist in the workspace: paste `*.PRJ` + `*.VC*` into `Projects\`, PAN/OND
  files into `ComposPV\PVmodules` / `ComposPV\Inverters`, site into `Sites\`, weather into `Meteo\`.

## 10. Licensing behaviour observed

* In trial mode each run printed:
  `PVsystCLI is running in TRIAL mode, remaining days: 60, remaining executions: <n>`.
* The trial counter decremented after `run-simulation` **and** `create-site` (250 → 249 → 248) although the
  docs state only `convert-meteo` and `run-simulation` count as a call.

## 11. End-to-end proof performed

In the real workspace, with the `_DEMO_Residential.PRJ` / `VC0` project (staged then removed):
```
run-simulation -w:"C:\Users\Lenovo\PVsyst8.1_Data" -p:"_DEMO_Residential.PRJ" -v:"VC0" -ocf:"…\real_res_VC0.csv" -rpf:"…\real_res_VC0.pdf" -ll:3
```
→ exit 0, 22 s wall time, stdout ended with `Simulation done in 0min 3sec` and
`Report PDF … generated`; produced a 333,252-byte CSV and a 600,774-byte PDF.
