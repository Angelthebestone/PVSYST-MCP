//! Encoding and serialization logic for PVsyst modern `.MEF` meteo formats.

use super::dominio::{MefModerno, PresenciaBom, VariableMef};
use crate::error::Result;
use crate::format::CRLF;

pub(crate) const BOM_UTF8: &[u8] = b"\xef\xbb\xbf";

const ESTIMACION_CAPACIDAD_BYTES: usize = 1200;
const BYTES_ESTIMADOS_POR_LINEA: usize = 40;
const ALBEDO_MENSUAL_DEFECTO: &str = "0.20,0.20,0.20,0.20,0.20,0.20,0.20,0.20,0.20,0.20,0.20,0.20,";
const CENTINELA_FECHA_FINAL: &str = "0";

pub(crate) fn encode_mef_moderno(m: &MefModerno) -> Result<Vec<u8>> {
    if let Some(lineas) = m.lineas_raw() {
        let mut out = Vec::with_capacity(lineas.len() * BYTES_ESTIMADOS_POR_LINEA);
        if m.bom() == PresenciaBom::Presente {
            out.extend_from_slice(BOM_UTF8);
        }
        for (idx, l) in lineas.iter().enumerate() {
            out.extend_from_slice(l.as_bytes());
            if idx + 1 < lineas.len() || !l.is_empty() {
                out.extend_from_slice(CRLF.as_bytes());
            }
        }
        return Ok(out);
    }
    generar_mef_moderno_sintetico(m)
}

pub(crate) fn generar_mef_moderno_sintetico(m: &MefModerno) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(ESTIMACION_CAPACIDAD_BYTES);
    out.extend_from_slice(BOM_UTF8);
    out.extend_from_slice(format!("PVObject_=pvMeteoFormat{CRLF}").as_bytes());
    out.extend_from_slice(format!("  Comment={}{CRLF}", m.comentario()).as_bytes());
    out.extend_from_slice(format!("  Version={}{CRLF}", m.version()).as_bytes());
    out.extend_from_slice(format!("  Flags=$C3{CRLF}  DateMode=ReadOnFile{CRLF}").as_bytes());
    out.extend_from_slice(format!("  DateFmtType={}{CRLF}", m.formato_fecha()).as_bytes());
    out.extend_from_slice(
        format!("  FileType=SingleFile{CRLF}  Separ={}{CRLF}", m.separador()).as_bytes(),
    );
    out.extend_from_slice(
        format!(
            "  MeasureStep={}{CRLF}  TypeInterv=3{CRLF}  UsesDST=False{CRLF}",
            m.paso_medicion_min()
        )
        .as_bytes(),
    );
    out.extend_from_slice(
        format!("  DateHEte=01/03/90{CRLF}  DateHHiver=01/11/90{CRLF}").as_bytes(),
    );
    out.extend_from_slice(
        format!("  RefTimeS={}{CRLF}  TypeRE=1{CRLF}", m.tiempo_referencia()).as_bytes(),
    );
    out.extend_from_slice(format!("  AlbedoSolMF={:.1}{CRLF}", m.albedo()).as_bytes());
    out.extend_from_slice(format!("  AlbedoMonMF={ALBEDO_MENSUAL_DEFECTO}{CRLF}").as_bytes());
    out.extend_from_slice(
        format!("  IsAlbedoMon=False{CRLF}  TypeJourFich=1{CRLF}  TypeMoisFich=1{CRLF}  NCarFMask=8{CRLF}")
            .as_bytes(),
    );

    let cols_str: Vec<String> = m.columnas_fecha().iter().map(|c| c.to_string()).collect();
    out.extend_from_slice(
        format!(
            "  NoFieldDate={},{},{CRLF}",
            cols_str.join(","),
            CENTINELA_FECHA_FINAL
        )
        .as_bytes(),
    );
    escribir_variables_ascii(&mut out, m.variables())?;
    out.extend_from_slice(
        format!("  LargChamps{CRLF}End of PVObject pvMeteoFormat{CRLF}").as_bytes(),
    );
    Ok(out)
}

fn escribir_variables_ascii(out: &mut Vec<u8>, variables: &[VariableMef]) -> Result<()> {
    out.extend_from_slice(
        format!(
            "  VarASCIIList, list of={} tVarASCII{CRLF}",
            variables.len()
        )
        .as_bytes(),
    );
    for (i, v) in variables.iter().enumerate() {
        out.extend_from_slice(format!("    VarASCII={}{CRLF}", i + 1).as_bytes());
        out.extend_from_slice(format!("      VarIdent={}{CRLF}", v.ident()).as_bytes());
        out.extend_from_slice(format!("      VarType={}{CRLF}", v.tipo()).as_bytes());
        out.extend_from_slice(format!("      NoField={}{CRLF}", v.columna()).as_bytes());
        out.extend_from_slice(format!("      FieldMult={:.5}{CRLF}", v.multiplicador()).as_bytes());
        out.extend_from_slice(format!("      UnitId={}{CRLF}", v.unidad()).as_bytes());
        out.extend_from_slice(format!("    End of TVarASCII{CRLF}").as_bytes());
    }
    out.extend_from_slice(format!("  End of List VarASCIIList{CRLF}").as_bytes());
    Ok(())
}
