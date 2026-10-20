use super::*;

#[test]
fn job_ids_count_per_instance() {
    let a = Estado::default();
    let b = Estado::default();
    assert_eq!(a.siguiente_trabajo(), "sim-1");
    assert_eq!(a.siguiente_trabajo(), "sim-2");
    assert_eq!(b.siguiente_trabajo(), "sim-1");
}
