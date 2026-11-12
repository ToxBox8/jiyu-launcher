//! Imágenes listas para dibujar: se leen, decodifican y reducen fuera del hilo de la interfaz,
//! y se entregan como píxeles. Así el dibujado nunca espera a disco ni a un decodificador PNG.

use iced::widget::image::Handle;
use std::path::{Path, PathBuf};

/// Lee una imagen y la deja como mucho en `ancho`×`alto` (sin agrandar).
pub fn reduce(ruta: &Path, ancho: u32, alto: u32, suave: bool) -> Option<Handle> {
    let img = image::open(ruta).ok()?;
    let img = if img.width() > ancho || img.height() > alto {
        let filtro = if suave { image::imageops::FilterType::Triangle } else { image::imageops::FilterType::Nearest };
        img.resize(ancho, alto, filtro)
    } else {
        img
    };
    let rgba = img.to_rgba8();
    Some(Handle::from_rgba(rgba.width(), rgba.height(), rgba.into_raw()))
}

/// Varias a la vez, en un hilo aparte.
pub async fn varias(rutas: Vec<PathBuf>, ancho: u32, alto: u32) -> Vec<(PathBuf, Handle)> {
    tokio::task::spawn_blocking(move || {
        rutas.into_iter().filter_map(|r| reduce(&r, ancho, alto, true).map(|h| (r, h))).collect()
    })
    .await
    .unwrap_or_default()
}
