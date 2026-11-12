//! La zona de instancias en sus tres vistas: Grupos (como Prism), Carpetas (como Nautilus) y Lista (árbol).
//! En las tres las carpetas pueden ir unas dentro de otras («A/B») y las instancias se arrastran a ellas.

use super::clic::{clic_derecho, clic_izquierdo};
use super::*;
use crate::ajustes::Vista;
use crate::app::{padre_de, QueMenu};
use iced::widget::{column, row};

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let aj = &app.aj;
    let filtro = text_input("Buscar instancia…  (Ctrl+F)", &app.filtro)
        .id(iced::widget::text_input::Id::new("buscar"))
        .on_input(Msg::Filtro)
        .padding([5, 8])
        .size(13)
        .style(tema::campo(p));
    let orden = iced::widget::pick_list(crate::ajustes::Orden::TODOS, Some(aj.orden), Msg::Orden).text_size(12).padding([4, 8]).style(tema::lista(p));
    let modos = segmentado(
        p,
        vec![
            ("Grupos".into(), aj.vista == Vista::Grupos, Some(Msg::Vista(Vista::Grupos))),
            ("Carpetas".into(), aj.vista == Vista::Carpetas, Some(Msg::Vista(Vista::Carpetas))),
            ("Lista".into(), aj.vista == Vista::Lista, Some(Msg::Vista(Vista::Lista))),
        ],
    );
    let mut col = Column::new().spacing(6).padding([8, 10]);
    if let Some(e) = &app.error_horizon {
        col = col.push(aviso(
            p,
            p.rust,
            column![txt("No puedo hablar con Horizon").font(SEMI), tenue(&p, e.clone()), btn(p, "Reintentar", Boton::Normal, Some(Msg::Recargar))]
                .spacing(6),
        ));
    }
    if app.todas().next().is_none() && !app.cargando {
        col = col.push(
            container(
                column![txt("Todavía no hay instancias").font(SEMI), btn(p, "+  Añadir instancia", Boton::Primario, Some(Msg::Abrir(Dlg::Nueva)))]
                    .spacing(10)
                    .align_x(Alignment::Center),
            )
            .center_x(Length::Fill)
            .padding(40),
        );
    }
    let cuerpo: E = match aj.vista {
        Vista::Grupos => grupos(app),
        Vista::Carpetas => carpetas(app),
        Vista::Lista => lista(app),
    };
    col = col.push(cuerpo);
    // Clic derecho en el fondo: nueva instancia, carpeta nueva, cambiar de vista.
    let zona = clic_derecho(desplaza(p, col).height(Length::Fill), |pos| Msg::AbrirMenu(QueMenu::Fondo, pos));
    column![
        container(row![filtro, orden, modos].spacing(8).align_y(Alignment::Center)).padding(Padding { top: 8.0, right: 10.0, bottom: 0.0, left: 10.0 }),
        zona
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Lo que hace a una zona «soltable»: avisa al entrar y salir mientras arrastras, y se marca.
fn soltable<'a>(app: &'a App, carpeta: &str, contenido: impl Into<E<'a>>) -> E<'a> {
    let activo = app.arrastrando && app.destino.as_deref() == Some(carpeta);
    mouse_area(container(contenido).style(tema::soltar(app.p, activo)))
        .on_enter(Msg::Destino(Some(carpeta.to_string())))
        .on_exit(Msg::Destino(None))
        .into()
}

fn nombre_corto(c: &str) -> String {
    if c.is_empty() {
        "Sin carpeta".into()
    } else {
        c.rsplit('/').next().unwrap_or(c).to_string()
    }
}

// ---------- Grupos ----------

fn grupos(app: &App) -> E<'_> {
    let p = app.p;
    let mut col = Column::new().spacing(6);
    let mut carpetas = app.todas_carpetas();
    carpetas.push(String::new());
    let buscando = !app.filtro.is_empty();
    for c in carpetas {
        let insts = app.en_carpeta(&c);
        let vacia = insts.is_empty();
        if vacia && (buscando || c.is_empty()) {
            continue;
        }
        let profundidad = c.matches('/').count() + if c.is_empty() { 0 } else { 1 };
        let clave = format!("c:{c}");
        let cerrado = app.aj.grupos_cerrados.contains(&clave);
        let flecha = button(
            image(if cerrado { app.flecha_cerrada.clone() } else { app.flecha_abierta.clone() })
                .width(10)
                .height(10),
        )
        .padding([4, 4])
        .style(tema::boton(p, Boton::Barra))
        .on_press(Msg::Grupo(clave.clone()));
        let mut titulo_g = row![text(nombre_corto(&c)).size(13).font(SEMI)].spacing(6).align_y(Alignment::Center);
        if profundidad > 1 {
            titulo_g = titulo_g.push(tenue(&p, padre_de(&c).replace('/', " › ")));
        }
        titulo_g = titulo_g.push(tenue(&p, format!("{}", insts.len())));
        let ico_g = image(app.icono_carpeta_p(&c)).width(16).height(16);
        // Pulsar el nombre del grupo abre su menú (crear una instancia aquí, carpeta nueva…).
        let c1 = c.clone();
        let c2 = c.clone();
        let nombre = clic_derecho(
            clic_izquierdo(container(titulo_g).padding([4, 6]), move |pos| Msg::AbrirMenu(QueMenu::Carpeta(c1.clone()), pos)),
            move |pos| Msg::AbrirMenu(QueMenu::Carpeta(c2.clone()), pos),
        );
        // Una línea a lo ancho tras el nombre, como en Prism, para separar los grupos.
        let raya = container(Space::new(Length::Fill, 1)).height(1).width(Length::Fill).style(tema::fondo(p.line));
        let cabecera = row![Space::new((profundidad.saturating_sub(1) * 18) as f32, 0), flecha, ico_g, nombre, raya]
            .spacing(4)
            .align_y(Alignment::Center);
        let mut seccion_g = Column::new().spacing(4).push(cabecera);
        if !cerrado {
            if vacia {
                seccion_g = seccion_g.push(
                    container(tenue(&p, "Vacía: arrastra aquí una instancia"))
                        .padding(Padding { top: 0.0, right: 0.0, bottom: 4.0, left: (profundidad * 18 + 12) as f32 }),
                );
            } else {
                seccion_g = seccion_g.push(
                    container(row(insts.into_iter().map(|i| casilla(app, i))).spacing(4).wrap())
                        .padding(Padding { top: 0.0, right: 0.0, bottom: 0.0, left: (profundidad.saturating_sub(1) * 18) as f32 }),
                );
            }
        }
        col = col.push(soltable(app, &c, seccion_g));
    }
    col.into()
}

// ---------- Carpetas (como Nautilus) ----------

fn carpetas(app: &App) -> E<'_> {
    let p = app.p;
    let actual = app.carpeta_actual.as_str();
    // Migas: Todas › A › B. Cada una abre esa carpeta y admite soltar.
    let mut migas = row![].spacing(2).align_y(Alignment::Center);
    let mut ruta = String::new();
    migas = migas.push(soltable(
        app,
        "",
        btn(p, "Todas", Boton::Pagina(actual.is_empty()), Some(Msg::EntrarCarpeta(String::new()))),
    ));
    for parte in actual.split('/').filter(|x| !x.is_empty()) {
        if !ruta.is_empty() {
            ruta.push('/');
        }
        ruta.push_str(parte);
        migas = migas.push(tenue(&p, "›"));
        migas = migas.push(soltable(
            app,
            &ruta,
            btn(p, parte.to_string(), Boton::Pagina(ruta == actual), Some(Msg::EntrarCarpeta(ruta.clone()))),
        ));
    }
    let mut col = Column::new().spacing(10).push(migas);
    let lado = app.aj.tamano.px();
    let ancho = (lado + 56.0).max(96.0);
    if !app.filtro.is_empty() {
        // Buscando: todas las que coinciden, estén donde estén.
        col = col.push(row(app.buscadas().into_iter().map(|i| casilla(app, i))).spacing(4).wrap());
        return col.into();
    }
    let mut piezas: Vec<E> = Vec::new();
    for c in app.subcarpetas(actual) {
        let n = app.cuantas_en(&c);
        let contenido = column![
            image(app.icono_carpeta(&c)).width(lado).height(lado),
            text(nombre_corto(&c)).size(12).font(MEDIA).width(ancho - 8.0).center(),
            tenue(&p, if n == 1 { "1 instancia".to_string() } else { format!("{n} instancias") }).size(10),
        ]
        .spacing(4)
        .align_x(Alignment::Center);
        let c1 = c.clone();
        let boton = button(contenido).width(ancho).padding([8, 4]).style(tema::boton(p, Boton::Casilla(false))).on_press(Msg::EntrarCarpeta(c.clone()));
        piezas.push(soltable(app, &c, clic_derecho(boton, move |pos| Msg::AbrirMenu(QueMenu::Carpeta(c1.clone()), pos))));
    }
    for i in app.en_carpeta(actual) {
        piezas.push(casilla(app, i));
    }
    if piezas.is_empty() {
        col = col.push(container(tenue(&p, "Carpeta vacía. Arrastra instancias aquí o crea una con clic derecho.")).padding(20));
    } else {
        col = col.push(row(piezas).spacing(4).wrap());
    }
    col.into()
}

// ---------- Lista (árbol) ----------

fn lista(app: &App) -> E<'_> {
    let p = app.p;
    let mut col = Column::new().spacing(1);
    col = col.push(
        container(
            row![
                tenue(&p, "Nombre").width(Length::Fill),
                tenue(&p, "Motor").width(70),
                tenue(&p, "Versión").width(150),
                tenue(&p, "Jugado").width(80),
                tenue(&p, "Última vez").width(110),
            ]
            .spacing(8),
        )
        .padding([4, 10]),
    );
    if !app.filtro.is_empty() {
        for i in app.buscadas() {
            col = col.push(fila_instancia(app, i, 0));
        }
        return col.into();
    }
    for e in rama(app, "", 0) {
        col = col.push(e);
    }
    col.into()
}

fn rama<'a>(app: &'a App, carpeta: &str, nivel: usize) -> Vec<E<'a>> {
    let p = app.p;
    let mut v: Vec<E<'a>> = Vec::new();
    for c in app.subcarpetas(carpeta) {
        let clave = format!("c:{c}");
        let cerrado = app.aj.grupos_cerrados.contains(&clave);
        let n = app.cuantas_en(&c);
        let flecha = image(if cerrado { app.flecha_cerrada.clone() } else { app.flecha_abierta.clone() })
            .width(10)
            .height(10);
        let contenido = row![
            Space::new((nivel * 18) as f32, 0),
            flecha,
            image(app.icono_carpeta_p(&c)).width(16).height(16),
            text(nombre_corto(&c)).size(13).font(SEMI),
            tenue(&p, format!("{n}")),
            horizontal_space()
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        let c1 = c.clone();
        let fila_c = button(contenido).width(Length::Fill).padding([5, 10]).style(tema::boton(p, Boton::Fila(false))).on_press(Msg::Grupo(clave));
        let mut bloque = Column::new().push(clic_derecho(fila_c, move |pos| Msg::AbrirMenu(QueMenu::Carpeta(c1.clone()), pos)));
        if !cerrado {
            for e in rama(app, &c, nivel + 1) {
                bloque = bloque.push(e);
            }
        }
        v.push(soltable(app, &c, bloque));
    }
    for i in app.en_carpeta(carpeta) {
        v.push(fila_instancia(app, i, nivel));
    }
    v
}

fn fila_instancia<'a>(app: &'a App, inst: &'a Instancia, nivel: usize) -> E<'a> {
    let p = app.p;
    let clave = inst.clave();
    let sel = app.sel.as_deref() == Some(clave.as_str());
    let version = match inst.motor {
        Motor::Rust => format!("Horizon {}", inst.subtitulo()),
        Motor::Java => match &inst.cargador {
            Some((c, _)) => format!("{} · {c}", inst.version.clone().unwrap_or_default()),
            None => inst.version.clone().unwrap_or_default(),
        },
    };
    let contenido = row![
        Space::new((nivel * 18 + 18) as f32, 0),
        icono(app, inst, 24.0),
        text(inst.nombre.clone()).size(13).font(MEDIA).width(Length::Fill),
        container(pildora(&p, inst.motor, inst.etiqueta_motor())).width(70),
        tenue(&p, version).width(150),
        tenue(&p, inst.tiempo_jugado()).width(80),
        tenue(&p, inst.ultima_ms.map(crate::datos::hace).unwrap_or_default()).width(110),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    let resaltado = sel || app.encima.as_deref() == Some(clave.as_str());
    let objetivo = app.arrastrando && app.destino_inst.as_deref() == Some(clave.as_str());
    pulsable(app, &clave, container(contenido).padding([3, 10]).width(Length::Fill).style(tema::fila_fondo(p, sel || objetivo, resaltado)))
}

/// Instancia que se puede pulsar (elegir, doble clic = jugar), arrastrar y abrir con clic derecho.
fn pulsable<'a>(app: &'a App, clave: &str, contenido: impl Into<E<'a>>) -> E<'a> {
    let c1 = clave.to_string();
    let interaccion = if app.arrastrando { iced::mouse::Interaction::Grabbing } else { iced::mouse::Interaction::Pointer };
    let zona = mouse_area(contenido)
        .on_press(Msg::Pulsar(clave.to_string()))
        .on_enter(Msg::Hover(Some(clave.to_string())))
        .on_exit(Msg::Hover(None))
        .interaction(interaccion);
    clic_derecho(zona, move |pos| Msg::AbrirMenu(QueMenu::Instancia(c1.clone()), pos)).into()
}

pub fn casilla<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let lado = app.aj.tamano.px();
    let ancho = (lado + 56.0).max(96.0);
    let clave = inst.clave();
    let sel = app.sel.as_deref() == Some(clave.as_str());
    let encima = app.encima.as_deref() == Some(clave.as_str());
    let mut col = column![icono(app, inst, lado), text(inst.nombre.clone()).size(12).font(MEDIA).width(ancho - 8.0).center()]
        .spacing(4)
        .align_x(Alignment::Center);
    if app.aj.etiqueta {
        let color = if inst.motor == Motor::Rust { p.rust } else { p.java };
        col = col.push(
            rich_text([span(inst.etiqueta_motor()).font(SEMI).color(color), span(format!(" · {}", inst.subtitulo())).color(p.dim)])
                .size(10)
                .width(ancho - 8.0)
                .align_x(Alignment::Center),
        );
    }
    // Mientras arrastras otra encima, se marca: al soltar, la otra se pone delante.
    let objetivo = app.arrastrando && app.destino_inst.as_deref() == Some(clave.as_str());
    pulsable(app, &clave, container(col).width(ancho).padding([8, 4]).style(tema::casilla_fondo(p, sel || objetivo, encima)))
}
