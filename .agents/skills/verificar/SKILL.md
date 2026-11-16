---
name: verificar
description: Verifica un cambio de pvsyst-mcp ejecutándolo como lo ejecuta PVsyst antes de hacer commit. Úsala al terminar cualquier unidad que cambie lo que PVsyst observa (archivos, CLI, GUI) y antes de escribir el trailer `Verified:`.
---

# Verificar un cambio

Sube por la escalera solo hasta donde el cambio lo necesita. Cada peldaño gasta más que el anterior.

1. **Comprobación acotada** (`agents.md` sección 8): `cargo check`, `cargo test --lib <modulo>::`, `cargo test --test <nombre>`.
2. **Fixtures herméticos**: `tests/fixtures/workspace` y `fake_pvsystcli`. Prueba el contrato sin gastar ejecuciones de prueba.
3. **`dry_run`**: acciones que lo implementan (`simular ejecutar`, `sitio crear`, `disenar_sistema`, `gui_actuar escena`…). Informa `costo_estimado`.
4. **Una ejecución real**, solo si lo anterior no puede probarlo: un archivo que PVsyst abre, una corrida de CLI o una ventana de GUI. Cuenta las ejecuciones de prueba gastadas.

## Antes de commit

- Lee `docs/checklist_fiabilidad.md` y ejecuta los ítems cuyo disparador coincide con el diff.
- Si tocaste `format/`: round-trip, idempotencia y, si reescribiste algo, comparación byte a byte contra la versión anterior sobre todos los fixtures.
- Si el cambio llega a la GUI: un solo agente a la vez, lanza PVsyst una vez y cierra `TDEMOForm` si aparece.
- Un test inestable no se reintenta ni se duerme: causa raíz o borrarlo con motivo.
- Tres rondas fallidas sobre la misma comprobación: detente y reporta qué se probó y por qué falló.

## Sello

Termina el mensaje de commit con una línea:

```
Verified: scoped, fake-cli, checklist R-01 R-02
```

o `Verified: none (motivo)`. Nombres válidos: `scoped`, `gate`, `fake-cli`, `dry-run`, `live xN`, `checklist <ids>`.
