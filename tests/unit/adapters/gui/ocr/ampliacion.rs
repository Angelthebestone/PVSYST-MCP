use super::*;

#[test]
fn should_ignore_pixels_outside_region_when_cropping_a_wider_capture() {
    let origen: Vec<u8> = [[10, 10, 10, 255], [250, 250, 250, 255]].concat();
    let imagen = ImagenBgra {
        pixeles: &origen,
        ancho: 2,
        alto: 1,
    };

    let salida = ampliar(imagen, Region { ancho: 1, alto: 1 }, 2);

    assert!(salida.chunks(BYTES_POR_PIXEL).all(|p| p[0] == 10));
}
