# Lista de fiabilidad

Trampas de PVsyst. Cada ítem tiene un **disparador** visible en un diff, una **comprobación** y su **origen** (medición o incidente). Antes de cada commit se ejecutan solo los ítems cuyo disparador coincide con el diff.

Un ítem nuevo nace de un error que llegó a `main` o de uno visto en dos revisiones, y cita su origen. Un ítem que ya no dispara o que otro cubre se borra al final de cada fase.

## Formato

```
### R-NN título corto
- Disparador: qué cambia en el diff.
- Comprobación: qué se ejecuta o se mira.
- Origen: medición o incidente, con fecha y versión de PVsyst.
```

## Ítems

### R-01 Flag «predefinido» de pvSubArray
- Disparador: el diff escribe `PMismMPP` u otro valor de pérdidas detalladas en un `.VC`.
- Comprobación: el bit `0x00040000` de `pvSubArray.Flags` queda limpio, o la herramienta avisa de que PVsyst ignora el valor. Releer el `.VC` y confirmar el PDF.
- Origen: medido el 2026-11-01 en 8.1.8; con el bit puesto, el PDF seguía mostrando 2,00 % con `PMismMPP=4.00`. Verificar si sigue vigente antes de aplicarlo.

### R-02 Guardado asíncrono de variantes
- Disparador: el diff lee un `.VC*` justo después de un guardado por GUI.
- Comprobación: espera a que el fichero cambie y dos lecturas iguales (`esperar_guardado`), sin `sleep` fijo.
- Origen: medido el 2026-11-09 en 8.1.8; el diálogo cierra antes de que PVsyst termine de escribir y la lectura dio «Archivo truncado».

### R-03 Proyecto en la línea de comandos
- Disparador: el diff lanza PVsyst con la ruta de un proyecto en argv.
- Comprobación: la carga pasa por «Cargar proyecto» (`TProjectLoadForm`) y se confirma que `TProjectForm` está abierto.
- Origen: medido el 2026-11-09 en 8.1.8; PVsyst ignora el proyecto en argv y solo abre `THomeForm`.

### R-04 Salida estándar
- Disparador: el diff añade o cambia un `print`, un `log` o un subproceso que hereda stdout.
- Comprobación: `rg "println!|eprintln!|dbg!" src` vacío y un arranque del servidor sin bytes extra en stdout.
- Origen: restricción del producto (`agents.md` sección 0).

### R-05 Cerrar un diálogo ajeno
- Disparador: el diff pulsa un botón de un diálogo modal que no abrió el propio flujo.
- Comprobación: la decisión sale de una función pura sobre la clase del diálogo, los rótulos de sus botones y el idioma; solo se pulsa el botón que esa función nombra, cualquier otro diálogo se devuelve como hoy, y la respuesta dice que se cerró en `advertencias`.
- Origen: informe de pruebas GUI visual del 2026-11-17 (§C.4): en modo DEMO cada acción moría en `TDEMOForm` («Compra | Activar | Cerrar | Ayuda») y el aviso había que cerrarlo a mano.
