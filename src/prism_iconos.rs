//! Los iconos de serie de Prism Launcher, los mismos que ves en Prism. No vienen con este launcher:
//! la primera vez se bajan de su repositorio oficial a tu caché (`~/.cache/jiyu-launcher/prism/`),
//! los SVG se pasan a PNG de 128 px, y desde entonces se usan sin red.

use std::path::PathBuf;

const AGENTE: &str = "jiyu-launcher/0.1";
const BASE: &str = "https://api.github.com/repos/PrismLauncher/PrismLauncher/contents/launcher/resources/multimc";
/// Carpetas del repositorio, de mejor a peor calidad: la primera que tenga una clave gana.
const CARPETAS: [&str; 4] = ["scalable/instances", "128x128/instances", "50x50/instances", "32x32/instances"];

pub fn carpeta() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"))
        .join("jiyu-launcher/prism")
}

/// El PNG de una clave de Prism (`flame`, `enderpearl_legacy`…), si ya está en la caché.
pub fn archivo(clave: &str) -> Option<PathBuf> {
    let f = carpeta().join(format!("{clave}.png"));
    f.is_file().then_some(f)
}

/// Todas las claves ya descargadas, ordenadas (para el selector de icono).
pub fn claves() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(carpeta())
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| e.file_name().to_string_lossy().strip_suffix(".png").map(str::to_string))
        .collect();
    v.sort();
    v
}

pub fn descargados() -> bool {
    carpeta().join(".hecho").is_file()
}

async fn curl(url: &str) -> Option<Vec<u8>> {
    let out = tokio::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "20", "-A", AGENTE, url])
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;
    out.status.success().then_some(out.stdout)
}

/// Baja los que falten. Devuelve cuántos hay en la caché al terminar.
pub async fn descargar() -> Result<usize, String> {
    let dir = carpeta();
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let mut pendientes: Vec<(String, String, bool)> = Vec::new();
    for sub in CARPETAS {
        let Some(lista) = curl(&format!("{BASE}/{sub}")).await else { continue };
        let Ok(v) = serde_json::from_slice::<serde_json::Value>(&lista) else { continue };
        for f in v.as_array().into_iter().flatten() {
            let (Some(nombre), Some(url)) = (f["name"].as_str(), f["download_url"].as_str()) else { continue };
            let (clave, svg) = match (nombre.strip_suffix(".svg"), nombre.strip_suffix(".png")) {
                (Some(c), _) => (c, true),
                (_, Some(c)) => (c, false),
                _ => continue,
            };
            if pendientes.iter().any(|(c, _, _)| c == clave) || dir.join(format!("{clave}.png")).is_file() {
                continue;
            }
            pendientes.push((clave.to_string(), url.to_string(), svg));
        }
    }
    if pendientes.is_empty() && claves().is_empty() {
        return Err("No pude leer el repositorio de Prism (¿sin conexión?)".into());
    }
    let tareas = pendientes.into_iter().map(|(clave, url, svg)| {
        let dir = dir.clone();
        async move {
            let datos = curl(&url).await?;
            tokio::task::spawn_blocking(move || {
                let destino = dir.join(format!("{clave}.png"));
                if svg {
                    rasteriza(&datos, &destino)
                } else {
                    std::fs::write(&destino, datos).ok()
                }
            })
            .await
            .ok()?
        }
    });
    iced::futures::future::join_all(tareas).await;
    let _ = std::fs::write(dir.join(".hecho"), "");
    Ok(claves().len())
}

/// SVG → PNG de 128×128.
fn rasteriza(svg: &[u8], destino: &std::path::Path) -> Option<()> {
    let arbol = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).ok()?;
    let tam = arbol.size();
    let lado = 128.0;
    let escala = (lado / tam.width()).min(lado / tam.height());
    let mut lienzo = resvg::tiny_skia::Pixmap::new(lado as u32, lado as u32)?;
    let dx = (lado - tam.width() * escala) / 2.0;
    let dy = (lado - tam.height() * escala) / 2.0;
    resvg::render(&arbol, resvg::tiny_skia::Transform::from_row(escala, 0.0, 0.0, escala, dx, dy), &mut lienzo.as_mut());
    lienzo.save_png(destino).ok()
}

/// El gato de Prism (Kitteh) y sus amigos Rory y Teawie, para la esquina, como en Prism.
pub fn gato(nombre: &str) -> Option<PathBuf> {
    let f = carpeta().join("gatos").join(format!("{nombre}.png"));
    f.is_file().then_some(f)
}

pub async fn descargar_gatos() -> Vec<String> {
    let dir = carpeta().join("gatos");
    let _ = std::fs::create_dir_all(&dir);
    let tareas = ["kitteh", "rory", "teawie"].into_iter().map(|n| {
        let dir = dir.clone();
        async move {
            let destino = dir.join(format!("{n}.png"));
            if destino.is_file() {
                return Some(n.to_string());
            }
            let url = format!("https://raw.githubusercontent.com/PrismLauncher/PrismLauncher/develop/launcher/resources/backgrounds/{n}.png");
            let datos = curl(&url).await?;
            std::fs::write(&destino, datos).ok()?;
            Some(n.to_string())
        }
    });
    iced::futures::future::join_all(tareas).await.into_iter().flatten().collect()
}
