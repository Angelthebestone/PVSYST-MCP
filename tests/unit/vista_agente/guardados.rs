use super::panel;
use crate::vista_agente::esquemas::entidad;

/// A module typed through `rellenar` (a different value in every field) and saved by PVsyst 8.1.8.
const MODULO_GUARDADO: &str =
    include_str!("../../fixtures/vista_agente/Modulo_distintos_8_1_8.PAN");

/// The same window with its size typed (`Dimensiones y tecnología`, in mm and kg) and saved by PVsyst 8.1.8.
const MODULO_CON_DIMENSIONES: &str =
    include_str!("../../fixtures/vista_agente/Modulo_con_dimensiones_8_1_8.PAN");

#[test]
fn should_find_every_file_key_of_the_panel_schema_in_the_module_pvsyst_saved() {
    let claves: Vec<&str> = MODULO_CON_DIMENSIONES
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let con_clave = panel().campos.iter().filter_map(|c| c.clave_archivo);
    for clave in con_clave {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

#[test]
fn should_keep_the_units_the_saved_module_shows_for_the_values_typed() {
    let valor = |k: &str| {
        MODULO_GUARDADO
            .lines()
            .find_map(|l| l.trim().strip_prefix(&format!("{k}=")).map(str::to_string))
            .and_then(|v| v.trim().parse::<f64>().ok())
    };
    // The values `rellenar` typed: W, A, V, mA/°C, °C and a count.
    let tecleado = [
        ("PNom", 410.5),
        ("Isc", 11.2),
        ("Voc", 48.3),
        ("Imp", 10.4),
        ("Vmp", 39.5),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(valor(clave), Some(esperado), "{clave}");
    }
    assert_eq!(valor("muISC"), Some(4.2));
    assert_eq!(valor("TRef"), Some(20.0));
    assert_eq!(valor("NCelS"), Some(72.0));
}

fn valor_guardado(archivo: &str, clave: &str) -> Option<f64> {
    archivo
        .lines()
        .find_map(|l| {
            l.trim()
                .strip_prefix(&format!("{clave}="))
                .map(str::to_string)
        })
        .and_then(|v| v.trim().parse::<f64>().ok())
}

/// A battery typed through `rellenar` and saved by PVsyst 8.1.8.
const BATERIA_GUARDADA: &str =
    include_str!("../../fixtures/vista_agente/Bateria_distintos_8_1_8.BTR");

#[test]
fn should_keep_the_scale_the_saved_battery_uses_for_the_values_typed() {
    // Typed: 6 cells, 2.1 V, 150.5 Ah, 25 °C, 3.5 mOhm (stored in ohm) and 95.5 % (stored as a fraction).
    let tecleado = [
        ("NCellSerie", 6.0),
        ("VNomCell", 2.1),
        ("CapNomC10", 150.5),
        ("BattTempRef", 25.0),
        ("ResInterne", 0.0035),
        ("EfficI", 0.955),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(
            valor_guardado(BATERIA_GUARDADA, clave),
            Some(esperado),
            "{clave}"
        );
    }
}

/// The orientation and the array layout typed through `rellenar` and saved by PVsyst 8.1.8 with
/// «Guardar la variante activa» (tilt 25, azimuth 10; `proyecto dimensionar` for 50 kWp).
const VARIANTE_GUARDADA: &str =
    include_str!("../../fixtures/vista_agente/Variante_orientacion_sistema_8_1_8.VC0");

#[test]
fn should_find_every_file_key_of_the_orientation_and_system_schemas_in_the_variant_pvsyst_saved() {
    let claves: Vec<&str> = VARIANTE_GUARDADA
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    for entidad_clave in ["variante.orientation", "variante.system"] {
        let esquema = entidad(entidad_clave).expect("schema");
        for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
            assert!(
                claves.contains(&clave),
                "{clave} no está en la variante guardada"
            );
        }
    }
}

/// 1722 x 1134 x 30 mm and 21.5 kg typed in the window: the file keeps the size in metres.
#[test]
fn should_save_the_module_size_typed_in_millimetres_as_metres() {
    let esperado = [
        ("Height", 1.722),
        ("Width", 1.134),
        ("Depth", 0.03),
        ("Weight", 21.5),
    ];
    for (clave, valor) in esperado {
        assert_eq!(
            valor_guardado(MODULO_CON_DIMENSIONES, clave),
            Some(valor),
            "{clave}"
        );
    }
}
