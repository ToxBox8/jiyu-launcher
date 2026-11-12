//! Tus servidores: se añaden a la lista Multijugador de todas tus instancias y se ve si están en línea.

use super::*;
use iced::widget::{column, row};

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let mut lista = Column::new().spacing(1);
    for (i, s) in app.aj.servidores.iter().enumerate() {
        let (icono, estado): (E, E) = match app.sv_estado.get(&s.direccion) {
            Some(Ok(e)) => (
                match &e.icono {
                    Some(h) => container(image(h.clone()).width(40).height(40)).style(tema::caja(p)).padding(1).into(),
                    None => super::logo(app, None, &s.nombre, 40.0),
                },
                column![
                    text(format!("{} / {} jugadores", e.en_linea, e.maximo)).size(12).color(p.ok),
                    tenue(&p, format!("{} ms · {}", e.ms, e.version)),
                ]
                .spacing(2)
                .align_x(Alignment::End)
                .into(),
            ),
            Some(Err(err)) => (super::logo(app, None, &s.nombre, 40.0), text(err.clone()).size(12).color(p.mal).into()),
            None => (super::logo(app, None, &s.nombre, 40.0), tenue(&p, "Mirando…").into()),
        };
        let motd = match app.sv_estado.get(&s.direccion) {
            Some(Ok(e)) if !e.motd.is_empty() => e.motd.clone(),
            _ => s.direccion.clone(),
        };
        lista = lista.push(
            container(
                row![
                    icono,
                    column![row![txt(s.nombre.clone()).font(SEMI), tenue(&p, s.direccion.clone())].spacing(8), tenue(&p, motd)]
                        .spacing(2)
                        .width(Length::Fill),
                    estado,
                    btn(p, "Quitar", Boton::Peligro, Some(Msg::SvQuitar(i))).padding([3, 8]),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            )
            .padding([8, 10]),
        );
    }
    if app.aj.servidores.is_empty() {
        lista = lista.push(container(tenue(&p, "Todavía no has añadido ninguno.")).padding(14));
    }
    let anadir = row![
        text_input("Nombre (opcional)", &app.sv_nombre).on_input(Msg::SvNombre).padding([6, 8]).size(13).width(200).style(tema::campo(p)),
        text_input("Dirección: play.ejemplo.com o 1.2.3.4:25565", &app.sv_dir)
            .on_input(Msg::SvDir)
            .on_submit(Msg::SvAnadir)
            .padding([6, 8])
            .size(13)
            .style(tema::campo(p)),
        btn(p, "Añadir", Boton::Primario, (!app.sv_dir.trim().is_empty()).then_some(Msg::SvAnadir)),
    ]
    .spacing(8);
    let cuerpo = column![
        titulo("Servidores"),
        tenue(&p, "Los que pongas aquí aparecen siempre en la lista Multijugador de tus instancias (Horizon y Prism). Los que ya tengas en cada una se quedan."),
        anadir,
        container(lista).padding(2).style(tema::caja(p)),
        interruptor(
            p,
            "Ponerlos en cada instancia al jugar",
            "Antes de arrancar se añaden los que falten a su servers.dat (se guarda una copia como servers.dat_old)",
            app.aj.servidores_siempre,
            Some(Box::new(|b| Msg::AFlag(crate::app::Flag::ServidoresSiempre, b))),
        ),
        row![
            btn(p, "Ponerlos ya en todas", Boton::Normal, (!app.aj.servidores.is_empty()).then_some(Msg::SvAplicar)),
            btn(p, "Actualizar estado", Boton::Normal, Some(Msg::SvRefrescar)),
        ]
        .spacing(8),
    ]
    .spacing(12);
    marco(p, "Servidores".into(), None, cuerpo.into(), Some(row![horizontal_space(), btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar))].into()))
}
