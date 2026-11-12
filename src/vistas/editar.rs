//! Editar instancia, con las páginas de Prism: versión, mods, paquetes, shaders, mundos, capturas, ajustes y registro.

use super::*;
use iced::widget::{column, row};
use crate::app::Archivo;
use iced::widget::{pick_list, slider};

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let ed = &app.editar;
    let Some(inst) = app.instancia(&ed.clave) else {
        return marco(p, "Editar".into(), None, tenue(&p, "Esta instancia ya no existe.").into(), None);
    };
    let pag = ed.pagina.unwrap_or(PagE::Version);
    let rust = inst.motor == Motor::Rust;
    let lista = vec![
        ("Versión".to_string(), None, pag == PagE::Version, Msg::EPagina(PagE::Version)),
        (
            if rust { "Mods nativos" } else { "Mods" }.to_string(),
            (pag == PagE::Mods).then(|| (ed.mods.len() + ed.mods_h.len()).to_string()),
            pag == PagE::Mods,
            Msg::EPagina(PagE::Mods),
        ),
        ("Paquetes de recursos".into(), None, pag == PagE::Recursos, Msg::EPagina(PagE::Recursos)),
        ("Shaders".into(), None, pag == PagE::Shaders, Msg::EPagina(PagE::Shaders)),
        ("Mundos".into(), None, pag == PagE::Mundos, Msg::EPagina(PagE::Mundos)),
        ("Capturas".into(), None, pag == PagE::Capturas, Msg::EPagina(PagE::Capturas)),
        ("Ajustes".into(), None, pag == PagE::Ajustes, Msg::EPagina(PagE::Ajustes)),
        ("Skin".into(), None, pag == PagE::Skin, Msg::EPagina(PagE::Skin)),
        ("Registro".into(), None, pag == PagE::Registro, Msg::EPagina(PagE::Registro)),
    ];
    let cabeza = column![
        row![icono(app, inst, 32.0), column![txt(inst.nombre.clone()).font(SEMI), pildora(&p, inst.motor, inst.etiqueta_motor())].spacing(4)]
            .spacing(8)
            .align_y(Alignment::Center),
        Space::new(0, 10),
        paginas(p, lista)
    ];

    let cuerpo: E = match pag {
        PagE::Version => version(app, inst),
        PagE::Mods => mods(app, inst),
        PagE::Recursos => archivos(app, "Paquetes de recursos", "resourcepacks", &ed.recursos, inst),
        PagE::Shaders => archivos(app, "Shaders", "shaderpacks", &ed.shaders, inst),
        PagE::Mundos => mundos(app),
        PagE::Capturas => capturas(app, inst),
        PagE::Ajustes => ajustes(app, inst),
        PagE::Skin => skin(app, inst),
        PagE::Registro => registro(app),
    };
    let pie = row![
        btn(p, "Abrir carpeta", Boton::Normal, Some(Msg::Carpeta(inst.carpeta.clone()))),
        horizontal_space(),
        btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar)),
        btn(p, "Jugar", Boton::Primario, Some(Msg::Jugar(inst.clave()))),
    ]
    .spacing(8);
    marco(p, format!("Editar · {}", inst.nombre), Some(cabeza.into()), cuerpo, Some(pie.into()))
}

fn version<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let mut col = column![row![
        titulo("Versión"),
        horizontal_space(),
        btn(p, "Cambiar icono", Boton::Normal, Some(Msg::AbrirIcono(inst.clave())))
    ]
    .align_y(Alignment::Center)]
    .spacing(12);
    if inst.origen == Origen::Horizon && inst.id != "horizon" {
        col = col.push(
            row![
                text_input("Nombre", &app.editar.nombre).on_input(Msg::ENombre).on_submit(Msg::ERenombrar).padding([6, 8]).size(13).style(tema::campo(p)),
                btn(p, "Renombrar", Boton::Normal, Some(Msg::ERenombrar)),
            ]
            .spacing(8),
        );
    }
    match (inst.motor, inst.origen) {
        (Motor::Rust, _) => {
            let mut opciones = vec!["latest".to_string()];
            opciones.extend(app.versiones.iter().map(|v| v.tag.clone()));
            let actual = if inst.sigue_ultima { "latest".to_string() } else { inst.version.clone().unwrap_or_default() };
            col = col
                .push(tenue(&p, "Horizon mantiene el juego; aquí eliges si esta instancia sigue la última versión o se queda en una."))
                .push(
                    row![
                        txt("Versión de Horizon").width(160),
                        pick_list(opciones, Some(actual), Msg::EVersion).text_size(13).padding([5, 8]).style(tema::lista(p))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .push(tenue(&p, "«latest» = seguir la última. Una versión que no tengas se descarga antes."));
        }
        (Motor::Java, Origen::Horizon) => {
            let actual = inst.cargador.as_ref().map(|c| c.0.clone()).unwrap_or_else(|| "Ninguno".into());
            let elegido = app.editar.cargador_nuevo.clone().unwrap_or(actual);
            let opciones: Vec<String> = ["Ninguno", "Fabric", "Quilt", "Forge", "NeoForge"].iter().map(|s| s.to_string()).collect();
            col = col
                .push(super::dato(&p, "Minecraft", inst.version.clone().unwrap_or_else(|| "?".into())))
                .push(
                    row![
                        txt("Cargador de mods").width(160),
                        pick_list(opciones, Some(elegido), Msg::ECargador).text_size(13).padding([5, 8]).style(tema::lista(p))
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .push(tenue(&p, "Al cambiarlo se pone su versión estable más nueva para tu Minecraft."));
        }
        (Motor::Java, Origen::Prism) => {
            col = col.push(super::dato(&p, "Minecraft", inst.version.clone().unwrap_or_else(|| "?".into())));
            if let Some((c, v)) = &inst.cargador {
                col = col.push(super::dato(&p, "Cargador", format!("{c} {v}")));
            }
            col = col.push(aviso(
                p,
                p.java,
                column![
                    txt("Esta instancia es de Prism"),
                    tenue(&p, "Sus componentes se cambian en Prism. Si quieres llevarla a Horizon, cópiala: la de Prism se queda igual."),
                    btn(p, "Copiar a Horizon", Boton::Normal, Some(Msg::CopiarAHorizon(inst.clave())))
                ]
                .spacing(6),
            ));
        }
    }
    col.into()
}

fn mods<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let ed = &app.editar;
    let mut col = column![row![
        titulo(if inst.motor == Motor::Rust { "Mods nativos" } else { "Mods" }),
        horizontal_space(),
        btn(p, "Descargar mods", Boton::Primario, Some(Msg::ExplorarPara(inst.clave(), "mod"))),
        btn(p, "Quitar", Boton::Peligro, ed.sel.map(|_| Msg::EQuitar)),
    ]
    .spacing(8)
    .align_y(Alignment::Center)]
    .spacing(10);

    if inst.motor == Motor::Rust && inst.origen == Origen::Horizon {
        col = col.push(compilar(app, inst));
    }

    if !ed.mods_h.is_empty() {
        col = col.push(seccion(&p, "Instalados con el buscador"));
        let mut t = Column::new().spacing(1);
        for m in &ed.mods_h {
            t = t.push(
                container(
                    row![
                        txt(m.titulo.clone()).font(MEDIA).width(Length::Fill),
                        tenue(&p, m.version.clone()).width(90),
                        tenue(&p, if m.nativo { "nativo".to_string() } else if m.dependencia { "dependencia".to_string() } else { m.fuente.clone() }).width(90),
                        btn(p, "Quitar", Boton::Peligro, Some(Msg::EQuitarH(m.proyecto.clone()))).padding([2, 8]),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                )
                .padding([4, 10]),
            );
        }
        col = col.push(container(t).padding(2).style(tema::caja(p)));
    }
    if !ed.mods.is_empty() || ed.mods_h.is_empty() {
        col = col.push(seccion(&p, "Archivos en mods/"));
        col = col.push(tabla(app, &ed.mods, "Sin mods todavía. Busca uno o suelta el .jar en la carpeta mods."));
    }
    col.into()
}

/// Compilar los mods nativos (.hnmod): Horizon tiene que compilar el juego con ellos para que carguen.
/// Pide confirmación porque su código corre con acceso total al ordenador.
fn compilar<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let clave = inst.clave();
    let nativos = app.editar.mods_h.iter().filter(|m| m.nativo).count();
    if let Some(l) = app.compilando.get(&clave) {
        return aviso(
            p,
            p.hl,
            column![
                text("Compilando el juego con tus mods nativos…").size(13).font(SEMI).color(p.hl),
                text(l.clone()).size(12).color(p.dim),
                tenue(&p, "Puedes cerrar esta ventana o seguir usando el launcher: sigue compilando y te aviso al terminar."),
            ]
            .spacing(4),
        );
    }
    if app.confirmar_compilar.as_deref() == Some(clave.as_str()) {
        return aviso(
            p,
            p.mal,
            column![
                text("Los mods nativos no van en una caja cerrada").size(13).font(SEMI),
                tenue(&p, "Se ejecutan con acceso total a tu ordenador, como cualquier programa. Compila solo si confías en quien los hizo (los del registro de Horizon están revisados; los de GitHub, no)."),
                tenue(&p, "La primera vez Horizon baja su propio Rust y tarda de 20 a 30 minutos. Después, solo unos minutos."),
                row![
                    horizontal_space(),
                    btn(p, "Cancelar", Boton::Normal, Some(Msg::PedirCompilar(None))),
                    btn(p, "Confío en ellos: compilar", Boton::Primario, Some(Msg::Compilar(clave.clone()))),
                ]
                .spacing(8),
            ]
            .spacing(6),
        );
    }
    let estado: E = match app.compilado.get(&clave) {
        Some(Ok(())) => text("✓ Compilado: ya cargan al jugar").size(12).color(p.ok).into(),
        Some(Err(e)) => text(format!("Falló: {e}")).size(12).color(p.mal).into(),
        None if nativos == 0 => tenue(&p, "No tienes mods nativos instalados. Búscalos en Descargar mods → Horizon (nativos).").into(),
        None => tenue(&p, "Después de instalar, quitar o actualizar mods nativos, compila para que carguen al jugar.").into(),
    };
    aviso(
        p,
        p.rust,
        row![
            column![text(format!("Mods nativos: {nativos}")).size(13).font(SEMI), estado].spacing(3).width(Length::Fill),
            btn(p, "Compilar mods", Boton::Primario, (nativos > 0).then(|| Msg::PedirCompilar(Some(clave.clone())))),
        ]
        .spacing(12)
        .align_y(Alignment::Center),
    )
}

/// Tabla de archivos con interruptor. La fila elegida usa el tinte de selección, distinto del botón «Quitar».
fn tabla<'a>(app: &'a App, lista: &'a [Archivo], vacio: &'a str) -> E<'a> {
    let p = app.p;
    if lista.is_empty() {
        return container(tenue(&p, vacio)).padding(14).width(Length::Fill).style(tema::caja(p)).into();
    }
    let mut t = Column::new().spacing(1);
    for (i, a) in lista.iter().enumerate() {
        let sel = app.editar.sel == Some(i);
        let archivo = a.nombre.trim_end_matches(".jar").trim_end_matches(".zip").to_string();
        let (id, titulo) = match &a.proyecto {
            Some((id, t)) if !t.is_empty() => (Some(id.as_str()), t.clone()),
            Some((id, _)) => (Some(id.as_str()), archivo.clone()),
            None => (None, archivo.clone()),
        };
        let nombre: E = if titulo != archivo {
            column![text(titulo.clone()).size(13).font(MEDIA).color(if a.activo { p.text } else { p.dim }), tenue(&p, archivo.clone())]
                .spacing(1)
                .width(Length::Fill)
                .into()
        } else {
            text(archivo.clone()).size(13).color(if a.activo { p.text } else { p.dim }).width(Length::Fill).into()
        };
        let r = row![
            toggler(a.activo).size(16).on_toggle(move |b| Msg::EActivar(i, b)),
            super::logo(app, id, &titulo, 32.0),
            nombre,
            tenue(&p, crate::datos::tamano(a.tamano)).width(70),
        ]
        .spacing(10)
        .align_y(Alignment::Center);
        t = t.push(fila(p, r, sel, Msg::ESel(i)));
    }
    container(t).padding(2).style(tema::caja(p)).into()
}

fn archivos<'a>(app: &'a App, nombre: &'a str, sub: &'a str, lista: &'a [Archivo], inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    column![
        row![
            titulo(nombre),
            horizontal_space(),
            btn(
                p,
                if sub == "shaderpacks" { "Descargar shaders" } else { "Descargar paquetes" },
                Boton::Primario,
                Some(Msg::ExplorarPara(inst.clave(), if sub == "shaderpacks" { "shader" } else { "resourcepack" })),
            ),
            btn(p, "Abrir carpeta", Boton::Normal, Some(Msg::Carpeta(inst.carpeta.join(sub)))),
            btn(p, "Quitar", Boton::Peligro, app.editar.sel.map(|_| Msg::EQuitar)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        tabla(app, lista, "Vacío. Suelta los .zip en la carpeta y vuelve a esta página."),
    ]
    .spacing(10)
    .into()
}

fn mundos(app: &App) -> E<'_> {
    let p = app.p;
    let mut t = Column::new().spacing(1);
    for m in &app.editar.mundos {
        t = t.push(
            container(
                row![
                    match &m.icono {
                        Some(h) => Element::from(container(image(h.clone()).width(40).height(40)).style(tema::caja(p)).padding(1)),
                        None => super::logo(app, None, &m.nombre, 40.0),
                    },
                    txt(m.nombre.clone()).font(MEDIA).width(Length::Fill),
                    tenue(&p, m.modo.clone()).width(90),
                    tenue(&p, m.version.clone()).width(60),
                    tenue(&p, if m.tamano > 0 { crate::datos::tamano(m.tamano) } else { String::new() }).width(70),
                    tenue(&p, m.ultima_ms.map(crate::datos::hace).unwrap_or_default()).width(110),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .padding([6, 10]),
        );
    }
    if app.editar.mundos.is_empty() {
        t = t.push(container(tenue(&p, "Sin mundos todavía.")).padding(14));
    }
    column![titulo("Mundos"), container(t).padding(2).style(tema::caja(p))].spacing(10).into()
}

fn capturas<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let fotos = app.editar.capturas.iter().map(|(f, h)| {
        button(image(h.clone()).width(196).height(110).content_fit(iced::ContentFit::Cover))
            .padding(2)
            .style(tema::boton(p, Boton::Casilla(false)))
            .on_press(Msg::Url(f.to_string_lossy().to_string()))
            .into()
    });
    let mut col = column![row![
        titulo("Capturas"),
        horizontal_space(),
        btn(p, "Abrir carpeta", Boton::Normal, Some(Msg::Carpeta(inst.carpeta.join("screenshots"))))
    ]
    .align_y(Alignment::Center)]
    .spacing(10);
    if app.editar.capturas.is_empty() {
        col = col.push(tenue(&p, "Las que hagas con F2 aparecen aquí."));
    } else {
        col = col.push(row(fotos).spacing(6).wrap());
    }
    col.into()
}

fn ajustes<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    let p = app.p;
    let ed = &app.editar;
    if inst.origen == Origen::Prism {
        return column![
            titulo("Ajustes de la instancia"),
            aviso(p, p.java, tenue(&p, "Memoria, Java y envoltorios de esta instancia se cambian en Prism, que es quien la arranca.")),
        ]
        .spacing(10)
        .into();
    }
    let mut col = column![titulo("Ajustes de la instancia")].spacing(10);
    if let Some(j) = &ed.java {
        let total = app.sis.memoria_mib as u32;
        col = col
            .push(seccion(&p, "Java"))
            .push(
                row![
                    txt("Memoria máxima").width(160),
                    slider(1024..=total.max(2048), j.memoria_mib, Msg::EMem).step(512u32).width(260),
                    txt(format!("{:.1} GB", j.memoria_mib as f32 / 1024.0)).font(SEMI),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            )
            .push(tenue(&p, "Argumentos de la JVM"))
            .push(text_input("-XX:+UseG1GC …", &j.argumentos).on_input(Msg::EArgs).padding([6, 8]).size(12).font(iced::Font::MONOSPACE).style(tema::campo(p)));
        if app.aj.gestionar_java {
            col = col.push(tenue(&p, "Ahora mismo el perfil de Rendimiento pisa esto al jugar. Si guardas aquí, deja de hacerlo."));
        }
    }
    match &ed.lanz {
        None => col = col.push(tenue(&p, "Leyendo los ajustes de Horizon…")),
        Some(l) => {
            col = col
                .push(seccion(&p, "Arranque"))
                .push(tenue(&p, "Envoltorio (el juego se ejecuta a través de esto)"))
                .push(text_input("gamemoderun, nice -n 5 …", &l.envoltorio).on_input(Msg::EEnvoltorio).padding([6, 8]).size(12).font(iced::Font::MONOSPACE).style(tema::campo(p)))
                .push(tenue(&p, "Variables de entorno (NOMBRE=valor separadas por espacios)"))
                .push(text_input("__GL_THREADED_OPTIMIZATIONS=1", &l.entorno).on_input(Msg::EEnv).padding([6, 8]).size(12).font(iced::Font::MONOSPACE).style(tema::campo(p)))
                .push(interruptor(
                    p,
                    "GameMode",
                    if l.hay_gamemode { "Pide al sistema prioridad para el juego" } else { "No está instalado" },
                    l.gamemode,
                    l.hay_gamemode.then(|| Box::new(Msg::EGamemode) as Box<dyn Fn(bool) -> Msg>),
                ))
                .push(interruptor(
                    p,
                    "MangoHud",
                    if l.hay_mangohud { "FPS y tiempos de cuadro encima del juego" } else { "No está instalado" },
                    l.mangohud,
                    l.hay_mangohud.then(|| Box::new(Msg::EMango) as Box<dyn Fn(bool) -> Msg>),
                ))
                .push(interruptor(
                    p,
                    "Gráfica dedicada",
                    if l.nvidia { "Usa la NVIDIA" } else { "Solo con dos gráficas" },
                    l.gpu_dedicada,
                    (l.nvidia || l.dos_gpus).then(|| Box::new(Msg::EGpu) as Box<dyn Fn(bool) -> Msg>),
                ));
        }
    }
    col = col.push(row![horizontal_space(), btn(p, "Guardar", Boton::Primario, Some(Msg::EGuardar))]);
    col.into()
}

fn registro(app: &App) -> E<'_> {
    let p = app.p;
    column![
        row![titulo("Registro"), horizontal_space(), btn(p, "Recargar", Boton::Normal, Some(Msg::ERegistro))].align_y(Alignment::Center),
        container(text(app.editar.registro.clone()).size(11).font(iced::Font::MONOSPACE))
            .padding(10)
            .width(Length::Fill)
            .style(tema::caja(p)),
    ]
    .spacing(10)
    .into()
}


/// Qué skin te pone al jugar esta instancia: la que llevas, una del armario o una importada.
fn skin<'a>(app: &'a App, inst: &'a Instancia) -> E<'a> {
    use iced::widget::image::FilterMethod;
    let p = app.p;
    let elegida = app.aj.skin_instancia.get(&inst.clave()).cloned();
    let mut lista = Column::new().spacing(2);
    lista = lista.push(fila(
        p,
        row![Space::new(24, 24), txt("La que llevas equipada").font(MEDIA), horizontal_space(), tenue(&p, "no cambia nada")].spacing(10).align_y(Alignment::Center),
        elegida.is_none(),
        Msg::ESkin(String::new()),
    ));
    for s in &app.armario.skins {
        let ruta = s.ruta.to_string_lossy().to_string();
        lista = lista.push(fila(
            p,
            row![image(s.cara.clone()).width(24).height(24).filter_method(FilterMethod::Nearest), txt(s.nombre.clone())]
                .spacing(10)
                .align_y(Alignment::Center),
            elegida.as_deref() == Some(ruta.as_str()),
            Msg::ESkin(ruta),
        ));
    }
    column![
        titulo("Skin al jugar"),
        tenue(&p, "Al darle a Jugar se pone esta skin en tu cuenta. Mojang guarda la skin en la cuenta, así que es la que verán todos hasta que juegues otra instancia o la cambies en el armario."),
        container(lista).padding(2).style(tema::caja(p)),
        row![
            btn(p, "Importar una skin…", Boton::Normal, Some(Msg::SImportar)),
            btn(p, "Abrir el armario", Boton::Normal, Some(Msg::Abrir(Dlg::Armario))),
        ]
        .spacing(8),
    ]
    .spacing(12)
    .into()
}
