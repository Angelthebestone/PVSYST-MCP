//! Argument descriptions of `proyecto`.

use super::{entrada, Entrada};

pub(super) const TABLA: &[Entrada] = &[
    entrada(
        "ancho_colector_m",
        "Ancho del colector del seguidor, m (defecto 4.25); menor que pitch_m.",
        "Tracker collector width, m (default 4.25); smaller than pitch_m.",
    ),
    entrada(
        "angulo_max",
        "Inclinación máxima del seguidor, ° (0-90; defecto 60).",
        "Tracker maximum tilt, ° (0-90; default 60).",
    ),
    entrada(
        "azimut_eje",
        "Azimut del eje del seguidor, ° (-180 a 180); si falta, se conserva el del archivo.",
        "Tracker axis azimuth, ° (-180 to 180); if omitted, the file's own is kept.",
    ),
    entrada(
        "backtracking",
        "true (defecto): el seguidor hace backtracking (UsesBacktracking).",
        "true (default): the tracker backtracks (UsesBacktracking).",
    ),
    entrada(
        "caudal_m3_dia",
        "Caudal de agua de 'bombeo', m³/día (máx. 10000); se escribe sin convertir en FlowYear.",
        "Water flow for 'bombeo', m³/day (max 10000); written unconverted to FlowYear.",
    ),
    entrada(
        "inflacion",
        "Inflación anual para 'economia', fracción 0-1 (no %).",
        "Annual inflation for 'economia', fraction 0-1 (not %).",
    ),
    entrada(
        "kwp",
        "Potencia DC objetivo de 'dimensionar', en kWp.",
        "DC target power for 'dimensionar', in kWp.",
    ),
    entrada(
        "lid_pct",
        "Pérdida por LID, % (0-20); se escribe en cada pvSubArray.",
        "LID loss, % (0-20); written to every pvSubArray.",
    ),
    entrada(
        "mismatch_pct",
        "Pérdida por mismatch, % (0-20); escribe PMismMPP de cada subarray.",
        "Mismatch loss, % (0-20); writes each subarray's PMismMPP.",
    ),
    entrada(
        "n_filas",
        "Filas de seguidores (NSheds; defecto 62, al menos 1).",
        "Tracker rows (NSheds; default 62, at least 1).",
    ),
    entrada(
        "n_inversores",
        "Inversores que escribe 'dimensionar' (NInverter); si falta, el de la variante.",
        "Inverters 'dimensionar' writes (NInverter); if omitted, the variant's.",
    ),
    entrada(
        "n_mod_serie",
        "Módulos en serie por cadena (NModSerie, 1-200); en 'crear' y 'dimensionar'.",
        "Modules in series per string (NModSerie, 1-200); in 'crear' and 'dimensionar'.",
    ),
    entrada(
        "n_paralelo",
        "Ramas de baterías en paralelo de 'almacenamiento' (NBattParall; defecto 20).",
        "Parallel battery strings in 'almacenamiento' (NBattParall; default 20).",
    ),
    entrada(
        "n_serie",
        "Baterías en serie del banco de 'almacenamiento' (NBattSerie; defecto 10).",
        "Batteries in series in the 'almacenamiento' bank (NBattSerie; default 10).",
    ),
    entrada(
        "n_strings",
        "Cadenas en paralelo (NStringCh y NStrOrient1); en 'crear' y 'dimensionar'.",
        "Strings in parallel (NStringCh and NStrOrient1); in 'crear' and 'dimensionar'.",
    ),
    entrada(
        "p_max_carga_kw",
        "Potencia máxima de carga de la batería, kW (defecto 380).",
        "Battery maximum charge power, kW (default 380).",
    ),
    entrada(
        "p_max_descarga_kw",
        "Potencia máxima de descarga de la batería, kW (defecto 170).",
        "Battery maximum discharge power, kW (default 170).",
    ),
    entrada(
        "pasos",
        "Pasos de 'configurar': objetos {accion:...} en orden, validados antes de aplicar.",
        "'configurar' steps: {accion:...} objects in order, all validated before running.",
    ),
    entrada(
        "perdida_separada",
        "true: el recorte por límite de inyección se registra como pérdida aparte.",
        "true: injection-limit clipping is booked as a separate loss.",
    ),
    entrada(
        "pitch_m",
        "Paso entre ejes de seguidores, m (defecto 8); debe superar ancho_colector_m.",
        "Spacing between tracker axes, m (default 8); must exceed ancho_colector_m.",
    ),
    entrada(
        "pnom_modulo",
        "Potencia nominal del módulo, W (máx. 3000), en lugar de la del .PAN.",
        "Module nominal power, W (max 3000), instead of the .PAN value.",
    ),
    entrada(
        "profundidad_estatica_m",
        "Profundidad estática del agua, m (máx. 1000); se escribe negativa.",
        "Static water depth, m (max 1000); written as a negative value.",
    ),
    entrada(
        "sitio",
        "Sitio que 'crear' vincula al proyecto: nombre o Sites/X.SIT existente.",
        "Site 'crear' links to the project: a name or an existing Sites/X.SIT.",
    ),
    entrada(
        "soc_inicial",
        "Carga inicial de la batería, fracción 0-1 (defecto 0.6).",
        "Initial battery charge, fraction 0-1 (default 0.6).",
    ),
    entrada(
        "suciedad_pct",
        "Pérdida por suciedad, % de la irradiancia (0-50).",
        "Soiling loss, % of irradiance (0-50).",
    ),
    entrada(
        "tan_phi",
        "tan(φ) de la reactiva en el límite de inyección (-1 a 1).",
        "tan(φ) of reactive power at the injection limit (-1 to 1).",
    ),
    entrada(
        "tarifa_venta_kwh",
        "Tarifa de venta por kWh en la moneda del proyecto (SummerPeakSaleTariff).",
        "Sale tariff per kWh in the project currency (SummerPeakSaleTariff).",
    ),
    entrada(
        "tasa_descuento",
        "Tasa de descuento para 'economia', fracción 0-1 (no %).",
        "Discount rate for 'economia', fraction 0-1 (not %).",
    ),
    entrada(
        "uc_termico",
        "Coeficiente de pérdidas térmicas U, W/m²K (0-100).",
        "Thermal loss coefficient U, W/m²K (0-100).",
    ),
    entrada(
        "vida_anios",
        "Vida del proyecto, años (1-100); escribe ProjectLife.",
        "Project life, years (1-100); writes ProjectLife.",
    ),
];
