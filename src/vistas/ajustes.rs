//! Ajustes del launcher: rendimiento, apariencia, Horizon y avanzado.

use super::*;
use iced::widget::{column, row};
use crate::ajustes::{Perfil, Recolector, Tamano};
use crate::app::{Flag, PagA};
use crate::tema::Tema;
use iced::widget::slider;

type Cambio<'a> = Option<Box<dyn Fn(bool) -> Msg + 'a>>;

fn flag<'a>(f: Flag) -> Cambio<'a> {
    Some(Box::new(move |b| Msg::AFlag(f, b)))
}

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let pag = app.pag_a;
    let lista = vec![
        ("Rendimiento".to_string(), None, pag == PagA::Rendimiento, Msg::APagina(PagA::Rendimiento)),
        ("Apariencia".into(), None, pag == PagA::Apariencia, Msg::APagina(PagA::Apariencia)),
        ("Horizon".into(), None, pag == PagA::Horizon, Msg::APagina(PagA::Horizon)),
        ("Red y sistema".into(), None, pag == PagA::Red, Msg::APagina(PagA::Red)),
        ("Avanzado".into(), None, pag == PagA::Avanzado, Msg::APagina(PagA::Avanzado)),
    ];
    let cuerpo = match pag {
        PagA::Rendimiento => rendimiento(app),
        PagA::Apariencia => apariencia(app),
        PagA::Horizon => horizon(app),
        PagA::Red => red(app),
        PagA::Avanzado => avanzado(app),
    };
    let pie = row![tenue(&p, "Los cambios se guardan al momento."), horizontal_space(), btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar))]
        .align_y(Alignment::Center);
    marco(p, "Ajustes".into(), Some(paginas(p, lista)), cuerpo, Some(pie.into()))
}

fn rendimiento(app: &App) -> E<'_> {
    let p = app.p;
    let aj = &app.aj;
    let sis = &app.sis;
    let perfil = |pf: Perfil| -> E<'_> {
        let sel = aj.perfil == pf;
        button(column![txt(pf.nombre()).font(SEMI), tenue(&p, pf.detalle())].spacing(4))
            .width(Length::Fill)
            .height(96)
            .padding(10)
            .style(tema::boton(p, Boton::Opcion(sel)))
            .on_press(Msg::APerfil(pf))
            .into()
    };
    let mut perfiles = row![perfil(Perfil::Maximo), perfil(Perfil::Equilibrado), perfil(Perfil::Silencioso)].spacing(8);
    if aj.perfil == Perfil::Propio {
        perfiles = perfiles.push(perfil(Perfil::Propio));
    }
    let gpu = sis.nvidia.clone().unwrap_or_else(|| "no detectada".into());
    let total = sis.memoria_mib as u32;
    let rec = sis.memoria_recomendada();

    column![
        titulo("Rendimiento"),
        tenue(&p, format!("Tu PC: {:.0} GB de RAM, {} hilos, gráfica {gpu}.", sis.memoria_mib as f32 / 1024.0, sis.hilos)),
        perfiles,
        seccion(&p, "Mientras juegas"),
        interruptor(
            p,
            "Cerrar el launcher al jugar",
            format!("Queda un vigilante de unos 4 MB que lo vuelve a abrir al salir. Ahora ocupa {} MB.", app.memoria_mb),
            aj.cerrar_al_jugar,
            flag(Flag::Cerrar)
        ),
        interruptor(
            p,
            "GameMode",
            if sis.gamemode { "Prioridad de CPU y gobernador de rendimiento mientras juegas" } else { "No está instalado (paquete gamemode)" },
            aj.gamemode && sis.gamemode,
            if sis.gamemode { flag(Flag::Gamemode) } else { None }
        ),
        interruptor(
            p,
            "MangoHud",
            if sis.mangohud { "FPS y tiempos de cuadro encima del juego" } else { "No está instalado (paquete mangohud)" },
            aj.mangohud && sis.mangohud,
            if sis.mangohud { flag(Flag::Mangohud) } else { None }
        ),
        interruptor(p, "Gráfica dedicada", format!("Arranca el juego en {gpu}"), aj.gpu_dedicada, sis.nvidia.is_some().then(|| flag(Flag::Gpu)).flatten()),
        interruptor(
            p,
            "Optimizaciones de hilos de NVIDIA",
            "__GL_THREADED_OPTIMIZATIONS: más FPS en Java cuando la CPU va justa",
            aj.hilos_nvidia,
            sis.nvidia.is_some().then(|| flag(Flag::HilosNvidia)).flatten()
        ),
        seccion(&p, "Java (instancias Java de Horizon)"),
        row![
            txt("Memoria máxima").width(160),
            slider(1024..=total.max(2048), aj.memoria_mib, Msg::AMem).step(512u32).width(260),
            txt(format!("{:.1} GB", aj.memoria_mib as f32 / 1024.0)).font(SEMI),
            tenue(&p, format!("recomendado {} GB", rec / 1024)),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        row![
            txt("Recolector").width(160),
            segmentado(
                p,
                vec![
                    ("G1 ajustado".into(), aj.recolector == Recolector::G1Aikar, Some(Msg::ARec(Recolector::G1Aikar))),
                    ("ZGC".into(), aj.recolector == Recolector::Zgc, Some(Msg::ARec(Recolector::Zgc))),
                    ("El de Java".into(), aj.recolector == Recolector::Normal, Some(Msg::ARec(Recolector::Normal))),
                ]
            )
        ]
        .align_y(Alignment::Center),
        interruptor(
            p,
            "Páginas grandes de memoria",
            "Transparent Huge Pages: menos trabajo de la CPU con mucha memoria",
            aj.paginas_grandes,
            flag(Flag::PaginasGrandes)
        ),
        interruptor(
            p,
            "Aplicar el perfil al jugar",
            "Antes de cada partida se pone esta memoria y estos argumentos a la instancia",
            aj.gestionar_java,
            flag(Flag::GestionarJava)
        ),
        container(text(aj.argumentos_jvm()).size(11).font(iced::Font::MONOSPACE)).padding(8).width(Length::Fill).style(tema::caja(p)),
        row![horizontal_space(), btn(p, "Aplicar ya a todas las Java de Horizon", Boton::Normal, Some(Msg::AAplicarJava))],
        seccion(&p, "El propio launcher"),
        interruptor(
            p,
            "Dibujar con la gráfica",
            "Más fluido al desplazar, pero el launcher pasa de unos 46 MB a unos 150 MB (carga el controlador de la gráfica). Se aplica al volver a abrir; mientras juegas está cerrado igualmente.",
            aj.grafica,
            flag(Flag::Grafica)
        ),
        seccion(&p, "Horizon (Rust)"),
        interruptor(
            p,
            "Servidor integrado en su propio hilo",
            "--server-thread: el mundo de un jugador no frena el dibujado",
            aj.hilo_servidor,
            flag(Flag::HiloServidor)
        ),
        aviso(
            p,
            p.hl,
            tenue(&p, "Las instancias de Prism usan los ajustes de Prism: este launcher solo le pide que las arranque.")
        ),
    ]
    .spacing(10)
    .into()
}

fn apariencia(app: &App) -> E<'_> {
    let p = app.p;
    let aj = &app.aj;
    let tarjeta = |t: Tema| -> E<'_> {
        let tp = t.paleta();
        let muestra = row![
            container(Space::new(22, 22)).style(tema::fondo(tp.base)),
            container(Space::new(22, 22)).style(tema::fondo(tp.panel)),
            container(Space::new(22, 22)).style(tema::fondo(tp.hl)),
            container(Space::new(22, 22)).style(tema::fondo(tp.rust)),
            container(Space::new(22, 22)).style(tema::fondo(tp.java)),
        ];
        button(column![container(muestra).style(tema::caja(tp)).padding(1), txt(t.nombre()).font(SEMI), tenue(&p, t.detalle())].spacing(6))
            .width(Length::Fill)
            .padding(10)
            .style(tema::boton(p, Boton::Opcion(aj.tema == t)))
            .on_press(Msg::ATema(t))
            .into()
    };
    column![
        titulo("Apariencia"),
        seccion(&p, "Tema"),
        row(Tema::TODOS.iter().map(|t| tarjeta(*t))).spacing(8),
        seccion(&p, "Rust o Java"),
        tenue(&p, "Cada pieza se enciende por separado. Apaga el logo y deja el hexágono para distinguirlas solo por la forma."),
        interruptor(p, "Logo del motor", "Engranaje para Rust, taza para Java, en la esquina del icono", aj.insignia, flag(Flag::Insignia)),
        interruptor(p, "Filo de color", "Una raya a la izquierda del icono", aj.filo, flag(Flag::Filo)),
        interruptor(p, "Forma hexagonal para Rust", "Las instancias de Horizon llevan una placa de seis lados", aj.hexagono, flag(Flag::Hexagono)),
        interruptor(p, "Etiqueta bajo el nombre", "RUST · nightly 11-08 o JAVA · Fabric", aj.etiqueta, flag(Flag::Etiqueta)),
        interruptor(p, "Mascota en la esquina", "Abajo del panel de la derecha, como el gato de Prism (también con el botón del gato)", aj.esquina, flag(Flag::Esquina)),
        row![
            txt("Mascota").width(160),
            segmentado(
                p,
                [("auto", "Según el tema"), ("kitteh", "Kitteh"), ("rory", "Rory"), ("teawie", "Teawie"), ("jiyu", "Jiyu")]
                    .iter()
                    .map(|(k, e)| (e.to_string(), aj.mascota == *k, Some(Msg::AMascota(k.to_string()))))
                    .collect()
            )
        ]
        .align_y(Alignment::Center),
        seccion(&p, "Iconos"),
        interruptor(
            p,
            "Bloques de tu Minecraft",
            "Los iconos de serie de Prism se cambian por texturas de tus archivos preparados de Horizon",
            aj.texturas_mc,
            flag(Flag::Texturas)
        ),
        row![
            txt("Tamaño").width(160),
            segmentado(
                p,
                vec![
                    ("Pequeño".into(), aj.tamano == Tamano::Pequeno, Some(Msg::ATamano(Tamano::Pequeno))),
                    ("Normal".into(), aj.tamano == Tamano::Normal, Some(Msg::ATamano(Tamano::Normal))),
                    ("Grande".into(), aj.tamano == Tamano::Grande, Some(Msg::ATamano(Tamano::Grande))),
                ]
            )
        ]
        .align_y(Alignment::Center),
        interruptor(p, "Mostrar las instancias de Prism", "Leídas de su carpeta, sin tocarlas", aj.ver_prism, flag(Flag::VerPrism)),
    ]
    .spacing(10)
    .into()
}

fn horizon(app: &App) -> E<'_> {
    let p = app.p;
    let Some(ah) = &app.ah else {
        return column![titulo("Horizon"), tenue(&p, "Preguntando a Horizon…")].spacing(10).into();
    };
    let perfiles = [
        ("performance", "Rendimiento", "Compila por partes: lo normal"),
        ("dev", "Desarrollo", "El mod se rehace antes, el juego va algo más lento"),
        ("release", "Publicación", "Tarda más en compilar y juega un poco mejor"),
    ];
    let cache = match app.cache_nativa {
        Some((usado, limite)) if limite > 0 => format!("{} de {}", crate::datos::tamano(usado), crate::datos::tamano(limite)),
        Some((usado, _)) => crate::datos::tamano(usado),
        None => "…".into(),
    };
    column![
        titulo("Horizon"),
        tenue(&p, format!("Versión elegida: {}", ah.version_elegida)),
        seccion(&p, "Mods nativos"),
        row![
            txt("Perfil de compilación").width(160),
            segmentado(p, perfiles.iter().map(|(k, e, _)| (e.to_string(), ah.perfil_nativo == *k, Some(Msg::AHPerfil(k)))).collect())
        ]
        .align_y(Alignment::Center),
        tenue(&p, perfiles.iter().find(|(k, _, _)| ah.perfil_nativo == *k).map(|x| x.2).unwrap_or("")),
        row![
            txt("Límite de la caché").width(160),
            segmentado(p, [10u64, 20, 30, 50].iter().map(|g| (format!("{g} GB"), ah.cache_gib == *g, Some(Msg::AHCache(*g)))).collect()),
            tenue(&p, format!("ocupa {cache}")),
        ]
        .spacing(10)
        .align_y(Alignment::Center),
        seccion(&p, "Versiones"),
        interruptor(
            p,
            "Instalar versiones nuevas solas",
            "Al abrir, si Horizon tiene una nueva, se baja en segundo plano",
            ah.instalar_nuevas,
            Some(Box::new(Msg::AHNuevas))
        ),
        interruptor(
            p,
            "Borrar versiones viejas",
            "Tras instalar una nueva, se quitan las viejas que no use ninguna instancia",
            ah.podar,
            Some(Box::new(Msg::AHPodar))
        ),
        row![
            txt("Versiones a guardar").width(160),
            segmentado(p, [1u64, 2, 3, 5].iter().map(|n| (n.to_string(), ah.guardar == *n, Some(Msg::AHGuardarN(*n)))).collect())
        ]
        .align_y(Alignment::Center),
        interruptor(
            p,
            "Buscar actualizaciones al abrir el launcher",
            "Una sola consulta al arrancar; nunca mientras juegas",
            app.aj.buscar_actualizaciones,
            flag(Flag::Actualizaciones)
        ),
        seccion(&p, "Argumentos del juego"),
        row![
            text_input("--server-thread", &app.ah_args).on_input(Msg::AHArgs).on_submit(Msg::AHGuardarArgs).padding([6, 8]).size(12).font(iced::Font::MONOSPACE).style(tema::campo(p)),
            btn(p, "Guardar", Boton::Normal, Some(Msg::AHGuardarArgs)),
        ]
        .spacing(8),
    ]
    .spacing(10)
    .into()
}

fn avanzado(app: &App) -> E<'_> {
    let p = app.p;
    let ruta = |r: Option<std::path::PathBuf>| r.map(|r| r.to_string_lossy().to_string()).unwrap_or_else(|| "no encontrado".into());
    column![
        titulo("Avanzado"),
        super::dato(&p, "Horizon", ruta(crate::motor::binario())),
        super::dato(&p, "Prism", ruta(crate::prism::raiz().map(|r| r.0))),
        super::dato(&p, "Ajustes", crate::ajustes::carpeta_config().to_string_lossy().to_string()),
        super::dato(&p, "Skins", crate::skins::carpeta().to_string_lossy().to_string()),
        row![
            btn(p, "Abrir la carpeta de ajustes", Boton::Normal, Some(Msg::Carpeta(crate::ajustes::carpeta_config()))),
            btn(p, "Recargar instancias", Boton::Normal, Some(Msg::Recargar)),
        ]
        .spacing(8),
        tenue(&p, "Para usar otro horizon-launcher, arranca con la variable JIYU_HORIZON=/ruta/al/binario."),
        tenue(&p, concat!("Jiyu Launcher ", env!("CARGO_PKG_VERSION"), " · sin telemetría")),
    ]
    .spacing(10)
    .into()
}

fn red(app: &App) -> E<'_> {
    let p = app.p;
    let aj = &app.aj;
    let tam = |d: std::path::PathBuf| -> String {
        let mut total = 0u64;
        let mut pila = vec![d];
        while let Some(x) = pila.pop() {
            for e in std::fs::read_dir(&x).into_iter().flatten().flatten() {
                match e.metadata() {
                    Ok(m) if m.is_dir() => pila.push(e.path()),
                    Ok(m) => total += m.len(),
                    _ => {}
                }
            }
        }
        crate::datos::tamano(total)
    };
    column![
        titulo("Red y sistema"),
        seccion(&p, "Proxy"),
        tenue(&p, "Para todo lo que se descarga: Minecraft y Java (Horizon), mods, logos e iconos (el launcher). Vacío = sin proxy."),
        row![
            text_input("http://host:puerto  o  socks5://host:puerto", &app.proxy_texto)
                .on_input(Msg::AProxy)
                .on_submit(Msg::AGuardarProxy)
                .padding([6, 8])
                .size(13)
                .font(iced::Font::MONOSPACE)
                .style(tema::campo(p)),
            btn(p, "Guardar", Boton::Normal, Some(Msg::AGuardarProxy)),
        ]
        .spacing(8),
        tenue(&p, if aj.proxy.is_empty() { "Ahora: sin proxy".to_string() } else { format!("Ahora: {}", aj.proxy) }),
        seccion(&p, "Al jugar"),
        interruptor(p, "Volver a abrir el launcher al cerrar el juego", "Si lo apagas, al salir del juego no vuelve a aparecer", aj.reabrir, flag(Flag::Reabrir)),
        seccion(&p, "Espacio"),
        row![
            column![txt("Logos de mods"), tenue(&p, tam(crate::modrinth::carpeta_cache()))].spacing(2).width(Length::Fill),
            btn(p, "Vaciar", Boton::Normal, Some(Msg::VaciarCache)),
        ]
        .align_y(Alignment::Center),
        row![
            column![txt("Iconos y gatos de Prism"), tenue(&p, tam(crate::prism_iconos::carpeta()))].spacing(2).width(Length::Fill),
        ]
        .align_y(Alignment::Center),
        seccion(&p, "Atajos de teclado"),
        tenue(&p, "Intro: jugar la elegida · Supr: eliminar · Ctrl+F: buscar · Ctrl+N: nueva instancia · F5: recargar · Esc: cerrar"),
        tenue(&p, "Doble clic en una instancia: jugar · Doble clic en un mod: marcarlo para instalar · Clic derecho: menú · Arrastrar: mover u ordenar"),
    ]
    .spacing(10)
    .into()
}
