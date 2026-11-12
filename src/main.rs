//! Jiyu Launcher: launcher con la estructura de Prism que usa Horizon como motor.
//!
//! `jiyu-launcher` abre la ventana. `jiyu-launcher --vigilar <pid> <carpeta>` es el vigilante: no abre ventana ni
//! carga la interfaz; espera a que el juego termine y vuelve a abrir el launcher. Así, mientras juegas,
//! el launcher no ocupa memoria ni CPU.

mod ajustes;
mod marca;
mod app;
mod datos;
mod iconos;
mod miniatura;
mod modrinth;
mod motor;
mod nbt;
mod prism;
mod prism_iconos;
mod servidores;
mod skins;
mod tema;
mod vistas;

use std::path::Path;
use std::time::Duration;

fn main() -> iced::Result {
    let args: Vec<String> = std::env::args().collect();
    if args.get(1).map(String::as_str) == Some("--vigilar") {
        let pid: u32 = args.get(2).and_then(|p| p.parse().ok()).unwrap_or(0);
        let carpeta = args.get(3).cloned().unwrap_or_default();
        vigilar(pid, Path::new(&carpeta));
        return Ok(());
    }

    migra_carpetas();
    // Cómo se dibuja la ventana: con la gráfica (fluido) o por software (menos memoria). Si la gráfica
    // no arranca, iced pasa solo al dibujo por software.
    let aj = ajustes::Ajustes::carga();
    // El proxy vale también para lo que baja el propio launcher (curl lo lee de estas variables).
    if !aj.proxy.trim().is_empty() {
        for v in ["ALL_PROXY", "HTTPS_PROXY", "HTTP_PROXY"] {
            std::env::set_var(v, aj.proxy.trim());
        }
    }
    if std::env::var_os("ICED_BACKEND").is_none() {
        std::env::set_var("ICED_BACKEND", if aj.grafica { "wgpu,tiny-skia" } else { "tiny-skia" });
    }

    let mut ventana = iced::window::Settings {
        size: iced::Size::new(1180.0, 760.0),
        min_size: Some(iced::Size::new(900.0, 600.0)),
        icon: iconos::icono_ventana(),
        ..Default::default()
    };
    #[cfg(target_os = "linux")]
    {
        ventana.platform_specific.application_id = "jiyu-launcher".into();
    }

    iced::application(app::App::titulo, app::App::update, vistas::vista)
        .theme(app::App::theme)
        .subscription(suscripciones)
        .window(ventana)
        .font(include_bytes!("../assets/fonts/inter_regular.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/inter_medium.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/inter_semibold.ttf").as_slice())
        .font(include_bytes!("../assets/fonts/inter_bold.ttf").as_slice())
        .default_font(vistas::INTER)
        .antialiasing(true)
        .run_with(app::App::nuevo)
}

fn suscripciones(app: &app::App) -> iced::Subscription<app::Msg> {
    use iced::keyboard::{key::Named, Key};
    use app::{Atajo, Msg};
    // Solo llegan las teclas que no ha usado un campo de texto (escribir «n» en el buscador no crea nada).
    let esc = iced::keyboard::on_key_press(|k, m| match k.as_ref() {
        Key::Named(Named::Escape) => Some(Msg::Cerrar),
        Key::Named(Named::Enter) => Some(Msg::Atajo(Atajo::Jugar)),
        Key::Named(Named::Delete) => Some(Msg::Atajo(Atajo::Borrar)),
        Key::Named(Named::F5) => Some(Msg::Atajo(Atajo::Recargar)),
        Key::Character("f") if m.control() => Some(Msg::Atajo(Atajo::Buscar)),
        Key::Character("n") if m.control() => Some(Msg::Atajo(Atajo::Nueva)),
        // Big Picture: F11 entra y sale; las flechas mueven la selección (solo hacen algo dentro).
        Key::Named(Named::F11) => Some(Msg::BPAlternar),
        Key::Named(Named::ArrowLeft) => Some(Msg::BPMover(-1, 0)),
        Key::Named(Named::ArrowRight) => Some(Msg::BPMover(1, 0)),
        Key::Named(Named::ArrowUp) => Some(Msg::BPMover(0, -1)),
        Key::Named(Named::ArrowDown) => Some(Msg::BPMover(0, 1)),
        Key::Character("e") if !m.control() => Some(Msg::BPEditar),
        _ => None,
    });
    iced::Subscription::batch([app.subscription(), esc])
}

/// ¿Sigue vivo el juego? El proceso que arrancamos, o cualquiera que trabaje en la carpeta de la instancia
/// (por si el lanzador de Horizon o Prism delega en otro proceso y termina antes que el juego).
fn juego_vivo(pid: u32, carpeta: &Path) -> bool {
    if pid != 0 && Path::new(&format!("/proc/{pid}")).exists() {
        let estado = std::fs::read_to_string(format!("/proc/{pid}/stat")).unwrap_or_default();
        // Un zombi (Z) ya terminó.
        if !estado.split(") ").nth(1).is_some_and(|r| r.starts_with('Z')) {
            return true;
        }
    }
    if carpeta.as_os_str().is_empty() {
        return false;
    }
    let yo = std::process::id();
    let Ok(dir) = std::fs::read_dir("/proc") else { return false };
    let texto = carpeta.to_string_lossy();
    dir.flatten().any(|e| {
        let Some(n) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { return false };
        if n == yo {
            return false;
        }
        let cwd = std::fs::read_link(e.path().join("cwd")).ok();
        if cwd.is_some_and(|c| c.starts_with(carpeta)) {
            return true;
        }
        std::fs::read(e.path().join("cmdline"))
            .map(|c| String::from_utf8_lossy(&c).contains(texto.as_ref()))
            .unwrap_or(false)
    })
}

fn vigilar(pid: u32, carpeta: &Path) {
    // Unos segundos de gracia: el juego tarda en aparecer cuando el lanzador lo delega.
    std::thread::sleep(Duration::from_secs(8));
    while juego_vivo(pid, carpeta) {
        std::thread::sleep(Duration::from_secs(3));
    }
    if let Ok(yo) = std::env::current_exe() {
        let _ = std::process::Command::new(yo)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }
}

/// Antes se llamaba RustCraft: si quedan sus carpetas (ajustes, datos, caché) y aún no hay de Jiyu,
/// se renombran, así no se pierde nada.
fn migra_carpetas() {
    let casa = std::path::PathBuf::from(std::env::var_os("HOME").unwrap_or_default());
    for (var, defecto) in [("XDG_CONFIG_HOME", ".config"), ("XDG_DATA_HOME", ".local/share"), ("XDG_CACHE_HOME", ".cache")] {
        let base = std::env::var_os(var).map(std::path::PathBuf::from).unwrap_or_else(|| casa.join(defecto));
        let (viejo, nuevo) = (base.join("rustcraft"), base.join("jiyu-launcher"));
        if viejo.is_dir() && !nuevo.exists() {
            let _ = std::fs::rename(&viejo, &nuevo);
        }
    }
}
