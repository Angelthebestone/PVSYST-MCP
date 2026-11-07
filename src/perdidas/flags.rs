//! `pvSubArray.Flags`: PVsyst's "use the default value" bits of the detailed-losses
//! window. While a bit is set PVsyst ignores the field it guards, so writing the
//! field alone changes nothing in the simulation.

/// Bit PVsyst reads as "mismatch is predefined", which makes it ignore `PMismMPP`.
/// Measured on 8.1.8 through the GUI: `$001E0002` (set) -> `$019A0002` (cleared)
/// -> `$019E0002` (set again, mismatch back to 2.00 %).
const BIT_MISMATCH_PREDEFINIDO: u32 = 0x0004_0000;

/// What an explicit `mismatch_pct` has to do to one `pvSubArray.Flags`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FlagsMismatch {
    /// The bit was set: write this value so PVsyst reads `PMismMPP`.
    Limpiar(String),
    /// No bit to clear: `PMismMPP` is honoured as written.
    YaLimpio,
    /// Not a `$`-hex value: the bit cannot be cleared without guessing.
    Desconocido,
}

/// [`FlagsMismatch`] for the current `Flags` text of a subarray.
pub(crate) fn al_escribir_mismatch(flags: Option<&str>) -> FlagsMismatch {
    let Some(digitos) = flags.and_then(|f| f.trim().strip_prefix('$')) else {
        return FlagsMismatch::Desconocido;
    };
    let Ok(valor) = u32::from_str_radix(digitos, 16) else {
        return FlagsMismatch::Desconocido;
    };
    let limpio = valor & !BIT_MISMATCH_PREDEFINIDO;
    if limpio == valor {
        return FlagsMismatch::YaLimpio;
    }
    let ancho = digitos.len().max(2);
    FlagsMismatch::Limpiar(format!("${limpio:0ancho$X}"))
}
