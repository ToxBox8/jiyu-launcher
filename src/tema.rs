//! Los cuatro temas de la maqueta (Prism Dark, Prism Bright, Matugen y Jiyu Pulse) y los estilos de cada pieza.
//! Los colores salen de `maqueta/estilo.css`; si cambian allí, se cambian aquí.

use iced::widget::{button, container, pick_list, scrollable, text_input};
use iced::{border, Background, Border, Color, Shadow, Theme};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Tema {
    #[default]
    Matugen,
    PrismDark,
    PrismBright,
    Jiyu,
}

impl Tema {
    pub const TODOS: [Tema; 4] = [Tema::Matugen, Tema::PrismDark, Tema::PrismBright, Tema::Jiyu];

    pub fn nombre(self) -> &'static str {
        match self {
            Tema::Matugen => "Matugen",
            Tema::PrismDark => "Prism Dark",
            Tema::PrismBright => "Prism Bright",
            Tema::Jiyu => "Jiyu Pulse",
        }
    }

    pub fn detalle(self) -> &'static str {
        match self {
            Tema::Matugen => "El tuyo de Prism: negro y lavanda",
            Tema::PrismDark => "El oscuro de Prism (Breeze)",
            Tema::PrismBright => "El claro de Prism",
            Tema::Jiyu => "Sumi y shu, con la mascota",
        }
    }

    pub fn paleta(self) -> Paleta {
        match self {
            Tema::PrismDark => Paleta {
                win: hex(0x31363b),
                chrome: hex(0x2a2e32),
                base: hex(0x232629),
                panel: hex(0x2c3035),
                text: hex(0xeff0f1),
                dim: hex(0xa3abb3),
                line: hex(0x4b5157),
                line_soft: hex(0x3a3f44),
                btn: hex(0x3b4045),
                btn_hover: hex(0x454b51),
                btn_press: hex(0x2f3337),
                hl: hex(0x3daee9),
                on_hl: hex(0x06141c),
                hl_tint: Color::from_rgba8(61, 174, 233, 0.22),
                link: hex(0x5cbcee),
                rust: hex(0xe08a4c),
                java: hex(0x7fa8da),
                on_badge: hex(0x16181a),
                ok: hex(0x5fbf6b),
                mal: hex(0xe0625a),
                r_sm: 3.0,
                r_md: 4.0,
                r_tile: 4.0,
                jiyu: false,
            },
            Tema::PrismBright => Paleta {
                win: hex(0xeff0f1),
                chrome: hex(0xe3e5e7),
                base: hex(0xfcfcfc),
                panel: hex(0xf4f5f6),
                text: hex(0x232629),
                dim: hex(0x5a6066),
                line: hex(0xbcc0c4),
                line_soft: hex(0xd8dbde),
                btn: hex(0xf7f8f8),
                btn_hover: hex(0xffffff),
                btn_press: hex(0xe1e4e6),
                hl: hex(0x3daee9),
                on_hl: hex(0x06141c),
                hl_tint: Color::from_rgba8(61, 174, 233, 0.20),
                link: hex(0x1f7fb5),
                rust: hex(0xb75a1f),
                java: hex(0x2f6db3),
                on_badge: hex(0xffffff),
                ok: hex(0x2e8b3d),
                mal: hex(0xb3261e),
                r_sm: 3.0,
                r_md: 4.0,
                r_tile: 4.0,
                jiyu: false,
            },
            Tema::Matugen => Paleta {
                win: hex(0x000000),
                chrome: hex(0x07070c),
                base: hex(0x000000),
                panel: hex(0x06060b),
                text: hex(0xe4ddff),
                dim: hex(0xa39dc3),
                line: hex(0x2b2944),
                line_soft: hex(0x1a1930),
                btn: hex(0x0d0c18),
                btn_hover: hex(0x17152a),
                btn_press: hex(0x050509),
                hl: hex(0x7676b0),
                on_hl: hex(0x000000),
                hl_tint: Color::from_rgba8(118, 118, 176, 0.26),
                link: hex(0xafd6ff),
                rust: hex(0xf2aa76),
                java: hex(0xafd6ff),
                on_badge: hex(0x000000),
                ok: hex(0x9ad8a4),
                mal: hex(0xff8f8f),
                r_sm: 3.0,
                r_md: 5.0,
                r_tile: 5.0,
                jiyu: false,
            },
            Tema::Jiyu => Paleta {
                win: hex(0x0e0e10),
                chrome: hex(0x0e0e10),
                base: hex(0x121215),
                panel: hex(0x16161a),
                text: hex(0xf3efe6),
                dim: hex(0xa8a39a),
                line: hex(0x2e2b28),
                line_soft: hex(0x211f1d),
                btn: hex(0x1d1c1f),
                btn_hover: hex(0x262428),
                btn_press: hex(0x141316),
                hl: hex(0xf4511e),
                // Regla de la marca: sobre shu siempre va sumi, nunca blanco.
                on_hl: hex(0x0e0e10),
                hl_tint: hex(0x3a1a10),
                link: hex(0xff6b3d),
                rust: hex(0xe3a93b),
                java: hex(0x8fb3d9),
                on_badge: hex(0x0e0e10),
                ok: hex(0x9fcf8a),
                mal: hex(0xff6b3d),
                r_sm: 8.0,
                r_md: 14.0,
                r_tile: 16.0,
                jiyu: true,
            },
        }
    }

    pub fn iced(self) -> Theme {
        let p = self.paleta();
        Theme::custom(
            self.nombre().to_string(),
            iced::theme::Palette {
                background: p.base,
                text: p.text,
                primary: p.hl,
                success: p.ok,
                danger: p.mal,
            },
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Paleta {
    pub win: Color,
    pub chrome: Color,
    pub base: Color,
    pub panel: Color,
    pub text: Color,
    pub dim: Color,
    pub line: Color,
    pub line_soft: Color,
    pub btn: Color,
    pub btn_hover: Color,
    pub btn_press: Color,
    pub hl: Color,
    pub on_hl: Color,
    pub hl_tint: Color,
    pub link: Color,
    pub rust: Color,
    pub java: Color,
    pub on_badge: Color,
    pub ok: Color,
    pub mal: Color,
    pub r_sm: f32,
    pub r_md: f32,
    pub r_tile: f32,
    pub jiyu: bool,
}

pub fn hex(h: u32) -> Color {
    Color::from_rgb8((h >> 16) as u8, (h >> 8) as u8, h as u8)
}

/// Mezcla `a` sobre `b` con opacidad `t` (lo que hace `color-mix` en la maqueta).
pub fn mezcla(a: Color, b: Color, t: f32) -> Color {
    Color {
        r: a.r * t + b.r * (1.0 - t),
        g: a.g * t + b.g * (1.0 - t),
        b: a.b * t + b.b * (1.0 - t),
        a: 1.0,
    }
}

fn borde(color: Color, ancho: f32, radio: f32) -> Border {
    Border { color, width: ancho, radius: border::radius(radio) }
}

// ---------- Botones ----------

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Boton {
    /// Botón normal de Prism: relieve suave y borde.
    Normal,
    /// La acción principal (Jugar, Instalar, Crear): fondo de resalte.
    Primario,
    /// Botón de la barra de herramientas: plano hasta que pasas por encima.
    Barra,
    /// Eliminar y similares.
    Peligro,
    /// Página de un diálogo (lista de la izquierda) o fuente de «Nueva instancia».
    Pagina(bool),
    /// Fila de una tabla (mods, versiones). Seleccionada NO usa el mismo color que el botón principal:
    /// lleva el tinte suave y un borde de resalte, y el texto sigue siendo el normal.
    Fila(bool),
    /// Casilla de instancia en la cuadrícula.
    Casilla(bool),
    /// Opción de un selector segmentado o tarjeta de preset/tema.
    Opcion(bool),
    /// Texto que se comporta como enlace.
    Enlace,
    /// Opción de un menú contextual. `true` = resalta al pasar por encima.
    Menu(bool),
}

pub fn boton(p: Paleta, tipo: Boton) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| {
        use button::Status::*;
        let encima = matches!(estado, Hovered);
        let pulsado = matches!(estado, Pressed);
        let apagado = matches!(estado, Disabled);
        let mut s = match tipo {
            Boton::Normal => button::Style {
                background: Some(Background::Color(if pulsado {
                    p.btn_press
                } else if encima {
                    p.btn_hover
                } else {
                    p.btn
                })),
                text_color: p.text,
                border: borde(p.line, 1.0, p.r_sm),
                shadow: Shadow::default(),
            },
            Boton::Primario => button::Style {
                background: Some(Background::Color(if pulsado {
                    mezcla(p.hl, p.base, 0.82)
                } else if encima {
                    mezcla(p.hl, p.text, 0.88)
                } else {
                    p.hl
                })),
                text_color: p.on_hl,
                border: borde(Color::TRANSPARENT, 0.0, if p.jiyu { 999.0 } else { p.r_sm }),
                shadow: Shadow::default(),
            },
            Boton::Barra => button::Style {
                background: if pulsado {
                    Some(Background::Color(p.btn_press))
                } else if encima {
                    Some(Background::Color(p.btn_hover))
                } else {
                    None
                },
                text_color: p.text,
                border: borde(if encima { p.line_soft } else { Color::TRANSPARENT }, 1.0, p.r_sm),
                shadow: Shadow::default(),
            },
            Boton::Peligro => button::Style {
                background: Some(Background::Color(if encima { p.btn_hover } else { p.btn })),
                text_color: if encima { p.mal } else { p.text },
                border: borde(if encima { p.mal } else { p.line }, 1.0, p.r_sm),
                shadow: Shadow::default(),
            },
            Boton::Pagina(sel) | Boton::Opcion(sel) => button::Style {
                background: Some(Background::Color(if sel {
                    p.hl_tint
                } else if encima {
                    p.btn_hover
                } else {
                    Color::TRANSPARENT
                })),
                text_color: p.text,
                border: borde(
                    if sel {
                        p.hl
                    } else if matches!(tipo, Boton::Opcion(_)) {
                        p.line
                    } else {
                        Color::TRANSPARENT
                    },
                    1.0,
                    p.r_sm,
                ),
                shadow: Shadow::default(),
            },
            Boton::Fila(sel) => button::Style {
                background: Some(Background::Color(if sel {
                    p.hl_tint
                } else if encima {
                    p.btn_hover
                } else {
                    Color::TRANSPARENT
                })),
                text_color: p.text,
                border: borde(if sel { p.hl } else { Color::TRANSPARENT }, 1.0, p.r_sm),
                shadow: Shadow::default(),
            },
            Boton::Casilla(sel) => button::Style {
                background: Some(Background::Color(if sel {
                    p.hl_tint
                } else if encima {
                    p.btn_hover
                } else {
                    Color::TRANSPARENT
                })),
                text_color: p.text,
                border: borde(if sel { p.hl } else { Color::TRANSPARENT }, 1.0, p.r_tile),
                shadow: Shadow::default(),
            },
            Boton::Menu(resalta) => button::Style {
                background: if resalta && (encima || pulsado) { Some(Background::Color(p.hl_tint)) } else { None },
                text_color: p.text,
                border: borde(Color::TRANSPARENT, 0.0, p.r_sm),
                shadow: Shadow::default(),
            },
            Boton::Enlace => button::Style {
                background: None,
                text_color: if encima { p.text } else { p.link },
                border: Border::default(),
                shadow: Shadow::default(),
            },
        };
        if apagado {
            s.text_color = Color { a: 0.45, ..s.text_color };
            if let Some(Background::Color(c)) = s.background {
                s.background = Some(Background::Color(Color { a: c.a * 0.5, ..c }));
            }
        }
        s
    }
}

// ---------- Contenedores ----------

pub fn fondo(color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style { background: Some(Background::Color(color)), ..Default::default() }
}

/// Recuadro con borde (las cajas de Prism).
pub fn caja(p: Paleta) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.base)),
        border: borde(p.line, 1.0, p.r_md),
        text_color: Some(p.text),
        ..Default::default()
    }
}

/// Aviso con filo de color (los `callout` de la maqueta).
pub fn aviso(p: Paleta, color: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(mezcla(color, p.base, 0.09))),
        border: borde(mezcla(color, p.base, 0.5), 1.0, p.r_md),
        text_color: Some(p.text),
        ..Default::default()
    }
}

/// Píldora rellena (insignia RUST / JAVA, chip de actualización).
pub fn pildora(color: Color, texto: Color) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: borde(Color::TRANSPARENT, 0.0, 999.0),
        text_color: Some(texto),
        ..Default::default()
    }
}

/// Ventana de diálogo sobre la cortina.
pub fn dialogo(p: Paleta) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(p.win)),
        border: borde(p.line, 1.0, if p.jiyu { 16.0 } else { 6.0 }),
        text_color: Some(p.text),
        // Sin sombra difuminada: con el dibujo por software se repinta encima del contenido al refrescar
        // solo una parte, y además cuesta CPU. La cortina ya separa el diálogo del fondo.
        shadow: Shadow::default(),
    }
}

// ---------- Campos ----------

pub fn campo(p: Paleta) -> impl Fn(&Theme, text_input::Status) -> text_input::Style {
    move |_, estado| text_input::Style {
        background: Background::Color(p.base),
        border: borde(
            if matches!(estado, text_input::Status::Focused) { p.hl } else { p.line },
            1.0,
            p.r_sm,
        ),
        icon: p.dim,
        placeholder: p.dim,
        value: p.text,
        selection: Color { a: 0.45, ..p.hl },
    }
}

pub fn lista(p: Paleta) -> impl Fn(&Theme, pick_list::Status) -> pick_list::Style {
    move |_, estado| pick_list::Style {
        text_color: p.text,
        placeholder_color: p.dim,
        handle_color: p.dim,
        background: Background::Color(if matches!(estado, pick_list::Status::Hovered) {
            p.btn_hover
        } else {
            p.btn
        }),
        border: borde(p.line, 1.0, p.r_sm),
    }
}

pub fn desplazable(p: Paleta) -> impl Fn(&Theme, scrollable::Status) -> scrollable::Style {
    move |tema, estado| {
        let mut s = scrollable::default(tema, estado);
        let barra = scrollable::Rail {
            background: None,
            border: Border::default(),
            scroller: scrollable::Scroller {
                color: p.line,
                border: borde(Color::TRANSPARENT, 0.0, 4.0),
            },
        };
        s.vertical_rail = barra;
        s.horizontal_rail = barra;
        s
    }
}

/// Recuadro relleno de un color con las esquinas del tema (logo de un mod sin imagen).
pub fn pildora_cuadrada(color: Color, radio: f32) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(color)),
        border: borde(Color::TRANSPARENT, 0.0, radio),
        ..Default::default()
    }
}

/// Muestra de color pulsable (selector de icono).
pub fn boton_color(color: Color, linea: Color) -> impl Fn(&Theme, button::Status) -> button::Style {
    move |_, estado| button::Style {
        background: Some(Background::Color(color)),
        text_color: Color::WHITE,
        border: borde(if matches!(estado, button::Status::Hovered) { Color::WHITE } else { linea }, 2.0, 6.0),
        shadow: Shadow::default(),
    }
}

/// Zona donde se puede soltar una instancia: se marca con el borde de resalte mientras arrastras encima.
pub fn soltar(p: Paleta, activo: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: activo.then_some(Background::Color(p.hl_tint)),
        border: borde(if activo { p.hl } else { Color::TRANSPARENT }, if activo { 2.0 } else { 0.0 }, p.r_md),
        ..Default::default()
    }
}

/// Fondo de una casilla de instancia: elegida (tinte y borde) o con el ratón encima.
pub fn casilla_fondo(p: Paleta, sel: bool, encima: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if sel {
            p.hl_tint
        } else if encima {
            p.btn_hover
        } else {
            Color::TRANSPARENT
        })),
        border: borde(if sel { p.hl } else { Color::TRANSPARENT }, 1.0, p.r_tile),
        text_color: Some(p.text),
        ..Default::default()
    }
}

/// Fondo de una fila de la lista de instancias.
pub fn fila_fondo(p: Paleta, sel: bool, encima: bool) -> impl Fn(&Theme) -> container::Style {
    move |_| container::Style {
        background: Some(Background::Color(if sel {
            p.hl_tint
        } else if encima {
            p.btn_hover
        } else {
            Color::TRANSPARENT
        })),
        border: borde(if sel { p.hl } else { Color::TRANSPARENT }, 1.0, p.r_sm),
        text_color: Some(p.text),
        ..Default::default()
    }
}
