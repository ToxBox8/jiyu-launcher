//! Big Picture: el launcher a pantalla completa para jugar desde el sofá, como el modo de Steam.
//! Arriba (fijo) la instancia elegida en grande con su botón de jugar; debajo, estantes («Seguir
//! jugando» y uno por carpeta) que se recorren con las flechas. Intro juega, E edita, Esc o F11 sale.
//!
//! Pensado para el dibujo por software: todo tiene tamaño fijo, los fondos son lisos y no hay nada
//! desplazable. Solo se construyen los estantes y las tarjetas que caben en pantalla; la «ventana» de
//! lo visible se mueve cuando la tarjeta elegida se sale de ella. Así, al moverse, solo se repintan
//! las dos tarjetas que cambian y la cabecera, y nada se dibuja fuera de su sitio.

use super::*;
use iced::widget::{column, mouse_area, row};
use iced::{Background, Color, Size};

pub const TARJETA: f32 = 176.0;
const ALTO_TARJETA: f32 = 224.0;
pub const HUECO: f32 = 18.0;
pub const MARGEN: f32 = 48.0;
/// Alto de un estante: título, tarjetas y el aire de debajo.
pub const ALTO_ESTANTE: f32 = 300.0;
const ALTO_CABECERA: f32 = 62.0;
const ALTO_HEROE: f32 = 236.0;
const ALTO_PIE: f32 = 54.0;
/// Lo que no es la zona de estantes (para saber cuánto se ve de ella).
pub const ALTO_FIJO: f32 = ALTO_CABECERA + ALTO_HEROE + ALTO_PIE + 2.0 + 32.0;

/// Cuántos estantes y cuántas tarjetas por estante caben enteros en la ventana.
pub fn cabe(ventana: Size) -> (usize, usize) {
    let filas = ((ventana.height - ALTO_FIJO) / ALTO_ESTANTE).floor().max(1.0) as usize;
    let cols = ((ventana.width - 2.0 * MARGEN + HUECO) / (TARJETA + HUECO)).floor().max(1.0) as usize;
    (filas, cols)
}

/// Flecha que avisa de que hay más tarjetas por ese lado.
fn mas<'a>(p: &Paleta, n: usize, izquierda: bool) -> E<'a> {
    let t = if n == 0 {
        String::new()
    } else if izquierda {
        format!("‹ {n}")
    } else {
        format!("{n} ›")
    };
    container(text(t).size(15).font(SEMI).color(p.dim)).width(MARGEN).height(ALTO_TARJETA).center(MARGEN).into()
}

fn corto(t: &str, n: usize) -> String {
    if t.chars().count() <= n {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(n - 1).collect::<String>().trim_end())
    }
}

fn tecla<'a>(p: &Paleta, t: &'a str, que: &'a str) -> E<'a> {
    let p = *p;
    row![
        container(text(t).size(13).font(SEMI)).padding([3, 9]).style(move |_: &iced::Theme| container::Style {
            background: Some(Background::Color(tema::mezcla(p.text, p.win, 0.12))),
            border: iced::border::rounded(6).color(tema::mezcla(p.text, p.win, 0.35)).width(1),
            text_color: Some(p.text),
            ..Default::default()
        }),
        text(que).size(14).color(p.dim),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn tarjeta<'a>(app: &'a App, i: &Instancia, f: usize, c: usize) -> E<'a> {
    let p = app.p;
    let elegida = app.bp == (f, c);
    let jugando = app.partidas.contains_key(&i.clave());
    let pie: E = if jugando {
        text("● jugando").size(13).color(p.ok).into()
    } else {
        tenue(&p, i.subtitulo()).size(13).into()
    };
    // Cada pieza en su caja de alto fijo: un nombre largo se corta, no empuja ni pisa nada.
    let contenido = column![
        container(icono(app, i, 92.0)).center_x(Length::Fill).height(118).align_y(Alignment::Center),
        container(text(corto(&i.nombre, 34)).size(16).font(SEMI).width(Length::Fill).center()).height(42).clip(true),
        container(pie).center_x(Length::Fill).height(18).clip(true),
    ]
    .spacing(6);
    let acento = if i.motor == Motor::Rust { p.rust } else { p.java };
    let fondo_elegida = tema::mezcla(p.hl, p.panel, 0.22);
    let fondo_encima = tema::mezcla(p.text, p.panel, 0.06);
    let borde = tema::mezcla(acento, p.panel, 0.25);
    button(contenido)
        .width(TARJETA)
        .height(ALTO_TARJETA)
        .padding([10, 10])
        .style(move |_: &iced::Theme, estado| {
            let encima = matches!(estado, button::Status::Hovered | button::Status::Pressed);
            button::Style {
                background: Some(Background::Color(if elegida {
                    fondo_elegida
                } else if encima {
                    fondo_encima
                } else {
                    p.panel
                })),
                text_color: p.text,
                border: iced::border::rounded(16).color(if elegida { p.hl } else { borde }).width(if elegida { 3.0 } else { 1.0 }),
                ..Default::default()
            }
        })
        .on_press(Msg::BPClic(f, c))
        .into()
}

fn heroe<'a>(app: &'a App, i: &Instancia) -> E<'a> {
    let p = app.p;
    let clave = i.clave();
    let acento = if i.motor == Motor::Rust { p.rust } else { p.java };
    let mut datos = vec![i.subtitulo()];
    if let Some(v) = i.version.clone().filter(|_| i.motor == Motor::Java) {
        datos.insert(0, v);
    }
    let jugando = app.partidas.get(&clave);
    let estado: E = match (jugando, app.compilando.get(&clave)) {
        (Some(e), _) => text(if e.en_juego { "En juego".to_string() } else { corto(&e.linea, 90) }).size(15).color(p.hl).into(),
        (None, Some(l)) => text(format!("Compilando mods: {}", corto(l, 80))).size(15).color(p.hl).into(),
        _ => text(format!(
            "{} jugado{}",
            i.tiempo_jugado(),
            i.ultima_ms.map(|m| format!(" · última vez {}", crate::datos::hace(m))).unwrap_or_default()
        ))
        .size(15)
        .color(p.dim)
        .into(),
    };
    let lineas = column![
        row![pildora(&p, i.motor, i.etiqueta_motor()), text(datos.join(" · ")).size(16).color(p.dim)].spacing(10).align_y(Alignment::Center),
        container(text(corto(&i.nombre, 40)).size(42).font(Font { weight: iced::font::Weight::Bold, ..INTER })).height(56).clip(true),
        container(estado).height(22).clip(true),
    ]
    .spacing(8);
    let jugar = button(text(if jugando.is_some() { "Jugando…" } else { "Jugar" }).size(20).font(SEMI))
        .padding([14, 44])
        .style(tema::boton(p, Boton::Primario))
        .on_press_maybe(jugando.is_none().then(|| Msg::Jugar(clave.clone())));
    let mut acciones = row![
        jugar,
        button(text("Editar").size(17)).padding([14, 26]).style(tema::boton(p, Boton::Normal)).on_press(Msg::AbrirEditar(clave.clone(), PagE::Version)),
        button(text("Mods").size(17)).padding([14, 26]).style(tema::boton(p, Boton::Normal)).on_press(Msg::AbrirEditar(clave.clone(), PagE::Mods)),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    if i.motor == Motor::Rust && i.origen == Origen::Horizon {
        let compilando = app.compilando.contains_key(&clave);
        acciones = acciones.push(
            button(text(if compilando { "Compilando…" } else { "Compilar mods" }).size(17))
                .padding([14, 26])
                .style(tema::boton(p, Boton::Normal))
                .on_press_maybe((!compilando).then(|| Msg::AbrirEditar(clave.clone(), PagE::Mods))),
        );
    }
    let fondo_placa = tema::mezcla(acento, p.win, 0.14);
    let borde_placa = tema::mezcla(acento, p.win, 0.4);
    let placa = container(icono(app, i, 150.0)).padding(16).style(move |_: &iced::Theme| container::Style {
        background: Some(Background::Color(fondo_placa)),
        border: iced::border::rounded(28).color(borde_placa).width(1),
        ..Default::default()
    });
    // Fondo liso (un degradado a pantalla completa cuesta mucho más de repintar por software).
    let fondo = tema::mezcla(acento, p.win, 0.09);
    container(row![placa, column![lineas, Space::new(0, 10), acciones].spacing(4).width(Length::Fill)].spacing(36).align_y(Alignment::Center))
        .padding([0, MARGEN as u16])
        .width(Length::Fill)
        .height(ALTO_HEROE)
        .align_y(Alignment::Center)
        .clip(true)
        .style(tema::fondo(fondo))
        .into()
}

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let estantes = app.estantes();
    let elegida = estantes.get(app.bp.0).and_then(|e| e.1.get(app.bp.1)).copied();

    // Cabecera: marca, tu cuenta y salir.
    let mut cuenta = row![].spacing(10).align_y(Alignment::Center);
    if let Some(h) = &app.cara {
        cuenta = cuenta.push(image(h.clone()).width(28).height(28).filter_method(FilterMethod::Nearest));
    }
    cuenta = cuenta.push(text(app.cuenta_activa().map(|c| c.nombre.clone()).unwrap_or_else(|| "Sin cuenta".into())).size(16));
    let cabecera = container(
        row![
            image(app.logo.clone()).width(30).height(30),
            text("Jiyu Launcher").size(22).font(SEMI),
            text("Big Picture").size(16).color(p.dim),
            horizontal_space(),
            cuenta,
            btn(p, "Salir de Big Picture", Boton::Normal, Some(Msg::BPAlternar)).padding([8, 16]),
        ]
        .spacing(14)
        .align_y(Alignment::Center),
    )
    .padding([0, MARGEN as u16])
    .height(ALTO_CABECERA)
    .align_y(Alignment::Center);

    let heroe_e: E = match elegida {
        Some(i) => heroe(app, i),
        None => container(
            column![text("Aún no hay instancias").size(30).font(SEMI), tenue(&p, "Sal de Big Picture y crea una con «Añadir instancia».").size(16)].spacing(8),
        )
        .padding([0, MARGEN as u16])
        .height(ALTO_HEROE)
        .align_y(Alignment::Center)
        .into(),
    };

    // Estantes: solo los que caben, y en cada uno solo las tarjetas que caben.
    let (filas, cols) = cabe(app.ventana);
    let fila0 = app.bp_fila0.min(estantes.len().saturating_sub(1));
    let mut estantes_col = Column::new().spacing(0).padding([16, 0]);
    for (f, (nombre, lista)) in estantes.iter().enumerate().skip(fila0).take(filas) {
        let mut titulo_e = row![
            text(nombre.replace('/', " › ")).size(20).font(SEMI).color(if f == app.bp.0 { p.text } else { p.dim }),
            text(lista.len().to_string()).size(15).color(p.dim),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        if estantes.len() > filas && f == fila0 {
            titulo_e = titulo_e.push(horizontal_space()).push(tenue(&p, format!("Estante {} de {}  ↑↓", app.bp.0 + 1, estantes.len())).size(14));
        }
        let c0 = app.bp_col0.get(&f).copied().unwrap_or(0).min(lista.len().saturating_sub(1));
        let fin = (c0 + cols).min(lista.len());
        let tarjetas = row(lista[c0..fin].iter().enumerate().map(|(k, i)| tarjeta(app, i, f, c0 + k))).spacing(HUECO);
        let fila_t = row![mas(&p, c0, true), tarjetas, mas(&p, lista.len() - fin, false)].align_y(Alignment::Center);
        estantes_col = estantes_col.push(
            column![container(titulo_e).padding([0, MARGEN as u16]).height(28).align_y(Alignment::Center), fila_t]
                .spacing(10)
                .height(ALTO_ESTANTE),
        );
    }
    // La rueda del ratón también se mueve por los estantes.
    let cuerpo = mouse_area(container(estantes_col).width(Length::Fill).height(Length::Fill).clip(true))
        .on_scroll(|d| {
            let (x, y) = match d {
                iced::mouse::ScrollDelta::Lines { x, y } => (x, y),
                iced::mouse::ScrollDelta::Pixels { x, y } => (x / 40.0, y / 40.0),
            };
            Msg::BPRueda(x, y)
        });

    let total: usize = estantes.iter().filter(|e| e.0 != "Seguir jugando").map(|e| e.1.len()).sum();
    let pie = container(
        row![
            tecla(&p, "← → ↑ ↓", "Moverse"),
            tecla(&p, "Intro", "Jugar"),
            tecla(&p, "E", "Editar"),
            tecla(&p, "Esc", "Salir"),
            horizontal_space(),
            text(format!("{total} instancias")).size(14).color(p.dim),
        ]
        .spacing(26)
        .align_y(Alignment::Center),
    )
    .padding([0, MARGEN as u16])
    .height(ALTO_PIE)
    .align_y(Alignment::Center);

    let todo = column![cabecera, heroe_e, cuerpo, linea(&p), pie];
    container(todo)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(tema::fondo(tema::mezcla(Color::BLACK, p.win, 0.25)))
        .into()
}
