use super::*;

#[test]
fn should_return_message_boxes_and_warnings_logged_after_the_mark() {
    // Given: a PVsyst log with an earlier message, then a mark, then new lines
    let dir = tempfile::tempdir().expect("dir");
    let ruta = dir.path().join("20261020_2337Err.log");
    let antes = "23:37:38|Warn|TEnvManager     |InitLicense         |No license file\n\
                 23:40:00|Norm|ShowMessageBox  |Advertencia         |Old message\n";
    std::fs::write(&ruta, antes).expect("log");
    let marca = Marca {
        ruta: ruta.clone(),
        desde: antes.len() as u64,
    };
    let despues = "23:50:39|Norm|ShowMessageBox  |Advertencia         |You have defined inverters with 4 MPPT inputs, and only 2 Strings\n\
                   \x20       |    |                |                    |The inverters will be largely underutilized\n\
                   23:50:39|Norm|TProjectForm    |SetWarningLabel     |VerifVCalcul result : \n\
                   \x20       |    |                |                    |Error code: SAG_NbStrings_LT_NbMPPT, Severity = Warning\n\
                   23:51:00|Warn|TListeObjetsPV  |ReadObjPV           |Los tamaños de este módulo no están definidos.\n\
                   23:51:01|Warn|TEnvManager     |InitLicense         |No license file\n";
    std::fs::write(&ruta, format!("{antes}{despues}")).expect("log");

    // When
    let mensajes = mensajes_desde(&marca);

    // Then: only the new message box (joined) and the new warning, no licence noise
    assert_eq!(
        mensajes,
        vec![
            "You have defined inverters with 4 MPPT inputs, and only 2 Strings The inverters will be largely underutilized",
            "Los tamaños de este módulo no están definidos.",
        ]
    );
}
