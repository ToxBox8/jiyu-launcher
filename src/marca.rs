//! La marca de Jiyu dentro del juego de Horizon:
//!
//! - **Logo en el juego**: el mod nativo `jiyu_logo` (en `horizon-mod/`, va dentro del launcher) dibuja el
//!   pulso de 16 px abajo a la izquierda. Se compila junto a los mods nativos de la instancia.
//! - **Icono de la ventana**: el juego de Horizon se presenta como `horizon` y el escritorio toma su icono
//!   de `horizon.desktop`; se cambia su `Icon=` por el logo de 16 px (y se devuelve al apagarlo).

use std::path::{Path, PathBuf};

/// Los archivos del mod, tal como van en una carpeta de mod desempaquetada (un `.hnmod` abierto).
const MOD: [(&str, &str); 5] = [
    ("horizon-native.json", include_str!("../horizon-mod/horizon-native.json")),
    ("mixins/jiyu_logo/horizon-mixin.toml", include_str!("../horizon-mod/mixins/jiyu_logo/horizon-mixin.toml")),
    ("mixins/jiyu_logo/runtime.rs", include_str!("../horizon-mod/mixins/jiyu_logo/runtime.rs")),
    ("mixins/jiyu_logo/list.rs", include_str!("../horizon-mod/mixins/jiyu_logo/list.rs")),
    ("mixins/jiyu_logo/logo.rs", include_str!("../horizon-mod/mixins/jiyu_logo/logo.rs")),
];

const LOGO_16: &str = include_str!("../assets/logo/jiyu-16.svg");
const NOMBRE_ICONO: &str = "jiyu-launcher-juego";

/// Prepara la carpeta que se le pasa a `native-build`: los mods nativos que la instancia ya tiene
/// (cada `.hnmod` abierto en su subcarpeta) y el del logo. Devuelve la carpeta.
pub fn prepara_compilacion(instancia: &str, hnmods: &[PathBuf]) -> Result<PathBuf, String> {
    let seguro: String = instancia.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '_' }).collect();
    let base = crate::modrinth::carpeta_cache().parent().map(Path::to_path_buf).unwrap_or_default().join("compilar").join(seguro);
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    for h in hnmods {
        let nombre = h.file_stem().and_then(|n| n.to_str()).unwrap_or("mod");
        let destino = base.join(nombre);
        std::fs::create_dir_all(&destino).map_err(|e| e.to_string())?;
        abre_zip(h, &destino)?;
    }
    let propio = base.join("jiyu_logo");
    for (ruta, contenido) in MOD {
        let f = propio.join(ruta);
        if let Some(d) = f.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(&f, contenido).map_err(|e| e.to_string())?;
    }
    Ok(base)
}

/// Abre un `.hnmod` (un zip) con lo que haya en el sistema: `bsdtar` o `unzip`.
fn abre_zip(zip: &Path, destino: &Path) -> Result<(), String> {
    let intentos: [(&str, Vec<&std::ffi::OsStr>); 2] = [
        ("bsdtar", vec!["-xf".as_ref(), zip.as_os_str(), "-C".as_ref(), destino.as_os_str()]),
        ("unzip", vec!["-oq".as_ref(), zip.as_os_str(), "-d".as_ref(), destino.as_os_str()]),
    ];
    for (programa, args) in intentos {
        if let Ok(s) = std::process::Command::new(programa).args(args).status() {
            if s.success() {
                return Ok(());
            }
        }
    }
    Err(format!("No pude abrir {} (hace falta bsdtar o unzip)", zip.display()))
}

fn datos() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share"))
}

/// Pone (o quita) el logo de 16 px como icono de la ventana del juego de Horizon.
pub fn icono_ventana_juego(poner: bool) -> Result<(), String> {
    let entrada = datos().join("applications/horizon.desktop");
    if poner {
        let icono = datos().join("icons/hicolor/scalable/apps").join(format!("{NOMBRE_ICONO}.svg"));
        if let Some(d) = icono.parent() {
            std::fs::create_dir_all(d).map_err(|e| e.to_string())?;
        }
        std::fs::write(&icono, LOGO_16).map_err(|e| e.to_string())?;
    }
    // Sin horizon.desktop (Horizon aún no lo ha creado), no hay nada que cambiar todavía.
    let Ok(texto) = std::fs::read_to_string(&entrada) else { return Ok(()) };
    let nuevo_icono = if poner { NOMBRE_ICONO } else { "horizon" };
    let mut cambiado = false;
    let lineas: Vec<String> = texto
        .lines()
        .map(|l| {
            if l.starts_with("Icon=") && l != format!("Icon={nuevo_icono}") {
                // Al quitarlo, solo se toca si el icono es el nuestro.
                if poner || l == format!("Icon={NOMBRE_ICONO}") {
                    cambiado = true;
                    return format!("Icon={nuevo_icono}");
                }
            }
            l.to_string()
        })
        .collect();
    if cambiado {
        let tmp = entrada.with_extension("desktop.tmp");
        std::fs::write(&tmp, lineas.join("\n") + "\n").map_err(|e| e.to_string())?;
        std::fs::rename(&tmp, &entrada).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    /// Sobre una copia (PRUEBA_DATOS hace de XDG_DATA_HOME): pone el icono y lo quita.
    #[test]
    fn pone_y_quita_el_icono() {
        let Ok(dir) = std::env::var("PRUEBA_DATOS") else { return };
        std::env::set_var("XDG_DATA_HOME", &dir);
        let f = std::path::Path::new(&dir).join("applications/horizon.desktop");
        super::icono_ventana_juego(true).unwrap();
        assert!(std::fs::read_to_string(&f).unwrap().contains("Icon=jiyu-launcher-juego\n"));
        assert!(std::path::Path::new(&dir).join("icons/hicolor/scalable/apps/jiyu-launcher-juego.svg").is_file());
        super::icono_ventana_juego(false).unwrap();
        assert!(std::fs::read_to_string(&f).unwrap().contains("Icon=horizon\n"));
    }

    #[test]
    fn prepara_la_carpeta_con_el_mod() {
        let Ok(dir) = std::env::var("PRUEBA_DATOS") else { return };
        std::env::set_var("XDG_CACHE_HOME", &dir);
        let hn = std::env::var("PRUEBA_HNMOD").ok().map(std::path::PathBuf::from).into_iter().collect::<Vec<_>>();
        let base = super::prepara_compilacion("horizon", &hn).unwrap();
        assert!(base.join("jiyu_logo/mixins/jiyu_logo/horizon-mixin.toml").is_file());
        for h in &hn {
            let n = h.file_stem().unwrap();
            assert!(base.join(n).join("horizon-native.json").is_file());
        }
        println!("{}", base.display());
    }
}
