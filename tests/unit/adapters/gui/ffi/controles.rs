use super::{escalar_a_dpi, DPI_BASE};

#[test]
fn should_keep_position_when_dpi_is_base() {
    assert_eq!(escalar_a_dpi(113, DPI_BASE), 113);
}

#[test]
fn should_scale_position_when_screen_is_125_percent() {
    assert_eq!(escalar_a_dpi(100, 120), 125);
}

#[test]
fn should_scale_position_when_screen_is_150_percent() {
    assert_eq!(escalar_a_dpi(448, 144), 672);
}

#[test]
fn should_not_overflow_when_position_and_dpi_are_extreme() {
    assert_eq!(escalar_a_dpi(i32::MAX, 384), i32::MAX);
}

#[test]
fn should_scale_negative_offsets_symmetrically() {
    assert_eq!(escalar_a_dpi(-96, 144), -144);
}
