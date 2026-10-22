//! Result-variable template (`.RVT`) structured editor.
//!
//! Provides variable listing, addition, removal, reordering, and catalog
//! querying with validation against real PVsyst 8.1.7 output variables.

mod aplicacion;
mod dominio;
pub(crate) use aplicacion::gestionar_rvt;
#[expect(unused_imports)]
pub(crate) use dominio::{
    anadir_variables, buscar_variable_valida, eliminar_variables, parse_rvt, reordenar_variables,
    serializar_rvt, VARIABLES_VALIDAS,
};
