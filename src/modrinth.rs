//! Logos de los mods. Horizon no los da, así que se piden a la API pública de Modrinth:
//! por id de proyecto (resultados de búsqueda) o por la huella SHA-1 de un .jar (mods instalados).
//! Se guardan reducidos a 64 px en `~/.cache/jiyu-launcher/logos/`, así la segunda vez no hay red.

use iced::widget::image::Handle;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

const AGENTE: &str = "jiyu-launcher/0.1";

pub fn carpeta_cache() -> PathBuf {
    cache()
}

fn cache() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache"))
        .join("jiyu-launcher/logos")
}

async fn curl(args: &[&str]) -> Option<Vec<u8>> {
    let out = tokio::process::Command::new("curl")
        .args(["-fsSL", "--max-time", "15", "-A", AGENTE])
        .args(args)
        .kill_on_drop(true)
        .output()
        .await
        .ok()?;
    out.status.success().then_some(out.stdout)
}

fn seguro(id: &str) -> String {
    id.chars().filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_').collect()
}

/// Logo ya guardado en disco, si lo hay (decodificado: no se llama desde el dibujado).
fn en_cache(id: &str) -> Option<Handle> {
    let f = cache().join(format!("{}.png", seguro(id)));
    if !f.is_file() {
        return None;
    }
    crate::miniatura::reduce(&f, 64, 64, true)
}

/// Baja (o lee de la caché) los logos de estos proyectos. Devuelve id → logo.
pub async fn logos(ids: Vec<String>) -> Vec<(String, Handle)> {
    let (mut hechos, faltan) = tokio::task::spawn_blocking(move || {
        let mut hechos = Vec::new();
        let mut faltan = Vec::new();
        for id in ids {
            match en_cache(&id) {
                Some(h) => hechos.push((id, h)),
                None => faltan.push(id),
            }
        }
        (hechos, faltan)
    })
    .await
    .unwrap_or_default();
    if faltan.is_empty() {
        return hechos;
    }
    let lista = serde_json::to_string(&faltan).unwrap_or_default();
    let url = format!("https://api.modrinth.com/v2/projects?ids={}", codifica(&lista));
    let Some(cuerpo) = curl(&[&url]).await else { return hechos };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&cuerpo) else { return hechos };
    let mut pares: Vec<(String, String)> = Vec::new();
    for p in v.as_array().into_iter().flatten() {
        let (Some(id), Some(icono)) = (p["id"].as_str(), p["icon_url"].as_str()) else { continue };
        // Puede venir pedido por slug: se guarda con los dos nombres.
        let slug = p["slug"].as_str().unwrap_or(id).to_string();
        let pedido = faltan.iter().find(|f| *f == id || **f == slug).cloned().unwrap_or(id.to_string());
        pares.push((pedido, icono.to_string()));
    }
    hechos.extend(logos_de(pares).await);
    hechos
}

/// Baja logos de los que ya sabemos la URL (la búsqueda de Modrinth la trae).
pub async fn logos_de(pares: Vec<(String, String)>) -> Vec<(String, Handle)> {
    let (mut hechos, pares) = tokio::task::spawn_blocking(move || {
        let mut hechos = Vec::new();
        let mut faltan = Vec::new();
        for (id, url) in pares {
            match en_cache(&id) {
                Some(h) => hechos.push((id, h)),
                None => faltan.push((id, url)),
            }
        }
        (hechos, faltan)
    })
    .await
    .unwrap_or_default();
    let bajadas = iced::futures::future::join_all(pares.into_iter().map(|(id, url)| async move {
        let bytes = curl(&[&url]).await?;
        let dir = cache();
        let destino = dir.join(format!("{}.png", seguro(&id)));
        let h = tokio::task::spawn_blocking(move || {
            reduce(&bytes, &destino)?;
            crate::miniatura::reduce(&destino, 64, 64, true)
        })
        .await
        .ok()??;
        let _ = dir;
        Some((id.clone(), h))
    }))
    .await;
    hechos.extend(bajadas.into_iter().flatten());
    hechos
}

/// Decodifica el logo (png, jpg o webp), lo deja en 64×64 y lo guarda como PNG.
fn reduce(bytes: &[u8], destino: &Path) -> Option<()> {
    let img = image::load_from_memory(bytes).ok()?;
    let img = img.resize(64, 64, image::imageops::FilterType::Triangle);
    std::fs::create_dir_all(destino.parent()?).ok()?;
    img.save_with_format(destino, image::ImageFormat::Png).ok()
}

fn codifica(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Identifica .jar por su SHA-1 en Modrinth: devuelve archivo → (id de proyecto, nombre del mod).
pub async fn identifica(archivos: Vec<PathBuf>) -> HashMap<PathBuf, (String, String)> {
    let mut res = HashMap::new();
    if archivos.is_empty() {
        return res;
    }
    // sha1sum de coreutils: rápido y sin dependencias nuevas.
    let Ok(out) = tokio::process::Command::new("sha1sum").args(&archivos).output().await else { return res };
    let mut por_hash: HashMap<String, PathBuf> = HashMap::new();
    for l in String::from_utf8_lossy(&out.stdout).lines() {
        if let Some((h, f)) = l.split_once("  ") {
            por_hash.insert(h.to_string(), PathBuf::from(f));
        }
    }
    let cuerpo = serde_json::json!({ "hashes": por_hash.keys().collect::<Vec<_>>(), "algorithm": "sha1" }).to_string();
    let Some(r) = curl(&["-X", "POST", "-H", "Content-Type: application/json", "-d", &cuerpo, "https://api.modrinth.com/v2/version_files"]).await
    else {
        return res;
    };
    let Ok(v) = serde_json::from_slice::<serde_json::Value>(&r) else { return res };
    let Some(mapa) = v.as_object() else { return res };
    let mut proyectos: Vec<String> = Vec::new();
    let mut hash_proyecto = Vec::new();
    for (h, ver) in mapa {
        if let Some(p) = ver["project_id"].as_str() {
            proyectos.push(p.to_string());
            hash_proyecto.push((h.clone(), p.to_string()));
        }
    }
    proyectos.sort();
    proyectos.dedup();
    // Los nombres de verdad de los proyectos.
    let mut nombres: HashMap<String, String> = HashMap::new();
    if !proyectos.is_empty() {
        let url = format!("https://api.modrinth.com/v2/projects?ids={}", codifica(&serde_json::to_string(&proyectos).unwrap_or_default()));
        if let Some(c) = curl(&[&url]).await {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&c) {
                for p in v.as_array().into_iter().flatten() {
                    if let (Some(id), Some(t)) = (p["id"].as_str(), p["title"].as_str()) {
                        nombres.insert(id.to_string(), t.to_string());
                    }
                }
            }
        }
    }
    for (h, p) in hash_proyecto {
        if let Some(f) = por_hash.get(&h) {
            res.insert(f.clone(), (p.clone(), nombres.get(&p).cloned().unwrap_or_default()));
        }
    }
    res
}


// ---------- Búsqueda e instalación directas ----------

/// Busca en Modrinth. `tipo`: mod, resourcepack, shader, modpack o datapack.
/// Resultados por página de búsqueda.
pub const POR_PAGINA: usize = 30;

pub async fn buscar(
    texto: String,
    tipo: String,
    cargador: Option<String>,
    version: Option<String>,
    desde: usize,
) -> Result<Vec<crate::motor::Encontrado>, String> {
    let mut facetas: Vec<Vec<String>> = Vec::new();
    if tipo == "datapack" {
        facetas.push(vec!["categories:datapack".into()]);
    } else {
        facetas.push(vec![format!("project_type:{tipo}")]);
    }
    if let Some(c) = cargador.filter(|_| tipo == "mod" || tipo == "modpack") {
        facetas.push(vec![format!("categories:{c}")]);
    }
    if let Some(v) = version {
        facetas.push(vec![format!("versions:{v}")]);
    }
    let url = format!(
        "https://api.modrinth.com/v2/search?limit={POR_PAGINA}&offset={desde}&query={}&facets={}",
        codifica(&texto),
        codifica(&serde_json::to_string(&facetas).unwrap_or_default())
    );
    let cuerpo = curl(&[&url]).await.ok_or("No pude hablar con Modrinth (¿sin conexión?)")?;
    let v: serde_json::Value = serde_json::from_slice(&cuerpo).map_err(|e| e.to_string())?;
    Ok(v["hits"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|m| crate::motor::Encontrado {
            id: m["slug"].as_str().or(m["project_id"].as_str()).unwrap_or_default().to_string(),
            modrinth: m["project_id"].as_str().map(str::to_string),
            icono_url: m["icon_url"].as_str().filter(|u| !u.is_empty()).map(str::to_string),
            titulo: m["title"].as_str().unwrap_or("?").to_string(),
            resumen: m["description"].as_str().unwrap_or_default().to_string(),
            autor: m["author"].as_str().unwrap_or_default().to_string(),
            descargas: m["downloads"].as_u64().unwrap_or(0),
            soporte: String::new(),
            pagina: format!("https://modrinth.com/{}/{}", m["project_type"].as_str().unwrap_or("mod"), m["slug"].as_str().unwrap_or_default()),
        })
        .collect())
}

/// Baja a la carpeta de la instancia la versión que vale para su Minecraft y cargador, con sus
/// dependencias obligatorias. Sirve para cualquier instancia (también las de Prism).
pub async fn instalar(
    proyecto: String,
    tipo: String,
    carpeta: PathBuf,
    mc: Option<String>,
    cargador: Option<String>,
) -> Result<Vec<String>, String> {
    let sub = match tipo.as_str() {
        "mod" => "mods",
        "resourcepack" => "resourcepacks",
        "shader" => "shaderpacks",
        otro => return Err(format!("No sé dónde va un «{otro}»")),
    };
    let destino = carpeta.join(sub);
    std::fs::create_dir_all(&destino).map_err(|e| e.to_string())?;
    let mut hechos = Vec::new();
    let mut pendientes = vec![proyecto];
    let mut vistos = std::collections::HashSet::new();
    while let Some(p) = pendientes.pop() {
        if !vistos.insert(p.clone()) || vistos.len() > 25 {
            continue;
        }
        let mut consulta = Vec::new();
        if tipo == "mod" {
            if let Some(c) = &cargador {
                consulta.push(format!("loaders={}", codifica(&format!("[\"{c}\"]"))));
            }
        }
        if let Some(v) = &mc {
            consulta.push(format!("game_versions={}", codifica(&format!("[\"{v}\"]"))));
        }
        let url = format!("https://api.modrinth.com/v2/project/{p}/version?{}", consulta.join("&"));
        let mut versiones: serde_json::Value =
            serde_json::from_slice(&curl(&[&url]).await.ok_or("Sin conexión con Modrinth")?).map_err(|e| e.to_string())?;
        // Los shaders y muchos paquetes no marcan versión: si no hay, se prueba sin filtrar.
        if versiones.as_array().is_none_or(|a| a.is_empty()) && tipo != "mod" {
            let url = format!("https://api.modrinth.com/v2/project/{p}/version");
            versiones = serde_json::from_slice(&curl(&[&url]).await.ok_or("Sin conexión con Modrinth")?).map_err(|e| e.to_string())?;
        }
        let Some(ver) = versiones.as_array().and_then(|a| a.first()).cloned() else {
            if hechos.is_empty() {
                return Err("No hay una versión para el Minecraft y el cargador de esta instancia".into());
            }
            continue;
        };
        let archivos = ver["files"].as_array().cloned().unwrap_or_default();
        let Some(f) = archivos.iter().find(|f| f["primary"].as_bool() == Some(true)).or(archivos.first()) else { continue };
        let (Some(url), Some(nombre)) = (f["url"].as_str(), f["filename"].as_str()) else { continue };
        let ruta = destino.join(nombre);
        if !ruta.exists() {
            let ok = curl(&["-o", &ruta.to_string_lossy(), url]).await.is_some();
            if !ok {
                let _ = std::fs::remove_file(&ruta);
                return Err(format!("No pude bajar {nombre}"));
            }
        }
        hechos.push(nombre.to_string());
        for d in ver["dependencies"].as_array().into_iter().flatten() {
            if d["dependency_type"] == "required" {
                if let Some(id) = d["project_id"].as_str() {
                    pendientes.push(id.to_string());
                }
            }
        }
    }
    Ok(hechos)
}

#[cfg(test)]
mod pruebas {
    /// Instala de verdad en una carpeta de prueba (PRUEBA_DESCARGA): un mod con dependencias, un shader y un paquete de texturas.
    #[tokio::test]
    async fn instala_de_verdad() {
        let Ok(dir) = std::env::var("PRUEBA_DESCARGA") else { return };
        let dir = std::path::PathBuf::from(dir);
        let t = std::time::Instant::now();
        let r = super::instalar("sodium-extra".into(), "mod".into(), dir.clone(), Some("1.21.1".into()), Some("fabric".into())).await;
        println!("mod: {r:?} {:?}", t.elapsed());
        let r = super::instalar("complementary-reimagined".into(), "shader".into(), dir.clone(), Some("1.21.1".into()), Some("fabric".into())).await;
        println!("shader: {r:?} {:?}", t.elapsed());
        let r = super::instalar("fresh-animations".into(), "resourcepack".into(), dir.clone(), Some("1.21.1".into()), Some("fabric".into())).await;
        println!("textura: {r:?} {:?}", t.elapsed());
    }
}
