use super::panel;
use crate::vista_agente::esquemas::entidad;

/// A module typed through `rellenar` (a different value in every field) and saved by PVsyst 8.1.8.
const MODULO_GUARDADO: &str =
    include_str!("../../fixtures/vista_agente/Modulo_distintos_8_1_8.PAN");

#[test]
fn should_find_every_file_key_of_the_panel_schema_in_the_module_pvsyst_saved() {
    let claves: Vec<&str> = MODULO_GUARDADO
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

/// An inverter typed through `rellenar` and saved by PVsyst 8.1.8.
const INVERSOR_GUARDADO: &str =
    include_str!("../../fixtures/vista_agente/Inversor_distintos_8_1_8.OND");

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

#[test]
fn should_find_every_file_key_of_the_inverter_schema_in_the_inverter_pvsyst_saved() {
    let claves: Vec<&str> = INVERSOR_GUARDADO
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let esquema = entidad("components.inverters").expect("inverter schema");
    for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

#[test]
fn should_keep_the_units_the_saved_inverter_shows_for_the_values_typed() {
    // Volts and kilowatts, as typed by `rellenar`.
    let tecleado = [
        ("VMppMin", 310.0),
        ("VmppNom", 600.0),
        ("VMPPMax", 820.0),
        ("VAbsMax", 1000.0),
        ("VOutConv", 400.0),
        ("PNomConv", 20.5),
        ("PMaxOUT", 22.5),
        ("PSeuil", 0.1),
    ];
    for (clave, esperado) in tecleado {
        assert_eq!(
            valor_guardado(INVERSOR_GUARDADO, clave),
            Some(esperado),
            "{clave}"
        );
    }
}

/// A battery typed through `rellenar` and saved by PVsyst 8.1.8.
const BATERIA_GUARDADA: &str =
    include_str!("../../fixtures/vista_agente/Bateria_distintos_8_1_8.BTR");

#[test]
fn should_find_every_file_key_of_the_battery_schema_in_the_battery_pvsyst_saved() {
    let claves: Vec<&str> = BATERIA_GUARDADA
        .lines()
        .filter_map(|l| l.trim().split_once('=').map(|(k, _)| k))
        .collect();
    let esquema = entidad("components.batteries").expect("battery schema");
    for clave in esquema.campos.iter().filter_map(|c| c.clave_archivo) {
        assert!(
            claves.contains(&clave),
            "{clave} no está en el archivo guardado"
        );
    }
}

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

#[test]
fn should_keep_the_values_the_saved_variant_shows_for_what_was_typed() {
    for (clave, tecleado) in [
        ("FieldTilt", 25.0),
        ("FieldAzim", 10.0),
        ("NModSerie", 28.0),
        ("NStringCh", 4.0),
        ("NInverter", 1.0),
    ] {
        assert_eq!(
            valor_guardado(VARIANTE_GUARDADA, clave),
            Some(tecleado),
            "{clave}"
        );
    }
}
