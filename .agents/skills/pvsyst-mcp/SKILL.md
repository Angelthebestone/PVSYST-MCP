---
name: pvsyst-mcp
description: Opera PVsyst 8.1.8 desde este servidor MCP (proyectos, sitios, meteo, simulación, pérdidas, escena 3D, evaluación económica). Úsala para cualquier tarea sobre PVsyst, energía fotovoltaica simulada, o para instalar/verificar el MCP pvsyst. No aplica a servicios de nube.
---

# PVSYST MCP

Servidor MCP en Rust que opera **PVsyst 8.1.8** manipulando únicamente sus
archivos de datos. No reimplementa el motor de simulación: los resultados salen
de PVsyst o no salen.

## Cuándo usar esta skill

- Crear, listar, leer o modificar proyectos y variantes de PVsyst.
- Sitios, horizonte, meteorología (incluido Meteonorm/MEF), series horarias.
- Dimensionado, pérdidas, sombreado, escena 3D, seguidores.
- Lanzar simulaciones y leer sus resultados.
- Evaluación económica, divisas, plantillas de variables.
- Instalar, reconstruir o verificar el MCP `pvsyst` (ver `references/instalacion.md`).

## Regla de oro

Los resultados deben venir de PVsyst. `PVSYST_MCP_APROXIMACIONES` está **desactivado**
por defecto: no lo actives para "rellenar" un dato que falta. Si una tool falla,
reporta el error; no inventes cifras de producción, PR ni pérdidas.

## Cómo trabajar

1. Usa las tools `mcp__pvsyst__*` en lugar de editar archivos `.PRJ`/`.VC*` a mano.
   El servidor valida rutas contra el workspace y respeta el formato de PVsyst.
2. Antes de una operación destructiva, lista el estado actual
   (`listar_proyectos`, lectura del proyecto) y confirma con el usuario.
3. Para dudas de formato o de CLI, lee los recursos que expone el servidor:
   `pvsyst://recetas`, `pvsyst://cobertura`, `pvsyst://formato`, `pvsyst://cli`,
   `pvsyst://gui`. Son la fuente verificada; no improvises el formato.
4. Las simulaciones son lentas: informan progreso y aceptan cancelación. No
   reintentes en bucle ni asumas fallo por timeout.
5. Nunca modifiques archivos dentro de `C:\Program Files\PVsyst8.1.8\`: es la
   instalación del programa, no datos del usuario.

## Alcance del proyecto

Este proyecto no tiene nada que ver con AWS. Las skills `aws-*`,
`amazon-bedrock`, `launch-with-aws`, `signing-in-to-aws` y
`setting-up-cloudwatch-observability` están **fuera de alcance**: no las cargues
ni las apliques aquí.

## Referencias

- `references/instalacion.md` — build, instalación y verificación del MCP.
- `README.md` (raíz del repo) — uso, tools y variables de entorno.
- `PVSYST_CLI_REFERENCE.md` — referencia del CLI de PVsyst.
