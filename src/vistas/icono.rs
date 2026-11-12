//! Selector de icono: bloques e ítems de tus archivos preparados, colores, tus iconos de Prism o el de serie.

use super::*;
use iced::widget::image::{FilterMethod, Handle};
use iced::widget::{column, row};

const COLORES: [u32; 10] = [0x3fa7d6, 0x59cd90, 0xfac05e, 0xf79d84, 0xee6352, 0xa77ba7, 0x7676b0, 0xf4511e, 0x5b6b7a, 0xe4ddff];

pub fn vista(app: &App) -> E<'_> {
    let p = app.p;
    let sel = &app.selector;
    // Para la instancia que se está creando, o para una que ya existe.
    let inst = app.instancia(&sel.clave);
    if inst.is_none() && !sel.para_nueva {
        return marco(p, "Icono".into(), None, tenue(&p, "Esta instancia ya no existe.").into(), None);
    }
    let nombre = inst.map(|i| i.nombre.clone()).unwrap_or_else(|| {
        if app.nueva.nombre.is_empty() { "la instancia nueva".to_string() } else { app.nueva.nombre.clone() }
    });
    let ocupado = sel.ocupado;
    let opcion = |h: Handle, spec: String| -> E<'_> {
        // Las texturas se escalan sin suavizar (píxeles nítidos); las fotos de Prism, suavizadas.
        let filtro = if spec.starts_with("prism:") { FilterMethod::Linear } else { FilterMethod::Nearest };
        button(image(h).width(40).height(40).filter_method(filtro))
            .padding(6)
            .style(tema::boton(p, Boton::Casilla(false)))
            .on_press_maybe((!ocupado).then(|| Msg::IElegir(spec)))
            .into()
    };

    let colores = row(COLORES.iter().map(|c| {
        button(Space::new(28, 28))
            .padding(0)
            .style(tema::boton_color(tema::hex(*c), p.line))
            .on_press_maybe((!ocupado).then(|| Msg::IElegir(format!("#{c:06x}"))))
            .into()
    }))
    .spacing(6);

    let bloques: E = if sel.previas.is_empty() {
        tenue(
            &p,
            if sel.texturas.is_empty() {
                "No encuentro texturas: prepara una versión de Horizon desde tu Minecraft."
            } else {
                "Ninguna textura se llama así."
            },
        )
        .into()
    } else {
        row(sel.previas.iter().map(|(spec, h)| opcion(h.clone(), spec.clone()))).spacing(4).wrap().into()
    };

    let mut cuerpo = column![
        match inst {
            Some(i) => Element::from(row![icono(app, i, 48.0), column![txt(nombre.clone()).font(SEMI), tenue(&p, donde(i))].spacing(2)]
                .spacing(12)
                .align_y(Alignment::Center)),
            None => column![txt(nombre.clone()).font(SEMI), tenue(&p, "Se le pondrá al crearla.")].spacing(2).into(),
        },
        text_input("Buscar bloque o ítem (diamond, tnt, cherry…)", &sel.filtro)
            .on_input(Msg::IFiltro)
            .padding([6, 8])
            .size(13)
            .style(tema::campo(p)),
        seccion(&p, if sel.filtro.trim().is_empty() { "Bloques e ítems sugeridos" } else { "Bloques e ítems" }),
        bloques,
        seccion(&p, "Colores"),
        colores,
    ]
    .spacing(12);
    if !sel.propios.is_empty() {
        cuerpo = cuerpo
            .push(seccion(&p, "Iconos de Prism (los tuyos y los de serie)"))
            .push(row(sel.propios.iter().map(|(spec, h)| opcion(h.clone(), spec.clone()))).spacing(4).wrap());
    }
    let pie = row![
        btn(p, "Volver al de serie", Boton::Normal, (!ocupado).then(|| Msg::IElegir("none".into()))),
        horizontal_space(),
        btn(p, "Cerrar", Boton::Normal, Some(Msg::Cerrar)),
    ]
    .spacing(8);
    marco(p, format!("Icono · {nombre}"), None, cuerpo.into(), Some(pie.into()))
}

fn donde(inst: &Instancia) -> &'static str {
    match inst.origen {
        Origen::Horizon => "Se guarda en Horizon: lo verás igual en su launcher.",
        Origen::Prism => "Se guarda en este launcher; la carpeta de Prism no se toca.",
    }
}
