//! Explorar: mods, paquetes de texturas, shaders, modpacks y data packs de Modrinth o del registro de
//! Horizon. Cada resultado se instala con un clic (o se marcan varios) y va a una cola de descargas que
//! sigue bajando aunque cambies de búsqueda, de tipo o de instancia. Lo que la instancia ya tiene sale
//! como «Instalado».

use super::*;
use crate::app::EstadoPedido;
use iced::widget::{checkbox, column, pick_list, row};

pub fn nombre_tipo(t: &str) -> &'static str {
    match t {
        "mod" => "Mod",
        "resourcepack" => "Texturas",
        "shader" => "Shader",
        "modpack" => "Modpack",
        "datapack" => "Data pack",
        _ => "Archivo",
    }
}

/// Etiqueta pequeña de color (estado de una descarga, «funciona en Horizon»…).
fn chip<'a>(p: &Paleta, color: iced::Color, t: impl text::IntoFragment<'a>) -> E<'a> {
    container(text(t).size(11).font(SEMI).color(color))
        .padding([1, 7])
        .style(tema::aviso(*p, color))
        .into()
}

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let x = &app.explorar;
    let destino_inst = x.instancia.as_ref().and_then(|o| app.instancia(&o.clave));
    let rust = destino_inst.is_some_and(|i| i.motor == Motor::Rust);

    // ---- Columna izquierda: dónde, qué y de dónde ----
    let mut tarjeta = Column::new().spacing(8);
    if let Some(i) = destino_inst {
        let mut detalle = i.version.clone().unwrap_or_else(|| "Horizon".into());
        if let Some((c, _)) = &i.cargador {
            detalle = format!("{detalle} · {c}");
        }
        tarjeta = tarjeta.push(
            row![icono(app, i, 28.0), column![txt(i.nombre.clone()).font(SEMI), tenue(&p, detalle)].spacing(1)]
                .spacing(6)
                .align_y(Alignment::Center),
        );
    }
    tarjeta = tarjeta.push(
        pick_list(app.opciones_instancias(), x.instancia.clone(), Msg::XInstancia)
            .placeholder("Elige instancia")
            .text_size(12)
            .padding([4, 8])
            .width(Length::Fill)
            .style(tema::lista(p)),
    );
    let tipos: Vec<(String, Option<String>, bool, Msg)> = [
        ("Mods", "mod"),
        ("Paquetes de texturas", "resourcepack"),
        ("Shaders", "shader"),
        ("Modpacks", "modpack"),
        ("Data packs", "datapack"),
    ]
    .iter()
    .map(|(e, t)| {
        let marcados = x.marcados.iter().filter(|(_, k)| k == t).count();
        (e.to_string(), (marcados > 0).then(|| format!("{marcados} ✓")), x.tipo == *t, Msg::XTipo(t))
    })
    .collect();
    let fuentes = vec![
        ("Modrinth".to_string(), None, x.fuente == "modrinth", Msg::XFuente("modrinth")),
        ("Horizon (nativos)".into(), None, x.fuente == "horizon", Msg::XFuente("horizon")),
        ("Favoritos".into(), Some(app.aj.favoritos.len().to_string()), x.fuente == "favoritos", Msg::XFuente("favoritos")),
    ];
    let lateral = column![
        seccion(&p, "Instalar en"),
        container(tarjeta).padding(8).style(tema::caja(p)),
        Space::new(0, 6),
        seccion(&p, "Qué buscas"),
        paginas(p, tipos),
        Space::new(0, 6),
        seccion(&p, "De dónde"),
        paginas(p, fuentes),
    ]
    .spacing(6);

    // ---- Búsqueda y filtros ----
    let pista = match x.tipo {
        "resourcepack" => "Busca paquetes de texturas…",
        "shader" => "Busca shaders…",
        "modpack" => "Busca modpacks…",
        "datapack" => "Busca data packs…",
        _ => "Busca mods…",
    };
    let barra = row![
        text_input(pista, &x.texto).id("explorar").on_input(Msg::XTexto).on_submit(Msg::XBuscar).padding([8, 10]).size(14).style(tema::campo(p)),
        btn(p, "Buscar", Boton::Normal, Some(Msg::XBuscar)).padding([7, 14]),
    ]
    .spacing(8)
    .align_y(Alignment::Center);

    let cargadores: Vec<(String, bool, Option<Msg>)> =
        [(None, "Cualquiera"), (Some("fabric"), "Fabric"), (Some("forge"), "Forge"), (Some("neoforge"), "NeoForge"), (Some("quilt"), "Quilt")]
            .iter()
            .map(|(c, e)| (e.to_string(), x.cargador == *c, Some(Msg::XCargador(*c))))
            .collect();
    let version_inst = destino_inst.and_then(|i| i.version.clone().filter(|_| i.motor == Motor::Java));
    let filtros_activos = x.fuente == "modrinth" && !(rust && x.tipo == "mod");
    let mut cuerpo = column![barra].spacing(12);
    if filtros_activos {
        let mut filtros = row![].spacing(8).align_y(Alignment::Center);
        if matches!(x.tipo, "mod" | "modpack") {
            filtros = filtros.push(tenue(&p, "Cargador")).push(segmentado(p, cargadores));
        }
        if let Some(v) = &version_inst {
            filtros = filtros
                .push(horizontal_space())
                .push(toggler(x.por_version).label(format!("Solo para {v}")).size(16).text_size(12).on_toggle(Msg::XPorVersion));
        }
        cuerpo = cuerpo.push(filtros);
    }
    if rust && x.tipo == "mod" && x.fuente == "modrinth" {
        cuerpo = cuerpo.push(aviso(
            p,
            p.rust,
            tenue(&p, "Instancia Rust: Horizon porta el mod Java al instalarlo. «Funciona» y «en parte» son lo que dice Horizon de cada uno."),
        ));
    }
    if rust && x.fuente == "horizon" && x.tipo == "mod" {
        if let Some(i) = destino_inst {
            cuerpo = cuerpo.push(aviso(
                p,
                p.rust,
                row![
                    tenue(&p, "Los mods nativos no cargan hasta que compilas el juego con ellos.").width(Length::Fill),
                    btn(p, "Ir a compilar", Boton::Normal, Some(Msg::AbrirEditar(i.clave(), PagE::Mods))),
                ]
                .spacing(10)
                .align_y(Alignment::Center),
            ));
        }
    }
    if x.tipo == "shader" && rust && x.fuente == "modrinth" {
        cuerpo = cuerpo.push(aviso(p, p.rust, tenue(&p, "Horizon (Rust) usa sus propios shaders: elige «Horizon (nativos)» en De dónde.")));
    }
    if let Some(v) = &x.relajado {
        if !x.resultados.is_empty() {
            cuerpo = cuerpo.push(aviso(
                p,
                p.rust,
                tenue(&p, format!("Nada hecho para {v}: te enseño los de cualquier versión. Si alguno no tiene versión para tu Minecraft, te lo digo al instalar.")),
            ));
        }
    }

    // ---- Cola de descargas ----
    if x.ver_cola && !x.cola.is_empty() {
        let mut lista = Column::new().spacing(0);
        for (k, pd) in x.cola.iter().enumerate() {
            let inst = app.instancia(&pd.inst).map(|i| i.nombre.clone()).unwrap_or_default();
            let (estado, quitar): (E, Option<Msg>) = match &pd.estado {
                EstadoPedido::Esperando => (chip(&p, p.dim, "En cola"), Some(Msg::XQuitarPedido(k))),
                EstadoPedido::Bajando => (chip(&p, p.hl, "Bajando…"), None),
                EstadoPedido::Hecho(t) => (chip(&p, p.ok, t.clone()), None),
                EstadoPedido::Fallo(e) => (text(e.clone()).size(12).color(p.mal).into(), None),
            };
            let mut r = row![
                super::logo(app, pd.m.modrinth.as_deref(), &pd.m.titulo, 26.0),
                column![txt(pd.m.titulo.clone()).font(MEDIA), tenue(&p, format!("{} · en {inst}", nombre_tipo(pd.tipo)))].spacing(1).width(Length::FillPortion(2)),
                container(estado).width(Length::FillPortion(3)),
            ]
            .spacing(10)
            .align_y(Alignment::Center);
            if let Some(m) = quitar {
                r = r.push(btn(p, "Quitar", Boton::Enlace, Some(m)).padding([2, 6]));
            }
            lista = lista.push(container(r).padding([6, 10]));
        }
        let cabeza = row![
            txt("Descargas").font(SEMI),
            horizontal_space(),
            btn(p, "Limpiar", Boton::Enlace, (!x.instalando).then_some(Msg::XLimpiarCola)).padding([2, 6]),
            btn(p, "Ocultar", Boton::Enlace, Some(Msg::XVerCola)).padding([2, 6]),
        ]
        .align_y(Alignment::Center);
        cuerpo = cuerpo.push(container(column![container(cabeza).padding([6, 10]), linea(&p), lista]).style(tema::caja(p)));
    }

    // ---- Resultados ----
    let estado_busqueda: E = if x.buscando {
        text("Buscando…").size(12).color(p.hl).into()
    } else {
        tenue(&p, format!("{}{} resultados", x.resultados.len(), if x.hay_mas { "+" } else { "" })).into()
    };
    cuerpo = cuerpo.push(estado_busqueda);

    let mut res = Column::new().spacing(0);
    for (i, m) in x.resultados.iter().enumerate() {
        let marcado = x.marcados.iter().any(|(k, _)| k.id == m.id);
        let favorito = app.aj.favoritos.iter().any(|f| f.id == m.id);
        let id = m.id.clone();
        let estrella = button(image(if favorito { app.estrella_llena.clone() } else { app.estrella_vacia.clone() }).width(18).height(18))
            .padding(4)
            .style(tema::boton(p, Boton::Barra))
            .on_press(Msg::XFavorito(i));
        // La acción de la fila: según esté en la cola, bajando o ya en la instancia.
        let accion: E = match app.estado_de(m) {
            None => btn(p, "Instalar", Boton::Primario, x.instancia.is_some().then_some(Msg::XInstalar(i))).padding([4, 12]).into(),
            Some(EstadoPedido::Esperando) => btn(p, "En cola", Boton::Normal, None).padding([4, 12]).into(),
            Some(EstadoPedido::Bajando) => btn(p, "Bajando…", Boton::Normal, None).padding([4, 12]).into(),
            Some(EstadoPedido::Hecho(_)) => container(text("✓ Instalado").size(12).font(SEMI).color(p.ok)).padding([5, 4]).into(),
            Some(EstadoPedido::Fallo(_)) => btn(p, "Reintentar", Boton::Normal, Some(Msg::XInstalar(i))).padding([4, 12]).into(),
        };
        let mut meta = row![].spacing(8).align_y(Alignment::Center);
        if m.descargas > 0 {
            meta = meta.push(tenue(&p, format!("{} descargas", descargas(m.descargas))));
        }
        match m.soporte.as_str() {
            "Full" | "Yes" => meta = meta.push(chip(&p, p.ok, "Funciona en Horizon")),
            "Partly" => meta = meta.push(chip(&p, p.rust, "En parte en Horizon")),
            "" => {}
            otro => meta = meta.push(chip(&p, p.dim, otro.to_lowercase())),
        }
        if !m.pagina.is_empty() {
            meta = meta.push(btn(p, "Ver en la web", Boton::Enlace, Some(Msg::Url(m.pagina.clone()))).padding([0, 2]));
        }
        let r = row![
            checkbox("", marcado).on_toggle(move |_| Msg::XMarcar(id.clone())).size(16),
            super::logo(app, m.modrinth.as_deref(), &m.titulo, 44.0),
            column![
                row![txt(m.titulo.clone()).font(SEMI).size(14), tenue(&p, format!("de {}", m.autor))].spacing(6).align_y(Alignment::Center),
                text(corta(&m.resumen, 160)).size(12).color(p.dim),
                meta,
            ]
            .spacing(3)
            .width(Length::Fill),
            estrella,
            container(accion).width(108).align_x(Alignment::End),
        ]
        .spacing(12)
        .align_y(Alignment::Center);
        res = res.push(fila(p, r, marcado || x.sel == Some(i), Msg::XSel(i)));
    }
    if x.hay_mas && !x.resultados.is_empty() {
        let mas = if x.pidiendo_mas {
            btn(p, "Cargando más…", Boton::Normal, None)
        } else {
            btn(p, "Cargar más", Boton::Normal, Some(Msg::XMas))
        };
        res = res.push(container(mas).center_x(Length::Fill).padding(10));
    }
    if x.resultados.is_empty() {
        let vacio = if x.buscando {
            "Buscando…"
        } else if x.fuente == "favoritos" {
            "Aún no tienes favoritos de este tipo: pulsa la estrella de un resultado."
        } else if filtros_activos && (x.por_version && version_inst.is_some() || x.cargador.is_some()) {
            "Nada para esta versión o cargador. Prueba a quitar «Solo para…» o a poner «Cualquiera»."
        } else {
            "Nada por aquí. Prueba con otra palabra."
        };
        res = res.push(container(tenue(&p, vacio)).padding(14));
    }
    cuerpo = cuerpo.push(container(res).padding(2).style(tema::caja(p)));
    if let Some(e) = &x.error {
        cuerpo = cuerpo.push(text(e.clone()).size(12).color(p.mal));
    }

    // ---- Pie: progreso de la cola y lo marcado ----
    let n = x.marcados.len();
    let total = x.cola.len();
    let hechas = x.cola.iter().filter(|c| !matches!(c.estado, EstadoPedido::Esperando | EstadoPedido::Bajando)).count();
    let fallos = x.cola.iter().filter(|c| matches!(&c.estado, EstadoPedido::Fallo(e) if e != "Quitada de la cola")).count();
    let mut pie = row![].spacing(8).align_y(Alignment::Center);
    if let Some(b) = x.cola.iter().find(|c| c.estado == EstadoPedido::Bajando) {
        pie = pie.push(text(format!("Bajando {} · {} de {total}", b.m.titulo, hechas + 1)).size(13).color(p.hl));
    } else if n > 0 {
        pie = pie
            .push(txt(format!("{n} marcados")).font(SEMI))
            .push(btn(p, "Quitar marcas", Boton::Enlace, Some(Msg::XDesmarcarTodo)));
    } else {
        pie = pie.push(tenue(&p, "Pulsa Instalar en un resultado, o marca varios y luego instálalos juntos"));
    }
    pie = pie.push(horizontal_space());
    if total > 0 {
        let etiqueta = if fallos > 0 { format!("Descargas ({fallos} con error)") } else { format!("Descargas ({hechas}/{total})") };
        pie = pie.push(btn(p, etiqueta, if fallos > 0 { Boton::Peligro } else { Boton::Normal }, Some(Msg::XVerCola)));
    }
    pie = pie.push(btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar)));
    if n > 0 {
        pie = pie.push(btn(p, format!("Instalar {n}"), Boton::Primario, x.instancia.is_some().then_some(Msg::XInstalarVarios)));
    }
    // Al llegar abajo, se piden solos los siguientes.
    marco_x(p, "Explorar".into(), Some(lateral.into()), cuerpo.into(), Some(pie.into()), Some(Msg::XDesplazado))
}

fn descargas(n: u64) -> String {
    match n {
        0..=999 => format!("{n}"),
        1000..=999_999 => format!("{:.0} mil", n as f64 / 1000.0),
        _ => format!("{:.1} M", n as f64 / 1_000_000.0),
    }
}

fn corta(t: &str, n: usize) -> String {
    if t.chars().count() <= n {
        t.to_string()
    } else {
        format!("{}…", t.chars().take(n).collect::<String>().trim_end())
    }
}
