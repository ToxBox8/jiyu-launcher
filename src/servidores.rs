//! Tus servidores de Minecraft: una lista que se añade sola a la lista Multijugador de todas tus instancias
//! (su `servers.dat`), y el estado de cada uno con la consulta de estado de Minecraft (jugadores, latencia,
//! descripción e icono), sin abrir el juego.

use crate::nbt::{self, Tag};
use iced::widget::image::Handle;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Servidor {
    pub nombre: String,
    pub direccion: String,
}

/// Añade a `servers.dat` de la carpeta del juego los servidores que falten. Devuelve cuántos añadió.
/// Escribe a un temporal y lo renombra, y deja la copia anterior como `servers.dat_old` (como hace el juego).
pub fn sincroniza(carpeta: &Path, lista: &[Servidor]) -> Result<usize, String> {
    if lista.is_empty() {
        return Ok(0);
    }
    let ruta = carpeta.join("servers.dat");
    let (nombre, mut raiz) = match std::fs::read(&ruta) {
        Ok(d) => nbt::lee(&d).ok_or("servers.dat está dañado: no lo toco")?,
        Err(_) => (String::new(), Tag::Compound(vec![("servers".into(), Tag::List(10, vec![]))])),
    };
    let Tag::Compound(campos) = &mut raiz else { return Err("servers.dat raro: no lo toco".into()) };
    if !campos.iter().any(|(k, _)| k == "servers") {
        campos.push(("servers".into(), Tag::List(10, vec![])));
    }
    let Some((_, Tag::List(_, servidores))) = campos.iter_mut().find(|(k, _)| k == "servers") else {
        return Err("servers.dat raro: no lo toco".into());
    };
    let mut nuevos = 0;
    for s in lista {
        let ya = servidores
            .iter()
            .any(|t| t.campo("ip").and_then(Tag::texto).is_some_and(|ip| ip.eq_ignore_ascii_case(&s.direccion)));
        if !ya {
            servidores.push(Tag::Compound(vec![
                ("name".into(), Tag::String(s.nombre.clone())),
                ("ip".into(), Tag::String(s.direccion.clone())),
                ("hidden".into(), Tag::Byte(0)),
            ]));
            nuevos += 1;
        }
    }
    if nuevos == 0 {
        return Ok(0);
    }
    std::fs::create_dir_all(carpeta).map_err(|e| e.to_string())?;
    if ruta.is_file() {
        let _ = std::fs::copy(&ruta, carpeta.join("servers.dat_old"));
    }
    let tmp = carpeta.join("servers.dat.tmp");
    std::fs::write(&tmp, nbt::escribe(&nombre, &raiz)).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, &ruta).map_err(|e| e.to_string())?;
    Ok(nuevos)
}

/// Estado de un servidor según su consulta de estado.
#[derive(Debug, Clone)]
pub struct Estado {
    pub en_linea: u32,
    pub maximo: u32,
    pub version: String,
    pub motd: String,
    pub ms: u32,
    pub icono: Option<Handle>,
}

fn varint(v: &mut Vec<u8>, mut x: i32) {
    loop {
        let mut b = (x & 0x7f) as u8;
        x = ((x as u32) >> 7) as i32;
        if x != 0 {
            b |= 0x80;
        }
        v.push(b);
        if x == 0 {
            break;
        }
    }
}

async fn lee_varint(s: &mut tokio::net::TcpStream) -> Result<i32, String> {
    let mut r = 0i32;
    for i in 0..5 {
        let b = s.read_u8().await.map_err(|e| e.to_string())?;
        r |= ((b & 0x7f) as i32) << (7 * i);
        if b & 0x80 == 0 {
            return Ok(r);
        }
    }
    Err("respuesta rara".into())
}

/// Texto de la descripción, que puede venir como texto o como componentes JSON.
fn texto_de(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Object(o) => {
            let mut t = o.get("text").and_then(|x| x.as_str()).unwrap_or("").to_string();
            for e in o.get("extra").and_then(|x| x.as_array()).into_iter().flatten() {
                t.push_str(&texto_de(e));
            }
            t
        }
        serde_json::Value::Array(a) => a.iter().map(texto_de).collect(),
        _ => String::new(),
    }
}

/// Quita los códigos de color «§x» de los MOTD antiguos.
fn sin_colores(s: &str) -> String {
    let mut r = String::new();
    let mut salta = false;
    for c in s.chars() {
        if salta {
            salta = false;
            continue;
        }
        if c == '§' {
            salta = true;
            continue;
        }
        r.push(c);
    }
    r.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub async fn estado(direccion: String) -> Result<Estado, String> {
    tokio::time::timeout(Duration::from_secs(5), consulta(direccion)).await.map_err(|_| "No contesta".to_string())?
}

async fn consulta(direccion: String) -> Result<Estado, String> {
    let (host, puerto) = match direccion.rsplit_once(':') {
        Some((h, p)) if p.parse::<u16>().is_ok() => (h.to_string(), p.parse().unwrap()),
        _ => (direccion.clone(), 25565u16),
    };
    let inicio = Instant::now();
    let mut s = tokio::net::TcpStream::connect((host.as_str(), puerto)).await.map_err(|_| "Sin conexión".to_string())?;
    let ms = inicio.elapsed().as_millis() as u32;
    // Saludo (estado siguiente = 1, estado) y petición de estado.
    let mut cuerpo = vec![0x00];
    varint(&mut cuerpo, 767);
    varint(&mut cuerpo, host.len() as i32);
    cuerpo.extend_from_slice(host.as_bytes());
    cuerpo.extend_from_slice(&puerto.to_be_bytes());
    varint(&mut cuerpo, 1);
    let mut paquete = Vec::new();
    varint(&mut paquete, cuerpo.len() as i32);
    paquete.extend(cuerpo);
    paquete.extend_from_slice(&[0x01, 0x00]);
    s.write_all(&paquete).await.map_err(|e| e.to_string())?;
    let _largo = lee_varint(&mut s).await?;
    let _id = lee_varint(&mut s).await?;
    let n = lee_varint(&mut s).await?.clamp(0, 1 << 21) as usize;
    let mut json = vec![0u8; n];
    s.read_exact(&mut json).await.map_err(|e| e.to_string())?;
    let v: serde_json::Value = serde_json::from_slice(&json).map_err(|_| "Respuesta rara".to_string())?;
    let icono = v["favicon"]
        .as_str()
        .and_then(|f| f.split_once("base64,").map(|(_, b)| b.to_string()))
        .and_then(|b| crate::skins::base64_publico(&b))
        .and_then(|png| image::load_from_memory(&png).ok())
        .map(|img| {
            let r = img.to_rgba8();
            Handle::from_rgba(r.width(), r.height(), r.into_raw())
        });
    Ok(Estado {
        en_linea: v["players"]["online"].as_u64().unwrap_or(0) as u32,
        maximo: v["players"]["max"].as_u64().unwrap_or(0) as u32,
        version: v["version"]["name"].as_str().unwrap_or_default().to_string(),
        motd: sin_colores(&texto_de(&v["description"])),
        ms,
        icono,
    })
}

#[cfg(test)]
mod pruebas {
    /// Sobre una copia en PRUEBA_DIR: añade un servidor, no lo duplica y conserva los que había.
    #[test]
    fn anade_sin_duplicar() {
        let Ok(dir) = std::env::var("PRUEBA_DIR") else { return };
        let dir = std::path::PathBuf::from(dir);
        let antes = crate::nbt::lee(&std::fs::read(dir.join("servers.dat")).unwrap()).unwrap().1;
        let n_antes = match antes.campo("servers") { Some(crate::nbt::Tag::List(_, v)) => v.len(), _ => 0 };
        let s = vec![super::Servidor { nombre: "Prueba".into(), direccion: "play.ejemplo.com".into() }];
        assert_eq!(super::sincroniza(&dir, &s).unwrap(), 1);
        assert_eq!(super::sincroniza(&dir, &s).unwrap(), 0);
        let despues = crate::nbt::lee(&std::fs::read(dir.join("servers.dat")).unwrap()).unwrap().1;
        let v = match despues.campo("servers") { Some(crate::nbt::Tag::List(_, v)) => v.clone(), _ => vec![] };
        assert_eq!(v.len(), n_antes + 1);
        assert!(dir.join("servers.dat_old").is_file());
        println!("antes {n_antes}, después {}", v.len());
    }
}
