//! Big Picture: el launcher a pantalla completa para jugar desde el sofá, como el modo de Steam.
//! Arriba, la instancia elegida en grande con su botón de jugar; debajo, estantes («Seguir jugando» y
//! uno por carpeta) que se recorren con las flechas. Intro juega, E edita, Esc o F11 sale.

use super::*;
use iced::widget::scrollable::{Direction, Scrollbar};
use iced::widget::{column, row, scrollable};
use iced::{gradient, Background, Color, Degrees};

pub const TARJETA: f32 = 176.0;
pub const HUECO: f32 = 18.0;
pub const ALTO_ESTANTE: f32 = 292.0;

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
    let mut pie = row![tenue(&p, i.subtitulo()).size(13)].spacing(6).align_y(Alignment::Center);
    if jugando {
        pie = pie.push(text("● jugando").size(12).color(p.ok));
    }
    let contenido = column![
        container(icono(app, i, 92.0)).center_x(Length::Fill).padding([10, 0]),
        text(corto(&i.nombre, 30)).size(16).font(SEMI).width(Length::Fill).center(),
        container(pie).center_x(Length::Fill),
    ]
    .spacing(6);
    let acento = if i.motor == Motor::Rust { p.rust } else { p.java };
    button(contenido)
        .width(TARJETA)
        .height(220)
        .padding([12, 10])
        .style(move |_: &iced::Theme, estado| {
            let encima = matches!(estado, button::Status::Hovered | button::Status::Pressed);
            let fondo = if elegida {
                tema::mezcla(p.hl, p.panel, 0.22)
            } else if encima {
                tema::mezcla(p.text, p.panel, 0.06)
            } else {
                p.panel
            };
            button::Style {
                background: Some(Background::Color(fondo)),
                text_color: p.text,
                border: iced::border::rounded(16)
                    .color(if elegida { p.hl } else { tema::mezcla(acento, p.panel, 0.25) })
                    .width(if elegida { 3.0 } else { 1.0 }),
                ..Default::default()
            }
        })
        .on_press(Msg::BPElegir(f, c))
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
    let mut lineas = column![
        row![pildora(&p, i.motor, i.etiqueta_motor()), text(datos.join(" · ")).size(16).color(p.dim)].spacing(10).align_y(Alignment::Center),
        text(i.nombre.clone()).size(46).font(Font { weight: iced::font::Weight::Bold, ..INTER }),
        text(format!(
            "{} jugado{}",
            i.tiempo_jugado(),
            i.ultima_ms.map(|m| format!(" · última vez {}", crate::datos::hace(m))).unwrap_or_default()
        ))
        .size(15)
        .color(p.dim),
    ]
    .spacing(8);
    if let Some(e) = app.partidas.get(&clave) {
        lineas = lineas.push(text(if e.en_juego { "En juego".to_string() } else { e.linea.clone() }).size(15).color(p.hl));
    }
    let jugar = button(text(if app.partidas.contains_key(&clave) { "Jugando…" } else { "Jugar" }).size(20).font(SEMI))
        .padding([14, 44])
        .style(tema::boton(p, Boton::Primario))
        .on_press_maybe((!app.partidas.contains_key(&clave)).then(|| Msg::Jugar(clave.clone())));
    let acciones = row![
        jugar,
        button(text("Editar").size(17)).padding([14, 26]).style(tema::boton(p, Boton::Normal)).on_press(Msg::AbrirEditar(clave.clone(), PagE::Version)),
        button(text("Descargar mods").size(17)).padding([14, 26]).style(tema::boton(p, Boton::Normal)).on_press(Msg::ExplorarPara(clave.clone(), "mod")),
    ]
    .spacing(12)
    .align_y(Alignment::Center);
    let placa = container(icono(app, i, 150.0)).padding(18).style(move |_: &iced::Theme| container::Style {
        background: Some(Background::Color(tema::mezcla(acento, p.win, 0.14))),
        border: iced::border::rounded(28).color(tema::mezcla(acento, p.win, 0.4)).width(1),
        ..Default::default()
    });
    let fondo_heroe = move |_: &iced::Theme| container::Style {
        background: Some(Background::Gradient(
            gradient::Linear::new(Degrees(90.0))
                .add_stop(0.0, tema::mezcla(acento, p.win, 0.2))
                .add_stop(0.55, tema::mezcla(acento, p.win, 0.05))
                .add_stop(1.0, p.win)
                .into(),
        )),
        ..Default::default()
    };
    container(row![placa, column![lineas, Space::new(0, 12), acciones].spacing(4)].spacing(36).align_y(Alignment::Center))
        .padding([34, 48])
        .width(Length::Fill)
        .style(fondo_heroe)
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
    let cabecera = row![
        image(app.logo.clone()).width(30).height(30),
        text("Jiyu Launcher").size(22).font(SEMI),
        text("Big Picture").size(16).color(p.dim),
        horizontal_space(),
        cuenta,
        btn(p, "Salir de Big Picture", Boton::Normal, Some(Msg::BPAlternar)).padding([8, 16]),
    ]
    .spacing(14)
    .align_y(Alignment::Center)
    .padding([16, 48]);

    let mut pagina = Column::new().spacing(0);
    match elegida {
        Some(i) => pagina = pagina.push(heroe(app, i)),
        None => {
            pagina = pagina.push(
                container(column![text("Aún no hay instancias").size(30).font(SEMI), tenue(&p, "Sal de Big Picture y crea una con «Añadir instancia».").size(16)].spacing(8))
                    .padding([60, 48]),
            )
        }
    }
    let mut estantes_col = Column::new().spacing(0).padding([10, 0]);
    for (f, (nombre, lista)) in estantes.iter().enumerate() {
        let titulo_e = row![
            text(nombre.replace('/', " › ")).size(20).font(SEMI).color(if f == app.bp.0 { p.text } else { p.dim }),
            text(lista.len().to_string()).size(15).color(p.dim),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        let fila_t = row(lista.iter().enumerate().map(|(c, i)| tarjeta(app, i, f, c))).spacing(HUECO).padding([6, 48]);
        let desliza = scrollable(fila_t)
            .id(scrollable::Id::new(format!("bp-{f}")))
            .direction(Direction::Horizontal(Scrollbar::new().width(0).scroller_width(0)))
            .width(Length::Fill);
        estantes_col = estantes_col.push(column![container(titulo_e).padding([0, 48]), desliza].spacing(10).height(ALTO_ESTANTE));
    }
    pagina = pagina.push(estantes_col);
    let cuerpo = scrollable(pagina).id(scrollable::Id::new("bp")).style(tema::desplazable(p)).height(Length::Fill);

    let pie = row![
        tecla(&p, "← → ↑ ↓", "Moverse"),
        tecla(&p, "Intro", "Jugar"),
        tecla(&p, "E", "Editar"),
        tecla(&p, "Esc", "Salir"),
        horizontal_space(),
        text(format!("{} instancias", estantes.iter().skip(usize::from(estantes.first().is_some_and(|e| e.0 == "Seguir jugando"))).map(|e| e.1.len()).sum::<usize>())).size(14).color(p.dim),
    ]
    .spacing(26)
    .align_y(Alignment::Center)
    .padding([14, 48]);

    let todo = column![cabecera, cuerpo, container(linea(&p)), pie];
    container(todo)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(tema::fondo(tema::mezcla(Color::BLACK, p.win, 0.25)))
        .into()
}

fn corto(t: &str, n: usize) -> String {
    if t.chars().count() <= n {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(n - 1).collect::<String>().trim_end())
    }
}
