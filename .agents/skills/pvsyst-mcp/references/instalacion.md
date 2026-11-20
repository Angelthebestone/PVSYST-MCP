# Instalación y verificación del MCP `pvsyst`

## 1. Build

```powershell
cargo build --release
# → target\release\pvsyst-mcp.exe
```

Toolchain: Rust stable. En esta máquina se usa el target GNU
(`stable-x86_64-pc-windows-gnu`) con MinGW-w64 en el `PATH`.

## 2. Instalar en una ruta sin espacios

El launcher de algunos clientes no entrecomilla la ruta del ejecutable, así que
un `command` con espacios (`...\PVSYST MCP\...`) falla en silencio. Instala en
`C:\pvsyst-mcp\`:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

El script compila, copia a `C:\pvsyst-mcp\pvsyst-mcp.exe` y resuelve el caso de
un servidor en ejecución (renombra el `.exe` bloqueado o detiene el proceso).
Repítelo tras cada rebuild.

## 3. Registro

Ya está registrado a nivel de proyecto en `.pi/mcp.json`, así que Pi lo carga al
abrir una sesión en este directorio. Para registrarlo a nivel de usuario:

```powershell
pi mcp add pvsyst --env PVSYST_WORKSPACE=C:\Users\Lenovo\PVsyst8.1_Data -- C:\pvsyst-mcp\pvsyst-mcp.exe
```

Otros clientes que leen el mapa `mcpServers` (Claude Desktop, Cursor, VS Code,
Command Code) usan la misma entrada; ver `mcp.example.json` y `.mcp.json`.

## 4. Verificar

```powershell
pi mcp list                                   # servidor conectado + tools
cargo test                                    # suite del proyecto
```

Dentro de una sesión: `/mcp` para inspeccionar el servidor, `/reload` tras
cambiar `.pi/` o `.agents/`.

## Variables de entorno

Todas opcionales; un valor vacío cuenta como no definido.

| Variable | Uso |
|---|---|
| `PVSYST_WORKSPACE` | Carpeta de datos del usuario (`%USERPROFILE%\PVsyst8.1_Data`). |
| `PVSYST_CLI` | `PVsystCLI.exe` (aquí `C:\Program Files\PVsyst8.1.8\PVsystCLI.exe`). |
| `PVSYST_GUI` | `PVsyst.exe` (aquí `C:\Program Files\PVsyst8.1.8\PVsyst.exe`). |
| `PVSYST_MCP_APROXIMACIONES` | `1` activa cálculos propios del MCP. Déjalo sin definir. |
| `PVSYST_MCP_ESCRITORIO` | `aislado` ejecuta PVsyst en un escritorio Win32 privado. |

## Problemas típicos

- **El servidor aparece pero no conecta**: la ruta del `command` tiene espacios.
  Reinstala en `C:\pvsyst-mcp\`.
- **`PVsystCLI.exe` no encontrado**: la instalación es 8.1.8, no 8.1.7. Ajusta
  `PVSYST_CLI` en `.pi/mcp.json`.
- **Simulación rechazada por licencia**: solo la simulación requiere licencia;
  archivos, sitios, dimensionado y GUI funcionan sin ella.
