# `.agents/` — configuración de agentes de este proyecto

Este directorio declara el alcance de los agentes que trabajan en **PVSYST MCP**:
qué skills aplican y qué MCP se usa.

## Alcance: skills de AWS desactivadas

Este proyecto es un servidor MCP en Rust que opera **PVsyst 8.1.8** sobre Windows.
No hay nada de AWS: ni Bedrock, ni CDK, ni Lambda, ni IAM, ni S3.

Por eso, en este proyecto **se ignoran todas las skills de AWS** que viven en
`~/.agents/skills/`:

```
amazon-bedrock              aws-observability            aws-storage
aws-ai-ml                   aws-sdk-js-v3-usage          launch-with-aws
aws-auth                    aws-sdk-python-usage         setting-up-cloudwatch-observability
aws-billing-and-cost-management  aws-sdk-swift-usage     signing-in-to-aws
aws-blocks                  aws-security
aws-cdk                     aws-serverless
aws-cloudformation          aws-messaging-and-streaming
aws-compute                 aws-networking
aws-containers              aws-database
aws-deployment              aws-iam
```

Regla: si una tarea parece pedir AWS, **no cargues esas skills**. Pregunta antes
de asumir que el proyecto necesita AWS.

La política se aplica en `.pi/APPEND_SYSTEM.md` (instrucciones del proyecto) y se
repite en `skills/pvsyst-mcp/SKILL.md`.

## MCP instalado: `pvsyst`

El servidor MCP de este repositorio está registrado a nivel de proyecto en
`.pi/mcp.json`, así que cualquier sesión de Pi abierta en este directorio lo tiene
disponible como `mcp__pvsyst__*` sin configuración extra.

- Binario: `C:\pvsyst-mcp\pvsyst-mcp.exe` (ruta sin espacios, ver `scripts/install.ps1`).
- Workspace: `C:\Users\Lenovo\PVsyst8.1_Data`.
- PVsyst: `C:\Program Files\PVsyst8.1.8\` (`PVsystCLI.exe`, `PVsyst.exe`).

## Estructura

```
.agents/
├── README.md                      # este archivo
└── skills/
    └── pvsyst-mcp/
        ├── SKILL.md               # cuándo y cómo usar el MCP (se autodescubre)
        └── references/
            └── instalacion.md     # build, install y verificación
```

`.agents/skills/` es la ubicación estándar de Agent Skills: Pi la descubre desde el
directorio de trabajo hacia arriba, sin tocar la configuración del usuario.

## Verificar

```powershell
pi mcp list          # el servidor pvsyst debe aparecer conectado
```

Dentro de una sesión: `/mcp` para inspeccionar, `/reload` tras cambiar archivos de
`.agents/` o `.pi/`.
