# Cobertura de la vista de agente en las ventanas de PVsyst (Tarea C)

Registro: `src/vista_agente/cobertura/ventanas/`. El test `should_register_every_window_class_of_the_map_in_exactly_one_state`
recorre las clases de `docs/gui_mapa.json` y falla si alguna no tiene exactamente un estado.

## Estados
1. **Entidad completa** (clave, etiqueta, unidad, rango, `clave_archivo`, tipo, ancla, acceso): **0 ventanas**. `TPVModuleForm`
   («Datos básicos», 16 campos) está validada a medias contra un archivo guardado por PVsyst 8.1.8 en PRUEBA, que **sí guarda
   componentes** (`OK` → `TSaveComponentForm` → «Guardar como» → `TSaveObjPVForm`): con un valor distinto en cada campo, el
   `.PAN` guardado (`tests/fixtures/vista_agente/Modulo_distintos_8_1_8.PAN`) trae `PNom=410.5`, `Isc=11.200`, `Voc=48.30`,
   `Imp=10.400`, `Vmp=39.50`, `muISC=4.20` (mA/°C), `TRef=20.0` y `NCelS=72`: las unidades y las `clave_archivo` están
   comprobadas. Siguen sin validarse los rangos (son los del MCP, no de PVsyst), `tecnologia` (no se pidió: el archivo trae el valor por
   defecto `Technol=mtNotSpec`) y `g_ref_w_m2`: se tecleó 900 y la ventana siguió en 1000 y el archivo trae `GRef=1000`. `mu_isc_pct_c`
   no se guarda: PVsyst lo calcula desde `muISC`. Por eso no pasa a «completa».
   `TGridInverterForm` («Parámetros principales») tiene ahora esquema `components.inverters` (15 campos de texto y número) y la
   misma validación contra el `.OND` que guardó PVsyst (`tests/fixtures/vista_agente/Inversor_distintos_8_1_8.OND`): `VMppMin=310`,
   `VmppNom=600.0`, `VMPPMax=820`, `VAbsMax=1000`, `VOutConv=400.0` (V), `PNomConv=20.500`, `PMaxOUT=22.500` y `PSeuil=0.10` (kW),
   además de `Model` y `Manufacturer`. No se aplicaron `voltaje_min_pnom_v`, `corriente_entrada_max_a`, `corriente_ca_nominal_a`
   y `corriente_ca_max_a` (quedan en «N/A» y deshabilitados tras rellenar: dependen de casillas que el esquema no modela), así que
   no tienen `clave_archivo`. Radios (fases) y casillas (50/60 Hz) no entran en el modelo de campos.
   `TBatteryForm` («Datos básicos») tiene esquema `components.batteries` (10 campos) con el `.BTR` guardado por PVsyst
   (`tests/fixtures/vista_agente/Bateria_distintos_8_1_8.BTR`): `NCellSerie=6`, `VNomCell=2.1` (V), `CapNomC10=150.50` (Ah),
   `BattTempRef=25.0` (°C), `BattTechnol=Lead_Acid`. **Escalas distintas entre ventana y archivo**: la ventana muestra la resistencia
   interna en mΩ (3.5) y el archivo la guarda en Ω (`ResInterne=0.0035`); la eficiencia coulómbica se muestra en % (95.5) y se
   guarda como fracción (`EfficI=0.955`). No se probaron los campos de la derecha (energía en DOD, capacidad de la hoja de datos,
   tasa de descarga) ni las pestañas 1 a 4.
   Tres esquemas de variante, validados contra una variante que PVsyst 8.1.8 guardó con «Guardar la variante activa»
   (`tests/fixtures/vista_agente/Variante_orientacion_sistema_8_1_8.VC0`; la PRUEBA **guarda variantes**):
   `variante.orientation` (`TOrientationForm`, plano fijo: `FieldType`, `FieldTilt=25.0`, `FieldAzim=10.0`, `BaseSlope`; el nombre de la
   orientación no se recalcula al cambiar la inclinación), `variante.system` (`TSystemegridForm`: `usar_potencia_planeada`/`usar_area_disponible`
   son opciones; con 50 kWp PVsyst propuso 28 módulos en serie × 4 cadenas y guardó `NModSerie=28`, `NStringCh=4`, `NInverter=1`,
   `PNomSizing=50.0`) y `proyecto.ventana` (`TProjectForm`: descripción, archivo meteo y variante activa). Los dos últimos reemplazan las
   coordenadas de `proyecto dimensionar` y de la apertura de la variante. Rangos: son los del MCP, no los de PVsyst. Otros tipos de campo
   (rastreadores, cobertizos) muestran otros controles: el esquema de orientación solo describe el plano fijo.
2. **Entidad básica** (clave por clase y posición a 96 DPI, valor leído por handle): **71** (6 con esquema propio: `TPVModuleForm`, `TGridInverterForm`, `TBatteryForm`, `TOrientationForm`, `TSystemegridForm`, `TProjectForm`).
3. **Solo acciones** (se describe por sus botones): **14**.
4. **No aplica** (dibujado o ajeno, solo captura): **4**.

## Cómo se midió
- Release, PVsyst 8.1.7 en PRUEBA, 150 % (144 DPI), 2026-10-27. Se abrió cada clase **una vez** con la primera función del
  catálogo que la abre (`gui_actuar accion:funcion`) y se leyó con `gui_observar que:vista` (campos vivos por handle, sin OCR).
  Cada ventana se cerró con `cerrar cancelar:true` (Cancelar/Cerrar). No se pulsó Guardar, OK, Aplicar ni Ejecutar.
- Los proyectos se abrieron como proyecto nuevo (`inicio.conectado_a_la_red`) sin guardarlo ni elegir sitio. Por eso hay
  funciones con botones deshabilitados (Horizonte, Sistema, Resultados…) que no se pudieron medir.
- **Solo se midió la pestaña que PVsyst muestra al abrir.** Los formularios con `TPageControl` no se recorrieron
  pestaña por pestaña, así que los campos de las demás pestañas figuran como no medidos.
- Un recorrido anterior abrió varias veces las mismas ventanas (una por función). Se descartó y se rehízo con una apertura por clase.
- Claves: `{clase sin T}_{y}_{x}` (p. ej. `edit_148_518`). No salen de etiquetas, así que no hay claves con erratas de
  OCR, pero tampoco nombre de dominio: dos formularios iguales con campos desplazados darían otras claves.
- Clases que PVsyst abrió y **no están en el mapa**: `TAboutForm`, `TLicenseMainForm` (puede mostrar datos de la licencia) y un
  diálogo de Windows (`#32770`). No se añadieron al mapa.

## Por clase (82 de 89 abiertas en vivo; 7 solo clasificadas)
| Clase | Dominio | Estado | Abierta en vivo | Campos vivos | Botones | Instancias en el mapa |
|---|---|---|---|---|---|---|
| `MicrosoftWindowsTooltip` | aplicacion | 4 | no | — | — | 1 |
| `TAdvancedSimForm` | variante | 2 | sí (`variante.simulacion_avanzada`) | 5 | 11 | 4 |
| `TAgingToolForm` | variante | 2 | sí (`variante.simulacion_avanzada.herramienta_de_envejecimiento`) | 5 | 6 | 1 |
| `TArrayCharactForm` | variante | 2 | sí (`herramientas.factor_transposicion`) | 14 | 4 | 1 |
| `TArrayLossesPlotForm` | resultados | 2 | sí (`variante.perdidas_detalladas.grafico_de_perdidas`) | 12 | 2 | 1 |
| `TArraySizingForm` | variante | 3 | sí (`variante.sistema.dimensionamiento`) | 0 | 1 | 1 |
| `TBacktrackingForm` | variante | 2 | no | — | — | 1 |
| `TBatchSimulForm` | variante | 2 | sí (`variante.simulacion_avanzada.simulacion_por_lotes`) | 6 | 4 | 1 |
| `TBatteryForm` | componentes | 2 | sí (`bases_de_datos.baterias.nuevo`) | 19 | 4 | 1 |
| `TBuildingObjectForm` | escena_3d | 3 | sí (`escena_3d.crear.edificio`) | 0 | 2 | 1 |
| `TChoixObjetPVForm` | componentes | 2 | sí (`bases_de_datos.inversores_red`) | 5 | 6 | 15 |
| `TContactForm` | componentes | 2 | sí (`bases_de_datos.fabricantes_minoristas.nuevo`) | 22 | 2 | 2 |
| `TCurrenciesForm` | variante | 2 | sí (`variante.evaluacion_economica.tasas`) | 100 | 4 | 1 |
| `TDamMainForm` | meteo | 3 | sí (`inicio.datos_medidos`) | 0 | 5 | 1 |
| `TDatabaseForm` | componentes | 3 | sí (`inicio.bases_de_datos`) | 0 | 17 | 1 |
| `TDatamesFusionForm` | meteo | 2 | sí (`datos_medidos.transformar_archivo`) | 3 | 5 | 1 |
| `TDetailedLossesForm` | variante | 2 | sí (`variante.perdidas_detalladas`) | 6 | 5 | 4 |
| `TDimensForm` | variante | 3 | sí (`diseno_preliminar.conectado_a_la_red`) | 0 | 5 | 3 |
| `TDimensSituationForm` | variante | 2 | sí (`diseno_preliminar.conectado_a_la_red.sitio_y_datos_meteo`) | 3 | 3 | 4 |
| `TDomesticUseForm` | variante | 2 | sí (`variante.necesidades_usuario_independiente`) | 35 | 6 | 1 |
| `TEcoEvaluationForm` | variante | 2 | sí (`variante.evaluacion_economica`) | 1 | 4 | 3 |
| `TEcoPriceListForm` | variante | 2 | sí (`bases_de_datos.precios`) | 2 | 3 | 1 |
| `TElementaryObjectForm` | escena_3d | 2 | sí (`escena_3d.crear.objeto_elemental`) | 8 | 2 | 1 |
| `TGenerSyntheForm` | variante | 2 | sí (`bases_de_datos.datos_sinteticos`) | 10 | 3 | 1 |
| `TGeneratorForm` | componentes | 2 | sí (`bases_de_datos.generadores.nuevo`) | 9 | 3 | 1 |
| `TGeoSiteForm` | componentes | 3 | sí (`bases_de_datos.sitios_geograficos.nuevo`) | 0 | 8 | 2 |
| `TGridInverterForm` | componentes | 2 | sí (`bases_de_datos.inversores_red.nuevo`) | 29 | 4 | 2 |
| `TGridStorageForm` | variante | 2 | sí (`variante.almacenamiento`) | 1 | 3 | 1 |
| `TGroundObjectForm` | escena_3d | 2 | sí (`escena_3d.crear.objeto_de_tierra`) | 11 | 2 | 1 |
| `THelpForm` | aplicacion | 4 | sí (`escena_3d.ayuda.paginas_de_ayuda`) | 0 | 0 | 1 |
| `THorizonForm` | variante | 2 | sí (`variante.horizonte`) | 9 | 7 | 4 |
| `TImportResultForm` | escena 3d | 2 | sí (Archivo ▸ Importar ▸ Importar una escena 3D, PVsyst 8.1.8) | 10 | 2 | 1 |
| `TTrackerFieldForm` | escena_3d | 2 | sí (`escena_3d.crear.conjunto_de_rastreadores`, PVsyst 8.1.8) | 19 | 3 | 1 |
| `TShedFieldForm` | escena_3d | 2 | sí (`escena_3d.crear.conjunto_de_tablas`, `…conjunto_de_cupulas` y `…conjunto_de_protectores_solares`: las tres abren esta clase) | 11 | 3 | 3 |
| `TRectFieldForm` | escena_3d | 2 | sí (`escena_3d.crear.plano_fv_rectangular`) | 9 | 3 | 1 |
| `TPolyFieldForm` | escena_3d | 2 | sí (`escena_3d.crear.plano_fv_poligonal`) | 7 | 3 | 1 |
| `TImportHorizonForm` | variante | 2 | sí (`variante.horizonte.leer_importar`) | 9 | 3 | 1 |
| `TMetWizard` | meteo | 3 | sí (`bases_de_datos.importar_archivo_personalizado`) | 0 | 2 | 1 |
| `TMeteoCompareForm` | meteo | 2 | sí (`bases_de_datos.comparar_meteo`) | 10 | 9 | 1 |
| `TMeteoConversionForm` | meteo | 2 | sí (`datos_medidos.importar_archivo_personalizado`) | 8 | 9 | 1 |
| `TMeteoImportForm` | meteo | 2 | sí (`bases_de_datos.importar_formato_conocido`) | 14 | 10 | 1 |
| `TMeteoMensForm` | meteo | 2 | sí (`herramientas.calculo_mensual_meteo`) | 11 | 5 | 1 |
| `TMeteoTablesGraphsForm` | meteo | 2 | sí (`bases_de_datos.tablas_graficos_meteo`) | 20 | 7 | 2 |
| `TMismatchStringForm` | variante | 2 | sí (`variante.perdidas_detalladas.estudio_detallado`) | 36 | 2 | 1 |
| `TOptimizationToolForm` | variante | 2 | sí (`variante.simulacion_avanzada.herramienta_de_optimizacion`) | 13 | 4 | 1 |
| `TOrientManageForm` | variante | 3 | sí (`escena_3d.herramientas.gestion_de_orientaciones`) | 0 | 1 | 1 |
| `TOrientToolForm` | variante | 2 | sí (`escena_3d.herramientas.herramienta_educativa_orientacion`) | 3 | 1 | 1 |
| `TOrientationForm` | variante | 2 | sí (`variante.orientacion`) | 9 | 4 | 5 |
| `TPVModuleForm` | componentes | 2 | sí (`bases_de_datos.modulos_pv.nuevo`) | 16 | 0 | 1 |
| `TParamEditForm` | meteo | 2 | sí (`configuraciones.parametros_avanzados`) | 4 | 2 | 1 |
| `TParamSolForm` | variante | 2 | sí (`herramientas.parametros_solares`) | 8 | 2 | 1 |
| `TPerspectiveForm` | escena_3d | 4 | sí (`variante.sombreados_cercanos.construccion_perspectiva`) | 0 | 0 | 1 |
| `TPlotLossDiagramF` | resultados | 4 | sí (`variante.resultados_detallados.diagrama_de_perdida`) | 0 | 0 | 1 |
| `TPreferencesForm` | aplicacion | 3 | sí (`configuraciones.preferencias`) | 0 | 4 | 1 |
| `TProjectForm` | variante | 2 | sí (`proyecto.nuevo_conectado_a_la_red`) | 3 | 16 | 4 |
| `TProjectLoadForm` | variante | 2 | sí (`proyecto.cargar`) | 1 | 3 | 2 |
| `TPumpControllerForm` | componentes | 2 | sí (`bases_de_datos.controladores_bombeo.nuevo`) | 15 | 4 | 1 |
| `TPumpForm` | componentes | 2 | sí (`bases_de_datos.bombas.nuevo`) | 20 | 4 | 1 |
| `TResultForm` | resultados | 2 | sí (`variante.resultados_detallados`) | 4 | 11 | 3 |
| `TResultPlotForm` | resultados | 3 | sí (`variante.resultados_detallados.graficos_predefinidos`) | 0 | 4 | 1 |
| `TResultPlotHorForm` | resultados | 2 | sí (`variante.resultados_detallados.graficos_por_hora`) | 27 | 3 | 1 |
| `TResultTabForm` | resultados | 2 | sí (`variante.resultados_detallados.tablas`) | 3 | 2 | 1 |
| `TSAControllerForm` | componentes | 2 | sí (`bases_de_datos.controladores_independientes.nuevo`) | 23 | 4 | 1 |
| `TSaveComponentForm` | componentes | 3 | sí (OK de un componente nuevo, PVsyst 8.1.8) | 1 | 2 | 1 |
| `TSaveObjPVForm` | componentes | 2 | sí (OK y «Guardar como» de un componente nuevo, o «Exportar escena (.SHD)»; PVsyst 8.1.8) | 2 | 2 | 1 |
| `TSettingsForm` | variante | 2 | sí (`ventana_proyecto.proyecto.configuracion`) | 20 | 2 | 1 |
| `TShadingsForm` | variante | 2 | sí (`variante.sombreados_cercanos`) | 6 | 10 | 4 |
| `TShdObjectSelectionForm` | variante | 2 | sí (`escena_3d.herramientas.lista_de_objetos`) | 1 | 2 | 1 |
| `TShedElecShadingForm` | variante | 2 | no | — | — | 1 |
| `TSimulFichForm` | variante | 2 | sí (`variante.simulacion_avanzada.archivo_de_salida`) | 11 | 3 | 1 |
| `TSimulPlotForm` | resultados | 2 | sí (`variante.simulacion_avanzada.graficos_especiales`) | 10 | 2 | 1 |
| `TSimulValHorForm` | resultados | 3 | sí (`variante.simulacion_avanzada.almacen_de_datos_por_hora`) | 0 | 2 | 1 |
| `TSummaryForm` | variante | 2 | sí (`escena_3d.herramientas.resumen_de_escena`) | 1 | 1 | 4 |
| `TSystemCircuitForm` | variante | 2 | sí (`variante.perdidas_detalladas.diagrama_unifilar`) | 5 | 1 | 2 |
| `TSystemPumpingForm` | variante | 2 | sí (`variante.sistema_bombeo`) | 18 | 5 | 1 |
| `TSystemeAddForm` | variante | 2 | sí (`variante.gestion_energia`) | 6 | 3 | 1 |
| `TSystemeBattForm` | variante | 2 | sí (`variante.sistema_independiente`) | 24 | 6 | 1 |
| `TSystemegridDCForm` | variante | 2 | no | — | — | 1 |
| `TSystemegridForm` | variante | 2 | sí (`variante.sistema`) | 22 | 10 | 1 |
| `TTableFieldForm` | meteo | 2 | sí (`escena_3d.crear.generador_fv`) | 7 | 3 | 1 |
| `TThinObjectsForm` | escena_3d | 2 | sí (`escena_3d.herramientas.sombreado_objetos_delgados`) | 3 | 1 | 1 |
| `TToolsForm` | aplicacion | 3 | sí (`inicio.herramientas`) | 0 | 6 | 1 |
| `TTranspositionForm` | variante | 2 | no | — | — | 1 |
| `TUserNeedsForm` | variante | 2 | sí (`variante.autoconsumo`) | 8 | 6 | 2 |
| `TVCManageForm` | variante | 2 | no | — | — | 1 |
| `TVoltageOptimiseForm` | variante | 2 | no | — | — | 1 |
| `TWaterUseForm` | variante | 2 | sí (`variante.necesidades_agua`) | 11 | 4 | 1 |
| `TWiringResistanceForm` | variante | 2 | sí (`variante.perdidas_detalladas.computacion_detallada`) | 9 | 4 | 1 |
| `TWorkspaceForm` | aplicacion | 3 | sí (`archivo.espacio_de_trabajo`) | 0 | 1 | 2 |

## Filas de listas y árboles (`gui_actuar accion:fila`)
Medido en vivo con el release (PVsyst 8.1.7; la tabla de objetos del editor 3D y `teclas` en 8.1.8, donde `fila` también funcionó en `TProjectLoadForm`, 144 DPI = 150 %, capturas a 96 DPI, 2026-10-28). Sin licencia (PRUEBA); todo se cerró con Cancelar/Cerrar y sin pulsar OK.

| Control | Ventana donde se midió | Geometría a 96 DPI | Filas probadas | Resultado |
|---|---|---|---|---|
| `TVirtualStringTree` | `TProjectLoadForm` (19 proyectos) | cabecera 21 px, fila 19 px, 29 huecos visibles | 0, 3, 18; 19 (hueco vacío); 40 | 0, 3 y 18 verificadas por banda; 19 error («PVsyst resaltó la 0»); 40 error de rango |
| `TListBox` | `TChoixObjetPVForm` de módulos (32 elementos) | borde 2 px, fila 13 px, 33 huecos visibles | 0, 3, 31; 32 (hueco vacío); 33 | 0, 3 y 31 verificadas; 32 error; 33 error de rango |
| `TVirtualStringTree` | `TShdObjectSelectionForm` («Lista y gestión de objetos» del editor 3D, 4 objetos; PVsyst 8.1.8, 144 DPI, 2026-10-29) | cabecera 32 px, fila 26 px, unos 20 huecos visibles | 0, 1, 3, 2 | las 4 verificadas por banda; no se abrió ninguna ventana |

- La cabecera de la lista de módulos es un `TStringGrid` aparte de una fila: se apunta a la `TListBox`, no a él (sin geometría: error).
- Solo selecciona. La banda de la fila *n* ocupa y∈[cabecera+alto·*n*, cabecera+alto·*(n+1)*] del control.
- El índice son huecos **visibles**: no hay desplazamiento. Una lista con más filas que huecos queda fuera de alcance `[INFERENCIA]`: no se probó.
- Un clic físico puede **activar** el elemento: en la primera medición, un clic sobre la lista de módulos (dy 100, con el foco fuera de la lista) abrió `TPVModuleForm` y cerró el selector; no se supo por qué `[INFERENCIA]`. `fila` comprueba que no apareció ni desapareció ninguna ventana y, antes del clic, da el foco al control: con el foco en un combo el primer clic solo movía el foco y la fila quedaba como estaba (tres de tres aciertos con el foco dado, tras `TAB` en el combo). Repetir el clic no es una salida: dos clics seguidos son un doble clic.
- Qué fila es no se lee: ni UIA ni MSAA dan texto. En módulos, inversores, baterías, controladores, bombas y fabricantes la fila se identifica solo por índice; el orden alfabético no se infiere.
- Con `PVSYST_MCP_ESCRITORIO=aislado` la acción se niega: no está medido. Otro DPI distinto de 144: `[INFERENCIA]`.
- La geometría vale para el control medido: clase y ventana anfitriona (`TVirtualStringTree` de `TProjectLoadForm`; `TListBox` de `TChoixObjetPVForm` titulado «Eligiendo un módulo FV»; `TVirtualStringTree` de `TShdObjectSelectionForm`). Los otros 14 selectores y cualquier otro control se rechazan con un error que lista dónde se midió.
- Antes del clic `fila` exige que la ventana esté al frente y el control tenga el foco, y que el punto de clic caiga dentro del control; se niega con menos de 3 huecos visibles. La banda se lee comparando la captura de antes y la de después, y se comprueba que ninguna ventana se abrió ni se cerró.

`que:esquema` trae `abierta_con` por clase: las funciones del catálogo que la abren. `herramientas.factor_transposicion` declara `TArrayCharactForm`, la ventana que se vio abrir; `TTranspositionForm` queda sin función que la abra.

## Pestañas (`gui_actuar accion:pagina`)
Recorridas en vivo con el release (PVsyst 8.1.7, 144 DPI, 2026-10-28, PRUEBA): cada clase se abrió una vez, se pidió `pagina i` hasta «fuera de rango» y tras cada cambio se leyó `que:vista`. Nada se guardó; los diálogos de componentes se abrieron con «Nuevo» y se cerraron con Cancelar/Cerrar.

| Clase | Función | Pestañas | Campos leídos por pestaña (vista genérica) |
|---|---|---|---|
| `TArrayCharactForm` | `herramientas.comportamiento_electrico` | 3 | 14/18/13 |
| `TBatchSimulForm` | `variante.simulacion_avanzada.simulacion_por_lotes` | 3 | 6/9/5 |
| `TBatteryForm` | `bases_de_datos.baterias.nuevo` | 5 | 19/17/24/5/13 |
| `TDatamesFusionForm` | `datos_medidos.transformar_archivo` | 3 | 3/3/2 |
| `TDetailedLossesForm` | `variante.perdidas_detalladas` | 9 | 6/16/8/15/18/6/28/9/11 |
| `TEcoEvaluationForm` | `variante.evaluacion_economica` | 5 | 1/12/11/1/10 |
| `TGeneratorForm` | `bases_de_datos.generadores.nuevo` | 2 | 9/11 |
| `TGridInverterForm` | `bases_de_datos.inversores_red.nuevo` | 6 | 29/12/34/15/11/13 |
| `TGridStorageForm` | `variante.almacenamiento` | 1 | 1 |
| `THorizonForm` | `variante.horizonte` | 2 | 9/1 |
| `TMeteoMensForm` | `herramientas.calculo_mensual_meteo` | 1 | 11 |
| `TOptimizationToolForm` | `variante.simulacion_avanzada.herramienta_de_optimizacion` | 2 | 13/9 |
| `TParamSolForm` | `herramientas.parametros_solares` | 3 | 8/10/17 |
| `TPreferencesForm` | `configuraciones.preferencias` | 9 | 0/0/0/0/0/0/0/0/0 |
| `TPumpControllerForm` | `bases_de_datos.controladores_bombeo.nuevo` | 2 | 15/15 |
| `TPumpForm` | `bases_de_datos.bombas.nuevo` | 4 | 20/9/4/11 |
| `TSAControllerForm` | `bases_de_datos.controladores_independientes.nuevo` | 4 | 23/12/4/13 |
| `TSystemCircuitForm` | `variante.sistema.diagrama_unifilar` | 2 | 5/9 |
| `TPolyFieldForm` | `escena_3d.crear.plano_fv_poligonal` | 3 | 7/(veto)/14 |
| `TRectFieldForm` | `escena_3d.crear.plano_fv_rectangular` | 3 | 9/12/14 |
| `TShedFieldForm` | `escena_3d.crear.conjunto_de_tablas` | 3 | 11/14/14 |
| `TSystemeAddForm` | `variante.gestion_energia` | 4 | 6/2/20/2 |
| `TTrackerFieldForm` | `escena_3d.crear.conjunto_de_rastreadores` | 4 | 19/15/6/14 |

- 0 vetos en las 19 clases de la tabla: todas las pestañas cambiaron. Los campos son los visibles de cada pestaña por clase y posición, no una entidad con esquema. `TPreferencesForm` sigue sin campos legibles en ninguna de sus 9 pestañas.
- `docs/gui_mapa.json` difiere del recorrido en el número de pestañas de `TGeneratorForm` (2; el mapa, 1), `TPumpControllerForm` (2; 1), `TGridInverterForm` (6; 5), `TEcoEvaluationForm` (5; el mapa lista 6 nombres, y no se midió por qué una no aparece), `TSystemeAddForm` (4; 3) y otras: el mapa solo registra las pestañas que se pisaron `[INFERENCIA]`.
- **`TPVModuleForm` (módulo nuevo, 5 pestañas):** `pagina 1` fue vetada con los campos obligatorios vacíos; tras `rellenar` con valores válidos (modelo, fabricante, potencia, Isc, Voc, Impp, Vmpp, μIsc) pasó la 0→1, y desde la 1 PVsyst vetó **todo** cambio, incluso volver a la 0, mientras la pestaña tuviera campos inválidos. Coincide con la hipótesis «PVsyst impide salir de una página mientras sus campos son inválidos» (observada, no probada contra el código). `rellenar` hoy solo cubre la pestaña 0: tras `pagina`, `que:vista` repite las 16 claves del esquema y el resto cae en `sin_mapear`.
- `TEcoEvaluationForm`: recorrer sus pestañas marca el formulario como modificado; cerrarlo pregunta «¿Quiere cancelar las modificaciones?» y no hay botón inocuo (No lo deja abierto, Sí está prohibido en la exploración). Se resolvió cerrando el proceso de PVsyst, que no tenía nada guardado.
- Sin abrir: ninguna de las 5 que faltaban (`TSettingsForm`, `TDomesticUseForm`, `TWaterUseForm`, `TSystemeBattForm`, `TSystemPumpingForm`) sigue sin abrir (PVsyst 8.1.8, 2026-10-29): las cuatro de sistema se abrieron con los proyectos demo `_DEMO_StandAlone` (fila 7 de «Cargar proyecto», `TDomesticUseForm` 35 campos y 2 pestañas, `TSystemeBattForm` 24 y 4) y `_DEMO_Pumping` (fila 4, `TWaterUseForm` 11 y 2, `TSystemPumpingForm` 18 y 3), y `TSettingsForm` (20 campos, 4 pestañas: Albedo, Condiciones de diseño, Otras limitaciones, Preferencias) con el botón «Configuración del proyecto» (`accion:clic_texto`): la función por menú falló porque un intento anterior dejó la barra de menús activada (la ventana mostraba «Proyecto» resaltado) y hubo que salir con `ESC`. `TGeoSiteForm` no se recorrió a propósito: una pestaña es un mapa web.

- **Diálogos de campo del editor 3D** (`TTrackerFieldForm`, `TShedFieldForm`, `TRectFieldForm`, `TPolyFieldForm`; PVsyst 8.1.8, 144 DPI, escena importada abierta, 2026-10-30): se abrieron con `accion:menu menu:1 item:N`, se leyó cada pestaña y se cerraron con Cancelar. Tras visitar una pestaña, Cancelar pregunta «¿Quiere mantener sus modificaciones en el objeto?» (Cancelar/No/Sí): se responde «No», que descarta el objeto; «Sí» lo añadiría a la escena. Las entradas 6, 7 y 8 del menú («Conjunto de tablas», «de cúpulas», «de protectores solares») abren la misma clase `TShedFieldForm` con el mismo título («Campo de colectores en cobertizos»): no se distinguen por clase ni por título. En `TPolyFieldForm` la pestaña «Dibujo poli» abre `TOrientationForm` («Gestión de orientaciones») y deja el formulario deshabilitado hasta cerrarlo con Cancelar; su pestaña «Partición» se leyó aparte. Tras la respuesta «No» el menú del editor puede quedar armado (el nombre «Archivo» resaltado) y el siguiente `accion:menu` falla con «El menú 0 … no se abrió»: una pulsación de Alt lo desarma.
- Los campos de estos diálogos incluyen `TUpDown` (flechas de los campos numéricos), que la vista no lee; los números de la tabla cuentan solo los controles legibles.

## No abiertas, solo clasificadas desde el mapa
`MicrosoftWindowsTooltip`, `TBacktrackingForm`, `TShedElecShadingForm`, `TSystemegridDCForm`, `TTranspositionForm`, `TVCManageForm`, `TVoltageOptimiseForm`.
Motivos: requieren un sitio elegido o un proyecto con resultados (botones deshabilitados en un proyecto nuevo), o la función del
catálogo abre otra ventana (`herramientas.factor_transposicion` abre `TArrayCharactForm`, no `TTranspositionForm`).
`MicrosoftWindowsTooltip` no es de PVsyst.

## Límites declarados
- Cobertura básica: sin unidad, rango ni `clave_archivo`; `rellenar` solo existe para las entidades con esquema
  (`components.panels`, `components.inverters`, `components.batteries`, `variante.orientation`, `variante.system`, `proyecto.ventana`).
- Un esquema describe los controles que la ventana muestra en un estado: los que aparecen o desaparecen con otra opción (el tipo de
  campo de la orientación, las casillas del inversor) salen como `presente:false` y sus valores quedan en `sin_mapear`.
- Las clases de control que la vista lee son `TEdit`, `TMemo`, `TLabeledEdit`, `TMaskEdit`, `TSpinEdit`, `TComboBox`,
  `TComboBoxEx`, `TCheckBox` y `TRadioButton`. No lee `TUpDown`, `TListBox`, `TDateTimePicker`, `TTrackBar`, árboles ni cuadrículas.
  `TBuildingObjectForm`, `TGeoSiteForm`, `TOrientManageForm` y `TPreferencesForm` mostraron 0 campos legibles y se pasaron a «solo acciones».
- Solo 150 % (144 DPI). Interfaz en español. Sin licencia no se valida nada de lo guardado.
- Instancias que comparten clase (p. ej. `TChoixObjetPVForm`, 15): la vista devuelve `titulo`; no se colapsan en una entidad por clase
  pero tampoco hay una entidad por título.
