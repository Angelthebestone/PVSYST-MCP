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

## 7. 3D Shading scenes: `.SHD`, `.BLD`, `.SHO` (binary blocks in old files, text tree in 8.1.8)

Near-shading 3D representations are stored across three file extensions in `Shadings/` and are **never written** by the MCP (`escribir_archivo` rejects them): PVsyst writes them (§7.7).
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

### 7.3 Binary form of the elementary shapes (old files, PROVEN for 4 shapes)

Files of older versions (and the demos in `DataRO`) keep an elementary object's dimensions in a 39-byte block (2 bytes shape ID + 6x Real48 slots + `/`):
* Shape ID `11` (`0x000B`): `Parallélépipède` -> [Length, Width, Height, 0, 0, 1].
* Shape ID `23` (`0x0017`): `Toiture + pignons` (gabled roof).
* Shape ID `25` (`0x0019`): `Prism - chimney`.
* Shape ID `21` (`0x0015`): `Tree` (tree obstacle).

The slot meaning that the first version of this document gave for IDs 23, 25 and 21 was **inferred and is superseded by §7.6**: PVsyst 8.1.8's own file shows a tree has six parameters, not three.

### 7.4 What is NOT proven

* Arbitrary polygonal mesh topologies (non-regular 3D polygons).
* Internal tracking algorithm state bits in `Calcul=$...`.
* Writing a scene by hand. The MCP never serializes a scene: PVsyst writes it (§7.6).

### 7.5 Text form of a scene (PVsyst 8.1.8, PROVEN)

A scene saved by PVsyst 8.1.8 (`Archivo > Exportar escena (.SHD)`) is a UTF-8 text file with a byte-order mark, the same `PVObject_` tree as `.PRJ`/`.VC0` (`PVObject_=pvShading` ... `End of PVObject pvShading`):

```
  PVObject_=pvShading
    Comment=...  Version=8.1.8  Flags=$00  Precision=1 ...
    ListeObjets, list of=1 TShdObject
      PVObject_1=pvShdElemObject
        Comment=Casa_A                 (the object's name)
        DOrigine=5.000000,2.000000,1.000000   (x, y, z in m)
        DInclin=0.000000  BaseSlope=0.000000  DAzimut=0.000000
        ObjetElemType=oeParallelepipede
        Params=10.000,8.000,3.000,
      End of PVObject pvShdElemObject
    End of ListeObjets
    Orientations, list of=1 TOrientGroup ...
  End of PVObject pvShading
```

`leer` reads both forms (`src/escena_3d/texto.rs`; geometry reports `tipo_pvsyst` and the values of `Params=`). Fixtures: `tests/fixtures/escena_3d/`.

### 7.6 The 27 shapes of the elementary-object dialog (PROVEN, measured live 2026-10-29 on PVsyst 8.1.8 in PRUEBA)

PVsyst 8.1.8 in trial mode **does save** scenes. Each shape of `TElementaryObjectForm` was created with its default values, the scene exported and the file read back (`tests/fixtures/escena_3d/Formas27_8_1_8.SHD`). Then the MCP built each shape with a **different value in every parameter** through `gui_actuar accion:escena`, PVsyst saved the scene and the file was compared field by field with the request: shapes 1-14 of the table (triangle … `casa_techo_dos_lados`, 14 objects, 132 values) matched with 0 differences (`Formas_A`, 149 s), and so did shapes 15-27 (`casa_techo_asimetrico` … `pilon_electrico`, 13 objects, `tests/fixtures/escena_3d/Formas13_distintos_8_1_8.SHD`, 163 s with the release build, 12.5 s per object: the time is the dialog's own pauses, not the build). So the order of `Params=` of all 27 shapes is proven, not inferred. The `cable` dialog has one more edit below its parameters, «Objeto delgado» (a percentage, default 40), which the MCP leaves as it is. The order of `Params=` is the order of the dialog, top to bottom. Ranges are the MCP's, not PVsyst's.

| # | Clave del agente | `ObjetoElemType` (archivo) | Entrada del combo | `Params=` en orden (etiqueta = valor por defecto) |
|---|---|---|---|---|
| 1 | `triangulo` | `oeTriangle` | Triángulo | Lado 1-2 = 1; Lado 2-3 = 1; Lado 3-1 = 1; Rotación en plano = 0 |
| 2 | `triangulo_isosceles` | `oeTriangleIsocele` | Triángulo Isósceles | Base = 1; Altura = 1; Rotación en plano = 0 |
| 3 | `triangulo_rectangulo` | `oeTriangleRectangle` | Triángulo Rectángulo | Base = 1; Altura = 1; Rotación en plano = 0 |
| 4 | `rectangulo` | `oeRectangle` | Rectángulo | Anchura = 1; Altura = 2; Rotación en plano = 0 |
| 5 | `trapecio` | `oeTrapeze` | Trapecio | Base inferior = 3; Base superior = 2; Altura = 1; Desviación izquierda = 0.5; Rotación en plano = 0 |
| 6 | `poligono_regular` | `oeRegularPolygon` | Polígono regular | Radio = 1; Núm. de lados = 6; Rotación en plano = 0 |
| 7 | `sector_seudocircular` | `oePseudoCircleSector` | Sector seudocircular | Radio = 1; Ángulo de apertura = 180; Núm. de segmentos = 8; Rotación en plano = 0 |
| 8 | `paralelepipedo` | `oeParallelepipede` | Paralelepípedo | Anchura (DX) = 1; Longitud (DY) = 1; Altura (DZ) = 1 |
| 9 | `piramide_cuadrada` | `oeSquarePyramid` | Pirámide Cuadrada | Base = 1; Altura = 2 |
| 10 | `prisma_triangular` | `oeTriangularprism` | Prisma Triangular | Lado 1-2 = 1; Lado 2-3 = 1; Lado 3-1 = 1; Altura = 1 |
| 11 | `prisma_hexagonal` | `oeHexagonalPrism` | Prisma Hexagonal | Diámetro = 1; Altura = 1 |
| 12 | `prisma_octogonal` | `oeOctogonalPrism` | Prisma Octogonal | Diámetro = 1; Altura = 1 |
| 13 | `porcion_de_cilindro` | `oePortionCylinder` | Porción de Cilindro | Radio = 1; Ángulo de apertura = 180; Núm. de segmentos = 9; Altura = 1 |
| 14 | `casa_techo_dos_lados` | `oeHouse` | Casa + techo de 2 lados | Anchura (DX) = 8; Longitud (DY) = 12; Altura (DZ) = 5; Ángulo de inclinación del techo = 30; Aleros a dos aguas = 0.5; Aleros laterales = 0.5 |
| 15 | `casa_techo_asimetrico` | `oeAsymHouse` | Casa, techo asimétrico | Anchura (DX) = 8; Longitud (DY) = 12; Altura en la parte superior = 5; Ángulo de inclinación del techo 1 = 30; Ángulo de inclinación del techo 2 = 0.5; Proporción del techo 1 = 0.5 |
| 16 | `techo_diedro` | `oeRoof_Like_Diedre` | En forma de techo - diedro | Ancho base = 8; Longitud superior = 12; Ángulo de inclinación del techo = 30; Aguilón 1 ángulo = 0; Aguilón 2 ángulo = 0 |
| 17 | `tejado_una_agua_aguilones` | `oeOne_Sided_Roof_Gables` | Tejado 1 fase + aguilones | Ancho base = 8; Longitud superior = 12; Ángulo de inclinación del techo = 30; Aguilón 1 ángulo = 0; Aguilón 2 ángulo = 0 |
| 18 | `techo_dos_aguas_hastial` | `oeTwo_Sided_Roof_Gables` | Techo de 2 aguas + Hastial | Ancho base = 8; Longitud superior = 12; Ángulo de inclinación del techo = 30; Aguilón 1 ángulo = 0; Aguilón 2 ángulo = 0 |
| 19 | `techo_cuatro_lados` | `oeFour_sided_Roof` | Techo de 4 lados | Ancho base = 8; Longitud base = 12; Longitud superior = 8; Ángulo de inclinación del techo = 30 |
| 20 | `buhardilla` | `oeMansard` | Buhardilla | Ancho frontal = 1.5; Altura frontal = 1.5; Ángulo de inclinación del techo = 45; Ángulo de inclinación base = 30 |
| 21 | `prisma_chimenea` | `oePrism_Chimney` | Prisma - chimenea | Profundidad = 1; Anchura = 1; Altura = 1; Ángulo de inclinación del techo = 30 |
| 22 | `arbol` | `oeTree` | Árbol | Altura punto medio = 3; Altura media = 3; Altura parte baja = 2; Altura de tronco = 2; Diámetro medio = 4; Diámetro de tronco = 0.5 |
| 23 | `cable` | `oeCable` | Cable | Longitud = 10; Pendiente = 1; Espesor = 0.2; Núm. de segmentos = 6; Ángulo de la pendiente = 0 |
| 24 | `barandilla_un_travesano` | `oeHandrail1Crossbar` | Barandilla un travesaño | Longitud = 10; Altura = 1; Espesor = 0.05; Número de peldaños = 5 |
| 25 | `barandilla_dos_travesanos` | `oeHandrail2Crossbar` | Barandilla dos travesaños | Longitud = 10; Altura = 1; Altura del 2do travesaño = 0.6; Espesor = 0.05; Número de peldaños = 5 |
| 26 | `turbina_eolica` | `oeWindTurbine` | Turbina eólica | Altura del soporte = 40; Soporte de menor diámetro = 4; Diámetro superior del soporte = 3; Pala de longitud completa = 30; Pala de longitud pequeña = 5; Ancho original de la pala = 3; Ancho mediano de la pala = 3.5; Ancho final de la pala = 1; Longitud del generador = 5; Diámetro del generador = 4; Ángulo de las palas = 0 |
| 27 | `pilon_electrico` | `oeElectricPylon` | Pilón eléctrico | Altura = 40; Longitud base = 8; Longitud superior = 5; Anchura = 4; Número de brazos = 3; Altura del primer brazo = 20; Longitud del brazo = 12; Ancho del brazo = 2; Longitud del aislador = 0.3; Altura del aislador = 3; Espesor de malla = 0.4 |

`turbina_eolica` and `pilon_electrico` have an 11th value (`pala_angulo_deg`, `espesor_malla`) that sits below the visible part of the dialog's scroll box; its label was read by scrolling the box (`WM_VSCROLL`, `SB_BOTTOM`). Collisions are only checked between `paralelepipedo` objects, with the position as the lower corner (`[INFERENCIA]`: the anchor is not proven).

### 7.7 Creating a scene (`gui_actuar accion:escena`)

With the 3D editor open on an empty scene (`gui_actuar accion:funcion funcion:variante.sombreados_cercanos.construccion_perspectiva`) the MCP: opens the elementary-object dialog, picks the shape in the combo, types the name and each value (focus first, Tab to commit), closes with «Cerrar objeto»; opens «Lista y gestión de objetos» and types X, Y, Z and azimuth into the cells (a click opens a `TVTEdit`; rows are 26 px apart at 96 DPI, 20 visible); exports the scene with a name that must not exist; reads the file and returns `{objeto, clave, pedido, leido, estado}`. Measured: about 13 s per object on a debug build, 14 objects in 149 s. The editor stays open on the scene. `simular:true` only validates. Importing meshes is `accion:escena_importar` (§7.8). Not done: mouse on the canvas and creating groups, PV tables, tracker or shed arrays by values (§7.8).

### 7.8 Reading the open scene, the «Crear» menu and importing meshes (measured live 2026-10-29, PVsyst 8.1.8 in PRUEBA)

* **`gui_actuar accion:escena_leer`** (`escena`): the table of objects has no controls per row, so PVsyst exports the open scene with «Exportar escena (.SHD)» and the file is read back (2 objects, 2.2 s). The file stays in `Shadings`.
* **Menu «Crear»** of `TPerspectiveForm` (menu 1), owner-drawn, read from a screenshot: Objeto de sombreado elemental (the 27 shapes), Edificio/Objeto Compuesto, Objeto de tierra, Generador FV individual, Conjunto de rastreadores, Conjunto de tablas, Conjunto de cúpulas, Conjunto de protectores solares, Plano FV rectangular (simple o múltiple), Plano FV poligonal. Only the first is automated; the others are **not measured**.
* **Menu «Archivo ▸ Importar»** (menu 0, item 4, a submenu) has five entries: «Importar una escena 3D (3DS, DAE, PVC, PVC2, PVZ2)», «Importar un archivo Helios3D (H2P)», «Importar datos terrestres (CSV, TIF)», «Importar una imagen de suelo» and «Descargar una imagen terrestre por satélite y/o datos terrestres». Menu keys: `Alt`, `Down`×5 to «Importar», `Right`, `Enter` (the first entry).
* **Import of a scene**: the standard Windows file dialog (`#32770`, title «Importar una escena 3D (3DS, DAE, PVC, PVC2, PVZ2)»; the file name goes in its `Edit` and «Abrir» is a `Button`), then `TImportResultForm` («Importar resultado»): counts (objects, vertices, faces), input size and units (combo, «Pulgadas (in)» by default), translation (`Automático` + x/y/z), «Aplicar rotación de 180°», PV-field orientation («Mejor azimut», «Borde más largo», «Este/Oeste») and «Crear múltiples orientaciones». With the sample `SHD_PVC_Fixed_Example.pvc` it reported 66 objects, 264 vertices.
* **A file under `Program Files` is refused** («Accès au fichier refusé», a `TMessageForm`): the mesh has to be copied to the user's workspace first.
* **Automated: `gui_actuar accion:escena_importar`** (`archivo`, `escena`, optional `unidades`, `simular`). The id of «Importar una escena 3D» is read from the `HMENU` of the submenu (`GetSubMenu` on the popup of menu 0, item 4): the keyboard only opens the top-level menu and the submenu is never entered (`accion:menu` takes `subitem` for the same purpose). The file name goes in the dialog's `Edit` by `WM_CHAR` and `IDOK` (command 1) is posted to the dialog. The unit is the first `TComboBox` of «Importar resultado». The text of that form comes from OCR (labels such as `atos de la escena`, `1x801`), so the counts are not read from it: the scene PVsyst exports afterwards is the evidence. Measured with `SHD_PVC_Fixed_Example.pvc` (copied out of `Program Files`): the form announced 66 objects and 264 vertices, and the exported file (`Imp_2.SHD`, 54 KB) holds 66 `pvShdArrayTable`, 2 `pvOrient` and 1 `pvShdObjectGroup`; the whole call took 20 s. PVsyst closes its export dialog before it has finished writing a large scene, so the file counts as written once its size stops changing.
* **Menu «Crear» (menu 1), measured 2026-10-30:** items 5 to 10 each open a field dialog with tabs, read tab by tab and closed with «Cancelar» (a visited tab makes PVsyst ask «¿Quiere mantener sus modificaciones en el objeto?»: «No» drops it): 5 «Conjunto de rastreadores» `TTrackerFieldForm` (General, Tamaños de los rastreadores, Parámetros de rastreo, Partición); 6 «Conjunto de tablas», 7 «Conjunto de cúpulas» and 8 «Conjunto de protectores solares» all `TShedFieldForm` (Parámetros básicos, Tamaños de los cobertizos, Partición); 9 «Plano FV rectangular» `TRectFieldForm` (Parámetros básicos, Tamaño rect., Partición); 10 «Plano FV poligonal» `TPolyFieldForm` (its «Dibujo poli» tab opens `TOrientationForm` and disables the form until it is cancelled). Pressing «Cerrar objeto» on the rectangular plane without touching a field adds a `pvShdArrayRect` at the origin (keys saved: `NModInHeight`, `NModInWidth`, `ModuleOrient`, `ModSpacingX/Y`, `DInclin`, `DAzimut`, `PanneauPVName`, `LongA`, `LargA`); its tilt and azimuth edits are disabled because they come from the scene's orientation, and the module comes from the database. The tables, trackers, domes, sun shields and polygonal plane were not closed with «Cerrar objeto», so what they add to the scene is not measured.
* **Not automated**: creating buildings, ground objects, PV tables, trackers, domes, sun shields and rectangular or polygonal planes by values. Each needs its own field-to-key map checked against the saved `.SHD`, which was not done.

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
