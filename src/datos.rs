//! Lo que el launcher sabe de cada instancia, venga de Horizon o de Prism.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Motor {
    /// Horizon: Minecraft reescrito en Rust.
    Rust,
    /// Minecraft: Java Edition de siempre.
    Java,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origen {
    /// Instancia de Horizon (`horizon-launcher instance list`): puede ser Rust o Java.
    Horizon,
    /// Instancia de Prism Launcher, leída de su carpeta sin tocarla. Se juega con Prism.
    Prism,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Icono {
    /// Una textura de bloque de los archivos preparados de Horizon (`crafting_table_front`).
    Textura(String),
    /// Un color plano (`#3fa7d6`), como deja elegir Horizon.
    Color(u32),
    /// Una clave de icono de Prism (`flame_legacy`, `viltrum`…).
    Prism(String),
    Defecto,
}

#[derive(Debug, Clone)]
pub struct Instancia {
    /// Para Horizon, su id; para Prism, el nombre de su carpeta.
    pub id: String,
    pub nombre: String,
    pub origen: Origen,
    pub motor: Motor,
    /// Versión de Minecraft (Java) o versión fijada de Horizon.
    pub version: Option<String>,
    /// Sigue la última versión de Horizon.
    pub sigue_ultima: bool,
    /// Cargador de mods y su versión (`Fabric 0.19.3`).
    pub cargador: Option<(String, String)>,
    pub grupo: Option<String>,
    /// La carpeta del juego (saves, mods, resourcepacks…).
    pub carpeta: PathBuf,
    pub jugado_s: u64,
    pub ultima_ms: Option<u64>,
    pub icono: Icono,
    /// La que «Jugar» de Horizon arranca por defecto.
    pub elegida: bool,
}

impl Instancia {
    /// Clave única entre las dos fuentes.
    pub fn clave(&self) -> String {
        match self.origen {
            Origen::Horizon => format!("h:{}", self.id),
            Origen::Prism => format!("p:{}", self.id),
        }
    }

    pub fn etiqueta_motor(&self) -> &'static str {
        match self.motor {
            Motor::Rust => "RUST",
            Motor::Java => "JAVA",
        }
    }

    /// El texto bajo el nombre: «RUST · nightly-11-08» o «JAVA · Fabric».
    pub fn subtitulo(&self) -> String {
        match self.motor {
            Motor::Rust => match &self.version {
                Some(v) if !self.sigue_ultima => corta_version(v),
                _ => "última".into(),
            },
            Motor::Java => match &self.cargador {
                Some((c, _)) => c.clone(),
                None => self.version.clone().unwrap_or_else(|| "Vanilla".into()),
            },
        }
    }

    pub fn tiempo_jugado(&self) -> String {
        duracion(self.jugado_s)
    }
}

/// `nightly-2026-11-08-fdc87ae` → `nightly 11-08`; `native-fdc87ae-…` → `nativa fdc87ae`.
pub fn corta_version(v: &str) -> String {
    if let Some(resto) = v.strip_prefix("nightly-") {
        let partes: Vec<&str> = resto.split('-').collect();
        if partes.len() >= 3 {
            return format!("nightly {}-{}", partes[1], partes[2]);
        }
    }
    if let Some(resto) = v.strip_prefix("native-") {
        let _ = resto;
        return "nativa".into();
    }
    v.to_string()
}

pub fn duracion(s: u64) -> String {
    if s == 0 {
        "sin jugar".into()
    } else if s < 3600 {
        format!("{} min", (s / 60).max(1))
    } else {
        let h = s / 3600;
        let m = (s % 3600) / 60;
        if m == 0 {
            format!("{h} h")
        } else {
            format!("{h} h {m} min")
        }
    }
}

pub fn tamano(bytes: u64) -> String {
    const K: f64 = 1024.0;
    let b = bytes as f64;
    if b >= K * K * K {
        format!("{:.1} GB", b / K / K / K)
    } else if b >= K * K {
        format!("{:.0} MB", b / K / K)
    } else if b >= K {
        format!("{:.0} KB", b / K)
    } else {
        format!("{bytes} B")
    }
}

/// Fecha relativa a partir de milisegundos UNIX.
pub fn hace(ms: u64) -> String {
    let ahora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    let s = ahora.saturating_sub(ms) / 1000;
    match s {
        0..=59 => "ahora mismo".into(),
        60..=3599 => format!("hace {} min", s / 60),
        3600..=86_399 => format!("hace {} h", s / 3600),
        86_400..=2_591_999 => format!("hace {} días", s / 86_400),
        _ => format!("hace {} meses", s / 2_592_000),
    }
}
