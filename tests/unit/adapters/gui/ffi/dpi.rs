use crate::adapters::gui::ffi::escalar_a_dpi;

#[test]
fn should_scale_position_when_screen_is_150_percent() {
    assert_eq!(escalar_a_dpi(448, 144), 672);
}
