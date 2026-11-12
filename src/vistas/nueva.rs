//! Nueva instancia: Horizon (Rust), Minecraft Java, un paquete de mods o copiar una de Prism.
//! El icono, la carpeta, la memoria y la skin se eligen ya aquí.

use super::*;
use iced::widget::{column, pick_list, row, slider};
use iced::widget::image::FilterMethod;
use crate::app::Fuente;

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let n = &app.nueva;
    let fuente = n.fuente.unwrap_or(Fuente::Horizon);

    let src = |nombre: &'static str, detalle: &'static str, color: iced::Color, f: Fuente| -> E<'_> {
        let sel = fuente == f;
        button(
            row![
                container(Space::new(10, 10)).style(tema::pildora(color, p.on_badge)),
                column![txt(nombre).font(if sel { SEMI } else { MEDIA }), tenue(&p, detalle)].spacing(1)
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([8, 10])
        .style(tema::boton(p, Boton::Pagina(sel)))
        .on_press(Msg::NFuente(f))
        .into()
    };
    let fuentes = column![
        seccion(&p, "Motor"),
        src("Horizon", "Minecraft en Rust", p.rust, Fuente::Horizon),
        src("Minecraft Java", "La edición de siempre", p.java, Fuente::Java),
        Space::new(0, 8),
        seccion(&p, "Importar"),
        src("Paquete de mods", "Modrinth o un .mrpack", p.hl, Fuente::Modpack),
        src("Desde Prism", "Copia una instancia tuya", p.line, Fuente::Prism),
    ]
    .spacing(4);

    // Icono + nombre, como en Prism: el icono se pulsa para elegirlo.
    let previa: E = match &n.icono {
        Some(spec) => {
            let h = app.selector.previas.iter().chain(app.selector.propios.iter()).find(|(k, _)| k == spec).map(|(_, h)| h.clone());
            match h {
                Some(h) => image(h).width(48).height(48).filter_method(FilterMethod::Nearest).into(),
                None => container(text(spec.clone()).size(10)).center(48).style(tema::caja(p)).into(),
            }
        }
        None => image(app.icono_defecto.clone().unwrap_or_else(|| crate::iconos::bloque(crate::iconos::Bloque::Tierra, "default")))
            .width(48)
            .height(48)
            .filter_method(FilterMethod::Nearest)
            .into(),
    };
    let boton_icono = button(previa).padding(4).style(tema::boton(p, Boton::Casilla(false))).on_press(Msg::NIcono);
    let nombre = row![
        column![boton_icono, tenue(&p, "Icono")].spacing(2).align_x(Alignment::Center),
        column![
            tenue(&p, "Nombre"),
            text_input("Mi instancia", &n.nombre).on_input(Msg::NNombre).padding([6, 8]).size(13).style(tema::campo(p))
        ]
        .spacing(4)
        .width(Length::Fill)
    ]
    .spacing(14)
    .align_y(Alignment::Center);

    let cuerpo: E = match fuente {
        Fuente::Horizon => {
            let mut filas = Column::new().spacing(1);
            filas = filas.push(fila(
                p,
                row![txt("Seguir la última").font(MEDIA).width(Length::Fill), tenue(&p, "se actualiza sola")],
                n.version_h.is_none(),
                Msg::NVersionH(None),
            ));
            for v in &app.versiones {
                let mut r = row![
                    txt(v.name.clone()).width(Length::Fill),
                    tenue(&p, v.date.clone()).width(90),
                ]
                .spacing(8)
                .align_y(Alignment::Center);
                r = r.push(if v.installed {
                    container(text("instalada").size(11).color(p.ok)).width(70)
                } else {
                    container(tenue(&p, if v.prerelease { "nightly" } else { "estable" })).width(70)
                });
                filas = filas.push(fila(p, r, n.version_h.as_deref() == Some(v.tag.as_str()), Msg::NVersionH(Some(v.tag.clone()))));
            }
            if app.versiones.is_empty() {
                filas = filas.push(tenue(&p, "Pidiendo las versiones a Horizon…"));
            }
            column![
                titulo("Instancia de Horizon"),
                tenue(&p, "Minecraft reescrito en Rust. Tus mundos y opciones de siempre, con mods nativos."),
                nombre,
                seccion(&p, "Versión"),
                container(desplaza(p, filas).height(240)).padding(2).style(tema::caja(p)),
                tenue(&p, "Si eliges una versión que no tienes, se descarga al crear la instancia."),
            ]
            .spacing(12)
            .into()
        }
        Fuente::Java => {
            let mut filas = Column::new().spacing(1);
            for v in n.java.iter().take(120) {
                let r = row![
                    txt(v.id.clone()).width(Length::Fill),
                    tenue(&p, if v.kind == "release" { "versión" } else { "snapshot" }).width(80),
                    tenue(&p, v.released.get(..10).unwrap_or("").to_string()).width(90),
                ]
                .spacing(8);
                filas = filas.push(fila(p, r, n.version_j.as_deref() == Some(v.id.as_str()), Msg::NVersionJ(v.id.clone())));
            }
            if n.java.is_empty() {
                filas = filas.push(tenue(&p, "Pidiendo las versiones a Mojang…"));
            }
            let cargadores: Vec<(String, bool, Option<Msg>)> = [None, Some("fabric"), Some("quilt"), Some("forge"), Some("neoforge")]
                .into_iter()
                .map(|c| {
                    let e = match c {
                        None => "Ninguno",
                        Some("fabric") => "Fabric",
                        Some("quilt") => "Quilt",
                        Some("forge") => "Forge",
                        _ => "NeoForge",
                    };
                    (e.to_string(), n.cargador == c, Some(Msg::NCargador(c)))
                })
                .collect();
            column![
                titulo("Minecraft: Java Edition"),
                tenue(&p, "Horizon descarga el juego y su Java la primera vez que juegas."),
                nombre,
                row![seccion(&p, "Versión"), horizontal_space(), toggler(n.snapshots).label("Snapshots").size(16).text_size(12).on_toggle(Msg::NSnapshots)]
                    .align_y(Alignment::Center),
                container(desplaza(p, filas).height(260)).padding(2).style(tema::caja(p)),
                seccion(&p, "Cargador de mods"),
                segmentado(p, cargadores),
            ]
            .spacing(12)
            .into()
        }
        Fuente::Modpack => {
            let mut filas = Column::new().spacing(1);
            for (i, m) in n.packs.iter().enumerate() {
                let r = row![
                    super::logo(app, m.modrinth.as_deref(), &m.titulo, 36.0),
                    column![row![txt(m.titulo.clone()).font(SEMI), tenue(&p, format!("de {}", m.autor))].spacing(6), tenue(&p, corta(&m.resumen, 120))]
                        .spacing(2)
                        .width(Length::Fill),
                ]
                .spacing(10)
                .align_y(Alignment::Center);
                filas = filas.push(fila(p, r, n.pack == Some(i), Msg::NPack(i)));
            }
            if n.packs.is_empty() {
                filas = filas.push(container(tenue(&p, "Buscando paquetes…")).padding(12));
            }
            let archivo: E = match &n.archivo {
                Some(f) => aviso(p, p.hl, row![txt(format!("Archivo: {f}")), horizontal_space(), btn(p, "Cambiar", Boton::Normal, Some(Msg::NPackArchivo))].align_y(Alignment::Center)),
                None => row![btn(p, "Importar un archivo .mrpack o .zip…", Boton::Normal, Some(Msg::NPackArchivo))].into(),
            };
            column![
                titulo("Paquete de mods"),
                tenue(&p, "Horizon lo instala con todo lo que lleva, como una instancia nueva."),
                nombre,
                text_input("Buscar paquetes (Better MC, Fabulously Optimized…)", &n.texto_pack)
                    .on_input(Msg::NPackTexto)
                    .on_submit(Msg::NPackBuscar)
                    .padding([6, 8])
                    .size(13)
                    .style(tema::campo(p)),
                container(desplaza(p, filas).height(260)).padding(2).style(tema::caja(p)),
                archivo,
            ]
            .spacing(12)
            .into()
        }
        Fuente::Prism => {
            let mut filas = Column::new().spacing(1);
            for i in &app.prism {
                let r = row![
                    txt(i.nombre.clone()).width(Length::Fill),
                    tenue(&p, i.version.clone().unwrap_or_default()).width(70),
                    tenue(&p, i.cargador.as_ref().map(|c| c.0.clone()).unwrap_or_else(|| "Vanilla".into())).width(80),
                ]
                .spacing(8);
                filas = filas.push(fila(p, r, n.prism.as_deref() == Some(i.nombre.as_str()), Msg::NPrism(i.nombre.clone())));
            }
            if app.prism.is_empty() {
                filas = filas.push(tenue(&p, "No encuentro instancias de Prism Launcher."));
            }
            column![
                titulo("Copiar desde Prism"),
                tenue(&p, "Se copian la versión, los mundos, los mods y las opciones a una instancia Java de Horizon. La de Prism no se toca."),
                nombre,
                container(filas).padding(2).style(tema::caja(p)),
            ]
            .spacing(12)
            .into()
        }
    };

    // Lo demás que se elige al crear: carpeta, memoria (Java) y skin.
    let mut carpetas: Vec<String> = vec![crate::app::RAIZ.to_string()];
    carpetas.extend(app.todas_carpetas());
    let carpeta_sel = if n.carpeta.is_empty() { crate::app::RAIZ.to_string() } else { n.carpeta.clone() };
    let mut skins: Vec<String> = vec!["La que llevas".to_string()];
    skins.extend(app.armario.skins.iter().map(|s| s.ruta.to_string_lossy().to_string()));
    let skin_sel = n.skin.clone().unwrap_or_else(|| "La que llevas".into());
    let mut extra = column![
        seccion(&p, "Al crearla"),
        row![
            txt("Carpeta").width(110),
            pick_list(carpetas, Some(carpeta_sel), Msg::NCarpeta).text_size(13).padding([5, 8]).style(tema::lista(p))
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            txt("Skin al jugar").width(110),
            pick_list(skins, Some(skin_sel), |s| Msg::NSkin(if s == "La que llevas" { String::new() } else { s })).text_size(13).padding([5, 8]).style(tema::lista(p))
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    ]
    .spacing(10);
    if matches!(fuente, Fuente::Java | Fuente::Modpack | Fuente::Prism) {
        let total = app.sis.memoria_mib as u32;
        let m = n.memoria.unwrap_or(app.aj.memoria_mib);
        extra = extra.push(
            row![
                txt("Memoria").width(110),
                slider(1024..=total.max(2048), m, Msg::NMemoria).step(512u32).width(240),
                txt(format!("{:.1} GB", m as f32 / 1024.0)).font(SEMI),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        );
    }
    let cuerpo: E = column![cuerpo, extra].spacing(18).into();

    let mut pie = row![].spacing(8).align_y(Alignment::Center);
    if let Some(e) = &n.error {
        pie = pie.push(text(e.clone()).size(12).color(p.mal));
    } else if n.ocupado {
        pie = pie.push(tenue(&p, "Creando… (si hay que descargar, tarda un poco)"));
    }
    pie = pie.push(horizontal_space()).push(btn(p, "Cancelar", Boton::Normal, Some(Msg::Cerrar))).push(btn(
        p,
        match fuente {
            Fuente::Prism => "Copiar",
            Fuente::Modpack => "Instalar",
            _ => "Crear",
        },
        Boton::Primario,
        (!n.ocupado).then_some(Msg::NCrear),
    ));

    marco(p, "Nueva instancia".into(), Some(fuentes.into()), cuerpo, Some(pie.into()))
}

fn corta(t: &str, n: usize) -> String {
    if t.chars().count() <= n {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(n).collect::<String>().trim_end())
    }
}
