//! El armario: tus skins guardadas (PNG de 64×64 o 64×32) y su vista de frente y de espalda.
//! Las skins viven en `~/.local/share/jiyu-launcher/skins/`. La skin de tu cuenta se baja de la API pública de Mojang.

use iced::widget::image::Handle;
use std::path::{Path, PathBuf};

pub fn carpeta() -> PathBuf {
    crate::ajustes::carpeta_datos().join("skins")
}

#[derive(Debug, Clone)]
pub struct Skin {
    pub nombre: String,
    pub ruta: PathBuf,
    pub frente: Handle,
    pub cara: Handle,
}

pub fn lista(delgado: bool) -> Vec<Skin> {
    let dir = carpeta();
    let _ = std::fs::create_dir_all(&dir);
    let mut v: Vec<Skin> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("png")))
        .filter_map(|p| carga(&p, delgado))
        .collect();
    v.sort_by(|a, b| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()));
    v
}

pub fn carga(ruta: &Path, delgado: bool) -> Option<Skin> {
    let img = image::open(ruta).ok()?.to_rgba8();
    let (w, h) = img.dimensions();
    if w != 64 || (h != 64 && h != 32) {
        return None;
    }
    let nombre = ruta.file_stem()?.to_string_lossy().replace(['_', '-'], " ");
    Some(Skin {
        nombre,
        ruta: ruta.to_path_buf(),
        frente: vista(&img, false, delgado),
        cara: cara(&img),
    })
}

type Img = image::RgbaImage;

/// Copia un rectángulo de la skin al lienzo; `espejo` lo da la vuelta (skins antiguas de 64×32).
fn pega(src: &Img, dst: &mut Img, sx: u32, sy: u32, w: u32, h: u32, dx: u32, dy: u32, espejo: bool, capa: bool) {
    for y in 0..h {
        for x in 0..w {
            let ox = if espejo { sx + w - 1 - x } else { sx + x };
            if ox >= src.width() || sy + y >= src.height() {
                continue;
            }
            let p = *src.get_pixel(ox, sy + y);
            if capa && p[3] < 10 {
                continue;
            }
            if !capa && p[3] == 0 {
                continue;
            }
            dst.put_pixel(dx + x, dy + y, image::Rgba([p[0], p[1], p[2], 255]));
        }
    }
}

/// La figura plana de 16×32: cabeza, cuerpo, brazos y piernas, con la segunda capa encima.
fn vista(s: &Img, espalda: bool, delgado: bool) -> Handle {
    let mut d = Img::new(16, 32);
    let nueva = s.height() == 64;
    let ab = if delgado { 3 } else { 4 };
    // (frente_x, espalda_x) de cada parte en la plantilla; y = fila.
    if !espalda {
        pega(s, &mut d, 8, 8, 8, 8, 4, 0, false, false);
        pega(s, &mut d, 20, 20, 8, 12, 4, 8, false, false);
        // Brazo derecho (a la izquierda de la imagen) y pierna derecha.
        pega(s, &mut d, 44, 20, ab, 12, 4 - ab, 8, false, false);
        pega(s, &mut d, 4, 20, 4, 12, 4, 20, false, false);
        if nueva {
            pega(s, &mut d, 36, 52, ab, 12, 12, 8, false, false);
            pega(s, &mut d, 20, 52, 4, 12, 8, 20, false, false);
            pega(s, &mut d, 20, 36, 8, 12, 4, 8, false, true);
            pega(s, &mut d, 44, 36, ab, 12, 4 - ab, 8, false, true);
            pega(s, &mut d, 52, 52, ab, 12, 12, 8, false, true);
            pega(s, &mut d, 4, 36, 4, 12, 4, 20, false, true);
            pega(s, &mut d, 4, 52, 4, 12, 8, 20, false, true);
        } else {
            pega(s, &mut d, 44, 20, ab, 12, 12, 8, true, false);
            pega(s, &mut d, 4, 20, 4, 12, 8, 20, true, false);
        }
        pega(s, &mut d, 40, 8, 8, 8, 4, 0, false, true);
    } else {
        pega(s, &mut d, 24, 8, 8, 8, 4, 0, false, false);
        pega(s, &mut d, 32, 20, 8, 12, 4, 8, false, false);
        let bx = 44 + ab + 4; // trasera del brazo derecho
        pega(s, &mut d, bx, 20, ab, 12, 12, 8, false, false);
        pega(s, &mut d, 12, 20, 4, 12, 8, 20, false, false);
        if nueva {
            let bxi = 36 + ab + 4;
            pega(s, &mut d, bxi, 52, ab, 12, 4 - ab, 8, false, false);
            pega(s, &mut d, 28, 52, 4, 12, 4, 20, false, false);
            pega(s, &mut d, 32, 36, 8, 12, 4, 8, false, true);
            pega(s, &mut d, bx, 36, ab, 12, 12, 8, false, true);
            pega(s, &mut d, 52 + ab + 4, 52, ab, 12, 4 - ab, 8, false, true);
            pega(s, &mut d, 12, 36, 4, 12, 8, 20, false, true);
            pega(s, &mut d, 12, 52, 4, 12, 4, 20, false, true);
        } else {
            pega(s, &mut d, bx, 20, ab, 12, 4 - ab, 8, true, false);
            pega(s, &mut d, 12, 20, 4, 12, 4, 20, true, false);
        }
        pega(s, &mut d, 56, 8, 8, 8, 4, 0, false, true);
    }
    Handle::from_rgba(16, 32, d.into_raw())
}

fn cara(s: &Img) -> Handle {
    let mut d = Img::new(8, 8);
    pega(s, &mut d, 8, 8, 8, 8, 0, 0, false, false);
    pega(s, &mut d, 40, 8, 8, 8, 0, 0, false, true);
    Handle::from_rgba(8, 8, d.into_raw())
}

/// Guarda en el armario la skin que lleva ahora tu cuenta (API pública de Mojang, con `curl`).
pub async fn bajar_de_cuenta(uuid: String, nombre: String) -> Result<PathBuf, String> {
    let perfil = curl(&format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}")).await?;
    let json: serde_json::Value = serde_json::from_slice(&perfil).map_err(|e| e.to_string())?;
    let valor = json["properties"]
        .as_array()
        .and_then(|a| a.iter().find(|p| p["name"] == "textures"))
        .and_then(|p| p["value"].as_str())
        .ok_or("Mojang no devolvió la skin de esta cuenta")?;
    let texturas: serde_json::Value =
        serde_json::from_slice(&base64(valor).ok_or("La respuesta de Mojang no se entiende")?).map_err(|e| e.to_string())?;
    let url = texturas["textures"]["SKIN"]["url"].as_str().ok_or("Esta cuenta lleva la skin por defecto")?;
    let png = curl(url).await?;
    let dir = carpeta();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let destino = dir.join(format!("{nombre} (cuenta).png"));
    std::fs::write(&destino, png).map_err(|e| e.to_string())?;
    Ok(destino)
}

async fn curl(url: &str) -> Result<Vec<u8>, String> {
    let out = tokio::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "15", url])
        .output()
        .await
        .map_err(|e| format!("No pude usar curl: {e}"))?;
    if !out.status.success() {
        return Err("No se pudo descargar (¿sin conexión?)".into());
    }
    Ok(out.stdout)
}

fn base64(s: &str) -> Option<Vec<u8>> {
    let mut salida = Vec::with_capacity(s.len() * 3 / 4);
    let mut acc = 0u32;
    let mut bits = 0;
    for c in s.bytes() {
        let v = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' | b'-' => 62,
            b'/' | b'_' => 63,
            b'=' | b'\n' | b'\r' => continue,
            _ => return None,
        } as u32;
        acc = (acc << 6) | v;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            salida.push((acc >> bits) as u8);
            acc &= (1 << bits) - 1;
        }
    }
    Some(salida)
}

// ---------- Tu cuenta: skins y capas de verdad ----------
//
// El token de Minecraft lo entrega Horizon (`account session`, un archivo que se lee una vez y se borra al
// momento). No se guarda en ningún sitio ni pasa por la línea de órdenes: va a curl por su entrada estándar.

#[derive(Debug, Clone)]
pub struct Capa {
    pub id: String,
    pub nombre: String,
    pub activa: bool,
    pub imagen: Option<Handle>,
}

#[derive(Debug, Clone, Default)]
pub struct Perfil {
    pub delgado: bool,
    pub capas: Vec<Capa>,
}

async fn token(cuenta: &str) -> Result<String, String> {
    let v = crate::motor::json(vec!["account".into(), "session".into(), cuenta.to_string()]).await?;
    let archivo = v["session_file"].as_str().ok_or("Horizon no pudo renovar la sesión de esta cuenta")?.to_string();
    let texto = std::fs::read_to_string(&archivo).map_err(|e| e.to_string());
    let _ = std::fs::remove_file(&archivo);
    let json: serde_json::Value = serde_json::from_str(&texto?).map_err(|e| e.to_string())?;
    json["access_token"].as_str().map(str::to_string).ok_or_else(|| "La sesión no trae token".into())
}

/// Llama a la API de Minecraft con el token por stdin (`curl -K -`), así no aparece en `ps`.
async fn api(cuenta: &str, metodo: &str, ruta: &str, extra: &[String]) -> Result<Vec<u8>, String> {
    use tokio::io::AsyncWriteExt;
    let tok = token(cuenta).await?;
    let mut hijo = tokio::process::Command::new("curl")
        .args(["-sS", "--max-time", "20", "-K", "-", "-X", metodo, "-w", "\n%{http_code}"])
        .args(extra)
        .arg(format!("https://api.minecraftservices.com{ruta}"))
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| format!("No pude usar curl: {e}"))?;
    if let Some(mut entrada) = hijo.stdin.take() {
        entrada
            .write_all(format!("header = \"Authorization: Bearer {tok}\"\n").as_bytes())
            .await
            .map_err(|e| e.to_string())?;
    }
    drop(tok);
    let out = hijo.wait_with_output().await.map_err(|e| e.to_string())?;
    let mut cuerpo = out.stdout;
    let fin = cuerpo.iter().rposition(|b| *b == b'\n').unwrap_or(0);
    let codigo: u16 = String::from_utf8_lossy(&cuerpo[fin..]).trim().parse().unwrap_or(0);
    cuerpo.truncate(fin);
    match codigo {
        200..=299 => Ok(cuerpo),
        401 => Err("Mojang no aceptó la sesión: vuelve a iniciar sesión en Cuentas".into()),
        429 => Err("Demasiados cambios seguidos: espera un minuto".into()),
        0 => Err("Sin conexión con Mojang".into()),
        c => Err(format!("Mojang contestó {c}")),
    }
}

pub async fn perfil(cuenta: String) -> Result<Perfil, String> {
    let cuerpo = api(&cuenta, "GET", "/minecraft/profile", &[]).await?;
    let v: serde_json::Value = serde_json::from_slice(&cuerpo).map_err(|e| e.to_string())?;
    let delgado = v["skins"]
        .as_array()
        .and_then(|a| a.iter().find(|s| s["state"] == "ACTIVE"))
        .is_some_and(|s| s["variant"] == "SLIM");
    // Las imágenes de las capas se bajan todas a la vez.
    let capas = iced::futures::future::join_all(v["capes"].as_array().into_iter().flatten().map(|c| async move {
        let imagen = match c["url"].as_str() {
            Some(url) => curl(url).await.ok().and_then(|png| frente_capa(&png)),
            None => None,
        };
        Capa {
            id: c["id"].as_str().unwrap_or_default().to_string(),
            nombre: c["alias"].as_str().unwrap_or("Capa").to_string(),
            activa: c["state"] == "ACTIVE",
            imagen,
        }
    }))
    .await;
    Ok(Perfil { delgado, capas })
}

/// La cara delantera de una capa (10×16 en la plantilla de 64×32).
fn frente_capa(png: &[u8]) -> Option<Handle> {
    let img = image::load_from_memory(png).ok()?.to_rgba8();
    let escala = img.width() / 64;
    if escala == 0 {
        return None;
    }
    let recorte = image::imageops::crop_imm(&img, escala, escala, 10 * escala, 16 * escala).to_image();
    Some(Handle::from_rgba(10 * escala, 16 * escala, recorte.into_raw()))
}

pub async fn poner_skin(cuenta: String, ruta: PathBuf, delgado: bool) -> Result<(), String> {
    let variante = if delgado { "slim" } else { "classic" };
    api(
        &cuenta,
        "POST",
        "/minecraft/profile/skins",
        &["-F".into(), format!("variant={variante}"), "-F".into(), format!("file=@{};type=image/png", ruta.display())],
    )
    .await
    .map(|_| ())
}

pub async fn poner_capa(cuenta: String, id: Option<String>) -> Result<(), String> {
    match id {
        Some(id) => api(
            &cuenta,
            "PUT",
            "/minecraft/profile/capes/active",
            &["-H".into(), "Content-Type: application/json".into(), "-d".into(), format!("{{\"capeId\":\"{id}\"}}")],
        )
        .await
        .map(|_| ()),
        None => api(&cuenta, "DELETE", "/minecraft/profile/capes/active", &[]).await.map(|_| ()),
    }
}

// ---------- Figura en 3D ----------
//
// Cada parte del cuerpo es una caja con su textura de la plantilla de 64×64. Se gira (guiñada y cabeceo),
// se proyecta en ortográfica y se rellena con búfer de profundidad. La segunda capa (sombrero, chaqueta,
// mangas, pantalón) va en cajas un poco más grandes y deja ver lo de dentro donde es transparente.

struct Caja {
    min: [f32; 3],
    tam: [f32; 3],
    /// Esquina de la tira de texturas en la plantilla y tamaño en píxeles de textura (w, h, d).
    uv: (u32, u32),
    tex: (u32, u32, u32),
    capa: bool,
}

fn cajas(delgado: bool, nueva: bool) -> Vec<Caja> {
    let ab = if delgado { 3.0 } else { 4.0 };
    let abi = ab as u32;
    let mut v = Vec::new();
    let mut parte = |min: [f32; 3], tam: [f32; 3], uv: (u32, u32), uv2: Option<(u32, u32)>| {
        let tex = (tam[0] as u32, tam[1] as u32, tam[2] as u32);
        v.push(Caja { min, tam, uv, tex, capa: false });
        if let Some(uv2) = uv2 {
            let g = if tam[1] >= 8.0 && tam[0] == 8.0 && tam[2] == 8.0 { 0.5 } else { 0.25 };
            v.push(Caja {
                min: [min[0] - g, min[1] - g, min[2] - g],
                tam: [tam[0] + 2.0 * g, tam[1] + 2.0 * g, tam[2] + 2.0 * g],
                uv: uv2,
                tex,
                capa: true,
            });
        }
    };
    // Cabeza y sombrero
    parte([-4.0, 24.0, -4.0], [8.0, 8.0, 8.0], (0, 0), Some((32, 0)));
    // Cuerpo
    parte([-4.0, 12.0, -2.0], [8.0, 12.0, 4.0], (16, 16), nueva.then_some((16, 32)));
    // Brazo derecho (a la izquierda de quien mira) e izquierdo
    parte([-4.0 - ab, 12.0, -2.0], [ab, 12.0, 4.0], (40, 16), nueva.then_some((40, 32)));
    parte([4.0, 12.0, -2.0], [ab, 12.0, 4.0], if nueva { (32, 48) } else { (40, 16) }, nueva.then_some((48, 48)));
    // Piernas
    parte([-4.0, 0.0, -2.0], [4.0, 12.0, 4.0], (0, 16), nueva.then_some((0, 32)));
    parte([0.0, 0.0, -2.0], [4.0, 12.0, 4.0], if nueva { (16, 48) } else { (0, 16) }, nueva.then_some((0, 48)));
    let _ = abi;
    v
}

/// Dibuja la skin girada `guinada` radianes (0 = de frente) y `cabeceo` hacia arriba o abajo.
pub fn figura_3d(skin: &Path, delgado: bool, guinada: f32, cabeceo: f32, ancho: u32, alto: u32) -> Option<Handle> {
    let img = image::open(skin).ok()?.to_rgba8();
    if img.width() != 64 {
        return None;
    }
    let nueva = img.height() == 64;
    let (sy, cy) = guinada.sin_cos();
    let (sp, cp) = cabeceo.sin_cos();
    let escala = alto as f32 / 40.0;
    let (ox, oy) = (ancho as f32 / 2.0, alto as f32 * 0.5);
    let gira = |p: [f32; 3]| -> [f32; 3] {
        let x = p[0] * cy + p[2] * sy;
        let z = -p[0] * sy + p[2] * cy;
        let y = p[1] - 16.0;
        let y2 = y * cp - z * sp;
        let z2 = y * sp + z * cp;
        [ox + x * escala, oy - y2 * escala, z2]
    };
    let mut color = vec![0u8; (ancho * alto * 4) as usize];
    let mut prof = vec![f32::NEG_INFINITY; (ancho * alto) as usize];
    for c in cajas(delgado, nueva) {
        let [x0, y0, z0] = c.min;
        let [x1, y1, z1] = [x0 + c.tam[0], y0 + c.tam[1], z0 + c.tam[2]];
        let (u, v) = c.uv;
        let (w, h, d) = c.tex;
        // (origen, eje u, eje v, rectángulo de textura, luz)
        let caras: [([f32; 3], [f32; 3], [f32; 3], (u32, u32, u32, u32), f32); 6] = [
            ([x0, y1, z1], [x1 - x0, 0.0, 0.0], [0.0, y0 - y1, 0.0], (u + d, v + d, w, h), 1.0),
            ([x1, y1, z0], [x0 - x1, 0.0, 0.0], [0.0, y0 - y1, 0.0], (u + 2 * d + w, v + d, w, h), 0.75),
            ([x0, y1, z0], [0.0, 0.0, z1 - z0], [0.0, y0 - y1, 0.0], (u, v + d, d, h), 0.82),
            ([x1, y1, z1], [0.0, 0.0, z0 - z1], [0.0, y0 - y1, 0.0], (u + d + w, v + d, d, h), 0.82),
            ([x0, y1, z0], [x1 - x0, 0.0, 0.0], [0.0, 0.0, z1 - z0], (u + d, v, w, d), 1.1),
            ([x0, y0, z1], [x1 - x0, 0.0, 0.0], [0.0, 0.0, z0 - z1], (u + d + w, v, w, d), 0.6),
        ];
        for (o, eu, ev, (tu, tv, tw, th), luz) in caras {
            let p0 = gira(o);
            let pu = gira([o[0] + eu[0], o[1] + eu[1], o[2] + eu[2]]);
            let pv = gira([o[0] + ev[0], o[1] + ev[1], o[2] + ev[2]]);
            let a = [pu[0] - p0[0], pu[1] - p0[1], pu[2] - p0[2]];
            let b = [pv[0] - p0[0], pv[1] - p0[1], pv[2] - p0[2]];
            let det = a[0] * b[1] - a[1] * b[0];
            if det.abs() < 1e-3 {
                continue;
            }
            // Con la pantalla hacia abajo, una cara que mira a la cámara sale con det > 0. Las de espaldas
            // no se dibujan (salvo la segunda capa, que también se ve por dentro).
            if det < 0.0 && !c.capa {
                continue;
            }
            let minx = p0[0].min(pu[0]).min(pv[0]).min(pu[0] + b[0]).floor().max(0.0) as u32;
            let maxx = p0[0].max(pu[0]).max(pv[0]).max(pu[0] + b[0]).ceil().min(ancho as f32 - 1.0) as u32;
            let miny = p0[1].min(pu[1]).min(pv[1]).min(pu[1] + b[1]).floor().max(0.0) as u32;
            let maxy = p0[1].max(pu[1]).max(pv[1]).max(pu[1] + b[1]).ceil().min(alto as f32 - 1.0) as u32;
            for py in miny..=maxy {
                for px in minx..=maxx {
                    let dx = px as f32 + 0.5 - p0[0];
                    let dy = py as f32 + 0.5 - p0[1];
                    let s = (dx * b[1] - dy * b[0]) / det;
                    let t = (a[0] * dy - a[1] * dx) / det;
                    if !(0.0..1.0).contains(&s) || !(0.0..1.0).contains(&t) {
                        continue;
                    }
                    let z = p0[2] + s * a[2] + t * b[2] + if c.capa { 0.01 } else { 0.0 };
                    let i = (py * ancho + px) as usize;
                    if z <= prof[i] {
                        continue;
                    }
                    let tx = tu + ((s * tw as f32) as u32).min(tw.saturating_sub(1));
                    let ty = tv + ((t * th as f32) as u32).min(th.saturating_sub(1));
                    if tx >= img.width() || ty >= img.height() {
                        continue;
                    }
                    let texel = img.get_pixel(tx, ty);
                    if texel[3] < 128 {
                        continue;
                    }
                    prof[i] = z;
                    let k = i * 4;
                    color[k] = (texel[0] as f32 * luz).min(255.0) as u8;
                    color[k + 1] = (texel[1] as f32 * luz).min(255.0) as u8;
                    color[k + 2] = (texel[2] as f32 * luz).min(255.0) as u8;
                    color[k + 3] = 255;
                }
            }
        }
    }
    Some(Handle::from_rgba(ancho, alto, color))
}

/// La cara de la skin que lleva tu cuenta (API pública de Mojang, sin sesión), para la barra de arriba.
/// Se guarda en la caché para no pedirla cada vez.
pub async fn cara_de_cuenta(uuid: String) -> Option<Handle> {
    let cache = crate::ajustes::carpeta_datos().join(format!("cara-{uuid}.png"));
    let fresca = cache
        .metadata()
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.elapsed().ok())
        .is_some_and(|e| e < std::time::Duration::from_secs(6 * 3600));
    if !fresca {
        let perfil = curl(&format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}")).await.ok()?;
        let json: serde_json::Value = serde_json::from_slice(&perfil).ok()?;
        let valor = json["properties"].as_array()?.iter().find(|p| p["name"] == "textures")?["value"].as_str()?.to_string();
        let tex: serde_json::Value = serde_json::from_slice(&base64(&valor)?).ok()?;
        let url = tex["textures"]["SKIN"]["url"].as_str()?;
        let png = curl(url).await.ok()?;
        let _ = std::fs::create_dir_all(cache.parent()?);
        std::fs::write(&cache, png).ok()?;
    }
    let img = image::open(&cache).ok()?.to_rgba8();
    Some(cara(&img))
}

/// Abre el selector de archivos del sistema (zenity) para elegir un PNG y lo copia al armario.
pub async fn importar() -> Result<Option<PathBuf>, String> {
    let out = tokio::process::Command::new("zenity")
        .args(["--file-selection", "--title=Importar skin", "--file-filter=Skins PNG | *.png"])
        .output()
        .await
        .map_err(|_| "Para importar hace falta zenity (el selector de archivos)".to_string())?;
    let ruta = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if ruta.is_empty() {
        return Ok(None);
    }
    let origen = PathBuf::from(&ruta);
    let img = image::open(&origen).map_err(|_| "Ese archivo no es una imagen".to_string())?;
    if img.width() != 64 || (img.height() != 64 && img.height() != 32) {
        return Err("Una skin mide 64×64 (o 64×32 si es antigua)".into());
    }
    let dir = carpeta();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let destino = dir.join(origen.file_name().unwrap_or_default());
    std::fs::copy(&origen, &destino).map_err(|e| e.to_string())?;
    Ok(Some(destino))
}

/// Decodificador base64 para otros módulos (iconos de servidores).
pub fn base64_publico(s: &str) -> Option<Vec<u8>> {
    base64(s)
}
