//! El archivador: grupos e instancias que no quieres ver en la ventana pero no quieres borrar.

use super::*;
use iced::widget::{column, row};

fn fila_instancia<'a>(app: &'a App, i: &Instancia, sangria: f32, suelta: bool) -> E<'a> {
    let p = app.p;
    let k = i.clave();
    let mut r = row![
        Space::new(sangria, 0),
        icono(app, i, 32.0),
        column![txt(i.nombre.clone()).font(SEMI), tenue(&p, format!("{} · {}", i.subtitulo(), i.tiempo_jugado()))].spacing(2).width(Length::Fill),
        pildora(&p, i.motor, i.etiqueta_motor()),
        btn(p, "Jugar", Boton::Normal, (!app.partidas.contains_key(&k)).then(|| Msg::Jugar(k.clone()))),
    ]
    .spacing(10)
    .align_y(Alignment::Center);
    if suelta {
        r = r.push(btn(p, "Sacar del archivador", Boton::Normal, Some(Msg::Archivar(k.clone(), false))));
    }
    r = r.push(btn(p, "Eliminar…", Boton::Peligro, Some(Msg::PedirBorrar(k))));
    container(r).padding([6, 10]).into()
}

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;

    // Grupos archivados, con lo que llevan dentro.
    let mut grupos = Column::new().spacing(1);
    for g in &app.aj.carpetas_archivadas {
        let dentro = app.en_grupo_archivado(g);
        let n = dentro.len();
        let cabeza = row![
            image(app.icono_carpeta_p(g)).width(18).height(18),
            column![
                txt(g.replace('/', " › ")).font(SEMI),
                tenue(&p, if n == 1 { "1 instancia".to_string() } else { format!("{n} instancias") }),
            ]
            .spacing(2)
            .width(Length::Fill),
            btn(p, "Sacar el grupo", Boton::Normal, Some(Msg::ArchivarCarpeta(g.clone(), false))),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        grupos = grupos.push(container(cabeza).padding([8, 10]));
        for i in dentro {
            grupos = grupos.push(fila_instancia(app, i, 18.0, false));
        }
        grupos = grupos.push(linea(&p));
    }

    let sueltas = app.archivadas();
    let mut lista = Column::new().spacing(1);
    for i in &sueltas {
        lista = lista.push(fila_instancia(app, i, 0.0, true));
    }

    let mut cuerpo = column![
        titulo("Archivadas"),
        tenue(&p, "No salen en la ventana ni en las búsquedas, pero siguen ahí con sus mundos y mods."),
    ]
    .spacing(12);
    if app.aj.carpetas_archivadas.is_empty() && sueltas.is_empty() {
        cuerpo = cuerpo.push(
            container(tenue(&p, "Vacío. Clic derecho en una instancia → Archivar, o en un grupo → Archivar grupo.")).padding(14).style(tema::caja(p)),
        );
    }
    if !app.aj.carpetas_archivadas.is_empty() {
        cuerpo = cuerpo.push(seccion(&p, "Grupos")).push(container(grupos).padding(2).style(tema::caja(p)));
    }
    if !sueltas.is_empty() {
        cuerpo = cuerpo.push(seccion(&p, "Instancias")).push(container(lista).padding(2).style(tema::caja(p)));
    }
    marco(
        p,
        "Archivadas".into(),
        None,
        desplaza(p, cuerpo).into(),
        Some(row![horizontal_space(), btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar))].into()),
    )
}
