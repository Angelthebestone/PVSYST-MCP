use crate::adapters::gui::fila::error::ErrorFila;
use crate::adapters::gui::fila::registro::{geometria_de, Control};

const LISTA_DE_MODULOS: Control<'static> = Control {
    clase: "TListBox",
    anfitrion: "TChoixObjetPVForm",
    titulo: "Eligiendo un módulo FV",
};

#[test]
fn should_refuse_a_list_of_another_chooser_because_only_the_module_list_was_measured() {
    let inversores = Control {
        titulo: "Eligiendo un inversor de red",
        ..LISTA_DE_MODULOS
    };
    assert!(matches!(
        geometria_de(&inversores),
        Err(ErrorFila::ClaseSinGeometria { .. })
    ));
}
