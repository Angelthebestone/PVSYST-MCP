# Cobertura de la vista de agente en las ventanas de PVsyst (Tarea C)

Registro: `src/vista_agente/cobertura/ventanas/`. El test `should_register_every_window_class_of_the_map_in_exactly_one_state`
recorre las clases de `docs/gui_mapa.json` y falla si alguna no tiene exactamente un estado.

## Estados
1. **Entidad completa** (clave, etiqueta, unidad, rango, `clave_archivo`, tipo, ancla, acceso): **0 ventanas**. Ni siquiera
   `TPVModuleForm` (16 campos en «Datos básicos») lo cumple: unidad y rango no se validan contra un archivo guardado y
   PVsyst en PRUEBA no guarda. Queda «no validado».
2. **Entidad básica** (clave por clase y posición a 96 DPI, valor leído por handle): **64**.
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

## Por clase (71 de 82 abiertas en vivo; 11 solo clasificadas)
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
| `TDomesticUseForm` | variante | 2 | no | — | — | 1 |
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
| `TProjectForm` | variante | 3 | sí (`proyecto.nuevo_conectado_a_la_red`) | 0 | 16 | 4 |
| `TProjectLoadForm` | variante | 2 | sí (`proyecto.cargar`) | 1 | 3 | 2 |
| `TPumpControllerForm` | componentes | 2 | sí (`bases_de_datos.controladores_bombeo.nuevo`) | 15 | 4 | 1 |
| `TPumpForm` | componentes | 2 | sí (`bases_de_datos.bombas.nuevo`) | 20 | 4 | 1 |
| `TResultForm` | resultados | 2 | sí (`variante.resultados_detallados`) | 4 | 11 | 3 |
| `TResultPlotForm` | resultados | 3 | sí (`variante.resultados_detallados.graficos_predefinidos`) | 0 | 4 | 1 |
| `TResultPlotHorForm` | resultados | 2 | sí (`variante.resultados_detallados.graficos_por_hora`) | 27 | 3 | 1 |
| `TResultTabForm` | resultados | 2 | sí (`variante.resultados_detallados.tablas`) | 3 | 2 | 1 |
| `TSAControllerForm` | componentes | 2 | sí (`bases_de_datos.controladores_independientes.nuevo`) | 23 | 4 | 1 |
| `TSettingsForm` | variante | 2 | sí (`ventana_proyecto.proyecto.configuracion`) | 20 | 2 | 1 |
| `TShadingsForm` | variante | 2 | sí (`variante.sombreados_cercanos`) | 6 | 10 | 4 |
| `TShdObjectSelectionForm` | variante | 2 | sí (`escena_3d.herramientas.lista_de_objetos`) | 1 | 2 | 1 |
| `TShedElecShadingForm` | variante | 2 | no | — | — | 1 |
| `TSimulFichForm` | variante | 2 | sí (`variante.simulacion_avanzada.archivo_de_salida`) | 11 | 3 | 1 |
| `TSimulPlotForm` | resultados | 2 | sí (`variante.simulacion_avanzada.graficos_especiales`) | 10 | 2 | 1 |
| `TSimulValHorForm` | resultados | 3 | sí (`variante.simulacion_avanzada.almacen_de_datos_por_hora`) | 0 | 2 | 1 |
| `TSummaryForm` | variante | 2 | sí (`escena_3d.herramientas.resumen_de_escena`) | 1 | 1 | 4 |
| `TSystemCircuitForm` | variante | 2 | sí (`variante.perdidas_detalladas.diagrama_unifilar`) | 5 | 1 | 2 |
| `TSystemPumpingForm` | variante | 2 | no | — | — | 1 |
| `TSystemeAddForm` | variante | 2 | sí (`variante.gestion_energia`) | 6 | 3 | 1 |
| `TSystemeBattForm` | variante | 2 | no | — | — | 1 |
| `TSystemegridDCForm` | variante | 2 | no | — | — | 1 |
| `TSystemegridForm` | variante | 2 | sí (`variante.sistema`) | 22 | 10 | 1 |
| `TTableFieldForm` | meteo | 2 | sí (`escena_3d.crear.generador_fv`) | 7 | 3 | 1 |
| `TThinObjectsForm` | escena_3d | 2 | sí (`escena_3d.herramientas.sombreado_objetos_delgados`) | 3 | 1 | 1 |
| `TToolsForm` | aplicacion | 3 | sí (`inicio.herramientas`) | 0 | 6 | 1 |
| `TTranspositionForm` | variante | 2 | no | — | — | 1 |
| `TUserNeedsForm` | variante | 2 | sí (`variante.autoconsumo`) | 8 | 6 | 2 |
| `TVCManageForm` | variante | 2 | no | — | — | 1 |
| `TVoltageOptimiseForm` | variante | 2 | no | — | — | 1 |
| `TWaterUseForm` | variante | 2 | no | — | — | 1 |
| `TWiringResistanceForm` | variante | 2 | sí (`variante.perdidas_detalladas.computacion_detallada`) | 9 | 4 | 1 |
| `TWorkspaceForm` | aplicacion | 3 | sí (`archivo.espacio_de_trabajo`) | 0 | 1 | 2 |

## Filas de listas y árboles (`gui_actuar accion:fila`)
Medido en vivo con el release (PVsyst 8.1.7, 144 DPI = 150 %, capturas a 96 DPI, 2026-10-28). Sin licencia (PRUEBA); todo se cerró con Cancelar/Cerrar y sin pulsar OK.

| Control | Ventana donde se midió | Geometría a 96 DPI | Filas probadas | Resultado |
|---|---|---|---|---|
| `TVirtualStringTree` | `TProjectLoadForm` (19 proyectos) | cabecera 21 px, fila 19 px, 29 huecos visibles | 0, 3, 18; 19 (hueco vacío); 40 | 0, 3 y 18 verificadas por banda; 19 error («PVsyst resaltó la 0»); 40 error de rango |
| `TListBox` | `TChoixObjetPVForm` de módulos (32 elementos) | borde 2 px, fila 13 px, 33 huecos visibles | 0, 3, 31; 32 (hueco vacío); 33 | 0, 3 y 31 verificadas; 32 error; 33 error de rango |

- La cabecera de la lista de módulos es un `TStringGrid` aparte de una fila: se apunta a la `TListBox`, no a él (sin geometría: error).
- Solo selecciona. La banda de la fila *n* ocupa y∈[cabecera+alto·*n*, cabecera+alto·*(n+1)*] del control.
- El índice son huecos **visibles**: no hay desplazamiento. Una lista con más filas que huecos queda fuera de alcance `[INFERENCIA]`: no se probó.
- Un clic físico puede **activar** el elemento: en la primera medición, un clic sobre la lista de módulos (dy 100, con el foco fuera de la lista) abrió `TPVModuleForm` y cerró el selector; no se supo por qué `[INFERENCIA]`. `fila` comprueba que no apareció ni desapareció ninguna ventana y, antes del clic, da el foco al control: con el foco en un combo el primer clic solo movía el foco y la fila quedaba como estaba (tres de tres aciertos con el foco dado, tras `TAB` en el combo). Repetir el clic no es una salida: dos clics seguidos son un doble clic.
- Qué fila es no se lee: ni UIA ni MSAA dan texto. En módulos, inversores, baterías, controladores, bombas y fabricantes la fila se identifica solo por índice; el orden alfabético no se infiere.
- Con `PVSYST_MCP_ESCRITORIO=aislado` la acción se niega: no está medido. Otro DPI distinto de 144: `[INFERENCIA]`.
- Con otras ventanas que usen estas mismas clases (otros `TChoixObjetPVForm`, 15 instancias) la geometría es la declarada por clase, medida solo en la de módulos `[INFERENCIA]`.

## No abiertas, solo clasificadas desde el mapa
`MicrosoftWindowsTooltip`, `TBacktrackingForm`, `TDomesticUseForm`, `TShedElecShadingForm`, `TSystemPumpingForm`, `TSystemeBattForm`, `TSystemegridDCForm`, `TTranspositionForm`, `TVCManageForm`, `TVoltageOptimiseForm`, `TWaterUseForm`.
Motivos: requieren un sitio elegido o un proyecto con resultados (botones deshabilitados en un proyecto nuevo), o la función del
catálogo abre otra ventana (`herramientas.factor_transposicion` abre `TArrayCharactForm`, no `TTranspositionForm`).
`MicrosoftWindowsTooltip` no es de PVsyst.

## Límites declarados
- Cobertura básica: sin unidad, rango ni `clave_archivo`; `rellenar` solo existe para las entidades con esquema
  (`components.panels`).
- Las clases de control que la vista lee son `TEdit`, `TMemo`, `TLabeledEdit`, `TMaskEdit`, `TSpinEdit`, `TComboBox`,
  `TComboBoxEx`, `TCheckBox` y `TRadioButton`. No lee `TUpDown`, `TListBox`, `TDateTimePicker`, `TTrackBar`, árboles ni cuadrículas.
  `TBuildingObjectForm`, `TGeoSiteForm`, `TOrientManageForm` y `TPreferencesForm` mostraron 0 campos legibles y se pasaron a «solo acciones».
- Solo 150 % (144 DPI). Interfaz en español. Sin licencia no se valida nada de lo guardado.
- Instancias que comparten clase (p. ej. `TChoixObjetPVForm`, 15): la vista devuelve `titulo`; no se colapsan en una entidad por clase
  pero tampoco hay una entidad por título.
