//! Lee las instancias de Prism Launcher tal como están en su carpeta, sin cambiar nada allí.
//! Se juegan con el propio Prism (`--launch`), así sus cuentas, Java y ajustes siguen valiendo.

use crate::datos::{Icono, Instancia, Motor, Origen};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Dónde vive Prism: primero el flatpak, luego la instalación normal.
pub fn raiz() -> Option<(PathBuf, bool)> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let flatpak = home.join(".var/app/org.prismlauncher.PrismLauncher/data/PrismLauncher");
    if flatpak.join("instances").is_dir() {
        return Some((flatpak, true));
    }
    let nativo = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local/share"))
        .join("PrismLauncher");
    if nativo.join("instances").is_dir() {
        return Some((nativo, false));
    }
    None
}

pub fn carpeta_iconos() -> Option<PathBuf> {
    raiz().map(|(r, _)| r.join("icons"))
}

/// Busca el archivo de un icono personalizado de Prism por su clave (`viltrum` → `viltrum.jpg`).
pub fn archivo_icono(clave: &str) -> Option<PathBuf> {
    let dir = carpeta_iconos()?;
    for ext in ["png", "jpg", "jpeg", "webp"] {
        let f = dir.join(format!("{clave}.{ext}"));
        if f.is_file() {
            return Some(f);
        }
    }
    None
}

fn lee_cfg(ruta: &Path) -> HashMap<String, String> {
    let mut mapa = HashMap::new();
    let Ok(texto) = std::fs::read_to_string(ruta) else { return mapa };
    let mut seccion = String::new();
    for linea in texto.lines() {
        let l = linea.trim();
        if l.starts_with('[') && l.ends_with(']') {
            seccion = l[1..l.len() - 1].to_string();
            continue;
        }
        if seccion != "General" && !seccion.is_empty() {
            continue;
        }
        if let Some((k, v)) = l.split_once('=') {
            mapa.insert(k.trim().to_string(), v.trim().trim_matches('"').to_string());
        }
    }
    mapa
}

fn lee_pack(ruta: &Path) -> (Option<String>, Option<(String, String)>) {
    let Ok(texto) = std::fs::read_to_string(ruta) else { return (None, None) };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&texto) else { return (None, None) };
    let mut mc = None;
    let mut cargador = None;
    for c in json["components"].as_array().into_iter().flatten() {
        let uid = c["uid"].as_str().unwrap_or("");
        let ver = c["version"].as_str().unwrap_or("").to_string();
        let nombre = match uid {
            "net.minecraft" => {
                mc = Some(ver);
                continue;
            }
            "net.fabricmc.fabric-loader" => "Fabric",
            "org.quiltmc.quilt-loader" => "Quilt",
            "net.minecraftforge" => "Forge",
            "net.neoforged" => "NeoForge",
            _ => continue,
        };
        cargador = Some((nombre.to_string(), ver));
    }
    (mc, cargador)
}

fn lee_grupos(raiz: &Path) -> HashMap<String, String> {
    let mut mapa = HashMap::new();
    let Ok(texto) = std::fs::read_to_string(raiz.join("instances/instgroups.json")) else { return mapa };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&texto) else { return mapa };
    if let Some(grupos) = json["groups"].as_object() {
        for (grupo, datos) in grupos {
            for inst in datos["instances"].as_array().into_iter().flatten() {
                if let Some(i) = inst.as_str() {
                    mapa.insert(i.to_string(), grupo.clone());
                }
            }
        }
    }
    mapa
}

/// Todas las instancias de Prism. Lectura de disco pequeña: se hace fuera del hilo de la interfaz.
pub fn instancias() -> Vec<Instancia> {
    let Some((raiz, _)) = raiz() else { return Vec::new() };
    let grupos = lee_grupos(&raiz);
    let Ok(dir) = std::fs::read_dir(raiz.join("instances")) else { return Vec::new() };
    let mut lista = Vec::new();
    for entrada in dir.flatten() {
        let ruta = entrada.path();
        let cfg_ruta = ruta.join("instance.cfg");
        if !cfg_ruta.is_file() {
            continue;
        }
        let carpeta_id = entrada.file_name().to_string_lossy().to_string();
        let cfg = lee_cfg(&cfg_ruta);
        let (mc, cargador) = lee_pack(&ruta.join("mmc-pack.json"));
        let juego = if ruta.join("minecraft").is_dir() { ruta.join("minecraft") } else { ruta.join(".minecraft") };
        let icono = match cfg.get("iconKey").map(String::as_str) {
            None | Some("") | Some("default") => Icono::Defecto,
            Some(k) => Icono::Prism(k.to_string()),
        };
        lista.push(Instancia {
            nombre: cfg.get("name").cloned().unwrap_or_else(|| carpeta_id.clone()),
            grupo: grupos.get(&carpeta_id).cloned(),
            id: carpeta_id,
            origen: Origen::Prism,
            motor: Motor::Java,
            version: mc,
            sigue_ultima: false,
            cargador,
            carpeta: juego,
            jugado_s: cfg.get("totalTimePlayed").and_then(|v| v.parse().ok()).unwrap_or(0),
            ultima_ms: cfg.get("lastLaunchTime").and_then(|v| v.parse().ok()).filter(|&v: &u64| v > 0),
            icono,
            elegida: false,
        });
    }
    lista.sort_by(|a, b| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()));
    lista
}

/// La orden que abre una instancia con Prism.
pub fn orden_jugar(carpeta_id: &str) -> Option<std::process::Command> {
    let (_, flatpak) = raiz()?;
    let mut c = if flatpak {
        let mut c = std::process::Command::new("flatpak");
        c.args(["run", "org.prismlauncher.PrismLauncher"]);
        c
    } else {
        std::process::Command::new("prismlauncher")
    };
    c.args(["--launch", carpeta_id]);
    Some(c)
}
