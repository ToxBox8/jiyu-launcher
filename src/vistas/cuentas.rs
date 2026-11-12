//! Cuentas de Microsoft. El inicio de sesión lo hace Horizon con su app ya aprobada por Mojang
//! (código de dispositivo); cuando la app propia de Azure esté aprobada, será la segunda opción.

use super::*;
use iced::widget::{column, row};

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let mut lista = Column::new().spacing(1);
    for c in &app.cuentas {
        let estado = if c.lista { text("Lista para jugar").size(12).color(p.ok) } else { text(c.mensaje.clone()).size(12).color(p.mal) };
        lista = lista.push(
            container(
                row![
                    column![txt(c.nombre.clone()).font(SEMI), estado].spacing(2).width(Length::Fill),
                    if c.elegida {
                        Element::from(container(text("Juega esta").size(11).font(SEMI)).padding([2, 8]).style(tema::pildora(p.hl, p.on_hl)))
                    } else {
                        btn(p, "Jugar con esta", Boton::Normal, Some(Msg::CElegir(c.uuid.clone()))).into()
                    },
                    btn(p, "Cerrar sesión", Boton::Peligro, Some(Msg::CSalir(c.uuid.clone()))),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            )
            .padding([8, 12]),
        );
    }
    if app.cuentas.is_empty() {
        lista = lista.push(container(tenue(&p, "Ninguna cuenta todavía.")).padding(14));
    }

    let acceso: E = if app.acceder.activo {
        let mut col = column![txt("Iniciando sesión con Microsoft").font(SEMI)].spacing(8);
        match &app.acceder.codigo {
            Some((codigo, enlace)) => {
                col = col
                    .push(tenue(&p, "Se ha abierto el navegador. Escribe este código allí:"))
                    .push(
                        row![
                            container(text(codigo.clone()).size(26).font(SEMI)).padding([6, 14]).style(tema::caja(p)),
                            btn(p, "Copiar", Boton::Normal, Some(Msg::CCopiar(codigo.clone()))),
                            btn(p, "Abrir la página", Boton::Normal, Some(Msg::Url(enlace.clone()))),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .push(tenue(&p, format!("Página: {enlace}")));
            }
            None => col = col.push(tenue(&p, "Pidiendo un código a Microsoft…")),
        }
        for l in &app.acceder.lineas {
            col = col.push(text(l.clone()).size(11).color(p.dim).font(iced::Font::MONOSPACE));
        }
        aviso(p, p.hl, col)
    } else {
        row![btn(p, "+  Añadir cuenta de Microsoft", Boton::Primario, Some(Msg::CEntrar))].into()
    };

    let cuerpo = column![
        titulo("Cuentas"),
        tenue(&p, "Para jugar hace falta una cuenta de Microsoft que tenga Minecraft: Java Edition."),
        container(lista).padding(2).style(tema::caja(p)),
        acceso,
        seccion(&p, "Cómo se inicia sesión"),
        segmentado(
            p,
            vec![
                ("A través de Horizon".into(), true, None),
                ("App propia · pendiente de Mojang".into(), false, None),
            ]
        ),
        tenue(&p, "Ahora se usa la app de Horizon, que Mojang ya aprobó. Tus tokens los guarda Horizon en el llavero del sistema; este launcher no los almacena."),
    ]
    .spacing(12);
    marco(p, "Cuentas".into(), None, cuerpo.into(), Some(row![horizontal_space(), btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar))].into()))
}
