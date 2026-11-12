//! Iconos de las instancias, la insignia de motor y la placa hexagonal.
//! Todo se genera una vez en memoria (RGBA) y se guarda; no hay ficheros de iconos propios.
//!
//! - Horizon: la textura de bloque de tus archivos preparados (como hace Horizon), o un bloque de ruido.
//! - Prism: tus iconos personalizados se leen de su carpeta; los de serie se dibujan aquí (originales, no
//!   copiados de Prism), o se toman de las texturas de tu Minecraft si eliges «Bloques de tu Minecraft».

use crate::datos::Icono;
use iced::widget::image::Handle;
use iced::Color;
use std::collections::HashMap;
use std::path::PathBuf;

pub const LADO: u32 = 16;

#[derive(Default)]
pub struct Cache {
    mapa: HashMap<String, Handle>,
    texturas: Option<Vec<PathBuf>>,
}

impl Cache {
    /// Olvida lo generado (cuando llegan los iconos de Prism o cambian las texturas).
    pub fn limpia(&mut self) {
        self.mapa.clear();
    }

    pub fn icono(&mut self, icono: &Icono, semilla: &str, texturas_mc: bool) -> Handle {
        let clave = format!("{icono:?}|{semilla}|{texturas_mc}");
        if let Some(h) = self.mapa.get(&clave) {
            return h.clone();
        }
        let h = self.genera(icono, semilla, texturas_mc);
        self.mapa.insert(clave, h.clone());
        h
    }

    fn genera(&mut self, icono: &Icono, semilla: &str, texturas_mc: bool) -> Handle {
        match icono {
            Icono::Textura(nombre) => self
                .textura(nombre)
                .unwrap_or_else(|| bloque(Bloque::por_nombre(nombre), semilla)),
            Icono::Color(c) => color_plano(*c),
            Icono::Prism(clave) => {
                // 1) Tus iconos personalizados de Prism; 2) los de serie de Prism (bajados a la caché);
                // 3) texturas de tu Minecraft si lo has elegido; 4) los dibujos propios.
                if let Some(h) = crate::prism::archivo_icono(clave).and_then(|f| crate::miniatura::reduce(&f, 128, 128, true)) {
                    return h;
                }
                if !texturas_mc {
                    if let Some(h) = crate::prism_iconos::archivo(clave).and_then(|f| crate::miniatura::reduce(&f, 128, 128, false)) {
                        return h;
                    }
                }
                if texturas_mc {
                    if let Some(h) = equivalente_mc(clave).and_then(|t| self.textura(t)) {
                        return h;
                    }
                }
                dibujo_prism(clave, semilla)
            }
            // El icono por defecto es el bloque de tierra de Minecraft (de tus archivos preparados, si los hay).
            Icono::Defecto => self.textura("dirt").unwrap_or_else(|| bloque(Bloque::Tierra, "default")),
        }
    }

    /// Nombres de todas las texturas de bloque e ítem (sin repetir), para el selector de icono.
    pub fn nombres_texturas(&mut self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for dir in self.carpetas().to_vec() {
            for sub in ["block", "item"] {
                for e in std::fs::read_dir(dir.join(sub)).into_iter().flatten().flatten() {
                    let n = e.file_name().to_string_lossy().to_string();
                    if let Some(base) = n.strip_suffix(".png") {
                        v.push(base.to_string());
                    }
                }
            }
        }
        v.sort();
        v.dedup();
        v
    }

    /// Carpetas `textures` de las versiones de Horizon preparadas.
    fn carpetas(&mut self) -> &[PathBuf] {
        self.texturas.get_or_insert_with(|| {
            let mut v = Vec::new();
            let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return v };
            let versiones = home.join(".local/share/horizon-launcher/versions");
            for ver in std::fs::read_dir(versiones).into_iter().flatten().flatten() {
                let cliente = ver.path().join(".local/horizon-client");
                for mc in std::fs::read_dir(cliente).into_iter().flatten().flatten() {
                    let t = mc.path().join("assets/minecraft/textures");
                    if t.is_dir() {
                        v.push(t);
                    }
                }
            }
            v
        })
    }

    pub fn textura(&mut self, nombre: &str) -> Option<Handle> {
        let nombre = nombre.trim_end_matches(".png").to_string();
        for dir in self.carpetas() {
            for sub in ["block", "item"] {
                let f = dir.join(sub).join(format!("{nombre}.png"));
                if f.is_file() {
                    // Las texturas animadas son tiras verticales: nos quedamos con el primer cuadro.
                    return cuadro_cuadrado(&f).or_else(|| Some(Handle::from_path(f)));
                }
            }
        }
        None
    }
}

fn cuadro_cuadrado(f: &std::path::Path) -> Option<Handle> {
    let img = image::open(f).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if h <= w {
        return None;
    }
    let recorte = image::imageops::crop_imm(&img, 0, 0, w, w).to_image();
    Some(Handle::from_rgba(w, w, recorte.into_raw()))
}

/// Qué textura de Minecraft se parece a cada icono de serie de Prism.
fn equivalente_mc(clave: &str) -> Option<&'static str> {
    Some(match clave.trim_end_matches("_legacy") {
        "flame" => "fire_charge",
        "enderpearl" => "ender_pearl",
        "diamond" => "diamond",
        "dirt" => "dirt",
        "meat" => "cooked_beef",
        "forge" => "anvil",
        "fabricmc" => "string",
        "creeper" => "tnt_side",
        "herobrine" => "obsidian",
        "grass" => "grass_block_side",
        "stone" => "stone",
        "gold" => "gold_block",
        "iron" => "iron_block",
        "planks" => "oak_planks",
        "brick" => "bricks",
        "chicken" => "cooked_chicken",
        "skeleton" => "bone",
        "squarecreeper" => "tnt_side",
        "steve" => "crafting_table_front",
        _ => return None,
    })
}

// ---------- Generador ----------

struct Lienzo {
    w: u32,
    h: u32,
    px: Vec<u8>,
}

impl Lienzo {
    fn nuevo(w: u32, h: u32) -> Self {
        Self { w, h, px: vec![0; (w * h * 4) as usize] }
    }
    fn pon(&mut self, x: u32, y: u32, c: u32, a: u8) {
        if x >= self.w || y >= self.h {
            return;
        }
        let i = ((y * self.w + x) * 4) as usize;
        self.px[i] = (c >> 16) as u8;
        self.px[i + 1] = (c >> 8) as u8;
        self.px[i + 2] = c as u8;
        self.px[i + 3] = a;
    }
    fn handle(self) -> Handle {
        Handle::from_rgba(self.w, self.h, self.px)
    }
}

/// El mismo generador que la maqueta (mulberry32 sobre FNV-1a), para que los bloques salgan iguales.
struct Azar(u32);
impl Azar {
    fn de(semilla: &str) -> Self {
        let mut h: u32 = 2166136261;
        for c in semilla.encode_utf16() {
            h = (h ^ c as u32).wrapping_mul(16777619);
        }
        Self(h)
    }
    fn sig(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x6D2B79F5);
        let mut t = self.0;
        t = (t ^ (t >> 15)).wrapping_mul(t | 1);
        t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 61));
        ((t ^ (t >> 14)) as f64 / 4294967296.0) as f32
    }
    fn elige(&mut self, de: &[u32]) -> u32 {
        de[((self.sig() * de.len() as f32) as usize).min(de.len() - 1)]
    }
}

#[derive(Clone, Copy)]
pub enum Bloque {
    Hierba,
    Tierra,
    Piedra,
    Cobre,
    Tablones,
    Purpur,
}

impl Bloque {
    fn por_nombre(n: &str) -> Self {
        if n.contains("copper") {
            Bloque::Cobre
        } else if n.contains("plank") || n.contains("crafting") {
            Bloque::Tablones
        } else if n.contains("purpur") {
            Bloque::Purpur
        } else if n.contains("dirt") {
            Bloque::Tierra
        } else if n.contains("stone") {
            Bloque::Piedra
        } else {
            Bloque::Hierba
        }
    }
    fn clave(self) -> &'static str {
        match self {
            Bloque::Hierba => "grass",
            Bloque::Tierra => "dirt",
            Bloque::Piedra => "stone",
            Bloque::Cobre => "copper",
            Bloque::Tablones => "planks",
            Bloque::Purpur => "purpur",
        }
    }
}

pub fn bloque(tipo: Bloque, semilla: &str) -> Handle {
    let mut r = Azar::de(&format!("{semilla}{}", tipo.clave()));
    let mut l = Lienzo::nuevo(LADO, LADO);
    let largo = semilla.encode_utf16().count() as u32;
    for y in 0..16u32 {
        for x in 0..16u32 {
            let c = match tipo {
                Bloque::Hierba => {
                    let mut c = r.elige(&[0x866043, 0x6c4a32, 0x9a7150, 0x79553a]);
                    let e = 3 + if r.sig() < 0.5 { 1 } else { 0 } + if x % 3 == 0 { 1 } else { 0 };
                    if y < e {
                        c = r.elige(&[0x5d9b3a, 0x6fb043, 0x4f8a2f]);
                    }
                    c
                }
                Bloque::Tierra => r.elige(&[0x866043, 0x6c4a32, 0x9a7150, 0x79553a, 0x5e412c]),
                Bloque::Piedra => r.elige(&[0x7d7d7d, 0x8f8f8f, 0x6b6b6b, 0x858585]),
                Bloque::Cobre => {
                    let mut c = r.elige(&[0xc06a3e, 0xe07b4b, 0xa45734, 0xd57245]);
                    if (x * 7 + y * 13 + largo) % 23 < 3 && x > 1 && x < 14 && y > 1 && y < 14 {
                        c = r.elige(&[0x4fae8e, 0x62c4a0]);
                    }
                    c
                }
                Bloque::Tablones => {
                    let mut c = r.elige(&[0xa8834f, 0x9a774a, 0xb8925c]);
                    let veta = if y % 8 < 4 { x == 5 } else { x == 11 };
                    if y % 4 == 3 || (veta && y % 4 != 3) {
                        c = 0x6e5230;
                    }
                    c
                }
                Bloque::Purpur => {
                    let mut c = r.elige(&[0xa77ba7, 0xb88cb8, 0x946894]);
                    if x == 0 || y == 0 || x == 15 || y == 15 {
                        c = 0x7a547a;
                    }
                    c
                }
            };
            l.pon(x, y, c, 255);
        }
    }
    l.handle()
}

fn color_plano(c: u32) -> Handle {
    let mut l = Lienzo::nuevo(LADO, LADO);
    for y in 0..16 {
        for x in 0..16 {
            let borde = x == 0 || y == 0 || x == 15 || y == 15;
            let osc = if borde { oscurece(c, 0.75) } else { c };
            l.pon(x, y, osc, 255);
        }
    }
    l.handle()
}

fn oscurece(c: u32, f: f32) -> u32 {
    let r = (((c >> 16) & 255) as f32 * f) as u32;
    let g = (((c >> 8) & 255) as f32 * f) as u32;
    let b = ((c & 255) as f32 * f) as u32;
    (r << 16) | (g << 8) | b
}

/// Dibujos originales de 16×16 para las claves de Prism (llama, perla, gema, carne, yunque, carrete, masa, capucha).
fn dibujo_prism(clave: &str, semilla: &str) -> Handle {
    let nombre = match clave.trim_end_matches("_legacy") {
        "flame" => "llama",
        "enderpearl" => "perla",
        "diamond" => "gema",
        "meat" => "carne",
        "forge" => "yunque",
        "fabricmc" => "carrete",
        "creeper" | "squarecreeper" => "masa",
        "herobrine" | "steve" => "capucha",
        "dirt" => return bloque(Bloque::Tierra, semilla),
        "stone" => return bloque(Bloque::Piedra, semilla),
        "planks" => return bloque(Bloque::Tablones, semilla),
        "grass" => return bloque(Bloque::Hierba, semilla),
        _ => return bloque(Bloque::Tierra, "default"),
    };
    let (paleta, mapa) = SPRITES.iter().find(|(n, _, _)| *n == nombre).map(|(_, p, m)| (*p, *m)).unwrap();
    let mut l = Lienzo::nuevo(LADO, LADO);
    for (y, fila) in mapa.iter().enumerate() {
        for (x, ch) in fila.chars().enumerate() {
            if let Some((_, c)) = paleta.iter().find(|(k, _)| *k == ch) {
                l.pon(x as u32, y as u32, *c, 255);
            }
        }
    }
    l.handle()
}

type Sprite = (&'static str, &'static [(char, u32)], &'static [&'static str; 16]);

const SPRITES: &[Sprite] = &[
    ("llama", &[('o', 0xff8c1a), ('R', 0xe04a1f), ('y', 0xffd23f), ('W', 0xfff3b0)], &[
        "................", ".......o........", "......oo........", "......oRo.......", ".....oRRo...o...", ".....RRRoo.oo...",
        "....oRRRRo.oR...", "...oRRyRRRoRR...", "...RRyyyRRRRRo..", "..oRyyWyyRRRRo..", "..RRyWWWyyRyRR..", "..RRyWWWWyyyRR..",
        "..oRyyWWWyyyRo..", "...RRyyyyyyRR...", "....RRRRRRRR....", "......RRRR......",
    ]),
    ("perla", &[('d', 0x0b3d38), ('t', 0x2f9e8f), ('T', 0x6fd3c4), ('W', 0xe6fffb), ('G', 0x124d45)], &[
        "................", "................", ".....dddddd.....", "....dttttttd....", "...dtTTttttTd...", "...dtTWtttttd...",
        "..dttTtttttttd..", "..dtttttGGtttd..", "..dttttGGGGttd..", "..dttttGGGGttd..", "..dtttttGGtttd..", "...dttttttttd...",
        "...dttttttttd...", "....dttttttd....", ".....dddddd.....", "................",
    ]),
    ("gema", &[('b', 0x1a8a91), ('L', 0x5ee6f0), ('W', 0xe9feff)], &[
        "................", "................", "....bbbbbbbb....", "...bLLLbbLLLb...", "..bLWWLbbLLLLb..", ".bLLLLLbbLLLLLb.",
        ".bbbbbbbbbbbbbb.", "..bLLLLbbLLLLb..", "...bLLLbbLLLb...", "....bLLbbLLb....", ".....bLbbLb.....", "......bbbb......",
        ".......bb.......", "................", "................", "................",
    ]),
    ("carne", &[('m', 0x7a2e1d), ('M', 0xc9563a), ('h', 0xe07a55), ('b', 0xa39c8c), ('W', 0xf2efe6)], &[
        "................", ".........mmmm...", "........mMhMMm..", ".......mMhhMMMm.", "......mMMhMMMMm.", "......mMMMMMMMm.",
        "......mMMMMMMm..", ".....mmMMMMMm...", "....bb.mmmmm....", "...bWb..........", "..bWWb..........", ".bWWb...........",
        ".bWb............", "..b.............", "................", "................",
    ]),
    ("yunque", &[('a', 0x2b2e33), ('A', 0x6b7179), ('H', 0x9aa1a9)], &[
        "................", "................", "................", "..aaaaaaaaaaaa..", ".aHHHHHHHHHHHAa.", ".aAAAAAAAAAAAAa.",
        "..aaaaAAAAAAaa..", "......aAAAAa....", "......aAAAa.....", ".....aAAAAAa....", "....aAAAAAAAa...", "...aAAAAAAAAAa..",
        "...aaaaaaaaaaa..", "................", "................", "................",
    ]),
    ("carrete", &[('w', 0x5b4a33), ('W', 0x8a7350), ('c', 0xc9b28a), ('C', 0xe6d3a8)], &[
        "................", "................", "...wwwwwwwwww...", "...wWWWWWWWWw...", "....cCcCcCcC....", "....CcCcCcCc....",
        "....cCcCcCcC....", "....CcCcCcCc....", "....cCcCcCcC....", "....CcCcCcCc....", "....cCcCcCcC....", "...wWWWWWWWWw...",
        "...wwwwwwwwww...", "................", "................", "................",
    ]),
    ("masa", &[('p', 0x3d2a73), ('P', 0x7c5cc4), ('L', 0xa68be6), ('k', 0x140d26), ('W', 0xefeaff)], &[
        "................", "................", ".....pppppp.....", "....pPLLPPPp....", "...pPLPPPPPPp...", "...pPPPPPPPPp...",
        "...pPWkPPWkPp...", "...pPkkPPkkPp...", "...pPPPPPPPPp...", "...pPPPkkPPPp...", "...pPPPPPPPPp...", "...pPPPPPPPPp...",
        "...pPpPPpPPpp...", "...pp.pp.pp.p...", "................", "................",
    ]),
    ("capucha", &[('h', 0x1b1d26), ('H', 0x363a4c), ('s', 0x2a2d3b), ('k', 0x7fd1ff)], &[
        "................", ".....hhhhhh.....", "....hHHHHHHh....", "...hHHHHHHHHh...", "...hHssssssHh...", "...hHssssssHh...",
        "...hHskssksHh...", "...hHssssssHh...", "...hHHssssHHh...", "..hHHHHHHHHHHh..", "..hHHHHHHHHHHh..", ".hHHHHHHHHHHHHh.",
        ".hHHHHHHHHHHHHh.", ".hhhhhhhhhhhhhh.", "................", "................",
    ]),
];

// ---------- Insignia y placa ----------

#[derive(Clone, Copy, PartialEq)]
pub enum Glifo {
    Engranaje,
    Taza,
}


fn rgb(c: Color) -> u32 {
    let [r, g, b, _] = c.into_rgba8();
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

/// Insignia redonda con el engranaje (Rust) o la taza (Java), para la esquina del icono.
pub fn insignia(glifo: Glifo, fondo: Color, tinta: Color, aro: Color) -> Handle {
    let k = hexc(tinta);
    let glifo_svg = if glifo == Glifo::Engranaje {
        // Engranaje de 8 dientes, con su agujero.
        let mut d = String::new();
        for i in 0..16 {
            let r = if (i / 2) % 2 == 0 { 6.4 } else { 4.7 };
            let a = (i as f32 * 22.5 - 11.25).to_radians();
            d.push_str(&format!("{}{:.2},{:.2}", if i == 0 { "M" } else { "L" }, 12.0 + r * a.cos(), 12.0 + r * a.sin()));
        }
        d.push('z');
        format!(r##"<path d="{d}" fill="{k}" stroke="{k}" stroke-width="0.9" stroke-linejoin="round"/><circle cx="12" cy="12" r="2" fill="{}"/>"##, hexc(fondo))
    } else {
        format!(
            r##"<path d="M6.6 9.5h8.4v3.6a3.6 3.6 0 0 1-3.6 3.6h-1.2a3.6 3.6 0 0 1-3.6-3.6z" fill="{k}"/><path d="M15 10.6h1.1a1.7 1.7 0 0 1 0 3.4H15" {}/><path d="M6 18.6h10" {}/><path d="M9.3 4.8c-.8.9.8 1.6 0 2.6M12.2 4.8c-.8.9.8 1.6 0 2.6" {}/>"##,
            trazo(&k, 1.3),
            trazo(&k, 1.5),
            trazo(&k, 1.1)
        )
    };
    let cuerpo = format!(r##"<circle cx="12" cy="12" r="10.6" fill="{}" stroke="{}" stroke-width="1.6"/>{glifo_svg}"##, hexc(fondo), hexc(aro));
    svg(&cuerpo, "0 0 24 24", 48)
}

/// Placa hexagonal (la forma de las instancias Rust), con el color de Rust a poca opacidad.
pub fn hexagono(lado: u32, color: Color, opacidad: f32) -> Handle {
    let mut l = Lienzo::nuevo(lado, lado);
    let s = lado as f32;
    // Vértices como en la maqueta: 50% 0, 95% 25%, 95% 75%, 50% 100%, 5% 75%, 5% 25%.
    let v = [(0.5, 0.0), (0.95, 0.25), (0.95, 0.75), (0.5, 1.0), (0.05, 0.75), (0.05, 0.25)];
    for y in 0..lado {
        for x in 0..lado {
            let (px, py) = ((x as f32 + 0.5) / s, (y as f32 + 0.5) / s);
            let mut dentro = false;
            let mut j = v.len() - 1;
            for i in 0..v.len() {
                let (xi, yi) = v[i];
                let (xj, yj) = v[j];
                if (yi > py) != (yj > py) && px < (xj - xi) * (py - yi) / (yj - yi) + xi {
                    dentro = !dentro;
                }
                j = i;
            }
            if dentro {
                l.pon(x, y, rgb(color), (opacidad * 255.0) as u8);
            }
        }
    }
    l.handle()
}

// ---------- Iconos vectoriales (suaves) ----------

fn hexc(c: Color) -> String {
    format!("#{:06x}", rgb(c))
}

fn hexu(c: u32) -> String {
    format!("#{:06x}", c & 0xff_ffff)
}

/// Dibuja un SVG (viewBox 24×24 salvo que diga otro) a `lado`×`lado` píxeles. Se pinta al doble del
/// tamaño en pantalla, así el escalado lineal lo deja nítido.
fn svg(cuerpo: &str, caja: &str, lado: u32) -> Handle {
    let fuente = format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{caja}" width="{lado}" height="{lado}">{cuerpo}</svg>"##);
    let pinta = || -> Option<Handle> {
        let arbol = resvg::usvg::Tree::from_str(&fuente, &resvg::usvg::Options::default()).ok()?;
        let mut lienzo = resvg::tiny_skia::Pixmap::new(lado, lado)?;
        let t = arbol.size();
        let e = (lado as f32 / t.width()).min(lado as f32 / t.height());
        resvg::render(&arbol, resvg::tiny_skia::Transform::from_scale(e, e), &mut lienzo.as_mut());
        // tiny-skia guarda el color premultiplicado; iced lo quiere sin premultiplicar.
        let mut px = lienzo.take();
        for c in px.chunks_exact_mut(4) {
            let a = c[3] as u32;
            if a > 0 && a < 255 {
                for k in 0..3 {
                    c[k] = ((c[k] as u32 * 255 + a / 2) / a).min(255) as u8;
                }
            }
        }
        Some(Handle::from_rgba(lado, lado, px))
    };
    pinta().unwrap_or_else(|| Lienzo::nuevo(lado, lado).handle())
}

fn trazo(color: &str, ancho: f32) -> String {
    format!(r##"fill="none" stroke="{color}" stroke-width="{ancho}" stroke-linecap="round" stroke-linejoin="round""##)
}

/// El símbolo de Jiyu (el «pulso»): dos arcos y la línea de encendido.
pub const JIYU_SIMBOLO: &str = "M12,13a11,11 0,1 0,16 0M16,16.078a6,6 0,1 0,8 0M20,9v12";
pub const SHU: u32 = 0xF4511E;

/// El logo de Jiyu en shu, a `lado` píxeles.
pub fn logo_jiyu(lado: u32) -> Handle {
    svg(&format!(r##"<path d="{JIYU_SIMBOLO}" {}/>"##, trazo(&hexu(SHU), 2.5)), "6.5 6.5 27 27", lado)
}

/// Icono de la ventana: el de la app de Jiyu (símbolo shu sobre sumi, con las esquinas redondeadas).
pub fn icono_ventana() -> Option<iced::window::Icon> {
    let cuerpo = format!(
        r##"<rect x="0" y="0" width="40" height="40" rx="9" fill="#0E0E10"/><g transform="translate(20 20) scale(0.94) translate(-20 -20.3)"><path d="{JIYU_SIMBOLO}" {}/></g>"##,
        trazo(&hexu(SHU), 2.5)
    );
    let lado = 128u32;
    let fuente = format!(r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 40" width="{lado}" height="{lado}">{cuerpo}</svg>"##);
    let arbol = resvg::usvg::Tree::from_str(&fuente, &resvg::usvg::Options::default()).ok()?;
    let mut lienzo = resvg::tiny_skia::Pixmap::new(lado, lado)?;
    resvg::render(&arbol, resvg::tiny_skia::Transform::from_scale(lado as f32 / 40.0, lado as f32 / 40.0), &mut lienzo.as_mut());
    let mut px = lienzo.take();
    for c in px.chunks_exact_mut(4) {
        let a = c[3] as u32;
        if a > 0 && a < 255 {
            for k in 0..3 {
                c[k] = ((c[k] as u32 * 255 + a / 2) / a).min(255) as u8;
            }
        }
    }
    iced::window::icon::from_rgba(px, lado, lado).ok()
}

/// Chevron para abrir y cerrar grupos (se ve a 10 px).
pub fn flecha(abierta: bool, color: Color) -> Handle {
    let d = if abierta { "M6 9.5l6 6 6-6" } else { "M9.5 6l6 6-6 6" };
    svg(&format!(r##"<path d="{d}" {}/>"##, trazo(&hexc(color), 2.8)), "0 0 24 24", 20)
}

/// Carpeta rellena del color elegido, suave: trasera en degradado oscuro, una hoja asomando y la tapa
/// delantera con degradado, brillo arriba y sombra abajo. `lado` es el tamaño de la imagen en píxeles
/// (el doble de como se ve en pantalla).
pub fn carpeta(color: Color, lado: u32) -> Handle {
    let base = rgb(color);
    let cuerpo = format!(
        r##"<defs>
<linearGradient id="t" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{tras1}"/><stop offset="1" stop-color="{tras2}"/></linearGradient>
<linearGradient id="f" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="{fre1}"/><stop offset="1" stop-color="{fre2}"/></linearGradient>
</defs>
<path d="M2.6 6.4C2.6 5.2 3.5 4.3 4.7 4.3h4.1c.6 0 1.1.2 1.5.6l1.3 1.3c.3.3.7.4 1.1.4h6.6c1.2 0 2.1.9 2.1 2.1v9.6c0 1.2-.9 2.1-2.1 2.1H4.7c-1.2 0-2.1-.9-2.1-2.1z" fill="url(#t)"/>
<rect x="4.4" y="7.6" width="15.2" height="6" rx="1" fill="#ffffff" opacity="0.82"/>
<path d="M2.6 10.9c0-1.1.9-2 2-2h14.8c1.1 0 2 .9 2 2v7.4c0 1.3-1 2.3-2.3 2.3H4.9c-1.3 0-2.3-1-2.3-2.3z" fill="url(#f)"/>
<path d="M4.5 9.75h15" stroke="{brillo}" stroke-width="0.7" stroke-linecap="round" opacity="0.75"/>"##,
        tras1 = hexu(oscurece(base, 0.78)),
        tras2 = hexu(oscurece(base, 0.62)),
        fre1 = hexu(aclara(base, 0.16)),
        fre2 = hexu(oscurece(base, 0.9)),
        brillo = hexu(aclara(base, 0.55)),
    );
    svg(&cuerpo, "0 0 24 24", lado)
}

fn aclara(c: u32, f: f32) -> u32 {
    let m = |v: u32| (v as f32 + (255.0 - v as f32) * f) as u32;
    (m(c >> 16 & 255) << 16) | (m(c >> 8 & 255) << 8) | m(c & 255)
}

/// Puntos de una estrella de cinco puntas centrada en (12, 12.6).
fn puntos_estrella() -> String {
    (0..10)
        .map(|i| {
            let r = if i % 2 == 0 { 9.6 } else { 4.4 };
            let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 5.0;
            format!("{:.2},{:.2}", 12.0 + r * a.cos(), 12.8 + r * a.sin())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Estrella de favoritos: dorada con degradado (llena) o solo el contorno.
pub fn estrella(llena: bool, color: Color) -> Handle {
    let pts = puntos_estrella();
    let cuerpo = if llena {
        format!(
            r##"<defs><linearGradient id="o" x1="0" y1="0" x2="0" y2="1"><stop offset="0" stop-color="#FFE07A"/><stop offset="1" stop-color="#F5A524"/></linearGradient></defs>
<polygon points="{pts}" fill="url(#o)" stroke="#D98710" stroke-width="1.6" stroke-linejoin="round"/>"##
        )
    } else {
        format!(r##"<polygon points="{pts}" {}/>"##, trazo(&hexc(color), 1.7))
    };
    svg(&cuerpo, "0 0 24 24", 40)
}

/// Iconos de la barra de arriba y de la barra de estado: trazo redondeado en el color del texto y un
/// detalle en el de acento. Se ven a 16 px.
pub fn barra(nombre: &str, color: Color, acento: Color) -> Handle {
    let (c, a) = (hexc(color), hexc(acento));
    let t = trazo(&c, 1.9);
    let cuerpo = match nombre {
        "anadir" => format!(r##"<path d="M12 5v14M5 12h14" {}/>"##, trazo(&c, 2.2)),
        "carpeta" => format!(r##"<path d="M3.5 7.5a2 2 0 0 1 2-2h3.7l2 2.2h7.3a2 2 0 0 1 2 2V17a2 2 0 0 1-2 2h-13a2 2 0 0 1-2-2z" {t}/>"##),
        "ajustes" => format!(
            r##"<path d="M4 7h9M18 7h2M4 12h3M12 12h8M4 17h11M20 17h0" {t}/><circle cx="15.5" cy="7" r="2.2" {ta}/><circle cx="9.5" cy="12" r="2.2" {ta}/><circle cx="17.5" cy="17" r="2.2" {ta}/>"##,
            ta = trazo(&a, 1.9)
        ),
        "explorar" => format!(r##"<circle cx="10.5" cy="10.5" r="6.5" {t}/><path d="M15.5 15.5 20 20" {}/><path d="M7.8 9.2a3 3 0 0 1 2.4-2" {}/>"##, trazo(&c, 2.2), trazo(&a, 1.6)),
        "recargar" => format!(r##"<path d="M19.5 12A7.5 7.5 0 1 1 17.3 6.7" {t}/><path d="M18.2 3.2v3.9h-3.9" {t}/>"##),
        "servidores" => format!(
            r##"<rect x="3.5" y="4" width="17" height="6.5" rx="2" {t}/><rect x="3.5" y="13.5" width="17" height="6.5" rx="2" {t}/><circle cx="7.5" cy="7.25" r="1.2" fill="{a}"/><circle cx="7.5" cy="16.75" r="1.2" fill="{a}"/><path d="M12 7.25h4.5M12 16.75h4.5" {t}/>"##
        ),
        "armario" => format!(r##"<path d="M8.5 3.8 3.6 6.6l1.9 3.9 2.1-.9v10.6h8.8V9.6l2.1.9 1.9-3.9-4.9-2.8a3.6 3.6 0 0 1-7 0z" {t}/>"##),
        "gato" => format!(
            r##"<path d="M4.5 18.5V9.3L5.6 4.2l4 3h4.8l4-3 1.1 5.1v9.2a1.5 1.5 0 0 1-1.5 1.5h-12a1.5 1.5 0 0 1-1.5-1.5z" {t}/><circle cx="9.3" cy="13" r="1.25" fill="{a}"/><circle cx="14.7" cy="13" r="1.25" fill="{a}"/><path d="M11 16.2h2" {}/>"##,
            trazo(&c, 1.6)
        ),
        "tele" => format!(r##"<rect x="2.8" y="4.5" width="18.4" height="12" rx="2.2" {t}/><path d="M8.5 20h7M12 16.5V20" {t}/><path d="M10.4 8.4v4.2l3.6-2.1z" fill="{a}" stroke="{a}" stroke-width="1" stroke-linejoin="round"/>"##),
        "archivo" => format!(r##"<rect x="3" y="4" width="18" height="4.5" rx="1.3" {t}/><path d="M5 8.5V18a2 2 0 0 0 2 2h10a2 2 0 0 0 2-2V8.5" {t}/><path d="M10 12.5h4" {}/>"##, trazo(&a, 2.0)),
        _ => String::new(),
    };
    svg(&cuerpo, "0 0 24 24", 32)
}

/// Colores que se pueden poner a una carpeta (el primero es el del tema).
pub const COLORES_CARPETA: [u32; 9] = [0, 0xe0625a, 0xf2a03d, 0xe3c545, 0x6cc070, 0x3fb8b0, 0x4f8fe0, 0x9a6ee0, 0xe07ab8];
