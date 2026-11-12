//! Preferencias del launcher (se guardan en `~/.config/jiyu-launcher/ajustes.json`) y lo que sabemos del PC.

use crate::tema::Tema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Perfil {
    Maximo,
    Equilibrado,
    Silencioso,
    Propio,
}

impl Perfil {
    pub fn nombre(self) -> &'static str {
        match self {
            Perfil::Maximo => "Máximo rendimiento",
            Perfil::Equilibrado => "Equilibrado",
            Perfil::Silencioso => "Silencioso",
            Perfil::Propio => "A mi manera",
        }
    }
    pub fn detalle(self) -> &'static str {
        match self {
            Perfil::Maximo => "GameMode, gráfica dedicada, G1 ajustado y páginas grandes. El launcher se cierra al jugar.",
            Perfil::Equilibrado => "G1 ajustado y la memoria justa. El launcher se cierra al jugar.",
            Perfil::Silencioso => "Lo que trae Java de serie y menos memoria. Para portátiles y partidas tranquilas.",
            Perfil::Propio => "Has tocado algo a mano: se guarda tal cual.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Recolector {
    /// G1 con los ajustes de Aikar: pausas cortas y estables (lo normal para jugar).
    G1Aikar,
    /// ZGC: pausas casi nulas, más CPU. Para Java 21 o más y mucha memoria.
    Zgc,
    /// Lo que pone Java por defecto.
    Normal,
}

/// Cómo se ven las instancias en la ventana principal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Vista {
    /// Por grupos, como Prism.
    Grupos,
    /// Carpetas que se abren, como Nautilus.
    Carpetas,
    /// Lista en árbol.
    Lista,
}

/// Cómo se ordenan las instancias dentro de cada carpeta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Orden {
    Nombre,
    Ultima,
    Jugado,
    /// Como tú las pongas, arrastrándolas.
    Manual,
}

impl Orden {
    pub const TODOS: [Orden; 4] = [Orden::Nombre, Orden::Ultima, Orden::Jugado, Orden::Manual];
}

impl std::fmt::Display for Orden {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Orden::Nombre => "Por nombre",
            Orden::Ultima => "Última vez jugada",
            Orden::Jugado => "Más jugadas",
            Orden::Manual => "A mano (arrastrando)",
        })
    }
}

/// Un mod (o pack) guardado en favoritos.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Favorito {
    pub id: String,
    pub modrinth: Option<String>,
    pub titulo: String,
    pub autor: String,
    pub tipo: String,
    pub icono_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Tamano {
    Pequeno,
    Normal,
    Grande,
}

impl Tamano {
    pub fn px(self) -> f32 {
        match self {
            Tamano::Pequeno => 32.0,
            Tamano::Normal => 48.0,
            Tamano::Grande => 64.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Ajustes {
    pub tema: Tema,
    // Apariencia: cada pieza que distingue Rust de Java se enciende por separado.
    pub insignia: bool,
    pub filo: bool,
    pub hexagono: bool,
    pub etiqueta: bool,
    pub esquina: bool,
    pub texturas_mc: bool,
    pub tamano: Tamano,
    pub ver_prism: bool,
    // Rendimiento
    pub perfil: Perfil,
    pub cerrar_al_jugar: bool,
    pub gestionar_java: bool,
    pub memoria_mib: u32,
    pub recolector: Recolector,
    pub paginas_grandes: bool,
    pub gamemode: bool,
    pub mangohud: bool,
    pub gpu_dedicada: bool,
    pub hilos_nvidia: bool,
    pub hilo_servidor: bool,
    // El propio launcher
    pub buscar_actualizaciones: bool,
    /// Dibujar el launcher con la gráfica (wgpu): desplazamiento fluido. Si falla, se usa el dibujo por software.
    pub grafica: bool,
    pub grupos_cerrados: Vec<String>,
    pub seleccion: Option<String>,
    pub modelo_delgado: bool,
    pub vista: Vista,
    pub orden: Orden,
    /// El orden a mano (claves de instancia), cuando `orden` es Manual.
    pub orden_manual: Vec<String>,
    /// Instancias archivadas: no salen en la ventana, solo en el archivador.
    pub archivadas: Vec<String>,
    /// Grupos archivados («A/B»): ellos, sus subcarpetas y lo de dentro van al archivador.
    #[serde(default)]
    pub carpetas_archivadas: Vec<String>,
    /// Color de cada carpeta («A/B» → 0xRRGGBB).
    pub colores_carpeta: std::collections::HashMap<String, u32>,
    /// Qué mascota sale en la esquina: auto (la de Jiyu con Jiyu Pulse, el gato de Prism con los demás),
    /// kitteh, rory, teawie o jiyu.
    pub mascota: String,
    /// Proxy para todo lo que baja el launcher y Horizon (http://… o socks5://…). Vacío = sin proxy.
    pub proxy: String,
    /// Al cerrar el juego, volver a abrir el launcher.
    pub reabrir: bool,
    /// El logo de Jiyu (16 px) abajo a la izquierda dentro del juego: un mod nativo que se añade al compilar.
    pub logo_juego: bool,
    /// El logo de Jiyu como icono de la ventana del juego de Horizon (su entrada `horizon.desktop`).
    pub icono_juego: bool,
    /// Tus servidores, que se añaden a la lista Multijugador de todas tus instancias.
    pub servidores: Vec<crate::servidores::Servidor>,
    pub servidores_siempre: bool,
    /// Carpetas creadas aquí (rutas «A/B»), aunque estén vacías.
    pub carpetas: Vec<String>,
    /// Carpeta de cada instancia de Prism movida aquí (Prism no se toca).
    pub carpetas_prism: std::collections::HashMap<String, String>,
    pub favoritos: Vec<Favorito>,
    /// Skin que se pone al jugar cada instancia (ruta del armario); si no está, la que lleves.
    pub skin_instancia: std::collections::HashMap<String, String>,
    /// Iconos elegidos aquí, por instancia («p:carpeta» → «tex:grass_block», «#3fa7d6», «prism:viltrum»).
    pub iconos: std::collections::HashMap<String, String>,
}

impl Default for Ajustes {
    fn default() -> Self {
        let mut a = Self {
            tema: Tema::Matugen,
            insignia: true,
            filo: true,
            hexagono: false,
            etiqueta: true,
            esquina: true,
            texturas_mc: false,
            tamano: Tamano::Normal,
            ver_prism: true,
            perfil: Perfil::Equilibrado,
            cerrar_al_jugar: true,
            gestionar_java: true,
            memoria_mib: 4096,
            recolector: Recolector::G1Aikar,
            paginas_grandes: false,
            gamemode: false,
            mangohud: false,
            gpu_dedicada: false,
            hilos_nvidia: false,
            hilo_servidor: false,
            buscar_actualizaciones: true,
            grafica: false,
            grupos_cerrados: Vec::new(),
            seleccion: None,
            modelo_delgado: false,
            iconos: Default::default(),
            vista: Vista::Grupos,
            orden: Orden::Nombre,
            orden_manual: Vec::new(),
            archivadas: Vec::new(),
            carpetas_archivadas: Vec::new(),
            colores_carpeta: Default::default(),
            mascota: "auto".into(),
            proxy: String::new(),
            reabrir: true,
            logo_juego: false,
            icono_juego: false,
            servidores: Vec::new(),
            servidores_siempre: true,
            carpetas: Vec::new(),
            carpetas_prism: Default::default(),
            favoritos: Vec::new(),
            skin_instancia: Default::default(),
        };
        a.memoria_mib = Sistema::detecta().memoria_recomendada();
        a
    }
}

pub fn carpeta_config() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config"))
        .join("jiyu-launcher")
}

pub fn carpeta_datos() -> PathBuf {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".local/share"))
        .join("jiyu-launcher")
}

impl Ajustes {
    pub fn carga() -> Self {
        std::fs::read_to_string(carpeta_config().join("ajustes.json"))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default()
    }

    pub fn guarda(&self) {
        let dir = carpeta_config();
        let _ = std::fs::create_dir_all(&dir);
        if let Ok(t) = serde_json::to_string_pretty(self) {
            // Escribe a un temporal y lo renombra: nunca deja el archivo a medias.
            let tmp = dir.join("ajustes.json.tmp");
            if std::fs::write(&tmp, t).is_ok() {
                let _ = std::fs::rename(tmp, dir.join("ajustes.json"));
            }
        }
    }

    pub fn aplica_perfil(&mut self, p: Perfil, sis: &Sistema) {
        self.perfil = p;
        let rec = sis.memoria_recomendada();
        match p {
            Perfil::Maximo => {
                self.cerrar_al_jugar = true;
                self.memoria_mib = rec;
                self.recolector = Recolector::G1Aikar;
                self.paginas_grandes = true;
                self.gamemode = sis.gamemode;
                self.gpu_dedicada = sis.nvidia.is_some();
                self.hilos_nvidia = sis.nvidia.is_some();
                self.hilo_servidor = true;
            }
            Perfil::Equilibrado => {
                self.cerrar_al_jugar = true;
                self.memoria_mib = rec;
                self.recolector = Recolector::G1Aikar;
                self.paginas_grandes = false;
                self.gamemode = false;
                self.gpu_dedicada = sis.nvidia.is_some();
                self.hilos_nvidia = false;
                self.hilo_servidor = false;
            }
            Perfil::Silencioso => {
                self.cerrar_al_jugar = false;
                self.memoria_mib = (rec / 2).max(2048);
                self.recolector = Recolector::Normal;
                self.paginas_grandes = false;
                self.gamemode = false;
                self.gpu_dedicada = false;
                self.hilos_nvidia = false;
                self.hilo_servidor = false;
            }
            Perfil::Propio => {}
        }
    }

    /// Los argumentos de la JVM que salen de las opciones (la memoria va aparte, con `--memory`).
    pub fn argumentos_jvm(&self) -> String {
        let mut a: Vec<&str> = match self.recolector {
            Recolector::G1Aikar => vec![
                "-XX:+UseG1GC",
                "-XX:+ParallelRefProcEnabled",
                "-XX:MaxGCPauseMillis=200",
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+DisableExplicitGC",
                "-XX:+AlwaysPreTouch",
                "-XX:G1NewSizePercent=30",
                "-XX:G1MaxNewSizePercent=40",
                "-XX:G1HeapRegionSize=8M",
                "-XX:G1ReservePercent=20",
                "-XX:G1HeapWastePercent=5",
                "-XX:G1MixedGCCountTarget=4",
                "-XX:InitiatingHeapOccupancyPercent=15",
                "-XX:G1MixedGCLiveThresholdPercent=90",
                "-XX:G1RSetUpdatingPauseTimePercent=5",
                "-XX:SurvivorRatio=32",
                "-XX:+PerfDisableSharedMem",
                "-XX:MaxTenuringThreshold=1",
            ],
            Recolector::Zgc => vec!["-XX:+UseZGC", "-XX:+AlwaysPreTouch", "-XX:+DisableExplicitGC"],
            Recolector::Normal => vec![],
        };
        if self.paginas_grandes {
            a.push("-XX:+UseTransparentHugePages");
        }
        a.join(" ")
    }

    /// Variables de entorno para el juego.
    pub fn entorno(&self) -> String {
        let mut v = Vec::new();
        if self.hilos_nvidia {
            v.push("__GL_THREADED_OPTIMIZATIONS=1");
            v.push("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP=1");
        }
        v.join(" ")
    }

    pub fn argumentos_rust(&self) -> Vec<String> {
        if self.hilo_servidor {
            vec!["--server-thread".into()]
        } else {
            vec![]
        }
    }
}

/// Lo que hay en este PC. Se mira una vez al abrir.
#[derive(Debug, Clone, Default)]
pub struct Sistema {
    pub memoria_mib: u64,
    pub hilos: usize,
    pub nvidia: Option<String>,
    pub gamemode: bool,
    pub mangohud: bool,
}

fn en_path(bin: &str) -> bool {
    std::env::var_os("PATH")
        .map(|p| std::env::split_paths(&p).any(|d| d.join(bin).is_file()))
        .unwrap_or(false)
}

impl Sistema {
    pub fn detecta() -> Self {
        let memoria_mib = std::fs::read_to_string("/proc/meminfo")
            .ok()
            .and_then(|t| {
                t.lines()
                    .find(|l| l.starts_with("MemTotal:"))
                    .and_then(|l| l.split_whitespace().nth(1))
                    .and_then(|k| k.parse::<u64>().ok())
            })
            .map(|kb| kb / 1024)
            .unwrap_or(8192);
        // El nombre de la gráfica NVIDIA sin abrir nvidia-smi (que tarda): lo deja el driver en /proc.
        let nvidia = std::fs::read_dir("/proc/driver/nvidia/gpus")
            .ok()
            .and_then(|mut d| d.next())
            .and_then(|e| e.ok())
            .and_then(|e| std::fs::read_to_string(e.path().join("information")).ok())
            .and_then(|t| {
                t.lines()
                    .find(|l| l.starts_with("Model:"))
                    .map(|l| l.trim_start_matches("Model:").trim().to_string())
            });
        Self {
            memoria_mib,
            hilos: std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4),
            nvidia,
            gamemode: en_path("gamemoderun"),
            mangohud: en_path("mangohud"),
        }
    }

    /// Un cuarto de la RAM, entre 2 y 8 GB, redondeado a GB: más no ayuda a Minecraft y quita al sistema.
    pub fn memoria_recomendada(&self) -> u32 {
        let cuarto = (self.memoria_mib / 4).clamp(2048, 8192);
        ((cuarto + 512) / 1024 * 1024) as u32
    }
}

/// Memoria que ocupa ahora mismo este proceso (VmRSS), en MB.
pub fn memoria_propia_mb() -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|t| {
            t.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1))
                .and_then(|k| k.parse::<u64>().ok())
        })
        .map(|kb| kb / 1024)
        .unwrap_or(0)
}
