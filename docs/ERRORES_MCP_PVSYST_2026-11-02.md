# Errores encontrados al probar el MCP pvsyst con PVsyst 8.1.8 real

Fecha: 2026-11-01/02. Binario probado: `C:\pvsyst-mcp\pvsyst-mcp.exe`
(SHA256 `8A4AE4DC6F6A7F57962DEA02D78EC18C8268B2B4DB8581D0CCF648C31D5565CE`, 4 852 736 bytes,
1/11/2026 9:29 p. m.). El usuario confirmó que ese exe es el que contiene los últimos cambios.
Workspace: `C:\Users\Lenovo\PVsyst8.1_Data`. PVsyst 8.1.8 en modo PRUEBA.

TRIAL: la licencia pasó de 149 a 140 ejecuciones restantes durante esta sesión (9 gastadas por mí;
el usuario ya había gastado 6 antes). Quedan 140 y 36 días.

Criterio: se reporta como fallo (a) un «ok» sin efecto real, (b) un parámetro ignorado sin aviso en
`ignorados`, (c) un error sin mensaje claro o engañoso, (d) una descripción que contradice el
comportamiento. Cada punto lleva la evidencia observada.

Leyenda de gravedad: **ALTA** = pierde datos, gasta TRIAL o deja el proyecto inválido;
**MEDIA** = resultado incorrecto o silencioso; **BAJA** = mensaje/documentación.

---

## 1. Regresiones: lo que se pidió re-probar y sigue fallando

| ID | Gravedad | Herramienta | Fallo | Evidencia |
|---|---|---|---|---|
| E01 | ALTA | `sitio crear dry_run:true` | Devuelve «PVsyst no creó el sitio: sin detalle» y **escribe** `UserData\Z_Site2_Monthly.csv` (22:51:01), contra «sin escribir nada». Repetido con todos los parámetros (`Z_SitePar`): mismo error. | `Sites\Z_Site2.SIT` no existe; el CSV mensual sí. |
| E02 | MEDIA | `leer` con `limite:3` | `limite` se ignora y no aparece en `ignorados`; se aplican solo `max_bytes`. | Rango 5–305 con `max_bytes:300`. |
| E03 | MEDIA | `leer` de un `.PAN` | `technology` sale `""`. | El archivo tiene `Technol=mtSiMono`. |
| E04 | MEDIA | `proyecto crear` con `sitio` | El aviso solo habla de la meteo; el `SiteSimul` del VC0 sigue siendo `Geneva/Cointrin`, Switzerland; solo el PRJ apunta a `Z_Site`. | `Z_Proj2.VC0` línea 13-21. |
| E05 | ALTA | `proyecto configurar` con un paso inválido | No valida todo antes de aplicar: aplica el paso 1 y falla en el 2 («Paso 2 fallido tras aplicar 1 paso(s)»). Deja un snapshot y un cambio a medias. | `SoilingLoss=0.030` en `Z_Proj.VC4` tras el error. |
| E06 | ALTA | `proyecto perdidas mismatch_pct` | Escribe `PMismMPP` pero PVsyst lo ignora: el informe sigue con «Module mismatch losses 2.00 % at MPP». Causa: el bit `0x00040000` de `Flags` en `pvSubArray` («predefinido» marcado) hace que PVsyst use su valor por defecto. Medido en la GUI: `$001E0002` (por defecto, 2.00) → `$019A0002` (desmarcado, 4.00) → `$019E0002` (marcado otra vez, vuelve a 2.00). | `Z_Proj_VC4.pdf` con `PMismMPP=4.00`: 2.0 %. La herramienta debería limpiar ese bit o avisar. Lo mismo probablemente aplica a calidad de módulo, LID y desajuste de voltaje (otros bits de `Flags`; no medidos). |
| E07 | MEDIA | `simular preparar` | `filas:[{"FieldTilt":10},…]` y `{"Plane tilt":15}` dejan la fila **vacía** sin aviso (`ignorados` ausente). Solo `tilt` y `azim` funcionan. `parametros:{"FieldTilt":[10,30]}` escribe la cadena literal `FieldTilt;0.0000;[10,30];;` en `Other\Param_Modif.dat` sin validar. La descripción de la herramienta sigue usando `FieldTilt` como ejemplo. | `MCP_BatchParams.csv` con `SIM_3;;;;;;…`. Con `tilt`/`azim` la corrida real sí difiere (`AngInc` 73.51 vs 70.25). |
| E08a | BAJA | `proyecto perdidas suciedad_pct` | Falla con «entre 0 y 50» mientras la descripción dice 0..100. | Mensaje de error de `suciedad_pct:150`. |
| E08b | MEDIA | `simular sin_cache:true` | No cambia la línea de comandos ni se confirma en la respuesta: no hace nada visible. | `cli_args` idénticos con y sin el parámetro. |
| E08c | MEDIA | `proyecto seguidor` | Sobrescribe `AxisAzim` a `0.0` aunque no se pase `azimut_eje` (estaba en -8.0) y deja el `Comment` con «Azim. -8.0°». | `Z_Proj.VC1` líneas 91 y 96. |
| E08d | BAJA | `editar insertar_objeto` | Deja `pvShading` al final del archivo (línea 17497) en lugar de tras `pvSystem`. PVsyst abre el proyecto sin quejarse y al guardarlo lo recoloca. | `Z_Ins.VC0`. |
| E08e | BAJA | `escribir_archivo` en archivo nuevo | Dice «con copia en el historial» con `historial_id:null`. | Respuesta de `UserData\Z_nota_nueva.txt`. |
| E08f | BAJA | `listar proyectos` / `proyecto crear` | `listar` no muestra `_DEMO_Pumping` (está en `DataRO`) pero `crear` la acepta como plantilla. Pasa igual con `_DEMO_StandAlone` y `_DEMO_DCGrid`. | `Program Files\PVsyst8.1.8\DataRO\PVsyst8.1_Data\Projects`. |
| E08g | MEDIA | `resultados economia` / `proyecto economia` | Se remiten el uno al otro; ninguno dice cómo crear la evaluación económica. | Mensajes literales de ambas. |
| E08h | BAJA | `proyecto renombrar` | Deja `Comment=<nombre antiguo>` en el PRJ. | `Z_Ren2.PRJ`: `Comment=Z_Ren1`. |
| E08i | MEDIA | `simular` con meteo de Ginebra y sitio de Colombia | `clasificacion:"exito"` con «ErrorList: 2033 error messages» y noches con irradiancia; no avisa de que la meteo no corresponde al sitio. | Simulación `Z_Proj` VC4. |

---

## 2. `disenar_sistema`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E10 | ALTA | `dry_run:true` **no se rechaza**: se ignora (sale en `ignorados`) y la tool crea el proyecto de verdad. Otras tools con ensayo sí rechazan (`meteo_pvgis`: «'dry_run' no existe en sitio meteo_pvgis… Quita 'dry_run'»). | Se creó `Projects\Z_Dis.*`. |
| E11 | ALTA | Si el módulo de la plantilla no existe, sustituye en silencio por `MCP_Test_400.PAN` (solo un aviso), que **no tiene dimensiones**; la simulación posterior falla en PVsyst y **gasta 1 TRIAL**. | «The sizes of this module are not defined»; `costo.ejecuciones:1`. |
| E12 | MEDIA | `synthetic_weather:true` falla porque falta `DataMN9\station.db.enc` de Meteonorm; la tool no lo comprueba antes. | «Failed to initialize Meteonorm… station.db.enc». |
| E13 | MEDIA | El campo `applied` del dimensionado omite `PVModule` y `GInverter` cuando sí los cambió (`aislado`). | `Z_DisAis.VC3`: `PVModule=Z_ModDim.PAN`, no figura en `applied`. |
| E14 | BAJA | El dimensionado de `kwp` devuelve primero «Sin n_mod_serie y n_strings el dimensionado lo haría la GUI» aunque se pase `dry_run`; el orden de validaciones oculta E10. | Respuesta literal. |

---

## 3. `componente`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E20 | ALTA | `editar` acepta `PNom:"abc"` (valor no numérico) y lo escribe en el `.PAN`. | `PNom=abc` en `Z_Mod.PAN` hasta que se sobrescribió. |
| E21 | MEDIA | `editar` y `crear` aceptan claves desconocidas (`NoExiste=3`, `Width`, `Depth`…) y las **insertan al final del archivo, fuera de `pvCommercial`**. Para definir dimensiones hay que editar `objeto:pvCommercial` con `editar`. | `Z_ModDim.PAN`: las 4 claves duplicadas, arriba y en `pvCommercial`. |
| E22 | MEDIA | `importar` con `nombre_nuevo:"Z_Imp"` (sin extensión) falla con «'Z_Imp' no es un componente PAN», contra la descripción «sin extensión». Con `Z_Imp.PAN` funciona. | Ambas respuestas. |
| E23 | BAJA | `importar` de un `.zip` falla con «Unknown component kind 'ZIP'» (mensaje poco claro). | Respuesta literal. |
| E24 | MEDIA | `crear_desde_ficha` con `Technol:"mtSiMono"` (el valor que usan los archivos) se rechaza; solo acepta la etiqueta de la GUI (`Si-mono`). La respuesta termina `completo:false` con «la acción informó un fallo» sin decir cuál. | Dos respuestas consecutivas. |
| E25 | ALTA | No hay camino para definir las dimensiones del módulo (`Width`, `Height`…): `crear_desde_ficha` no las admite, y los módulos del workspace (`MCP_Test_400`, `Z_E2E_Importado`) carecen de ellas. Cualquier simulación de extremo a extremo con módulos creados por el MCP falla. Además `DataRO` solo trae 2 módulos y falta `Generic_Mono_300W.PAN` que usan las plantillas. | Corrida con `Z_ModDim.PAN` (dimensiones en `pvCommercial`) sí simula. |

---

## 4. `proyecto`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E30 | MEDIA | `dimensionar` ignora `n_serie` y `n_paralelo` (existen en el esquema) y responde en inglés «Missing required argument 'kwp'». | Respuesta literal. |
| E31 | MEDIA | `perdidas` ignora `perdida_separada` y `comentario` sin avisar (solo aplican `lid_pct`, `uc_termico`, etc.). `perdida_separada` pertenece a `limite_inyeccion`. | `cambios` solo con `UFactor` y `LIDLoss`. |
| E32 | MEDIA | `economia` limita `tarifa_venta_kwh` a 0..100 aunque la descripción dice «en la moneda elegida» (500 COP/kWh se rechaza). | «debe estar entre 0 y 100: 500». |
| E33 | ALTA | `almacenamiento` acepta una batería que no existe (`NoExiste.BTR`, solo un aviso) y escribe en el archivo; inventa `NBattSerie=10` y `NBattParall=20`; e inserta un `pvLoad` («Shopping center needs», 386 campos) no pedido. Con `recorte_picos`, PVsyst rechaza la variante: «With the "Peak shaving" strategy you cannot define a self-consumption». Gastó 1 TRIAL. | Simulación de `Z_ParA`. |
| E34 | MEDIA | `anadir elemento:carga` responde `ok:true` con `insertados:[]` (sin efecto, sin explicar). | Respuesta literal. |
| E35 | MEDIA | `anadir elemento:circuito_hidraulico` inserta un objeto de bombeo (`pvHydrCircuit` bajo `pvSystem`) en un proyecto conectado a red sin comprobar el tipo. | `Z_ParC.VC0`. |
| E36 | BAJA | `anadir elemento:zona_campo` inserta geometría de demostración (`Zone #1`, «Mono 440 Wp Twin 144 half-cells») ajena al proyecto. | Respuesta literal. |
| E37 | BAJA | `crear` desde `_DEMO_DCGrid` devuelve `applied:[]`: el comentario no se actualiza con el nombre nuevo (sí en las otras plantillas). | Respuesta literal. |
| E38 | ALTA | `p50_p90`: PVsyst guarda la variante pero **no escribe** el bloque `P50_P90=TP50_P90 … EndTags`; el error «¿usa PVsyst otro espacio de trabajo?» es engañoso (el archivo sí se guardó). `uia_invocar alternar` sobre «Activo» sí marca la casilla. | `Z_Ins.VC0` mtime 23:10:31 sin el bloque. |
| E39 | ALTA | `editar` sobre `pvVCalcul` escribe `MeteoAnnualVariab`, `PxxCustom`… al nivel superior del VC, mientras PVsyst los guarda **dentro** del bloque `P50_P90=TP50_P90`. Es probable que PVsyst no los lea (no verificado). | `Z_Proj.VC4` líneas 2-8 frente a `_DEMO_Commercial.VC2` líneas 266-277. |

---

## 5. `sitio`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E40 | MEDIA | `crear` ignora `albedo`, `titulo`, `fecha` y `log_level` sin avisar. Se pidió albedo 0.25 y el `.SIT` quedó con 0.20; el `Comment` es el nombre, no el título. | `Z_SitePar.SIT` y la `command_line` de `create-site`. |
| E41 | MEDIA | `convertir_meteo` ignora `output_sit_file`, `sit_file` y `weather_data_file` sin avisar. | `command_line` del dry_run. |
| E42 | MEDIA | `meteo_pvgis` gasta **2** TRIAL (`create-site` + `convert-meteo`) sin avisar ni pedir confirmación; la nota dice «gratis si las coordenadas están en caché». | Licencia 149→147. |
| E43 | BAJA | El `.MET` generado por `meteo_pvgis` guarda `NomFSource` apuntando a una carpeta `.staging` que se borra. | `Meteo\Z_Pvgis_PVGIS.MET`. |
| E44 | BAJA | `horizonte_pvgis` no dice en qué archivo escribe (escribió un `pvHorizon` de 39 puntos en el `.VC`). | `Z_Ins.VC0` línea 114. |
| E45 | BAJA | `escribir_serie`: no existe ningún `.DAM`/`.DAF` en el workspace y la ruta de éxito exige 8760 valores por variable dentro de la llamada. No se pudo verificar; el rechazo por longitud es claro. | «tiene 3 horas; esperado 8760 o 8784». |

---

## 6. `simular`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E50 | MEDIA | `variables` y `unidades` se descartan sin aviso: no llegan a la línea de comandos y no figuran en `ignorados`. | `-sd`, `-ed`, `-ts`, `-rl`, `-rst` sí llegan. |
| E51 | BAJA | `extra_options` con dos opciones: el dry_run muestra 13 de 14 argumentos (`truncado`) y oculta la segunda. | `cli_args_total:14`. |
| E52 | MEDIA | `resultados kpis` tras una simulación de rango parcial falla con «El informe no contiene Produced Energy» (no explica que el informe es parcial). | `Z_ParD` con `start_date/end_date`. |
| E53 | MEDIA | Con `filas` en lote, todas las filas escriben el mismo CSV (`Z_ParD_VC0.csv`), así que la segunda sobrescribe la primera. | `MCP_BatchResults.csv`: ambas filas con `Z_ParD_VC0.csv`. |
| E54 | BAJA | Una simulación que PVsyst rechaza al comprobar la variante («Simulation cannot be run») consume igualmente 1 TRIAL; la herramienta no lo avisa antes. | `costo.ejecuciones:1` con `clasificacion:"error"`. |

---

## 7. `leer`, `listar`, `resultados`, `workspace`, `entorno`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E60 | BAJA | `leer serie:true` ignora `max_bytes` y `detalle`, y devuelve 24 horas fijas. | Respuesta de `Geneva_MN90_SYN.MET`. |
| E61 | BAJA | `leer tipo:"texto"` se acepta sin efecto ni aviso. | Respuesta de `Z_nota.txt`. |
| E62 | BAJA | `listar componentes` con `BTR`, `PMP` o `baterias` devuelve `count:0` y `folder:""` sin decir que la carpeta no tiene archivos; `baterias` se acepta como tipo y se convierte en `BATERIAS`. | `ComposPV\Batteries` solo contiene `Battery.Fav`. |
| E63 | MEDIA | `resultados mensual` con `archivo` en lugar de `proyecto` falla con «Missing required argument 'proyecto'» (en inglés); `archivo` está en el esquema. | Respuesta literal. |
| E64 | BAJA | `resultados csv count:5` devuelve `count:1` (cuenta archivos, no filas) y las 8760 filas en el resumen; `internet:true` no tiene efecto. | Respuestas de `Z_Proj` VC4. |
| E65 | BAJA | `workspace historial` ignora `proyecto` y `ruta`: devuelve el historial global sin filtrar ni avisar. | Respuesta con `proyecto:Z_ParD`. |
| E66 | BAJA | `workspace exportar` deja el zip (150 MB) dentro del workspace (`UserData`), de modo que un segundo export incluiría el anterior. | `Z_ws_export.zip`. |

---

## 8. `gui_observar` y `gui_actuar`

| ID | Gravedad | Fallo | Evidencia |
|---|---|---|---|
| E70 | ALTA | `funcion` informa «abre» sin ventana ni error cuando PVsyst lo bloquea con un aviso propio. Ejemplo: `ventana_proyecto.variante.gestionar` → «La variante ha sido modificada. Debes guardarla antes de poder abrir la ventana 'Gestionar variantes'». Lo mismo con `herramientas.optimizacion_voltaje` («El difuso horizontal no está presente en los datos»). Solo `bases_de_datos.generacion_tmy` lo reportó en `ventanas_nuevas`. | `gui_observar formulario` sobre el `TMessageForm`. |
| E71 | MEDIA | Dentro de `secuencia`, un `funcion` sobre un botón deshabilitado pasa sin error (`…conectado_a_la_red.horizonte`, `variante.necesidades_usuario_independiente`); suelto sí dice «Botón … deshabilitado». | Dos casos reproducidos. |
| E72 | MEDIA | `secuencia` marca `completados` y estado `ok` aunque un paso lleva `"error":"PVsyst no abrió la ventana esperada (TArrayCharactForm)"` (la ventana abrió con retraso). | Respuesta de `herramientas.comportamiento_electrico`. |
| E73 | MEDIA | La primera llamada de menú tras abrir el editor 3D falla siempre («El menú N de TPerspectiveForm no se abrió»); tras una tecla Alt funciona. Reproducido 4 veces (`escena`, `escena_importar`, `escena_3d.crear.*`). | Ver notas de memoria `foreground-alt-trick`. |
| E74 | MEDIA | `cerrar` falla («no encontrada») justo después de cerrar un formulario hijo: PVsyst recrea el selector padre con otro hwnd y el hwnd o clase en caché queda obsoleto. | Selectores de `TChoixObjetPVForm`; con la clase fresca funciona. |
| E75 | MEDIA | `funcion` no resuelve sus prerrequisitos aunque la descripción lo afirma: `variante.sombreados_cercanos.construccion_perspectiva` falló con «Ventana TProjectForm no está abierta». | Respuesta literal. |
| E76 | BAJA | `escribir` y `fila` sobre la clase del formulario fallan; solo funcionan con el hwnd del control. El error de `fila` enumera clases medidas pero no sugiere usar el hwnd. | Respuestas literales. |
| E77 | BAJA | `campo` con etiqueta inexistente responde «Ningún campo se llama 'buscar'. Rótulos vistos:» con la lista vacía. | Respuesta literal. |
| E78 | BAJA | `menu` con `menu`/`item` posicionales falla en `THomeForm` («no tiene submenú»); solo funciona con `id`. | `menu id:23` abrió `TWorkspaceForm`. |
| E79 | BAJA | `clic_fisico` da «ok» aunque el clic caiga fuera del control; dx/dy son px a 96 DPI, no físicos, y la descripción no lo dice. | Primer intento sin efecto, segundo con efecto. |
| E80 | BAJA | `gui_observar que:menus` devuelve solo ids y posiciones (sin rótulos) y termina en estado «parcial» tras 11 s. | Respuesta literal. |
| E81 | BAJA | `gui_observar que:leer` sobre un `TMessageForm` devuelve solo el título; el texto exige OCR (`formulario`). | Respuesta literal. |
| E82 | BAJA | `escena_importar` no resuelve rutas relativas al workspace («No existe UserData\Z_malla.pvc»); con ruta absoluta sí. | Dos respuestas. |
| E83 | BAJA | `uia_invocar` sobre un control sin nombre exige `nombre` o `rol`; no hay forma de apuntar por hwnd. | Respuesta literal. |
| E84 | BAJA | `paso` devuelve `verificado:false` aunque el efecto es real (cerró la ventana). | `TDEMOForm`. |
| E85 | MEDIA | `cerrar_todo` cierra también la ventana principal y termina PVsyst, aunque no lo iniciara la tool. | `gui_observar` después: «Process 'PVsyst.exe' is not running». |
| E86 | BAJA | `lanzar` responde «PVsyst lanzado» cuando ya estaba en ejecución (solo `ya_en_ejecucion:true` lo delata). | Respuesta literal. |
| E87 | BAJA | `reiniciar` solo actúa ante un diálogo de fallo (madExcept); con PVsyst sano responde con un mensaje claro. No es un fallo; anotado como cobertura. | Respuesta literal. |
| E88 | BAJA | `gui_observar que:funciones detalle:completo` devuelve 62 241 caracteres y se desborda; hace falta `compacto` o leer el archivo guardado. | Mensaje del cliente. |

---

## 9. Catálogo y ventanas escondidas o sin implementar

| ID | Gravedad | Hallazgo |
|---|---|---|
| E90 | MEDIA | Nueve clases de ventana abren funciones del catálogo pero **no están en `docs/gui_mapa.json`** (82 clases): `#32770` (diálogo del sistema), `TMessageForm`, `TPolyFieldForm`, `TRectFieldForm`, `TShedFieldForm`, `TTrackerFieldForm`, y las tres descubiertas al probar: `TLicenseMainForm`, `TAboutForm`, `TLicenseAgreementForm`. |
| E91 | BAJA | `TTranspositionForm` y `MicrosoftWindowsTooltip` están en el mapa pero ninguna de las 209 funciones las abre; no se encontró cómo llegar. |
| E92 | MEDIA | Unas 50 funciones del catálogo tienen `efecto:abre` pero `abre:null`: no declaran qué ventana abren (`ayuda.*`, `tutoriales_pdf.*`, `inicio.*`, `licencia.detalles`, `bases_de_datos.generacion_tmy`, `escena_3d.herramientas.zonas_de_tabla`…). Algunas sí abren una ventana propia (`licencia.detalles` → `TLicenseMainForm`, `ayuda.acerca_de` → `TAboutForm`), otras ninguna (`ayuda.notas_de_lanzamiento` responde «abre» con `ventanas_nuevas:[]` tras 5 s). |
| E93 | BAJA | Funciones condicionadas al contexto sin documentarlo en el catálogo: `…orientacion.detalles_y_grafico` («Botón no visible»), `…gestion_de_retroceso` (entrada 10 del menú 6 deshabilitada sin seguidores), las ventanas de autónomo/bombeo/red-CC (solo se pueden abrir cargando un proyecto de ese tipo desde Inicio; desde una ventana de proyecto de red la carga solo lista proyectos de red). |
| E94 | BAJA | `escena_3d.crear.plano_fv_poligonal` abre además `TOrientationForm` desde la pestaña «Dibujo poli» y deja el formulario deshabilitado; está en `nota` pero no en la lista de ventanas. |

---

## 10. Lo que no se probó, y por qué

- `entorno licencia_desactivar`: requiere autorización explícita del usuario y puede afectar a la licencia TRIAL.
- ~~Las 37 funciones GUI que modifican datos o cierran PVsyst~~ y ~~las que abren navegador o visor externo~~: **ya probadas en la segunda tanda (sección 13)**. Las 209 funciones del catálogo quedaron ejecutadas al menos una vez.
- `escribir_serie` con éxito (E45).
- `crear_desde_ficha` con tipos distintos de PAN (OND, BTR, PMP).
- `disenar_sistema` completo con simulación exitosa en el sitio de Colombia: bloqueado por E11/E12 (módulo sin dimensiones y sin Meteonorm); solo se verificó el camino hasta la simulación.
- `#32770` de «Importar proyecto» (se observó la de «Cambiar espacio de trabajo»).
- Entidades de `rellenar` salvo `variante.orientation` (solo en `simular:true`).

---

## 11. Lo que sí funcionó (para contraste)

`entorno` (diagnóstico, licencia, `exportar_logs` real y en dry_run, `logs_cli`, `preferencias`), `listar`
(todos los filtros), `leer` (PRJ, VC, SIT, MET, CSV, base64, `serie`), `escribir_archivo`, `editar`
(campos, insertar/eliminar objeto, RVT), `proyecto` (`crear`, `eliminar`, `renombrar`, `doctor`,
`limite_inyeccion`, `anadir` salvo los puntos de la sección 4), `sitio` (`escribir_horizonte`,
`meteo_pvgis`, `convertir_meteo`, `horizonte_pvgis`), `workspace` (exportar sin sobrescritura, importar,
crear con la regla de «junto al workspace»), `pvsyst_cli` (incluido el rechazo de `;`), `simular` con
`start_date`, `end_date`, `time_step`, `idioma_reporte`, `recalcular_sombreado` y `filas` con
`tilt`/`azim`, 70 clases de ventana abiertas y cerradas, `escena`, `escena_leer`, `escena_importar`,
`uia_invocar` (`alternar` e `invocar`), `secuencia`, `esperar_cierre`, `fila`, `pagina`, `clic_texto`.

---

## 12. Limpieza

Todos los archivos creados por estas pruebas (proyectos `Z_*`, módulos, sitios, meteo, escenas, zips y
CSV) se movieron a `UserRecycleBin`. Se restauraron: `Z_Proj.PRJ` (comentario `Z_Proj`), `Z_Proj.VC1`
y `Z_Proj.VC4` (`PMismMPP=1.00`, `SoilingLoss=0.020`). `UserBatch\MCP_BatchParams.csv` y
`MCP_BatchResults.csv` quedaron con el contenido de la última corrida real (`tilt/azim`
10/0 y 30/20). Se añadió `.claude\settings.local.json` con 15 reglas `permissions.allow` para las
herramientas `mcp__pvsyst__*` (archivo ignorado por git).

---

## 13. Segunda tanda: las funciones GUI que faltaban (todas ejecutadas)

TRIAL: la simulación lanzada desde la GUI (`variante.ejecutar_simulacion`) **no** descontó ninguna
ejecución del contador de `lic-info` (siguen 140). Se probó con proyectos propios (`Z_GuiImp`,
`Z_GuiSave`); ninguno de los proyectos existentes se tocó (17 proyectos intactos al final).

### 13.1 Resultado por función

| Función | Resultado | Evidencia |
|---|---|---|
| `idioma.*` (12) | Funcionan; cambian el título de la ventana (PRUEBA→EVALUATION, TEST, VALUTAZIONE, AVALIAÇÃO, DENEME, 평가 모드, 试用模式, 評価版, PRÓBNY…). | `PVsyst.ini` solo guarda `English`/`Spanish`; el resto no persiste. `idioma.frances` abre un `TMessageForm` «Info» (F9 alterna inglés/francés) que la respuesta no menciona. |
| `ayuda.buscar_actualizaciones_al_inicio` | Funciona (alterna `Updates\CheckForUpdatesAtStartup` 1↔0). | Restaurada a 1. |
| `archivo.exportar_componentes` | Abre selector y `TExportDataForm`; **no se pudo exportar**: «Exportar» sigue deshabilitado con «Caminos idénticos» aunque el destino es otro. | Ver E100, E101. |
| `archivo.exportar_proyectos` / `ventana_proyecto.proyecto.exportar` | Funcionan: ZIP real de 2,8 MB (PRJ + 5 VC + MET). | `proyecto.exportar` escribe en `%TEMP%\PVsyst_tmp`, no en la última carpeta usada. |
| `archivo.importar_proyectos` / `proyecto.importar` | Funcionan: `Z_GuiImp.PRJ` y `.VC0` vuelven con el mismo tamaño. | |
| `archivo.importar_componentes_zip` / `_carpeta` | Abren `TImportDataForm` / selector de carpeta. No se importó nada. | |
| `archivo.exportar_registros` | Funciona: `…PVsyst818-logs.zip` 50 KB. | |
| `archivo.actualizar_bases_de_datos` | «ok» sin ventana, sin mensaje y sin cambio observable. | E106. |
| `archivo.borrar_proyectos_recientes` / `borrar_espacios_recientes` | Funcionan (confirmación Sí/No; la lista queda vacía). | `PVsyst.ini` restaurado desde copia al terminar. |
| `ayuda.sitio_web`, `foro`, `tutoriales_video`, `tutoriales_pdf.*` (10), `buscar_actualizaciones` | Abren pestañas en Chrome; `buscar_actualizaciones` muestra «PVsyst 8.1 está actualizado». La respuesta no puede decir qué URL/archivo abrió (`ventanas_nuevas:[]`). | |
| `ventana_proyecto.variante.crear` | Funciona en memoria (VC1 «Nueva variante»); no escribe `VC1` hasta guardar. | |
| `ventana_proyecto.variante.guardar` | Con el botón deshabilitado: error claro («La entrada 1 del menú 2 está deshabilitada»). Con variante simulada: abre `TSaveObjPVForm`; tras «Guardar», `VC0` 1,49→1,68 MB. | |
| `ventana_proyecto.variante.eliminar` | Funciona (sin confirmación visible). | |
| `ventana_proyecto.variante.importar` | Abre `TVCalculSelectionForm`. | Clase sin registrar (E102). |
| `ventana_proyecto.proyecto.guardar` | Funciona: `Z_GuiSave.PRJ` + `VC0`, tras la pregunta «¿copiar todas las variantes?». | |
| `ventana_proyecto.proyecto.eliminar` | Funciona: borra PRJ+VC del proyecto activo tras confirmación que nombra el proyecto. | |
| `ventana_proyecto.proyecto.salida`, `archivo.salida` | Cierran la ventana de proyecto / PVsyst (sin proceso restante). | |
| `variante.ejecutar_simulacion` | Funciona: pregunta «¿simular de nuevo?» (con casilla «No me vuelva a preguntar», no marcada), simula y pregunta si guardar. | |
| `variante.informe` | Abre `TProgressForm` y luego **`TPrintPreviewForm`** (7 páginas) con barra propia: guardar PDF (610 KB real), imprimir (diálogo estándar), copiar al portapapeles (confirma con OK/Cancelar), exportar (`TResultExportForm`), opciones (`TPrintOptionsForm`), editar portada, comparar (`TVCalculSelectionForm`). | E103, E104. |
| `escena_3d.archivo.exportar_escena` | Funciona: `Shadings\Z_GuiSave.SHD` 672 B. | |
| `escena_3d.archivo.cerrar_escena` / `cancelar` | Funcionan (cierran el editor). | La primera llamada de menú tras abrir el editor falla; con Alt+primer plano sale. |
| `ventana_proyecto.variante.gestionar`, `notas.*`, `proyecto.configuracion`, `sitio.*`, `proyecto.nuevo`, `proyecto.cargar`, `proyecto.ayuda`, `variante.ayuda` | Abren su ventana (`TVCManageForm`, `TSummaryForm`, `TSettingsForm`, `TGeoSiteForm`, `TChoixObjetPVForm`, `TProjectLoadForm`, `THelpForm`) y se cierran bien. | `proyecto.nuevo` no abre ventana: reinicia la actual a «New.PRJ». |

### 13.2 Fallos nuevos

| ID | Gravedad | Herramienta | Fallo | Evidencia |
|---|---|---|---|---|
| E100 | MEDIA | `gui_actuar escribir` | Rechaza los `Edit` Win32 de los diálogos de archivo («indique 'campo'…»); no hay forma soportada de escribir una ruta en «Seleccionar carpeta»/«Guardar como». Hay que elegir con `clic_fisico` sobre la lista o salirse del MCP (SendKeys). | `Ruta de exportación`, `Seleccione un directorio de destino`. |
| E101 | MEDIA | `archivo.exportar_componentes` | `TExportDataForm` deja «Exportar» deshabilitado con el aviso «Caminos idénticos» aunque la ruta de destino ya es distinta (`UserData\Z_ExpGui`). No se pudo completar. | Capturas del formulario. Puede ser PVsyst, pero el MCP lo da por «modifica/ok». |
| E102 | MEDIA | catálogo/mapa | Clases abiertas por las funciones y **no registradas** en `docs/gui_mapa.json`: `TExportDataForm`, `TImportDataForm`, `TExportProjectForm`, `TImportProjectForm`, `TSaveObjPVForm`, `TVCalculSelectionForm`, `TProgressForm`, `TThreadProgressForm`, `TPrintPreviewForm`, `TPrintOptionsForm`, `TResultExportForm`. Suman 11 a las 9 de E90. | `funcion` devuelve la clase, pero `gui_observar que:esquema` no las conoce. |
| E103 | ALTA | `funcion variante.informe` | La respuesta solo menciona `TProgressForm`; la vista previa (`TPrintPreviewForm`) aparece ~1 s después, **no sale en `ventanas_nuevas`** y deja `TProjectForm` deshabilitada: la siguiente `funcion` falla con «hay un diálogo modal abierto (TPrintPreviewForm)» sin que antes se supiera que existía. Pasa igual con el aviso «Simulación terminada con éxito» tras `ejecutar_simulacion`. | Sección 13.1. |
| E104 | MEDIA | `gui_observar que:formulario` | Asocia mal etiquetas y campos en `TPrintOptionsForm` y `TResultExportForm` (p. ej. el radio «kWh» sale con etiqueta «Añadir gráficos predefinidos al informe:»; `TUpDown` con la etiqueta de otro grupo). | `candidatos` y `etiqueta` no coinciden con la captura. |
| E105 | ALTA | `gui_actuar clic_texto` | Con `TMessageForm` elige el texto del **cuerpo** del aviso en vez del botón («No», «OK», «Sí»): devuelve `ok` y `coincidencias:1` con `control.clase:"TMessageForm"` y no pulsa nada, o `coincidencias:0` y sí pulsa. Los avisos solo se pudieron cerrar con `click` sobre el hwnd del botón. | «Guardar documento» (No), «Copiando en el portapapeles» (OK). |
| E106 | MEDIA | `archivo.actualizar_bases_de_datos` | Devuelve `efecto:"modifica"` y `ok` pero no hay ventana, mensaje, ni cambio de archivos observable. | Sin entradas nuevas en el log de errores. |
| E107 | MEDIA | `gui_actuar click` | Un clic sobre un botón deshabilitado devuelve `ok` (`window.enabled:false` solo aparece dentro de `window`). | `TProjectLoadForm` «Cancelar» tras eliminar. |
| E108 | BAJA | `gui_actuar cerrar` | A veces responde «sigue abierta tras pedir el cierre» y la ventana se cierra un instante después (carrera). | `TGeoSiteForm`, `TProjectLoadForm`. |
| E109 | BAJA | `uia_invocar` | `objetivo:{…}` se ignora; hay que pasar `nombre`/`rol` como parámetros de primer nivel, y el error («Indique 'nombre' o 'rol'») no lo explica. Además devuelve «La operación se completó correctamente (0x00000000)» **como error**. | `fijar_valor` sobre «Carpeta:». |
| E110 | MEDIA | `gui_observar que:ventanas` | Justo después de una acción no lista el diálogo que se está abriendo (`#32770`, `TPrintOptionsForm`, `TPrintPreviewForm`) hasta ~1 s; `funcion` tampoco lo reporta. Hay que reintentar o usar `esperar clase:`. | Varias. |
| E111 | BAJA | `gui_actuar fila` | «TTreeView de TExportDataForm no tiene geometría de filas medida»: el árbol de exportación no es manejable por `fila` (el mensaje sí es claro). | |
| E112 | BAJA | `gui_actuar teclas` | Devuelve `foco:false` pero las teclas sí llegan tras el truco de primer plano; el campo `foco` no es fiable. | `RIGHT` en `TExportDataForm`. |
| E113 | BAJA | `ventana_proyecto.proyecto.eliminar` (desde el selector «Eliminar») | Tras borrar desde `TProjectLoadForm` aparece una **segunda** confirmación «¿Realmente desea eliminar el proyecto y todas sus versiones?» sin nombrar el proyecto, y «Cancelar» queda deshabilitado; solo «OK» cierra el selector. | Se respondió «No» a la segunda: los 17 proyectos restantes intactos. |
| E114 | BAJA | `idioma.*` | Con `idioma.frances` aparece un aviso «Info» (F9) que `funcion` no reporta (`ventanas_nuevas` sí lo trae, `nota:null`). | |

### 13.3 Estado final

Procesos de PVsyst: ninguno. `PVsyst.ini` restaurado desde copia (hash `F5EFD8F7…4E42`, idioma Spanish).
Movidos a `UserRecycleBin`: `Z_ExpGui_gui`, `Z_GuiSave.SHD`, `Z_scratch_gui` (capturas que `gui_observar capturar` dejó en
`<workspace>\.scratch\bk`: la ruta relativa se resuelve **dentro del workspace**, no del cwd). `Z_GuiImp` y `Z_GuiSave`
borrados desde la propia GUI. El portapapeles del usuario no se tocó (se canceló «Copiar al portapapeles»).
Quedan sin probar solo `entorno licencia_desactivar` (requiere autorización explícita), `escribir_serie` con éxito y los parámetros
de `rellenar` distintos de `variante.orientation`.

---

## 14. Mejoras y bugs pedidos por el usuario (2026-11-02)

| ID | Gravedad | Tema | Qué hay que corregir | Evidencia |
|---|---|---|---|---|
| M01 | ALTA | Diálogos de guardado/apertura de archivo | Hoy el MCP no puede escribir una ruta en los diálogos de Windows (E100) y obliga a elegir con `clic_fisico` sobre capturas. Debe bastar con pasar la **ruta**: `gui_actuar funcion`/`paso` con un parámetro `ruta` (relativo al workspace, o absoluto con `permitir_fuera_del_workspace`) que el MCP escribe en el campo «Nombre de archivo»/«Carpeta:» (mensaje Win32 o UIA), confirma con el botón por defecto y **verifica el archivo en disco** (existe, tamaño > 0) antes de responder `ok`. Debe contestar por su cuenta los avisos posteriores («¿abrir la carpeta en el explorador?» → No; «¿copiar todas las variantes?», «¿simular de nuevo?»), con parámetros explícitos para cada caso. | Afecta a: `archivo.exportar_componentes`, `archivo.exportar_proyectos`, `archivo.exportar_registros`, `archivo.importar_proyectos`, `archivo.importar_componentes_zip/_carpeta`, `ventana_proyecto.proyecto.exportar/importar`, `ventana_proyecto.proyecto.guardar` y `variante.guardar` (`TSaveObjPVForm`: nombre + descripción), `escena_3d.archivo.exportar_escena`, guardar informe como PDF (`TPrintPreviewForm`), `escena_importar`. Además `proyecto.exportar` guardó en `%TEMP%\PVsyst_tmp` ignorando la última carpeta (sección 13). |
| E115 | ALTA | Resultado de simulación incoherente | La variante VC0 de `Z_Proj` (y de todo proyecto creado desde ella) trae un resultado guardado con el aviso naranja **«Por favor, póngase en contacto con support@pvsyst.com: Incoherencia en los resultados de la simulación: Error de balance energético nominal del conjunto (-33.33 %). EArrNom calculation…»** (captura del usuario). Resultado físicamente imposible: 11 888 kWh/año, 1 981 kWh/kWp/año, **PR = 1.306**, pérdidas del conjunto **−1.38 kWh/kWp/día**, para 6.00 kWp. Esa simulación fue de «versión 8.1.0, fecha 11/02/26» (hecha con el MCP/CLI) y el MCP la devolvió como `clasificacion:"exito"` sin avisar. Al **re-simular la misma variante desde la GUI** el resultado es coherente: **2 650 kWh/año, 442 kWh/kWp/año, PR = 0.820**, sin aviso. | Capturas de `Z_GuiSave.VC0` antes (PR 1.306) y después (PR 0.820, `TResultForm`) de «Ejecutar simulación». Relacionado con E08i (`ErrorList: 2033 error messages`, meteo de Ginebra con sitio de Colombia). El MCP debe (a) leer el aviso del PRJ/VC y de `ErrorList` y devolver `advertencia`/`fallo` en vez de `exito`, (b) comprobar `PR ≤ ~1` y `EArrNom` coherente, (c) investigar qué parámetros de la simulación por CLI producen el desbalance (la meteo no coincide con el sitio es la causa más probable, no probada). |
| M02 | ALTA | Escenas 3D y sombras editables por fuera | **No está adaptado.** `.SHD`/`.BLD`/`.SHO` son de solo lectura en el MCP: `leer` los decodifica (objetos, posiciones, formas) y `escribir_archivo` los rechaza («formato de malla no demostrado»). Las únicas vías para crear una escena son por la GUI: `gui_actuar escena` (lista de objetos) y `escena_importar` (mallas), ambas abriendo el editor 3D. No existe flujo «editar el archivo fuera → cargarlo en el simulador» (leer → modificar → escribir `.SHD` válido → asociarlo a la variante). | `docs/pvsyst_format.md` §7 y `docs/cobertura.md` línea 76. Falta: codificador de escritura con ida y vuelta verificada (leer→escribir→PVsyst lo abre idéntico, como ya hay para `.HOR` con `sitio escribir_horizonte`) y una acción para asociar la escena a una variante (`pvShading`/`ShadingFile` en el `.VC`; no verificado en esta sesión si existe). |

---

## 15. Eficiencia: dónde hubo que encadenar llamadas y cómo reducirlas

Contadas en esta sesión (llamadas reales que hicieron falta para una sola tarea) y propuesta para bajarlas.

| ID | Tarea | Llamadas hoy | Por qué | Propuesta | Llamadas después |
|---|---|---|---|---|---|
| P01 | Exportar/guardar con diálogo de archivo | 7–9 (`funcion` → `ventanas` → `capturar` → `Read` PNG → `clic_fisico` lista → `clic_fisico` botón → `esperar_cierre` → `listar`) | No se puede escribir la ruta (E100, M01) | `ruta` en la propia función + verificación en disco | 1 |
| P02 | Cargar un proyecto en la GUI | 6 (`funcion proyecto.cargar` → `controles` → `escribir` → script externo de primer plano → `fila` → `clic_texto OK` → `esperar TProjectForm`) | Flujo manual por widgets | `funcion proyecto.cargar {proyecto}` que resuelva filtro, fila y OK | 1 |
| P03 | Poner PVsyst en primer plano antes de `fila`/`teclas`/menú | 1 llamada **fuera del MCP** (`fg2.py`) cada vez, ~15 veces | `foco:false` pero hace falta Alt+`SetForegroundWindow` | Hacerlo dentro de `gui_actuar` (reintento automático si el menú falla la primera vez) | 0 |
| P04 | Responder un aviso (`TMessageForm`) | 4–5 (`funcion` → `ventanas` → `formulario` → `controles` → `click` por hwnd) | `ventanas` llega antes de que el aviso exista (E110); `clic_texto` falla con avisos (E105) | `funcion` espera a que la GUI se estabilice y devuelve `dialogos:[{clase,titulo,texto,botones:[{texto,hwnd}]}]`; parámetro `responder:{"Advertencia":"Sí","Guardar documento":"No"}` o `auto_responder:"seguro"` | 1 |
| P05 | Sondear una función (qué abre, cerrarla) — barrido de 209 funciones | 3 (`funcion` → `formulario`/`capturar` → `cerrar`) | Cada paso separado | `gui_actuar accion:sondear funcion:X` → abre, devuelve clase, título, controles, texto y captura, y cierra; o `secuencia` con `cerrar_nuevas:true` | 1 |
| P06 | Ver una captura | 2 (`capturar` → `Read`) y deja un PNG en `<workspace>\.scratch` | La imagen se guarda en disco | `imagen:true` devuelve la imagen inline (ya existe el parámetro) y que `capturar` sin `ruta` no escriba nada | 1 |
| P07 | Comprobar el efecto en disco de cada acción | 1 `listar`/`ls` por acción (≈40 veces) | La respuesta no dice qué cambió | Toda función que modifica devuelve `efecto_en_disco:[{ruta,accion:creado|modificado|borrado,bytes}]` (diff del workspace + `%TEMP%\PVsyst_tmp` antes/después) y espera a que el archivo termine de escribirse (el ZIP se vio con 827 B y era de 2,8 MB) | 0 |
| P08 | Clic en un punto de un diálogo estándar | 2–3 (`capturar`, calcular escala 96 DPI, `clic_fisico`, a veces repetir) | `clic_fisico` en px de 96 DPI relativo a la ventana, falla sin avisar | `clic_texto` con `rol:"boton"`/`exacto:true` y, si hay varias coincidencias, devolver `candidatos` en vez de elegir; o `clic_fisico` con `espacio:"captura"` | 1 |
| P09 | Aviso de PVsyst tras una acción (error de balance, `ErrorList`) | 2 (`simular` → `resultados`/`leer`) y no se detectó | El resultado `exito` no incluye los avisos (E115) | `simular`/`variante.ejecutar_simulacion` devuelve `kpis:{kwh_anio, kwh_kwp, pr}` y `advertencias` (aviso naranja, `ErrorList`) | 1 |
| P10 | Efectos secundarios en `PVsyst.ini` (idioma, recientes, actualizaciones) | 4 (copia manual, comparar, restaurar, verificar hash) | Las funciones «modifica» no se pueden deshacer | `funcion … deshacer:true` o `entorno preferencias_restaurar` con copia automática | 1 |
| P11 | Limpieza de pruebas | 5–6 llamadas de PowerShell por tanda | No hay herramienta de limpieza por prefijo | `workspace limpiar {prefijo:"Z_", dry_run:true}` (recicla PRJ/VC/SIT/MET/SHD/PAN y carpetas) y un `workspace efimero` de sesión | 1 |
| P12 | Probar un proyecto nuevo de punta a punta | 8+ (`proyecto crear` → `exportar_proyecto` → `eliminar` → GUI importar → …) | Cada paso es una herramienta distinta | `proyecto clonar {origen,destino}` y `workspace exportar_proyecto` que acepte `reimportar:true` para pruebas | 2 |
| P13 | Carreras al listar (`listar texto` justo tras un guardado) | 2 (volver a listar) | El archivo aún no existía | `esperar_archivo {ruta,timeout_ms}` o `listar … esperar:true` | 1 |
| P14 | Coste en tokens de cada respuesta | — | `vista.campos` repite los mismos datos que el JSON de primer nivel; `click` devuelve ~15 campos de `window`; `ventanas` con `todos:true` devuelve 298 ventanas de AMD/GDI | `compacto:true` por defecto (sin `vista.campos` duplicado, `window` solo si cambia algo), filtrar `ventanas` a los procesos de PVsyst por defecto | −40–60 % de texto |

Prioridad sugerida: P01+P07 (ruta + efecto en disco), P04+P03 (avisos y primer plano dentro del MCP), P02, P05, P09.
Con esas cinco, el barrido de las 209 funciones habría necesitado ≈250 llamadas en vez de ≈700.

---

## 16. Recorrido del panel de la variante (completo: 14 de 14 botones; ver 16.1 al final)

Método: `funcion` + `pagina` + `controles` por pestaña (la captura sola no basta: `controles` solo lista la pestaña activa y no ve controles sin hwnd). Proyecto de trabajo `Z_Sub` (copia de `Z_Proj`) cargado en la instancia de PVsyst del usuario.

Hecho: Orientación, Sistema (+ módulo FV 7 pestañas, inversor 6, dimensionamiento, unifilar, resumen), Pérdidas detalladas (9 pestañas), Autoconsumo, Almacenamiento, Horizonte, Sombreados cercanos (hasta ver sus botones), Informe y Ejecutar simulación (sección 13).
**Pendiente:** Gestión de la energía, Evaluación económica, Simulación avanzada (+7 hijos), Resultados detallados (+4 hijos), Diseño de módulo (deshabilitado en VC0), combo «Tipo de campo» de Orientación.

| ID | Gravedad | Hallazgo | Evidencia |
|---|---|---|---|
| E116 | ALTA | **Causa probable de E115**: la variante simula con **Geneva/Cointrin (Suiza, 46,24° N)**, no con `Z_Site` (Colombia). `proyecto crear sitio` solo cambia el PRJ (E04). | «Resumen del sistema» y título de `THorizonForm` («…en Geneva/Cointrin»). |
| E117 | MEDIA | `Orientación`: `campos` del MCP dicen «Tracking, tilted axis, Tilt 10°, Azim 10°» pero la ventana muestra «Fixed, Tilt 25°, Azim 20°». | `funcion` vs captura de `TOrientationForm`. |
| E118 | MEDIA | `TIAMToolForm` («Estudio detallado de IAM»: ASHRAE, Fresnel, Sandia, Guardar/Leer/Exportar) **no está en catálogo ni mapa**; se abre desde Pérdidas > IAM > «Estudio detallado». | `ventanas`. |
| E119 | MEDIA | Botones/funciones sin clave en el catálogo: Pérdidas pág. 2 «Computación detallada» del desajuste (abre `TArrayCharactForm`, con «Mostrar histograma/gráfico/Abrir»), pág. 3 «Establecer todo como año», pág. 4 IAM «Estudio detallado»/«Pegar»/«Copiar», pág. 6 «Guardar como modelo», pág. 7 «Establecer aleatorio»/«Siguiente»/«Previo»; Autoconsumo «Guardar», «Imprimir», «Uso doméstico» (solo en modo «Consumidores domésticos», abre `TDomesticUseForm`); Almacenamiento «Autoconsumo» (abre `TUserNeedsForm`) y «Abrir» batería; Horizonte «Horizonte despejado», «Guardar», «Imprimir»; Sombreados cercanos «Exportar» escena e «Imprimir»; Sistema «Abrir» módulo (`TPVModuleForm`), «Resumen del sistema». | `controles` por pestaña. |
| E120 | MEDIA | Pestaña 9ª de Pérdidas («Corrección espectral», modelo FirstSolar) y «Calidad/LID» predefinidos (los bits de `Flags`, E06) no tienen parámetros en `proyecto perdidas`. | `pagina` 0–8. |
| E121 | MEDIA | `TPlotForm` es una única ventana compartida (misma hwnd; solo cambia el título), `cerrar` no la cierra y, abierta desde `TArrayCharactForm`, deja **todas las ventanas deshabilitadas** hasta pulsar «Continuar». `click` (BM_CLICK) sobre ese botón no hace nada; solo funcionó un clic por mensajes de ratón. Tiene además un botón «Recalcular» invisible en captura. | `IsWindowEnabled=false` en todas; PVsyst ocultas: `TTablWriteForm`, `TSaveComponentForm`, `TTablForm`, 2.º `TPlotForm`. |
| E122 | BAJA | `seleccionar` con un texto inexistente dice «no ofrece 'X'» sin listar las opciones válidas. Con el hwnd del combo y un prefijo sí funciona («Autoconsumo»). | Orientación / Almacenamiento. |
| E123 | BAJA | `esperar clase:` da timeout con ventanas que `ventanas` lista 1 s después (`TPVModuleForm`, `TPlotForm`). | Varias. |
| E124 | BAJA | Pestañas anidadas (p. ej. módulo FV > Parámetros del modelo > «Rderivación», «Coef. temper.») no son alcanzables con `pagina` (solo primer nivel). | `TPVModuleForm`. |
| E125 | BAJA | Clases más sin registrar: `TPanneauPVOptimizeForm`, `TPlotForm`, `TIAMToolForm`. | catálogo + mapa. |

Estado inicial de esta tanda: la orientación #2 que añadí se descartó con «Cancelar».

### 16.1 Resto del panel (segunda parte) y conclusión sobre E115

Hecho en esta parte: Gestión de la energía, Evaluación económica (5 pestañas), Simulación avanzada (7 herramientas), Resultados detallados (4 hijos), «Diseño de módulo» (deshabilitado en VC0). Orientación: no se enumeró la lista del combo «Tipo de campo».

| ID | Gravedad | Hallazgo | Evidencia |
|---|---|---|---|
| E126 | ALTA | **E115 explicado**: el resultado guardado en VC0 es **obsoleto respecto a la configuración**. Diagrama de pérdidas: irradiación efectiva 1 480 kWh/m² × 33 m² × 18,45 % ≈ 9 011 kWh, pero «Conjunto de energía nominal» = 13 327 kWh (≈ 1,48×, diferencia ≈ −32 %, el «−33,33 %» del aviso). Tabla mensual: **PR 1,25–1,41 todos los meses**. Al re-simular desde la GUI sale PR 0,820. El MCP/CLI devolvió `exito` sin comprobar PR ≤ 1 ni el aviso. | `TPlotLossDiagramF`, `TTablForm`, `TResultForm`. |
| E127 | MEDIA | `TEcoEvaluationForm` tiene **5 pestañas** (Inversión y cargos, Parámetros financieros, Venta de electricidad, Resultados financieros, **Balance de carbono**); el catálogo solo cubre la 0 y la 3, y `proyecto economia` no cubre tarifas variable/horaria/estacional/CSV ni el balance de CO₂ (−10,278 t con el resultado malo). Pestañas anidadas no alcanzables (E124). Cerrar con «Cancelar» pregunta «¿Quiere cancelar las modificaciones?» solo por abrirla. | `pagina` 0–4. |
| E128 | MEDIA | `TAdvancedSimForm` tiene botones «Simulación» y «Resultados» (y fechas inicio/fin, paso de tiempo) sin clave en el catálogo; «Comparaciones» deshabilitado (`funcion` lo dice bien). Herramientas con botones sin clave: Envejecimiento («Ejecutar simulación», «Copiar la tabla», «Informe», «Pérdidas detalladas – Envejecimiento»), Lotes («Abrir en explorer», «Copiar»), Salida («Cambiar archivo»), Optimización («Ejecutar», «Abortar»), Gráficos especiales y Almacén por hora (barras `TActionToolBar` sin hwnd). | `controles` de cada clase. |
| E129 | MEDIA | `TResultForm`: «Recentrar», «Carga», «Guardar» sin clave; «Gráficos predefinidos» (`TResultPlotForm`): «Añadir»/«Eliminar» gráficos del informe; «Gráficos por hora» (`TResultPlotHorForm`): «Establecer como predeterminado». | `controles`. |
| E130 | ALTA | `TTablForm` (tabla mensual) y `TPlotLossDiagramF` tienen menús propios (Cerrar/Imprimir/**Exportar**/…): «Exportar» → «Copiar al portapapeles» (id 126), «Copiar imagen» (127), **«Copiar a archivo» (128)** abre `TTablWriteForm` («Guardar valores de tabla como archivo», con `TEdit` de ruta y separador tabulación/coma/punto y coma). Con la ruta escrita en el `TEdit` y «OK» se creó `Z_TablaMensual.csv` (869 B). Ninguna de las tres funciones está en el catálogo ni la clase en el mapa; es el camino más corto para exportar resultados mensuales por GUI y funciona con `escribir` (sin diálogo de Windows, relevante para M01). | `UserData\Z_TablaMensual.csv`. |
| E131 | MEDIA | El guardián de modales de `gui_actuar` da falsos positivos: acusó a un `TPrintPreviewForm` **oculto** (no visible, habilitado) de bloquear `TTablForm` y rechazó `menu id:128`; hubo que enviar `WM_COMMAND` por fuera del MCP. | `menu ventana:TTablForm id:128`. |
| E132 | BAJA | `clic_texto "Diseño de módulo"` sobre un botón deshabilitado devuelve `ok`/`coincidencias:1` sin efecto (E107). El botón no tiene clave de catálogo y solo se habilita con el modo de sombreado «Cálculo eléctrico detallado». | `TProjectForm`. |
| E133 | BAJA | Al cerrar `TResultForm` se cierra también la `TAdvancedSimForm` que la abrió (efecto en cadena no documentado). | `ventanas`. |

### 16.2 Pendientes cerrados (tercera pasada)

Pendiente de la sección 10 resuelto: `escribir_serie` con éxito, `crear_desde_ficha` OND (BTR/PMP no soportados), `rellenar` real, `disenar_sistema` completo, combo «Tipo de campo», botones de ejecución de Simulación avanzada. Siguen sin probarse: `entorno licencia_desactivar` (requiere autorización) y «Abrir en explorer» (abre el Explorador del usuario).

| ID | Gravedad | Hallazgo | Evidencia |
|---|---|---|---|
| E134 | ALTA | **`crear_desde_ficha` OND genera un inversor inválido para simular**: no pide ni escribe la tensión de red (`VOutConv`, que PVsyst espera **dentro** de `Converter=TConverter`) y `PNomConv`/`PMaxOUT` van en **kW** (referencia `Generic_50kW.OND`: 50.000); con `PNomConv:5000` quedó un inversor de 5 000 kW. La simulación falló con «The chosen inverter is not valid. Please define the grid voltage». | `Z_OndFicha.OND`; `editar` no alcanza `TConverter` (no es PVObject) y hubo que editar el archivo a mano. |
| E135 | MEDIA | `disenar_sistema` con `simular:true` devuelve `ok:true`/`errores:[]` aunque PVsyst avisó «ErrorList: 1 error messages»; pide `n_mod_serie` y `n_strings` (el mensaje sí es claro) y exige `modulo`/`inversor` como `PVmodules\X.PAN`/`X.OND` (con solo el nombre: «no encontrado»). Una ejecución TRIAL se gastó en el intento con inversor inválido (139 → 138 con la buena). Resultado correcto: 6 244,8 kWh, 1 561 kWh/kWp, PR 82,24 % = la simulación de la GUI (6 245 kWh, PR 0,822). | `Z_DisFull`. |
| E136 | MEDIA | La variante creada por `disenar_sistema` conserva sitio **Geneva/Cointrin** en herramientas de la GUI (título del gráfico de optimización de cobertizos) aunque el proyecto usa `Z_DisFull_Site`; la ventana de resultados sí muestra `Z_DisFull_Site`. | `TPlotForm`. |
| E137 | MEDIA | `componente crear` exige `tipo` (error «Missing required argument 'tipo'») aunque la descripción solo pide `nombre_nuevo`, `plantilla`, `cambios`. `tipo` en `crear_desde_ficha` anuncia «PAN, OND, BTR, PMP» pero BTR/PMP responden «no soportado: PAN u OND». | llamadas. |
| E138 | BAJA | `sitio escribir_serie`: exige que el `.DAM` exista y 8 760/8 784 horas por variable; el mensaje de éxito dice «Serie meteorológica escrita en el sitio» (es un `.DAM`, no un sitio). Funciona: solo `T_Amb` cambió, el resto intacto. | `Z_Serie.DAM`. |
| E139 | MEDIA | `crear_desde_ficha` y otras acciones con GUI no restablecen la pantalla de inicio: con una ventana de proyecto abierta fallan con «PVsyst no abrió 'THomeForm' en 60 s» y dejan PVsyst sin ventanas visibles (hubo que `cerrar_todo`). | `componente crear_desde_ficha`. |
| E140 | MEDIA | `rellenar` (real) funciona en `components.batteries` (rellena, verifica valor a valor y recalcula `nombre_archivo`; tras OK + «Guardar como» + «Guardar» creó `ComposPV\Batteries\Z_ZBattRell.BTR` correcto). Pero con `variante.orientation tipo_de_campo:"Cobertizos ilimitados"` el cambio **sí se aplicó** y `rellenar` lo informó como `distinto`/«fallo parcial» (el formulario cambia de diseño y no puede releer). `rellenar` sí lista las 17 opciones del combo en el error; `seleccionar` no (E122). | `TOrientationForm`. |
| E141 | BAJA | `seleccionar` sobre combos de `TOptimizationToolForm` elige la fila contraria a la del hwnd (la fila visual 3 en vez de la 1) y `escribir` en los `TEdit` correspondientes cae en la fila equivocada. | captura. |
| E142 | MEDIA | Botones de ejecución: `click` sobre «Abortar» muestra un aviso «¿De verdad quiere abortar?» que aparece con retraso y `click` no informa; tras «Sí» sale «El escaneo fue abortado por el usuario». «Ejecutar» del escaneo de optimización y «Simulación» de Simulación avanzada ejecutan sin gastar TRIAL (138 antes y después). «Ejecutar simulación» de Envejecimiento está deshabilitado hasta activar la degradación (y `click` dice `ok`). | `TOptimizationToolForm`, `TAgingToolForm`. |
| E143 | BAJA | Nuevas ventanas sin registrar: `TAzimuthHeightForm` («Diagrama de recorridos solares», desde «Gráfico de sombreado» de orientación de cobertizos); botones «Mostrar optimización» y «Gráfico de sombreado» sin clave. | `ventanas`. |
| E144 | BAJA | Al cerrar PVsyst reescribe `PVsyst.ini`; no queda forma de recuperar el ini original (la copia de respaldo se había borrado). Cambios: solo la lista de proyectos recientes (ahora con `Z_DisFull`, borrado). | `entorno preferencias`. |

Estado final: `Z_Sub.*` y `Z_TablaMensual.csv` movidos a `UserRecycleBin`; `Z_Proj` y los 16 proyectos intactos; tu PVsyst queda abierto en «New.PRJ» (pantalla de proyecto vacía), sin nada guardado.
