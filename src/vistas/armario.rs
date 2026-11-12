//! El armario: tus skins guardadas, la figura de frente o de espalda, y las capas de tu cuenta.

use super::*;
use iced::widget::{column, row};
use iced::widget::image::FilterMethod;

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let ar = &app.armario;
    let delgado = app.aj.modelo_delgado;

    // Lista de skins (izquierda).
    let mut lista = Column::new().spacing(2);
    for (i, s) in ar.skins.iter().enumerate() {
        let r = row![image(s.cara.clone()).width(24).height(24).filter_method(FilterMethod::Nearest), txt(s.nombre.clone())]
            .spacing(8)
            .align_y(Alignment::Center);
        lista = lista.push(fila(p, r, ar.sel == Some(i), Msg::SSel(i)));
    }
    if ar.skins.is_empty() {
        lista = lista.push(tenue(&p, "Sin skins. Trae la de tu cuenta o suelta PNG de 64×64 en la carpeta."));
    }
    let izquierda = column![
        seccion(&p, "Tus skins"),
        lista,
        Space::new(0, 6),
        btn(p, "Traer la de mi cuenta", Boton::Normal, app.cuenta_activa().map(|_| Msg::SBajar)).width(Length::Fill),
        btn(p, "Importar una skin…", Boton::Normal, Some(Msg::SImportar)).width(Length::Fill),
        btn(p, "Abrir la carpeta", Boton::Normal, Some(Msg::Carpeta(crate::skins::carpeta()))).width(Length::Fill),
        btn(p, "Recargar", Boton::Normal, Some(Msg::SRecargar)).width(Length::Fill),
    ]
    .spacing(6);

    // Figura (centro).
    let skin = ar.sel.and_then(|i| ar.skins.get(i));
    // La figura en 3D: arrastra para girarla (izquierda/derecha) e inclinarla (arriba/abajo).
    let figura: E = match (skin, &ar.figura) {
        (Some(_), Some(h)) => image(h.clone()).width(260).height(400).filter_method(FilterMethod::Nearest).into(),
        (Some(s), None) => image(s.frente.clone()).width(160).height(320).filter_method(FilterMethod::Nearest).into(),
        (None, _) => container(tenue(&p, "Elige una skin")).center_x(260).center_y(400).into(),
    };
    let escenario = mouse_area(
        container(column![figura, tenue(&p, "Arrastra para girarla")].spacing(4).align_x(Alignment::Center))
            .center_x(Length::Fill)
            .padding(12)
            .style(tema::caja(p)),
    )
    .on_press(Msg::SGirar(true))
    .on_release(Msg::SGirar(false))
    .on_move(Msg::SMover)
    .interaction(if ar.girando.is_some() { iced::mouse::Interaction::Grabbing } else { iced::mouse::Interaction::Grab });
    let controles = row![
        segmentado(
            p,
            vec![
                ("Clásico".into(), !delgado, Some(Msg::SDelgado(false))),
                ("Delgado".into(), delgado, Some(Msg::SDelgado(true))),
            ]
        ),
        horizontal_space(),
        segmentado(
            p,
            vec![
                ("Frente".into(), !ar.espalda, Some(Msg::SEspalda(false))),
                ("Espalda".into(), ar.espalda, Some(Msg::SEspalda(true))),
            ]
        ),
    ]
    .align_y(Alignment::Center);
    let cuenta = app.cuenta_activa().map(|c| c.nombre.clone()).unwrap_or_else(|| "tu cuenta".into());
    let poner = btn(
        p,
        "Ponérmela",
        Boton::Primario,
        (skin.is_some() && app.cuenta_activa().is_some() && !ar.cargando).then_some(Msg::SPoner),
    );

    // Capas de la cuenta (derecha).
    let mut capas = Column::new().spacing(2);
    match &ar.perfil {
        Some(perfil) => {
            let ninguna = !perfil.capas.iter().any(|c| c.activa);
            capas = capas.push(fila(p, row![Space::new(20, 32), txt("Sin capa")].spacing(8).align_y(Alignment::Center), ninguna, Msg::SCapa(None)));
            for c in &perfil.capas {
                let img: E = match &c.imagen {
                    Some(h) => image(h.clone()).width(20).height(32).filter_method(FilterMethod::Nearest).into(),
                    None => Space::new(20, 32).into(),
                };
                capas = capas.push(fila(p, row![img, txt(c.nombre.clone())].spacing(8).align_y(Alignment::Center), c.activa, Msg::SCapa(Some(c.id.clone()))));
            }
            if perfil.capas.is_empty() {
                capas = capas.push(tenue(&p, "Tu cuenta no tiene capas."));
            }
        }
        None if ar.cargando => capas = capas.push(tenue(&p, "Leyendo tu cuenta…")),
        None => capas = capas.push(tenue(&p, "Inicia sesión para ver tus capas.")),
    }

    let mut centro = column![escenario, controles, row![tenue(&p, format!("Se pone en {cuenta}.")), horizontal_space(), poner].align_y(Alignment::Center)].spacing(10);
    if let Some(e) = &ar.error {
        centro = centro.push(text(e.clone()).size(12).color(p.mal));
    }

    let cuerpo = row![
        container(izquierda).width(220),
        container(centro).width(Length::Fill),
        container(column![seccion(&p, "Capas de tu cuenta"), capas, tenue(&p, "Al tocar una se pone al momento.")].spacing(6)).width(220),
    ]
    .spacing(18);
    marco(
        p,
        "Armario".into(),
        None,
        cuerpo.into(),
        Some(
            row![
                tenue(&p, "Los cambios van a Mojang con la sesión de Horizon. Los demás jugadores los ven al volver a entrar."),
                horizontal_space(),
                btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar))
            ]
            .align_y(Alignment::Center)
            .into(),
        ),
    )
}
