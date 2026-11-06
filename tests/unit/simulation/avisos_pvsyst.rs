use super::*;

#[test]
fn should_keep_pvsyst_error_summary_and_drop_hourly_detail_and_license_noise() {
    // Given: the shape of a real PVsystCLI log after a run with Vmpp below the inverter window
    let log_excerpt = concat!(
        "23:16:40|Norm|TSimulationHandl|ExecSimulation      |LErrorList.Count = 3Incident global = 657 W/m², :The MPP voltage is lower (by 3 V) than the minimum MPP voltage of the inverter.\n",
        "23:16:40|Norm|TSimulationHandl|ExecSimulation      |LErrorList.Count = 4Incident global = 645 W/m², :The MPP voltage is lower (by 3 V) than the minimum MPP voltage of the inverter.\n",
        "23:16:49|Warn|TSimulation     |HandleErrorMessages |Errors during the simulation:\n",
        "        |    |                |                    |ErrorList: 1882 error messages\n",
        "        |    |                |                    |The array Vmpp at 60 °C  is lower than the inverter minimum operating voltage.\n",
        "        |    |                |                    |This may result in some undervoltage loss. \n",
        "        |    |                |                    |\n",
        "        |    |                |                    |1879 occurrences of MPP voltage lower than min. voltage input of the inverter\n",
        "        |    |                |                    |\n",
        "        |    |                |                    |Incident global = 657 W/m², :The MPP voltage is lower (by 3 V) than the minimum MPP voltage of the inverter.\n",
        "23:16:49|Warn|TEnvManager     |InitLicense         |No license file\n",
        "23:16:49|Err |PVsystCLI       |CMD OUTPUT          |The license file \"C:\\ProgramData\\PVsystCLI\\PVsystCLI.lic\" is missing.\n"
    );

    // When
    let avisos = parsear_log(log_excerpt.as_bytes());

    // Then: PVsyst's own summary, in order, and nothing else
    assert_eq!(
        avisos,
        vec![
            "ErrorList: 1882 error messages",
            "The array Vmpp at 60 °C  is lower than the inverter minimum operating voltage.",
            "This may result in some undervoltage loss.",
            "1879 occurrences of MPP voltage lower than min. voltage input of the inverter",
        ]
    );
}
