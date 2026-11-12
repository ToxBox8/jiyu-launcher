// Jiyu Logo: el pulso de Jiyu (16×16 píxeles de GUI) abajo a la izquierda, en el juego y en los menús.
// Este archivo es a la vez la biblioteca que se prueba suelta y el módulo `crate::game::jiyu_logo`.

pub const ID: &str = "jiyu_logo";
pub const NAME: &str = "Jiyu Logo";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "The Jiyu Launcher pulse in the bottom-left corner.";

/// Fondo (sumi) y tinta (shu), en ARGB.
pub const FONDO: u32 = 0xE6_0E0E10;
pub const TINTA: u32 = 0xFF_F4511E;
pub const LADO: f32 = 16.0;
/// Separación del borde de la ventana, en píxeles de GUI.
pub const MARGEN: f32 = 4.0;

/// El dibujo, fila a fila (bit 15 = columna 0).
pub const PULSO: [u16; 16] = [
    0b0000000000000000,
    0b0000000110000000,
    0b0000000110000000,
    0b0011000110001100,
    0b0111000110001110,
    0b0110010110100110,
    0b0110110110110110,
    0b0110110110110110,
    0b0110110110110110,
    0b0110110000110110,
    0b0110011111100110,
    0b0111001111001110,
    0b0011100000011100,
    0b0001111111111000,
    0b0000011111100000,
    0b0000000000000000,
];

/// Tramos horizontales de tinta: (x, y, largo). Menos rectángulos que píxel a píxel.
pub fn tramos() -> Vec<(u8, u8, u8)> {
    let mut v = Vec::new();
    for (y, fila) in PULSO.iter().enumerate() {
        let mut x = 0u8;
        while x < 16 {
            if fila & (0x8000 >> x) != 0 {
                let inicio = x;
                while x < 16 && fila & (0x8000 >> x) != 0 {
                    x += 1;
                }
                v.push((inicio, y as u8, x - inicio));
            } else {
                x += 1;
            }
        }
    }
    v
}

/// Dónde va la esquina de arriba a la izquierda del logo en una GUI de `ancho`×`alto`.
pub fn sitio(_ancho: f32, alto: f32) -> [f32; 2] {
    [MARGEN, alto - LADO - MARGEN]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_dibujo_tiene_su_tinta() {
        let pixeles: u32 = PULSO.iter().map(|f| f.count_ones()).sum();
        let en_tramos: u32 = tramos().iter().map(|t| t.2 as u32).sum();
        assert_eq!(pixeles, en_tramos);
        assert!(pixeles > 60);
    }

    #[test]
    fn abajo_a_la_izquierda() {
        assert_eq!(sitio(480.0, 270.0), [4.0, 250.0]);
    }
}
