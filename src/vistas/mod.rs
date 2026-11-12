//! Pantallas. `principal` es la ventana de Prism; el resto son los diálogos que se abren encima.

mod ajustes;
mod archivo;
mod armario;
mod servidores;
pub mod clic;
mod menu;
mod principal;
mod cuentas;
mod editar;
mod explorar;
pub mod grande;
mod icono;
mod nueva;

use crate::app::{App, Dlg, Msg, PagE};
use crate::datos::{Instancia, Motor, Origen};
use crate::tema::{self, Boton, Paleta};
use iced::font::{self, Font};
use iced::widget::image::FilterMethod;
use iced::widget::{
    button, center, column, container, horizontal_space, image, mouse_area, rich_text, row, scrollable, span, stack, text,
    text_input, toggler, Column, Space, Text,
};
use iced::{Alignment, Element, Length, Padding};

pub const INTER: Font = Font::with_name("Inter Variable");
pub const SEMI: Font = Font { weight: font::Weight::Semibold, ..INTER };
pub const MEDIA: Font = Font { weight: font::Weight::Medium, ..INTER };

pub type E<'a> = Element<'a, Msg>;

// ---------- Piezas comunes ----------

pub fn txt<'a>(s: impl text::IntoFragment<'a>) -> Text<'a> {
    text(s).size(13)
}

pub fn tenue<'a>(p: &Paleta, s: impl text::IntoFragment<'a>) -> Text<'a> {
    text(s).size(12).color(p.dim)
}

pub fn titulo<'a>(s: impl text::IntoFragment<'a>) -> Text<'a> {
    text(s).size(17).font(SEMI)
}

pub fn seccion<'a>(p: &Paleta, s: &str) -> Text<'a> {
    text(s.to_uppercase()).size(11).font(SEMI).color(p.dim)
}

pub fn btn<'a>(p: Paleta, etiqueta: impl text::IntoFragment<'a>, tipo: Boton, msg: Option<Msg>) -> button::Button<'a, Msg> {
    let fuente = if tipo == Boton::Primario { SEMI } else { INTER };
    button(text(etiqueta).size(13).font(fuente)).padding([5, 12]).style(tema::boton(p, tipo)).on_press_maybe(msg)
}

/// Fila de opción con interruptor: título, explicación y el interruptor a la derecha.
pub fn interruptor<'a>(
    p: Paleta,
    nombre: impl text::IntoFragment<'a>,
    detalle: impl text::IntoFragment<'a>,
    valor: bool,
    al_cambiar: Option<Box<dyn Fn(bool) -> Msg + 'a>>,
) -> E<'a> {
    let t = toggler(valor).size(18).on_toggle_maybe(al_cambiar);
    row![column![txt(nombre).font(MEDIA), tenue(&p, detalle)].spacing(2).width(Length::Fill), t]
        .spacing(16)
        .align_y(Alignment::Center)
        .padding([6, 0])
        .into()
}

pub fn segmentado<'a>(p: Paleta, opciones: Vec<(String, bool, Option<Msg>)>) -> E<'a> {
    row(opciones.into_iter().map(|(e, sel, m)| {
        button(text(e).size(12).font(if sel { SEMI } else { INTER }))
            .padding([4, 12])
            .style(tema::boton(p, Boton::Opcion(sel)))
            .on_press_maybe(m)
            .into()
    }))
    .spacing(4)
    .into()
}

pub fn linea<'a>(p: &Paleta) -> E<'a> {
    container(Space::new(Length::Fill, 1)).height(1).style(tema::fondo(p.line)).into()
}

pub fn aviso<'a>(p: Paleta, color: iced::Color, contenido: impl Into<E<'a>>) -> E<'a> {
    container(contenido).padding([10, 12]).width(Length::Fill).style(tema::aviso(p, color)).into()
}

pub fn pildora<'a>(p: &Paleta, motor: Motor, texto: impl text::IntoFragment<'a>) -> E<'a> {
    let c = if motor == Motor::Rust { p.rust } else { p.java };
    container(text(texto).size(11).font(SEMI)).padding([1, 8]).style(tema::pildora(c, p.on_badge)).into()
}

/// Fila clicable de una tabla. Seleccionada lleva el tinte y el borde de resalte, no el relleno del botón
/// principal (así no se confunde con «Instalar» o «Crear»).
pub fn fila<'a>(p: Paleta, contenido: impl Into<E<'a>>, sel: bool, msg: Msg) -> E<'a> {
    button(contenido).width(Length::Fill).padding([6, 10]).style(tema::boton(p, Boton::Fila(sel))).on_press(msg).into()
}

pub fn desplaza<'a>(p: Paleta, contenido: impl Into<E<'a>>) -> scrollable::Scrollable<'a, Msg> {
    scrollable(contenido).style(tema::desplazable(p))
}

/// El marco de todos los diálogos: título, páginas a la izquierda, contenido y pie.
pub fn marco<'a>(p: Paleta, nombre: String, paginas: Option<E<'a>>, cuerpo: E<'a>, pie: Option<E<'a>>) -> E<'a> {
    marco_x(p, nombre, paginas, cuerpo, pie, None)
}

/// Como `marco`, pero avisa con `al_bajar(posición 0–1)` cada vez que se desplaza el contenido.
pub fn marco_x<'a>(
    p: Paleta,
    nombre: String,
    paginas: Option<E<'a>>,
    cuerpo: E<'a>,
    pie: Option<E<'a>>,
    al_bajar: Option<fn(f32) -> Msg>,
) -> E<'a> {
    let cabecera = row![
        text(nombre).size(15).font(SEMI),
        horizontal_space(),
        button(text("×").size(16)).padding([2, 8]).style(tema::boton(p, Boton::Barra)).on_press(Msg::Cerrar)
    ]
    .align_y(Alignment::Center)
    .padding([8, 12]);
    let mut barra_v = desplaza(p, container(cuerpo).padding(18).width(Length::Fill)).height(Length::Fill);
    if let Some(f) = al_bajar {
        barra_v = barra_v.on_scroll(move |v| f(v.relative_offset().y));
    }
    let contenido = container(barra_v)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(tema::fondo(p.base));
    let medio: E = match paginas {
        Some(pg) => row![
            container(desplaza(p, container(pg).padding(8))).width(200).height(Length::Fill).style(tema::fondo(p.panel)),
            container(Space::new(1, Length::Fill)).width(1).style(tema::fondo(p.line)),
            contenido
        ]
        .height(Length::Fill)
        .into(),
        None => contenido.into(),
    };
    let mut col = column![cabecera, linea(&p), medio];
    if let Some(pie) = pie {
        col = col.push(linea(&p)).push(container(pie).padding([10, 12]).width(Length::Fill));
    }
    container(col).width(Length::Fill).height(Length::Fill).max_width(1160).max_height(820.0).style(tema::dialogo(p)).into()
}

/// Lista de páginas de un diálogo.
pub fn paginas<'a>(p: Paleta, lista: Vec<(String, Option<String>, bool, Msg)>) -> E<'a> {
    Column::with_children(lista.into_iter().map(|(nombre, cuenta, sel, msg)| {
        let mut r = row![txt(nombre).font(if sel { SEMI } else { INTER }).width(Length::Fill)].align_y(Alignment::Center);
        if let Some(c) = cuenta {
            r = r.push(tenue(&p, c));
        }
        button(r).width(Length::Fill).padding([6, 10]).style(tema::boton(p, Boton::Pagina(sel))).on_press(msg).into()
    }))
    .spacing(2)
    .into()
}

// ---------- Icono compuesto (placa, filo e insignia) ----------

pub fn icono<'a>(app: &'a App, inst: &Instancia, lado: f32) -> E<'a> {
    let p = &app.p;
    let aj = &app.aj;
    let caja = lado + 12.0;
    let mut capas = stack![Space::new(caja, caja)];
    if inst.motor == Motor::Rust && aj.hexagono {
        capas = capas.push(image(app.hexagono.clone()).width(caja).height(caja));
    }
    if let Some(h) = app.iconos.get(&inst.clave()) {
        capas = capas.push(
            container(image(h.clone()).width(lado).height(lado).filter_method(FilterMethod::Nearest)).center(caja),
        );
    }
    if aj.insignia {
        let ins = if inst.motor == Motor::Rust { &app.insignia_rust } else { &app.insignia_java };
        let t = (lado * 0.42).clamp(16.0, 24.0);
        capas = capas.push(
            container(image(ins.clone()).width(t).height(t))
                .align_right(caja)
                .align_bottom(caja),
        );
    }
    let color_filo = if inst.motor == Motor::Rust { p.rust } else { p.java };
    let filo = container(Space::new(3, lado))
        .style(tema::pildora(if aj.filo { color_filo } else { iced::Color::TRANSPARENT }, p.on_badge));
    row![filo, capas].spacing(3).align_y(Alignment::Center).into()
}

// ---------- Ventana principal ----------

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let principal = column![barra(app), linea(&p), row![principal::vista(app), lateral(app)].height(Length::Fill), linea(&p), estado(app)];
    let fondo = container(principal).width(Length::Fill).height(Length::Fill).style(tema::fondo(p.win));

    let encima: Option<E> = if let Some(clave) = &app.borrar {
        Some(confirmar_borrar(app, clave))
    } else if let Some(pn) = &app.pide_nombre {
        Some(pedir_nombre(app, pn))
    } else {
        app.dlg.map(|d| match d {
            Dlg::Nueva => nueva::vista(app),
            Dlg::Editar => editar::vista(app),
            Dlg::Ajustes => ajustes::vista(app),
            Dlg::Cuentas => cuentas::vista(app),
            Dlg::Armario => armario::vista(app),
            Dlg::Explorar => explorar::vista(app),
            Dlg::Icono => icono::vista(app),
            Dlg::Servidores => servidores::vista(app),
            Dlg::Archivo => archivo::vista(app),
        })
    };
    // Con un diálogo abierto no se dibuja la ventana de debajo: el dibujo por software no recorta bien el
    // texto de capas inferiores al repintar a trozos, y así además hay menos que pintar. Como en Prism,
    // cada diálogo es su propia «ventana».
    match encima {
        Some(d) => container(center(d))
            .padding(16)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(tema::fondo(tema::mezcla(p.win, iced::Color::BLACK, 0.6)))
            .into(),
        None if app.grande => grande::vista(app),
        None => match &app.menu {
            Some(m) => stack![
                fondo,
                // Pulsar fuera del menú lo cierra.
                mouse_area(Space::new(Length::Fill, Length::Fill)).on_press(Msg::CerrarMenu).on_right_press(Msg::CerrarMenu),
                menu::vista(app, m)
            ]
            .into(),
            None => fondo.into(),
        },
    }
}

/// Botón de la barra de arriba: icono y texto.
pub fn boton_barra<'a>(app: &'a App, icono_n: &'static str, etiqueta: &'a str, msg: Msg) -> E<'a> {
    let p = app.p;
    let mut r = row![].spacing(6).align_y(Alignment::Center);
    if let Some(h) = app.barra.get(icono_n) {
        r = r.push(image(h.clone()).width(16).height(16));
    }
    if !etiqueta.is_empty() {
        r = r.push(text(etiqueta).size(13));
    }
    button(r).padding([5, 9]).style(tema::boton(p, Boton::Barra)).on_press(msg).into()
}

fn barra(app: &App) -> E<'_> {
    let p = app.p;
    let b = |n: &'static str, e: &'static str, m: Msg| boton_barra(app, n, e, m);
    // El gato enciende o apaga la mascota de la esquina; resaltado cuando está.
    let gato = button(
        image(app.barra.get("gato").cloned().unwrap_or_else(|| app.carpeta_ico.clone())).width(16).height(16),
    )
    .padding([5, 8])
    .style(tema::boton(p, Boton::Opcion(app.aj.esquina)))
    .on_press(Msg::AFlag(crate::app::Flag::Esquina, !app.aj.esquina));
    // La marca: el pulso de Jiyu y el nombre.
    let marca = row![image(app.logo.clone()).width(32).height(32), text("Jiyu Launcher").size(14).font(SEMI)]
        .spacing(7)
        .align_y(Alignment::Center);
    let mut r = row![
        container(marca).padding([0, 8]),
        container(Space::new(1, 18)).style(tema::fondo(p.line)),
        Space::new(4, 0),
        b("anadir", "Añadir instancia", Msg::NuevaEn(String::new())),
        b("carpeta", "Carpetas", Msg::Carpeta(crate::ajustes::carpeta_datos())),
        b("ajustes", "Ajustes", Msg::Abrir(Dlg::Ajustes)),
        b("explorar", "Explorar mods", Msg::Abrir(Dlg::Explorar)),
        b("servidores", "Servidores", Msg::Abrir(Dlg::Servidores)),
        b("recargar", "Recargar", Msg::Recargar),
        horizontal_space(),
        b("tele", "Big Picture", Msg::BPAlternar),
        gato,
    ]
    .spacing(2)
    .align_y(Alignment::Center);
    if let Some(v) = crate::motor::nueva(&app.versiones) {
        let chip = row![
            image(app.insignia_rust.clone()).width(16).height(16),
            text(format!("Horizon {}", crate::datos::corta_version(&v.tag))).size(12),
            button(text("Instalar").size(12).font(SEMI))
                .padding([2, 8])
                .style(move |_: &iced::Theme, _| button::Style {
                    background: Some(iced::Background::Color(p.rust)),
                    text_color: p.on_badge,
                    border: iced::border::rounded(if p.jiyu { 999.0 } else { p.r_sm }),
                    ..Default::default()
                })
                .on_press(Msg::ActualizarHorizon(v.tag.clone()))
        ]
        .spacing(8)
        .align_y(Alignment::Center);
        r = r.push(container(chip).padding([3, 4]).style(tema::aviso(p, p.rust)));
    }
    // Tu cuenta, con la cabeza de tu skin al lado del nombre.
    let mut cuenta = row![].spacing(8).align_y(Alignment::Center);
    if let Some(h) = &app.cara {
        cuenta = cuenta.push(image(h.clone()).width(20).height(20).filter_method(FilterMethod::Nearest));
    }
    cuenta = cuenta.push(text(app.cuenta_activa().map(|c| c.nombre.clone()).unwrap_or_else(|| "Iniciar sesión".into())).size(13));
    r = r
        .push(b("armario", "Armario", Msg::Abrir(Dlg::Armario)))
        .push(button(cuenta).padding([4, 10]).style(tema::boton(p, Boton::Barra)).on_press(Msg::Abrir(Dlg::Cuentas)));
    container(r).padding([5, 8]).width(Length::Fill).style(tema::fondo(p.chrome)).into()
}

pub fn dato<'a>(p: &Paleta, k: &'a str, v: String) -> E<'a> {
    row![tenue(p, k).width(84), txt(v)].spacing(6).into()
}

fn lateral(app: &App) -> E<'_> {
    let p = app.p;
    let sel = app.sel.as_ref().and_then(|s| app.instancia(s));
    let mut col = Column::new().spacing(10).padding(14).width(Length::Fill);
    match sel {
        None => col = col.push(tenue(&p, "Elige una instancia para ver qué puedes hacer con ella.")),
        Some(inst) => {
            let clave = inst.clave();
            let motor_txt = match (inst.motor, inst.origen) {
                (Motor::Rust, _) => "RUST · Horizon".to_string(),
                (Motor::Java, Origen::Horizon) => "JAVA · vía Horizon".to_string(),
                (Motor::Java, Origen::Prism) => "JAVA · Prism".to_string(),
            };
            col = col
                .push(
                    container(
                        // Pulsar el icono abre el selector para cambiarlo.
                        button(icono(app, inst, 64.0))
                            .padding(4)
                            .style(tema::boton(p, Boton::Casilla(false)))
                            .on_press(Msg::AbrirIcono(clave.clone())),
                    )
                    .center_x(Length::Fill),
                )
                .push(container(tenue(&p, "Pulsa el icono para cambiarlo")).center_x(Length::Fill))
                .push(text(inst.nombre.clone()).size(15).font(SEMI).width(Length::Fill).center())
                .push(container(pildora(&p, inst.motor, motor_txt)).center_x(Length::Fill));
            let mut datos = Column::new().spacing(4);
            match inst.motor {
                Motor::Rust => {
                    let v = if inst.sigue_ultima {
                        "la última".into()
                    } else {
                        inst.version.as_deref().map(crate::datos::corta_version).unwrap_or_default()
                    };
                    datos = datos.push(dato(&p, "Horizon", v));
                }
                Motor::Java => {
                    datos = datos.push(dato(&p, "Minecraft", inst.version.clone().unwrap_or_else(|| "?".into())));
                    if let Some((c, v)) = &inst.cargador {
                        datos = datos.push(dato(&p, "Cargador", format!("{c} {v}")));
                    }
                }
            }
            datos = datos.push(dato(&p, "Jugado", inst.tiempo_jugado()));
            if let Some(ms) = inst.ultima_ms {
                datos = datos.push(dato(&p, "Última vez", crate::datos::hace(ms)));
            }
            col = col.push(datos);
            let ancho = |b: button::Button<'static, Msg>| b.width(Length::Fill);
            let mut botones = column![
                match app.partidas.get(&clave) {
                    // Mientras Horizon prepara o el juego está abierto, no se puede volver a pulsar.
                    Some(e) => ancho(btn(p, if e.en_juego { "En juego" } else { "Arrancando…" }, Boton::Primario, None).padding([8, 12])),
                    None => ancho(btn(p, "Jugar", Boton::Primario, Some(Msg::Jugar(clave.clone()))).padding([8, 12])),
                },
                ancho(btn(p, "Editar", Boton::Normal, Some(Msg::AbrirEditar(clave.clone(), PagE::Version)))),
                ancho(btn(p, "Mods", Boton::Normal, Some(Msg::AbrirEditar(clave.clone(), PagE::Mods)))),
                ancho(btn(p, "Mundos", Boton::Normal, Some(Msg::AbrirEditar(clave.clone(), PagE::Mundos)))),
                ancho(btn(p, "Registro", Boton::Normal, Some(Msg::AbrirEditar(clave.clone(), PagE::Registro)))),
                ancho(btn(p, "Abrir carpeta", Boton::Normal, Some(Msg::Carpeta(inst.carpeta.clone())))),
            ]
            .spacing(4);
            match inst.origen {
                Origen::Prism => {
                    botones = botones.push(ancho(btn(p, "Copiar a Horizon", Boton::Normal, Some(Msg::CopiarAHorizon(clave.clone())))));
                }
                Origen::Horizon if inst.id != "horizon" => {
                    botones = botones.push(ancho(btn(p, "Eliminar", Boton::Peligro, Some(Msg::PedirBorrar(clave.clone())))));
                }
                _ => {}
            }
            if let Some(e) = app.partidas.get(&clave) {
                col = col.push(aviso(p, p.hl, text(e.linea.clone()).size(12)));
            }
            col = col.push(botones);
            if inst.origen == Origen::Prism {
                col = col.push(tenue(&p, "Se juega con Prism, con sus cuentas, su Java y sus ajustes."));
            }
        }
    }
    let mut lado = column![desplaza(p, col).height(Length::Fill)];
    // La mascota de Jiyu ocupa la esquina, como el gato en Prism. Una por pantalla.
    if let Some(m) = app.mascota() {
        lado = lado.push(container(image(m).width(200).height(200).content_fit(iced::ContentFit::Contain)).center_x(Length::Fill));
    }
    row![
        container(Space::new(1, Length::Fill)).width(1).style(tema::fondo(p.line)),
        container(lado).width(250).height(Length::Fill).style(tema::fondo(p.panel))
    ]
    .into()
}

fn estado(app: &App) -> E<'_> {
    let p = app.p;
    let n = app.visibles().count();
    let total: u64 = app.todas().map(|i| i.jugado_s).sum();
    let izq: E = match &app.aviso {
        Some((t, true)) => text(t.clone()).size(12).color(p.mal).into(),
        Some((t, false)) => text(t.clone()).size(12).into(),
        None if !app.partidas.is_empty() => {
            let e = app.partidas.values().next().unwrap();
            text(e.linea.clone()).size(12).color(p.hl).into()
        }
        None if app.cargando => tenue(&p, "Cargando instancias…").into(),
        None => tenue(&p, "Listo").into(),
    };
    // El archivador, abajo a la derecha: ahí van las instancias archivadas.
    let n_arch = app.n_archivo();
    let mut archivo = row![].spacing(6).align_y(Alignment::Center);
    if let Some(h) = app.barra.get("archivo") {
        archivo = archivo.push(image(h.clone()).width(16).height(16));
    }
    archivo = archivo.push(text(if n_arch == 0 { "Archivadas".to_string() } else { format!("Archivadas ({n_arch})") }).size(12));
    let r = row![
        izq,
        horizontal_space(),
        tenue(&p, format!("{n} instancias · {} jugado", crate::datos::duracion(total))),
        tenue(&p, "·"),
        tenue(&p, format!("Launcher: {} MB", app.memoria_mb)),
        button(archivo).padding([2, 8]).style(tema::boton(p, Boton::Barra)).on_press(Msg::Abrir(Dlg::Archivo)),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    container(r).padding([4, 10]).width(Length::Fill).style(tema::fondo(p.chrome)).into()
}

fn confirmar_borrar<'a>(app: &'a App, clave: &str) -> E<'a> {
    let p = app.p;
    let inst = app.instancia(clave);
    let nombre = inst.map(|i| i.nombre.clone()).unwrap_or_default();
    let detalle = match inst.map(|i| i.origen) {
        Some(Origen::Prism) => "Es de Prism: su carpeta entera (mundos, mods, opciones) va a la papelera y Prism dejará de verla.",
        _ => "Su carpeta (mundos, mods, opciones, paquetes) va a la papelera. Desde allí puedes recuperarla.",
    };
    let listo = app.borrar_texto.trim() == "BORRAR";
    container(
        column![
            text(format!("¿Eliminar «{nombre}»?")).size(15).font(SEMI),
            tenue(&p, detalle),
            tenue(&p, "Para confirmarlo, escribe BORRAR:"),
            text_input("BORRAR", &app.borrar_texto)
                .on_input(Msg::BorrarTexto)
                .on_submit(Msg::Borrar(clave.to_string()))
                .padding([6, 8])
                .size(14)
                .style(tema::campo(p)),
            row![
                horizontal_space(),
                btn(p, "Cancelar", Boton::Normal, Some(Msg::CancelarBorrar)),
                btn(p, "Eliminar", Boton::Peligro, listo.then(|| Msg::Borrar(clave.to_string()))),
            ]
            .spacing(8)
        ]
        .spacing(12),
    )
    .padding(18)
    .width(460)
    .style(tema::dialogo(p))
    .into()
}

fn pedir_nombre<'a>(app: &'a App, pn: &'a crate::app::PideNombre) -> E<'a> {
    let p = app.p;
    let titulo_txt = match &pn.para {
        crate::app::ParaNombre::NuevaCarpeta(padre) if padre.is_empty() => "Carpeta nueva".to_string(),
        crate::app::ParaNombre::NuevaCarpeta(padre) => format!("Carpeta nueva dentro de «{}»", padre.replace('/', " › ")),
        crate::app::ParaNombre::RenombrarCarpeta(c) => format!("Renombrar «{}»", c.replace('/', " › ")),
    };
    container(
        column![
            text(titulo_txt).size(15).font(SEMI),
            text_input("Nombre", &pn.valor).on_input(Msg::NombreTexto).on_submit(Msg::NombreAceptar).padding([6, 8]).size(14).style(tema::campo(p)),
            row![
                horizontal_space(),
                btn(p, "Cancelar", Boton::Normal, Some(Msg::Cerrar)),
                btn(p, "Aceptar", Boton::Primario, (!pn.valor.trim().is_empty()).then_some(Msg::NombreAceptar)),
            ]
            .spacing(8)
        ]
        .spacing(12),
    )
    .padding(18)
    .width(440)
    .style(tema::dialogo(p))
    .into()
}

/// Logo de un mod (de Modrinth) o, si no lo hay, la inicial sobre un color sacado de su nombre.
pub fn logo<'a>(app: &'a App, id: Option<&str>, nombre: &str, lado: f32) -> E<'a> {
    if let Some(h) = id.and_then(|i| app.logos.get(i)) {
        return container(image(h.clone()).width(lado).height(lado)).style(tema::caja(app.p)).padding(1).into();
    }
    let mut h: u32 = 2166136261;
    for b in nombre.bytes() {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    let color = iced::Color::from_rgb8(70 + (h % 120) as u8, 70 + ((h >> 8) % 120) as u8, 70 + ((h >> 16) % 120) as u8);
    let letra: String = nombre.chars().find(|c| c.is_alphanumeric()).map(|c| c.to_uppercase().to_string()).unwrap_or_default();
    container(text(letra).size(lado * 0.45).font(SEMI).color(iced::Color::WHITE))
        .center(lado + 2.0)
        .style(tema::pildora_cuadrada(color, app.p.r_sm))
        .into()
}
