//! Menú contextual: sale donde pulsaste (clic derecho en una instancia, en una carpeta o en el fondo,
//! o clic en el nombre de un grupo). Se cierra al elegir, al pulsar fuera o con Esc.

use super::*;
use crate::ajustes::Vista;
use crate::app::{Menu, ParaNombre, QueMenu, RAIZ};
use iced::widget::column;

const ANCHO: f32 = 230.0;

pub fn vista<'a>(app: &'a App, m: &'a Menu) -> E<'a> {
    let p = app.p;
    // Con el dibujo por software, resaltar al pasar repinta a trozos y deja restos de lo de debajo:
    // solo se resalta cuando se dibuja con la gráfica.
    let resalta = app.aj.grafica;
    let item = |t: String, msg: Option<Msg>| -> E<'a> {
        button(text(t).size(13))
            .width(Length::Fill)
            .padding([6, 10])
            .style(tema::boton(p, Boton::Menu(resalta)))
            .on_press_maybe(msg)
            .into()
    };
    let peligro = |t: &str, msg: Msg| -> E<'a> {
        button(text(t.to_string()).size(13).color(p.mal))
            .width(Length::Fill)
            .padding([6, 10])
            .style(tema::boton(p, Boton::Menu(resalta)))
            .on_press(msg)
            .into()
    };
    let mut col = Column::new().spacing(0);
    match &m.que {
        QueMenu::Instancia(clave) => {
            let Some(inst) = app.instancia(clave) else { return Space::new(0, 0).into() };
            let k = clave.clone();
            col = col
                .push(container(txt(inst.nombre.clone()).font(SEMI)).padding([6, 10]))
                .push(linea(&p))
                .push(item("Jugar".into(), (!app.partidas.contains_key(clave)).then(|| Msg::Jugar(k.clone()))))
                .push(item("Editar".into(), Some(Msg::AbrirEditar(k.clone(), PagE::Version))))
                .push(item("Cambiar icono…".into(), Some(Msg::AbrirIcono(k.clone()))))
                .push(item("Mods".into(), Some(Msg::AbrirEditar(k.clone(), PagE::Mods))))
                .push(item("Descargar mods, shaders, texturas…".into(), Some(Msg::ExplorarPara(k.clone(), "mod"))))
                .push(item("Skin al jugar…".into(), Some(Msg::AbrirEditar(k.clone(), PagE::Skin))))
                .push(item(if m.mover { "Mover a…  ↑".into() } else { "Mover a…  ↓".into() }, Some(Msg::MoverDesplegar)));
            if m.mover {
                let actual = app.carpeta_de(inst);
                let mut destinos = vec![String::new()];
                destinos.extend(app.todas_carpetas());
                for d in destinos.into_iter().filter(|d| *d != actual) {
                    let etiqueta = if d.is_empty() { format!("    {RAIZ}") } else { format!("    {}", d.replace('/', " › ")) };
                    col = col.push(item(etiqueta, Some(Msg::Mover(k.clone(), d))));
                }
            }
            col = col.push(item("Abrir carpeta".into(), Some(Msg::Carpeta(inst.carpeta.clone()))));
            if inst.origen == crate::datos::Origen::Horizon {
                col = col
                    .push(item("Duplicar".into(), Some(Msg::Duplicar(k.clone()))))
                    .push(item("Acceso directo en el escritorio".into(), Some(Msg::AccesoDirecto(k.clone()))));
            }
            let archivada = app.aj.archivadas.contains(clave);
            col = col.push(item(
                if archivada { "Sacar del archivador".into() } else { "Archivar".into() },
                Some(Msg::Archivar(k.clone(), !archivada)),
            ));
            if inst.origen == crate::datos::Origen::Prism {
                col = col.push(item("Copiar a Horizon".into(), Some(Msg::CopiarAHorizon(k.clone()))));
            }
            if !(inst.origen == crate::datos::Origen::Horizon && inst.id == "horizon") {
                col = col.push(linea(&p)).push(peligro("Eliminar…", Msg::PedirBorrar(k)));
            }
        }
        QueMenu::Carpeta(c) => {
            let nombre = if c.is_empty() { "Sin carpeta".to_string() } else { c.replace('/', " › ") };
            let cerrado = app.aj.grupos_cerrados.contains(&format!("c:{c}"));
            col = col
                .push(container(txt(nombre).font(SEMI)).padding([6, 10]))
                .push(linea(&p))
                .push(item("Nueva instancia aquí".into(), Some(Msg::NuevaEn(c.clone()))))
                .push(item("Carpeta nueva dentro…".into(), Some(Msg::PedirNombre(ParaNombre::NuevaCarpeta(c.clone())))));
            if app.aj.vista == Vista::Carpetas {
                col = col.push(item("Abrir".into(), Some(Msg::EntrarCarpeta(c.clone()))));
            } else {
                col = col.push(item(if cerrado { "Desplegar".into() } else { "Plegar".into() }, Some(Msg::Grupo(format!("c:{c}")))));
            }
            if !c.is_empty() {
                // Color de la carpeta: una fila de muestras.
                let actual = app.aj.colores_carpeta.get(c).copied().unwrap_or(0);
                let muestras = iced::widget::row(crate::iconos::COLORES_CARPETA.iter().map(|k| {
                    let color = if *k == 0 { p.java } else { tema::hex(*k) };
                    button(Space::new(16, 16))
                        .padding(0)
                        .style(tema::boton_color(color, if *k == actual { p.text } else { p.line }))
                        .on_press(Msg::ColorCarpeta(c.clone(), *k))
                        .into()
                }))
                .spacing(5);
                col = col
                    .push(container(column![tenue(&p, "Color"), muestras].spacing(4)).padding([6, 10]))
                    .push(item("Renombrar…".into(), Some(Msg::PedirNombre(ParaNombre::RenombrarCarpeta(c.clone())))))
                    .push(item("Archivar grupo".into(), Some(Msg::ArchivarCarpeta(c.clone(), true))))
                    .push(linea(&p))
                    .push(peligro("Quitar carpeta (lo de dentro sube)", Msg::BorrarCarpeta(c.clone())));
            }
        }
        QueMenu::Fondo => {
            let dentro = if app.aj.vista == Vista::Carpetas { app.carpeta_actual.clone() } else { String::new() };
            col = col
                .push(item("Nueva instancia".into(), Some(Msg::NuevaEn(dentro.clone()))))
                .push(item("Carpeta nueva…".into(), Some(Msg::PedirNombre(ParaNombre::NuevaCarpeta(dentro)))))
                .push(linea(&p))
                .push(item("Ver por grupos".into(), (app.aj.vista != Vista::Grupos).then_some(Msg::Vista(Vista::Grupos))))
                .push(item("Ver como carpetas".into(), (app.aj.vista != Vista::Carpetas).then_some(Msg::Vista(Vista::Carpetas))))
                .push(item("Ver como lista".into(), (app.aj.vista != Vista::Lista).then_some(Msg::Vista(Vista::Lista))))
                .push(linea(&p))
                .push(item("Recargar".into(), Some(Msg::Recargar)));
        }
    }
    let caja = container(desplaza(p, col).height(Length::Shrink)).width(ANCHO).max_height(app.ventana.height - 20.0).padding(4).style(tema::dialogo(p));
    // Que no se salga de la ventana: si no cabe a la derecha o abajo, se pega al borde.
    let alto_estimado = 34.0 * 12.0;
    let x = m.pos.x.min(app.ventana.width - ANCHO - 8.0).max(4.0);
    let y = m.pos.y.min((app.ventana.height - alto_estimado).max(4.0)).max(4.0);
    container(column![caja])
        .padding(Padding { top: y, left: x, right: 0.0, bottom: 0.0 })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
