# PVsyst 8.1.7 — File formats observed from the outside

This document describes the PVsyst data formats **as observed from the outside**,
by reading the shipped data files and the official documentation
(`C:\Program Files\PVsyst8.1.7\help\...`) with a text/hex viewer.

> **Scope / rules honoured**
> * No PVsyst source code, no decompilation. `PVsyst.exe` / `PVsystCLI.exe` were
>   only ever *run* black-box (the CLI `help`), never opened in Ghidra/IDA.
> * No PVsyst calculation logic is reproduced here or in the MCP. The MCP only
>   reads/writes files and shells out to the *documented* `PVsystCLI` command.
> * Everything labelled **PROVEN** below was cross-checked against the actual
>   bytes and the official docs. **INFERRED** marks the few fields whose exact
>   role was not independently confirmed.

---

## 1. Where the data lives

| Thing | Path (this machine) |
|---|---|
| Program | `C:\Program Files\PVsyst8.1.7\` |
| GUI | `C:\Program Files\PVsyst8.1.7\PVsyst.exe` |
| Headless CLI | `C:\Program Files\PVsyst8.1.7\PVsystCLI.exe` |
| **User workspace** | `C:\Users\Lenovo\PVsyst8.1_Data\` |
| Read-only bundled DB | `C:\Program Files\PVsyst8.1.7\DataRO\PVsyst8.1_Data\` |
| Per-user config | `C:\Users\Lenovo\AppData\Roaming\PVsyst SA\PVsyst 8.1.7\` |

The workspace is created on first run and contains fixed sub-folders
(source: `help\getting-started\file-organisation\user-data\workspace`):

```
PVsyst8.1_Data/
  ComposPV/      component database (one sub-folder per kind)
  DataMeas/      measured-data interpretation/format files (.DAF, .DAM)
  Meteo/         weather data (.MET, .MEF, .TFT)
  Models/        templates & models (.LOD, .SPL, .SFI, .RVT)
  Projects/      projects and variants (.PRJ, .VCi, .CMi, .PSG, .PSS, .PSP)
  Shadings/      horizon & shading scenes (.HOR, .SHD, .BLD, .SHO)
  Other/         miscellaneous
  Sites/         geographical sites (.SIT)
  UserData/      user export files (Components.xls, result CSVs)
  Templates/     templates (empty until "Reload templates")
  UserBatch/     batch parameter CSVs for the CLI
  UserHourly/ UserHourlyParams/ UserImages/ UserOptimization/ UserRecycleBin/
```

Component sub-folders (`ComposPV/`):

| Folder | Extension | Contents |
|---|---|---|
| `PVModules/` | `.PAN` | PV module parameters & characteristics |
| `Inverters/` | `.OND` | grid inverter limits + efficiency profile |
| `Batteries/` | `.BTR` | battery characteristics |
| `Pumps/` | `.PMP` | pump device |
| `Regulators/` | `.RLT` | charge/discharge regulator (stand-alone) |
| `RegulPmp/` | `.RLP` | pumping-system controller |
| `Gensets/` | `.GEN` | back-up generator |
| `Sellers/` | `.REV` | manufacturer/reseller addresses |
| `Clients/` | `.CLI`? | client data |

**Format evolution** (source: `help\...\format-of-pvsyst-files.html`):
* ≤ 6.39 — everything binary, not readable.
* ≥ **6.40 — all files text** (project, variant, weather, components…).
* ≥ 6.60 — `.VCi` project-variant format improved.
* ≥ **6.80 — encoding switched to UTF-8** (international characters).

The workspace folder name encodes the format generation
(`PVsyst6_Data`, `PVsyst640_Data`, `PVsyst660_Data`, `PVsyst680_Data`,
`PVsyst8.1_Data`); formats are **not** backward compatible.

> **See also:** the consolidated extension table and the full PVsyst feature
> inventory (mapped to each file/CLI artifact) are in
> [`pvsyst_features.md`](./pvsyst_features.md); the tool-by-tool coverage map is
> in [`cobertura.md`](./cobertura.md).

---

## 2. The common "PVObject" text format (≥ 6.40)

Every ≥ 6.40 file (`.PRJ`, `.VCi`, `.OND`, `.MET`, `.SIT`, `.PAN`, …) is a tree of
typed objects. Lines are `<indent>Key=Value`; nested objects use:

```
PVObject_<Role>=<Type>      ← open a child object
  Key=Value
End of PVObject <Type>      ← close it
```

* Indentation is cosmetic (2 spaces per level) — the tree is driven by the
  open/close markers.
* `PVObject_=<Type>` (no role) marks the file's root object.
* Some files add `EndTags`, `End Remarks`, `/` and `... Start` / `... End` /
  `X End;` markers — they carry no scalar data for our purposes.

### 2.1 `.PRJ` — project definition (PROVEN)

Real `_DEMO_Commercial.PRJ` (1746 bytes, UTF-8/ASCII, CRLF):

```
PVObject_=pvProject
  Comment=DEMO commercial system Oakland CA
  Version=8.1.0
  Flags=$00080010
PrintSettings=TPrintSettings
  PrintCoverPage=True
  ...
EndTags
MeteoFileName=Oakland CA_MN90_SYN.MET
MeteoFileDate=28/08/24 13:36
PVObject_SitePrj=pvSite
  Comment=Oakland CA;United States;North_America;
  Flags=$0006283F
  NomF=Oakland CA_MN90.SIT
  Site=Oakland CA
  Country=United States
  Region=North_America
  Source=Meteonorm 9.0
  Latitude=37.8029
  Longitude=-122.2579
  Altitude=47
  TimeZone=-8.0
  Albedo=0.20
  GlobalH=70.1,90.3,...    (12 monthly + annual)
  DiffuseH=...
  TAmb=...
  WindVel=...
  Linke=...
  RelHum=...
  Extraterr=...
  YrToYrVar=2.302
End of PVObject pvSite
Albedo=0.200,...
DesignTempMin=-10
TempOper=50
...
TranspositionAlgorithm=CircSolSeparate
ACLossesRef=STC
End of PVObject pvProject
```

Key fields the MCP reads: `Comment`, `Version`, `MeteoFileName`,
`PVObject_SitePrj` → `Site`, `Latitude`, `Longitude`, `NomF` (the `.SIT`).

### 2.2 `.VCi` — simulation variant (PROVEN)

`<ProjectName>.VCi`, last char of the extension is the version number `0..9`, `A..Z`
(so `VC0`, `VC1`, … `VCA`, …). Same name as the project; large (≈1.5–3 MB) because
it also stores simulation results. Root object is `pvVCalcul`.

The **module reference** lives inside each `pvSubArray`:

```
  SubArrays=Start
    PVObject_=pvSubArray
      Comment=PV Array
      SubArrayId=1
      PVModule=Generic_Mono_440W_Half.PAN     ← the .PAN used by this sub-array
      GInverter=Generic_60kW.OND              ← the .OND used
      NModSerie=24
      NStringCh=48
      NInverter=6
      ...
```

`crear_proyecto` rewrites the `PVModule=` line(s) — this is the only
edit needed to point a variant at a different module.

`<ProjectName>.CMi` = simulation-vs-measured-data comparison variant.

### 2.3 `.OND` — grid inverter (PROVEN, fully text)

Real `Generic_50kW.OND`:

```
PVObject_=pvGInverter
  Comment=Generic	50 kWac inverter	Generic device
  Version=7.3.4
  Flags=$0570
  PVObject_Commercial=pvCommercial
    Manufacturer=Generic
    Model=50 kWac inverter
    ...
  End of PVObject pvCommercial
  Transfo=Without
  Converter=TConverter
    PNomConv=50.000
    PMaxOUT=50.000
    VOutConv=400.0
    VMppMin=200
    VMPPMax=1000
    VAbsMax=1800
    EfficMax=98.10
    EfficEuro=97.80
    ModeOper=MPPT
    ProfilPIO=TCubicProfile
      NPtsEff=8
      Point_1=248,0
      Point_2=1500,1370
      ...
    End of TCubicProfile
  End of TConverter
  NbInputs=4
  NbMPPT=2
  MasterSlave=No_M_S
  Night_Loss=4.80
End of PVObject pvGInverter
```

### 2.4 `.MET` — weather data (text header; hourly data)

Root `pvMeteo`, with a nested `pvSite` and monthly summary values
(`GlobalH`, `DiffuseH`, `TAmb`, `WindVel`, `Linke`, `RelHum`, `Extraterr`).
The hourly records may follow in a compact block; the header is plain text.

### 2.5 `.SIT` — geographical site (text)

Root `pvSite` with `Site`, `Country`, `Region`, `Latitude`, `Longitude`,
`Altitude`, `TimeZone`, `Albedo` and the twelve monthly climatological values.
Created headlessly by `PVsystCLI create-site`.

### 2.6 `.RVT` / `.SFI` — result templates (text)

`Models/*.RVT` = list of output variables for CSV exports; `Models/*.SFI` =
export format definitions. Both are PVObject text.

---

## 3. `.PAN` — PV module (two layouts!)

### 3.1 Modern (PVsyst ≥ 6.40) — text

A `PVObject_=pvModule` document with the "Basic data" + "Manufacturer
specifications" fields (names from
`help\component-database\photovoltaics-modules\...\pv-modules-basic-data`):
`Manufacturer`, `Model`, `DataSource`, `Technology`, `PNom` (nominal power),
`Gref`/`Tref` (measurement reference conditions), **`Isc`**, **`Voc`**,
`Imp`/`Vmpp` (max-power point), `muIsc` (Isc temperature coefficient). The
`.OND` above is the same format, so a v8 `.PAN` is fully text.

`leer_modulo` maps a rich alias list (`PNom`/`Pnom`/`NominalPower`,
`Impp`/`Imp`/`IMPP`, `Vmpp`/`Vmp`/`VMPP`, `muIsc`/`MuISC`, …).

### 3.2 Legacy (< 6.40) — hybrid ASCII + **Delphi `Real48`** binary (PROVEN)

The two demo modules shipped with 8.1.7 are legacy files (`Version 5.4` /
`5.67`). Layout:

```
pvModule;Soitec<TAB>CX-M500 1.00.086<TAB>Manufacturer;    ASCII  header
Version 5.67; Calcul=$00003043;                            ASCII
GaInP2/GaAs/Ge;240;10;2;0.000;0.000;                       ASCII  tech;NCelS;NCelP;NDiode;;;
<300 bytes = 50 × 6-byte Real48>                            BINARY module block
/\r\n                                                       terminator 0x2F 0D 0A
pvCommercial;;                                             ASCII
Version 5.67; Calcul=$00000041;
;
EUR;;;
<52 bytes = Real48>                                         BINARY commercial block
/\r\n
End Remarks
```

* **Block 1** starts 2 bytes after the **3rd CR** and runs until the `0x2F`
  (`/`) terminator → `(end-start)/6` values (50 for these files).
* **`Real48`**, PVsyst's legacy Delphi real (6 bytes; PVsyst author confirms
  "the old DELPHI encoding of reals on 6 bytes"). Decode:

  ```
  exp      = b0 - 129                      (b0 == 0  →  value is 0.0)
  mantissa = b1 | b2<<8 | b3<<16 | b4<<24 | (b5 & 0x7F)<<32
  sign     = (b5 & 0x80) ? -1 : +1
  value    = sign * (1 + mantissa / 2^39) * 2^exp
  ```

* Decoded slot layout (slot = 6-byte offset from block start):

| slot | field | notes |
|---|---|---|
| 0 | **PNom** (W) | validated exactly (`CX-M500` → 2450.0) |
| 1 | VMaxIEC (V) | 1000 on the file that specifies it |
| 2 | tolerance | |
| 3 | Area (m²) | = Width × Height |
| 4 | Cell area (cm²) | = Area·10⁴ /(NCelS·NCelP) |
| 5 | **Gref** (W/m²) | measurement reference irradiance |
| 6 | **Tref** (°C) | measurement reference temperature |
| 7 | Absorb (INFERRED) | 0.9 |
| 8 | **Isc** (A) | |
| 9 | muIsc (mA/°C) | |
| 10 | **Voc** (V) | |
| 11 | muVoc (mV/°C) | |
| 12 | **Imp** (A) | |
| 13 | **Vmp** (V) | |
| 14 | bypass-diode V | −0.70 |
| 17 | Rshunt (Ω) | |
| 18 | Rserie (Ω) | |
| 23 | Rp0 (Ω), 24 RpExp | model defaults |
| 25 | muPmpReq (%/K) (INFERRED) | |
| 28–49 | model defaults / coefficients | roles partly unidentified |

  Decoded example — `Soitec CX-M500` (a high-voltage concentrator module,
  240 cells in series): PNom 2450 W, Gref 943, Tref 64, Isc 3.861 A,
  Voc 695.7 V, Imp 3.564 A, Vmp 606.6 V, muVoc −1042.7 mV/°C.
  `Concentrix CPV Gen II`: PNom 87 W, Isc 0.642 A, Voc 141.7 V.

* **Block 2** (commercial) = `[Width, Height, Depth, Weight]` again as Real48.
  For `CX-M500`: 2.286 m × 3.428 m × 0.102 m × 210 kg (matches the datasheet).

Validation of the decoder: slot 0 = exact datasheet P_MPP; slot 3 = W×H;
slot 4 = Area·10⁴/cells; per-cell Voc (695.7/240 = 2.90 V) correct for
triple-junction CPV; muVoc/Voc = −0.141 %/K matches the −0.14 %/K datasheet.

> The legacy decoder is a **reader only**. It reconstructs the stored numbers;
> it contains none of PVsyst's simulation/one-diode logic.
> A standalone reference implementation of this exact decoder lives in
> [`tools/pan_decode.py`](../tools/pan_decode.py); `src/components/pan.rs`
> (`real48`) is the Rust port.

> **Note:** PVsyst itself manages the `ComposPV/` component folders — running a
> simulation rewrites/cleans them. The MCP therefore reads modules from the
> workspace when present and falls back to the bundled read-only database
> (`DataRO\PVsyst8.1_Data\ComposPV`) otherwise.


---

## 4. Result CSVs (`leer_resultados_csv`)

`PVsystCLI run-simulation -ocf:<file>` (or the default CSV) writes a
semicolon-separated file: a `PVSYST 8.1.7` banner + project metadata lines,
then a header row whose first cell is `date`, then a units row
(`;W/m²;…`), a blank line, and hourly rows `dd/mm/yy HH:MM;…`.

`leer_resultados_csv` finds the `date;…` header, reads the units row, and
returns metadata + header + per-column totals + sample rows.

---

## 5. The bundled database CSVs are encrypted

`DataRO\*.csv` (`PVModuleDB.csv` 8.7 MB, `InverterDB.csv` 5.6 MB,
`BatteriesDB.csv`, `Generators.csv`, …) are **not plain text** — they begin with
non-UTF-8 bytes (`qÚ…`) and are PVsyst-encrypted. They are therefore *not* used
as a data source. The MCP reads the per-component files (`.PAN`, `.OND`, …) and
the workspace structure instead.

---

## 6. PVsystCLI — the only way the MCP triggers a simulation

The MCP does **not** compute anything. For simulations it calls the documented
headless CLI (see `PVSYST_CLI_REFERENCE.md` for the exhaustive reference):

```
PVsystCLI.exe run-simulation -w:"<workspace>" -p:"<Project>.PRJ" -v:"<VC0>"
                  [-ocf:"<out.csv>"] [-rpf:"<out.pdf>"]
                  [-imf:"<met>"] [-isf:"<sfi>"]
                  [-bpf:"<batchParams.csv>"] [-brf:"<batchVars.rvt>"]   ← batch mode
                  [-ll:3]
```

* Options use a **colon** (`-w:"…"`), never `=` or a space.
* The process **always exits 0** — success is detected from stdout
  (`Simulation done in …`).
* Batch/parametric mode = `-bpf` + `-brf`.
* `--workspace` must be a genuine `*_Data` workspace folder.

Other commands: `create-site`, `convert-meteo`, `export-logs`, `lic-*`, `version`.

---

## 7. 3D Shading scenes: `.SHD`, `.BLD`, `.SHO` (Read-only; Hybrid ASCII + Delphi Real48)

Near-shading 3D representations are stored across three file extensions in `Shadings/` and are strictly **read-only** in the MCP (writing is rejected with `Error::Invalid`):
* `.SHD`: Complete 3D scene (root `pvShading;`).
* `.BLD`: Building or obstacle cluster (root `pvShdBuilding;`).
* `.SHO`: Shading object definition or tracker array (root `pvShdObjects;`).
### 7.1 General structure (PROVEN)

Every file consists of two-line ASCII object headers followed by binary blocks delimited by `/` (`0x2F`):
1. Root header:
   * Line 0: `<RootClass>;<Description>;\r\n` (e.g. `pvShading;...;`, `pvShdBuilding;...;`, `pvShdObjects;...;`).
   * Line 1: `Version <Ver>; Calcul=$<HexFlags>;\r\n`.
   * Lines 2..: Root metadata / parameters ending with `/\r\n` before first child object.
2. Child objects (`pvShdArrayShed`, `pvShdBuilding`, `pvShdElemObject`, `pvShdArrayTracking`):
   * Line 0: `<ObjectClass>;<ObjectName>;\r\n`.
   * Line 1: `Version <Ver>; Calcul=$<HexFlags>;\r\n`.
   * Line 2 (optional): `<Count1>; <Count2>/\r\n`.

### 7.2 Spatial placement block (PROVEN)

For placed objects, a fixed 44-byte binary block (43 bytes payload + `/`) encodes position and rotation:

| Byte offset | Length | Type | Role |
|---|---|---|---|
| 0..3 | 4 bytes | u32 LE | Object ID / status flags |
| 4..7 | 4 bytes | u32 LE | Delphi TColor (render color, e.g. `$0000C0FF`) |
| 8..11 | 4 bytes | u32 LE | Group / selection flags |
| 12..17 | 6 bytes | Delphi Real48 | X coordinate (meters) |
| 18..23 | 6 bytes | Delphi Real48 | Y coordinate (meters) |
| 24..29 | 6 bytes | Delphi Real48 | Z coordinate (meters) |
| 30..35 | 6 bytes | Delphi Real48 | Primary rotation / Azimuth (radians/degrees) |
| 36..41 | 6 bytes | Delphi Real48 | Secondary rotation / Tilt |
| 42 | 1 byte | u8 | Visibility flag (`0x01` = visible) |
| 43 | 1 byte | ASCII `/` | Block terminator (`0x2F`) |

### 7.3 Elementary geometric shapes (`pvShdElemObject`) (PROVEN)

Elementary objects define their dimensions via a 39-byte block (2 bytes shape ID + 6x Real48 slots + `/`):
* Shape ID `11` (`0x000B`): `Parallélépipède` -> [Length, Width, Height, 0, 0, 1].
* Shape ID `23` (`0x0017`): `Toiture + pignons` (gabled roof) -> [Length, Width, Tilt (°), 0, 0, 1].
* Shape ID `25` (`0x0019`): `Prism - chimney` -> [Width, Length, Height, Tilt (°), 1, 1].
* Shape ID `21` (`0x0015`): `Tree` (tree obstacle) -> [Crown radius, Trunk radius, Height, ...].

### 7.4 What is NOT proven (INFERRED) & Read-only policy

* Arbitrary polygonal mesh topologies (non-regular 3D polygons).
* Internal tracking algorithm state bits in `Calcul=$...`.
* Scene creation/serialization: mesh structure is not proven. Writing is rejected with `Error::Invalid("escena 3D: solo lectura; el formato de las mallas no está demostrado, créala con la GUI de PVsyst (Construcción / Perspectiva)")`.
---

### 7.5 Shape list of the elementary-object dialog (measured live, 2026-10-27)

`TElementaryObjectForm` (`escena_3d.crear.objeto_elemental`, opened with the editor menu and closed with Cancelar; nothing saved)
offers 27 shapes in a `TComboBox`, grouped as Superficies, Volúmenes, Elementos de construcción and Formas especiales:

  - Triángulo — no demostrada
  - Triángulo Isósceles — no demostrada
  - Triángulo Rectángulo — no demostrada
  - Rectángulo — no demostrada
  - Trapecio — no demostrada
  - Polígono regular — no demostrada
  - Sector seudocircular — no demostrada
  - Paralelepípedo — **demostrada** (id 11 Parallélépipède)
  - Pirámide Cuadrada — no demostrada
  - Prisma Triangular — no demostrada
  - Prisma Hexagonal — no demostrada
  - Prisma Octogonal — no demostrada
  - Porción de Cilindro — no demostrada
  - Casa + techo de 2 lados — no demostrada
  - Casa, techo asimétrico — no demostrada
  - En forma de techo - diedro — no demostrada
  - Tejado 1 fase + aguilones — no demostrada
  - Techo de 2 aguas + Hastial — **demostrada** (id 23 Toiture + pignons; la correspondencia del nombre español es [INFERENCIA])
  - Techo de 4 lados — no demostrada
  - Buhardilla — no demostrada
  - Prisma - chimenea — **demostrada** (id 25 Prism - chimney)
  - Árbol — **demostrada** (id 21 Tree)
  - Cable — no demostrada
  - Barandilla un travesaño — no demostrada
  - Barandilla dos travesaños — no demostrada
  - Turbina eólica — no demostrada
  - Pilón eléctrico — no demostrada

Only the four marked *demostrada* were confirmed by reading a file PVsyst saved (§7.3). The rest stay **unproven and
read-only**: PVsyst in TRIAL mode does not save, so no file exists to compare, and the MCP does not write scenes by hand.
The dialog also has a name field, three parameter fields (`Param1Edit`…`Param3Edit` placeholders), a colour/size field and two
checkboxes; `TGroundObjectForm` (11 fields), `TTableFieldForm` (7, with an orientation list) and `TBuildingObjectForm`
(no readable fields) were also opened. Raw measurements: `.scratch/exp/escena_3d_formas.json` (not versioned).

`gui_actuar accion:escena` with `simular:true` validates a request for the four proven shapes (names `paralelepipedo`,
`tejado_dos_aguas`, `chimenea`, `arbol`) and reports the file slot of each dimension. It never writes a scene; creating one in the
editor is not implemented. Ranges are the MCP's (PVsyst's were not measured) and collisions assume the position is the lower corner
(`[INFERENCIA]`).

## 8. Weather format specifications: `.MEF`

`.MEF` files define how PVsyst imports external CSV/text weather time series into `.MET` binary files.

### 8.1 Modern format (≥ 6.40, PVObject `pvMeteoFormat`) (PROVEN)

Standard UTF-8 text object tree starting with `PVObject_=pvMeteoFormat` and ending with `End of PVObject pvMeteoFormat`:
* `Comment`, `Version`, `Flags`: format metadata.
* `DateFmtType`: date parsing pattern (e.g. `MM_DD_hh_mm` or `YYxMMxDDxhhxmm`).
* `Separ`: delimiter byte (e.g. `$003B` for `;`, `$002C` for `,`).
* `MeasureStep`: time step in minutes (e.g. `60`).
* `RefTimeS`: time reference (`LegalTime`, `SolarTime`).
* `AlbedoSolMF`: ground albedo fraction (e.g. `0.2`).
* `NoFieldDate`: 1-based column indices for date components, followed by `0` as an unused date component sentinel (e.g. `NoFieldDate=2,3,4,5,0,`).
* `VarASCIIList`: sub-object array of variables to import (`GlobHor`, `DiffHor`, `T_Amb`, `WindVel`):
  * `VarIdent`: variable identifier.
  * `VarType`: physical quantity category (`Irrad`, `Temper`, `WindVel`).
  * `NoField`: 1-based CSV column index.
  * `FieldMult`: multiplier to convert values to standard PVsyst units (e.g. `1.00000` or `0.00360`).
  * `UnitId`: target unit (`W_m2`, `DegC`, `m_s`).

### 8.2 Legacy format (< 6.40, Delphi ASCII + Real48) (PROVEN / READ-ONLY)

Legacy MEF is strictly **read-only** (writing is rejected with `Error::Invalid("no hay escritor legacy demostrado")`).
The parser decodes:
* Line 0: `pvMeteoFormat;<Description>;\r\n` -> `comentario`.
* Line 1: `Version <Ver>; Calcul=$<HexFlags>;\r\n` -> `version` and `calcul` (with prefixes `Version ` and `Calcul=` stripped).
* Column mapping lines: `<VarIdent>;<VarType>;<ColIndex>;<Min>;<Max>;<Mult>;\r\n` -> `VarIdent`, `VarType`, 1-based column index, multiplier, and unit derived from `VarType` (`DegC` for `Temper`, `m_s` for `WindVel`, `W_m2` for `Irrad`).
* Tail: binary Delphi Real48 parameters (such as default albedo) are not decoded into domain fields.
---

## 9. Currency exchange rate table: `Currency.dat` (Read-only; PROVEN)

Located in `DataRO\Currency.dat`, this semicolon-delimited text file defines foreign currency exchange rates relative to the base currency (EUR). It is strictly **read-only** (writing is rejected with `Error::Invalid("solo lectura: PVsyst lo lee de su instalación (DataRO) y no está demostrado que lea una copia del workspace")` because PVsyst reads it from DataRO and has not been proven to read a workspace copy):
* Line 0: Format header `CURRENCY_RATES_V2;\r\n`.
* Line 1: `<Count>; <BaseCurrencyIndex>;<Flag>;\r\n` (e.g. `33; 1;1;\r\n` where 33 is total rows, 1 is 1-based base currency EUR).
* Lines 2..34: Currency rows `<Code>;<Name>;<FormattedRate>;` (33 rows).
  * `<Code>`: 3-letter uppercase ISO currency code (`EUR`, `USD`, `GBP`...).
  * `<Name>`: English name in Latin-1 / DOS OEM encoding (e.g. `Icelandic Króna`).
  * `<FormattedRate>`: Exchange rate formatted in a strict 10-character right-aligned column:
    `width = 10`, `decimals = min(6, 10 - 1 - integer_digits)`.
  * Note: The final line does not include a trailing `\r\n`.
* GUI check (PVsyst 8.1.7, `Eco → Tasas`, `TCurrenciesForm`): the window shows `EUR - Euro` as reference currency and lists GBP, CHF, AUD, CAD, CNY, JPY, BGN, BRL, CZK, DKK, HKD with the same code, name and rate as `leer` returns for those rows (e.g. `GBP 0.89470000`, `JPY 131.57890000`). Only those 11 rows were visible in the viewport; the USD row and rows 14–33 were not compared. The grid rows are TEdit controls whose text could not be read by handle (they return their component name), so the comparison is from a capture.
