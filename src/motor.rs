//! El motor: todo lo que hace el juego de verdad lo hace `horizon-launcher` con `--json`.
//! Este launcher solo le pide cosas y pinta lo que contesta; no reimplementa descargas,
//! inicio de sesión ni arranque del juego.

use crate::datos::{Icono, Instancia, Motor, Origen};
use serde::Deserialize;
use serde_json::Value;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::OnceLock;
use tokio::process::Command;

static BINARIO: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Busca `horizon-launcher`: en el PATH y en las rutas donde lo deja su instalador.
pub fn binario() -> Option<PathBuf> {
    BINARIO
        .get_or_init(|| {
            if let Some(propio) = std::env::var_os("JIYU_HORIZON").or_else(|| std::env::var_os("RUSTCRAFT_HORIZON")).map(PathBuf::from) {
                if propio.is_file() {
                    return Some(propio);
                }
            }
            if let Some(path) = std::env::var_os("PATH") {
                for dir in std::env::split_paths(&path) {
                    let f = dir.join("horizon-launcher");
                    if f.is_file() {
                        return Some(f);
                    }
                }
            }
            let home = PathBuf::from(std::env::var_os("HOME")?);
            [
                "Applications/horizon-launcher-linux-x86_64/horizon-launcher",
                ".local/bin/horizon-launcher",
                ".local/share/horizon-launcher/bin/horizon-launcher",
            ]
            .iter()
            .map(|r| home.join(r))
            .find(|f| f.is_file())
        })
        .clone()
}

fn orden(args: &[&str]) -> Result<Command, String> {
    let bin = binario().ok_or("No encuentro horizon-launcher. Instálalo o pon su ruta en JIYU_HORIZON.")?;
    let mut c = Command::new(bin);
    c.args(args).arg("--json").stdin(Stdio::null()).kill_on_drop(true);
    Ok(c)
}

/// Ejecuta una orden y devuelve su JSON (o `Null` si no imprime nada).
pub async fn json(args: Vec<String>) -> Result<Value, String> {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    let salida = orden(&refs)?.output().await.map_err(|e| format!("No pude ejecutar Horizon: {e}"))?;
    if !salida.status.success() {
        let err = String::from_utf8_lossy(&salida.stderr);
        let err = err.trim().trim_start_matches("error: ");
        return Err(if err.is_empty() { "Horizon terminó con un error".into() } else { err.to_string() });
    }
    let texto = String::from_utf8_lossy(&salida.stdout);
    if texto.trim().is_empty() {
        return Ok(Value::Null);
    }
    serde_json::from_str(&texto).map_err(|e| format!("Horizon contestó algo que no entiendo: {e}"))
}

fn args(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn texto_ref(v: &Value) -> Option<String> {
    texto(v)
}

fn texto(v: &Value) -> Option<String> {
    v.as_str().map(str::to_string).filter(|s| !s.is_empty())
}

// ---------- Instancias ----------

fn instancia(v: &Value) -> Instancia {
    let java = &v["java"];
    let es_java = v["kind"].as_str() == Some("java") || java.is_object();
    let home = PathBuf::from(v["home"].as_str().unwrap_or_default());
    let carpeta = if home.join("minecraft").is_dir() { home.join("minecraft") } else { home };
    let icono = match &v["icon"] {
        Value::Object(o) => {
            if let Some(t) = o.get("texture").and_then(Value::as_str) {
                Icono::Textura(t.to_string())
            } else if let Some(c) = o.get("color").and_then(Value::as_str) {
                u32::from_str_radix(c.trim_start_matches('#'), 16).map(Icono::Color).unwrap_or(Icono::Defecto)
            } else {
                Icono::Defecto
            }
        }
        Value::String(s) if s.starts_with('#') => {
            u32::from_str_radix(&s[1..], 16).map(Icono::Color).unwrap_or(Icono::Defecto)
        }
        Value::String(s) => Icono::Textura(s.clone()),
        _ => Icono::Defecto,
    };
    let (version, cargador) = if es_java {
        let mc = texto(&java["minecraft"]).or_else(|| texto(&java["version"])).or_else(|| texto(&v["minecraft"]));
        // Horizon da el cargador como objeto ({"kind": "fabric", "version": "0.20.1"}); versiones viejas, como texto.
        let (tipo_c, version_c) = match &java["loader"] {
            Value::Object(o) => (o.get("kind").and_then(texto_ref), o.get("version").and_then(texto_ref)),
            otro => (texto(otro), texto(&java["loader_version"])),
        };
        let cargador = tipo_c.filter(|l| l != "vanilla").map(|l| {
            let nombre = match l.as_str() {
                "fabric" => "Fabric",
                "quilt" => "Quilt",
                "forge" => "Forge",
                "neoforge" => "NeoForge",
                otro => return (otro.to_string(), version_c.clone().unwrap_or_default()),
            };
            (nombre.to_string(), version_c.clone().unwrap_or_default())
        });
        (mc, cargador)
    } else {
        (texto(&v["version"]), None)
    };
    Instancia {
        id: v["id"].as_str().unwrap_or_default().to_string(),
        nombre: v["name"].as_str().unwrap_or("Sin nombre").to_string(),
        origen: Origen::Horizon,
        motor: if es_java { Motor::Java } else { Motor::Rust },
        sigue_ultima: !es_java && v["version"].is_null(),
        version,
        cargador,
        grupo: texto(&v["group"]),
        carpeta,
        jugado_s: v["played_secs"].as_u64().unwrap_or(0),
        ultima_ms: v["last_played_ms"].as_u64(),
        icono,
        elegida: v["selected"].as_bool().unwrap_or(false),
    }
}

pub async fn instancias() -> Result<Vec<Instancia>, String> {
    let v = json(args(&["instance", "list"])).await?;
    Ok(v.as_array().map(|a| a.iter().map(instancia).collect()).unwrap_or_default())
}

pub async fn crear_horizon(nombre: String, version: Option<String>) -> Result<(), String> {
    json(vec!["instance".into(), "create".into(), nombre.clone()]).await?;
    if let Some(v) = version {
        // La instancia nueva se busca por su nombre, que Horizon acepta igual que el id.
        json(vec!["instance".into(), "version".into(), nombre, v]).await?;
    }
    Ok(())
}

pub async fn crear_java(nombre: String, mc: String, cargador: Option<String>) -> Result<(), String> {
    let mut a = vec!["instance".into(), "create".into(), nombre, "--minecraft".into(), mc];
    if let Some(c) = cargador {
        a.push("--loader".into());
        a.push(c);
    }
    json(a).await.map(|_| ())
}

pub async fn copiar_de_prism(nombre: String, como: String) -> Result<(), String> {
    json(vec!["instance".into(), "copy-from".into(), nombre, "--as".into(), como]).await.map(|_| ())
}

pub async fn renombrar(id: String, nombre: String) -> Result<(), String> {
    json(vec!["instance".into(), "rename".into(), id, nombre]).await.map(|_| ())
}

pub async fn borrar(id: String) -> Result<(), String> {
    json(vec!["instance".into(), "delete".into(), id]).await.map(|_| ())
}

pub async fn fijar_version(id: String, version: String) -> Result<(), String> {
    json(vec!["instance".into(), "version".into(), id, version]).await.map(|_| ())
}

#[derive(Debug, Clone, Default)]
pub struct Lanzamiento {
    pub envoltorio: String,
    pub entorno: String,
    pub gamemode: bool,
    pub mangohud: bool,
    pub gpu_dedicada: bool,
    pub hay_gamemode: bool,
    pub hay_mangohud: bool,
    pub dos_gpus: bool,
    pub nvidia: bool,
}

pub async fn lanzamiento(id: String) -> Result<Lanzamiento, String> {
    let v = json(vec!["instance".into(), "launch".into(), id]).await?;
    let entorno = v["env"]
        .as_object()
        .map(|o| {
            o.iter()
                .map(|(k, v)| format!("{k}={}", v.as_str().unwrap_or_default()))
                .collect::<Vec<_>>()
                .join(" ")
        })
        .unwrap_or_default();
    Ok(Lanzamiento {
        envoltorio: v["wrapper"].as_str().unwrap_or_default().to_string(),
        entorno,
        gamemode: v["gamemode"].as_bool().unwrap_or(false),
        mangohud: v["mangohud"].as_bool().unwrap_or(false),
        gpu_dedicada: v["dedicated_gpu"].as_bool().unwrap_or(false),
        hay_gamemode: v["found"]["gamemode"].as_bool().unwrap_or(false),
        hay_mangohud: v["found"]["mangohud"].as_bool().unwrap_or(false),
        dos_gpus: v["found"]["two_gpus"].as_bool().unwrap_or(false),
        nvidia: v["found"]["nvidia"].as_bool().unwrap_or(false),
    })
}

pub async fn guardar_lanzamiento(id: String, l: Lanzamiento) -> Result<(), String> {
    let onoff = |b: bool| if b { "on".to_string() } else { "off".to_string() };
    let mut a = vec![
        "instance".into(),
        "launch".into(),
        id,
        "--wrapper".into(),
        l.envoltorio,
        "--env".into(),
        l.entorno,
    ];
    if l.hay_gamemode {
        a.extend(["--gamemode".into(), onoff(l.gamemode)]);
    }
    if l.hay_mangohud {
        a.extend(["--mangohud".into(), onoff(l.mangohud)]);
    }
    if l.dos_gpus || l.nvidia {
        a.extend(["--dedicated-gpu".into(), onoff(l.gpu_dedicada)]);
    }
    json(a).await.map(|_| ())
}

#[derive(Debug, Clone, Default)]
pub struct JavaInstancia {
    pub memoria_mib: u32,
    pub argumentos: String,
}

pub async fn java_de(id: String) -> Result<JavaInstancia, String> {
    let v = json(vec!["instance".into(), "java".into(), id]).await?;
    Ok(JavaInstancia {
        memoria_mib: v["memory"].as_u64().or_else(|| v["memory_mib"].as_u64()).unwrap_or(4096) as u32,
        argumentos: v["args"].as_str().or_else(|| v["jvm_args"].as_str()).unwrap_or_default().to_string(),
    })
}

pub async fn guardar_java(id: String, j: JavaInstancia) -> Result<(), String> {
    json(vec![
        "instance".into(),
        "java".into(),
        id,
        "--memory".into(),
        j.memoria_mib.to_string(),
        "--args".into(),
        j.argumentos,
    ])
    .await
    .map(|_| ())
}

// ---------- Versiones ----------

#[derive(Debug, Clone, Deserialize)]
pub struct VersionHorizon {
    pub tag: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub date: String,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub installed: bool,
}

pub async fn versiones_horizon() -> Result<Vec<VersionHorizon>, String> {
    let v = json(args(&["versions"])).await?;
    serde_json::from_value(v).map_err(|e| e.to_string())
}

#[derive(Debug, Clone, Deserialize)]
pub struct VersionJava {
    pub id: String,
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub released: String,
}

pub async fn versiones_java(snapshots: bool) -> Result<Vec<VersionJava>, String> {
    let mut a = args(&["instance", "versions"]);
    if snapshots {
        a.push("--snapshots".into());
    }
    let v = json(a).await?;
    serde_json::from_value(v).map_err(|e| e.to_string())
}

// ---------- Cuentas ----------

#[derive(Debug, Clone)]
pub struct Cuenta {
    pub nombre: String,
    pub uuid: String,
    pub elegida: bool,
    pub lista: bool,
    pub mensaje: String,
}

pub async fn cuentas() -> Result<Vec<Cuenta>, String> {
    let v = json(args(&["account", "list"])).await?;
    Ok(v["accounts"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| Cuenta {
            nombre: c["name"].as_str().unwrap_or("?").to_string(),
            uuid: c["uuid"].as_str().unwrap_or_default().to_string(),
            elegida: c["selected"].as_bool().unwrap_or(false),
            lista: c["authentication"]["state"].as_str() == Some("ready"),
            mensaje: c["authentication"]["message"].as_str().unwrap_or_default().to_string(),
        })
        .collect())
}

pub async fn elegir_cuenta(uuid: String) -> Result<(), String> {
    json(vec!["account".into(), "select".into(), uuid]).await.map(|_| ())
}

pub async fn cerrar_sesion(uuid: String) -> Result<(), String> {
    json(vec!["account".into(), "sign-out".into(), uuid]).await.map(|_| ())
}

/// Lo que va diciendo `account sign-in` mientras inicias sesión en el navegador.
#[derive(Debug, Clone)]
pub enum Acceso {
    Codigo { codigo: String, enlace: String },
    Linea(String),
    Hecho(Result<(), String>),
}

/// Inicia sesión con código de dispositivo, a través de la app de Horizon (ya aprobada por Mojang).
/// Va pasando cada línea; el código y el enlace se sacan de ellas.
pub fn iniciar_sesion() -> impl iced::futures::Stream<Item = Acceso> {
    use iced::futures::SinkExt;
    use tokio::io::{AsyncBufReadExt, BufReader};
    iced::stream::channel(32, |mut tx| async move {
        let Some(bin) = binario() else {
            let _ = tx.send(Acceso::Hecho(Err("No encuentro horizon-launcher.".into()))).await;
            return;
        };
        // Sin --json: queremos el texto que va imprimiendo, línea a línea.
        let hijo = Command::new(bin)
            .args(["account", "sign-in"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn();
        let mut hijo = match hijo {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Acceso::Hecho(Err(e.to_string()))).await;
                return;
            }
        };
        let mut salida = BufReader::new(hijo.stdout.take().expect("stdout")).lines();
        let mut errores = BufReader::new(hijo.stderr.take().expect("stderr")).lines();
        let mut ultimo_error = String::new();
        let mut err_abierto = true;
        loop {
            tokio::select! {
                l = salida.next_line() => match l {
                    Ok(Some(l)) => { let _ = tx.send(analiza_linea(&l)).await; }
                    _ => break,
                },
                l = errores.next_line(), if err_abierto => match l {
                    Ok(Some(l)) => {
                        ultimo_error = l.clone();
                        let _ = tx.send(analiza_linea(&l)).await;
                    }
                    _ => err_abierto = false,
                },
            }
        }
        let fin = match hijo.wait().await {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => Err(if ultimo_error.is_empty() { "No se pudo iniciar sesión".into() } else { ultimo_error }),
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(Acceso::Hecho(fin)).await;
    })
}

fn analiza_linea(l: &str) -> Acceso {
    let enlace = l.split_whitespace().find(|p| p.starts_with("http")).map(|p| p.trim_end_matches(['.', ',', ')']).to_string());
    // Un código de dispositivo de Microsoft: 8 o 9 caracteres en mayúsculas y cifras, a veces con guion.
    let codigo = l
        .split(|c: char| c.is_whitespace() || c == ':' || c == '"')
        .map(|p| p.trim_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-'))
        .find(|p| {
            let limpio: String = p.chars().filter(|c| *c != '-').collect();
            (8..=9).contains(&limpio.len())
                && limpio.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit())
                && limpio.chars().any(|c| c.is_ascii_uppercase())
        })
        .map(str::to_string);
    match codigo {
        Some(codigo) => Acceso::Codigo {
            codigo,
            enlace: enlace.unwrap_or_else(|| "https://www.microsoft.com/link".into()),
        },
        None => Acceso::Linea(l.trim().to_string()),
    }
}

// ---------- Mods ----------

#[derive(Debug, Clone)]
pub struct Encontrado {
    pub id: String,
    /// Id del proyecto en Modrinth, para pedir su logo.
    pub modrinth: Option<String>,
    /// Logo que ya trae la búsqueda de Modrinth (así no hay que pedirlo aparte).
    pub icono_url: Option<String>,
    pub titulo: String,
    pub resumen: String,
    pub autor: String,
    pub descargas: u64,
    pub soporte: String,
    pub pagina: String,
}

pub async fn buscar(texto: String, tipo: String, fuente: String, instancia: Option<String>) -> Result<Vec<Encontrado>, String> {
    let mut a = vec!["mods".into(), "search".into()];
    a.extend(texto.split_whitespace().map(str::to_string));
    let es_modrinth = fuente == "modrinth";
    a.extend(["--kind".into(), tipo, "--source".into(), fuente]);
    if let Some(i) = instancia {
        a.extend(["--instance".into(), i]);
    }
    let v = json(a).await?;
    Ok(v.as_array()
        .into_iter()
        .flatten()
        .map(|m| Encontrado {
            modrinth: if es_modrinth { m["id"].as_str().map(str::to_string) } else { None },
            icono_url: None,
            id: texto_o(&m["slug"], &m["id"]),
            titulo: m["title"].as_str().unwrap_or("?").to_string(),
            resumen: m["summary"].as_str().unwrap_or_default().to_string(),
            autor: m["author"].as_str().unwrap_or_default().to_string(),
            descargas: m["downloads"].as_u64().unwrap_or(0),
            soporte: m["support"].as_str().unwrap_or_default().to_string(),
            pagina: m["page"].as_str().unwrap_or_default().to_string(),
        })
        .collect())
}

fn texto_o(a: &Value, b: &Value) -> String {
    texto(a).or_else(|| texto(b)).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct ModHorizon {
    pub proyecto: String,
    pub titulo: String,
    pub version: String,
    pub fuente: String,
    pub dependencia: bool,
    /// Mod nativo de Horizon (.hnmod): hay que compilar el juego con él para que cargue.
    pub nativo: bool,
}

/// Lo que va pasando al compilar los mods nativos de una instancia.
#[derive(Debug, Clone)]
pub enum Compila {
    Linea(String),
    Fin(Result<(), String>),
}

/// Compila el juego con los mods nativos de la instancia (`native-build`). La primera vez Horizon baja su
/// Rust y tarda 20–30 minutos; luego es incremental. Va contando la última línea que escribe.
pub fn compilar(instancia: String) -> impl iced::futures::Stream<Item = Compila> {
    use iced::futures::SinkExt;
    use tokio::io::{AsyncBufReadExt, BufReader};
    iced::stream::channel(32, move |mut tx| async move {
        let Some(bin) = binario() else {
            let _ = tx.send(Compila::Fin(Err("No encuentro horizon-launcher".into()))).await;
            return;
        };
        let hijo = tokio::process::Command::new(bin)
            .args(["native-build", "--instance", &instancia, "--trust-native-code"])
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(false)
            .spawn();
        let mut hijo = match hijo {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Compila::Fin(Err(format!("No pude empezar a compilar: {e}")))).await;
                return;
            }
        };
        // Salida y errores por el mismo canal: las dos cuentan el progreso.
        let (txl, mut rxl) = tokio::sync::mpsc::unbounded_channel::<String>();
        if let Some(o) = hijo.stdout.take() {
            let t = txl.clone();
            tokio::spawn(async move {
                let mut l = BufReader::new(o).lines();
                while let Ok(Some(x)) = l.next_line().await {
                    let _ = t.send(x);
                }
            });
        }
        if let Some(e) = hijo.stderr.take() {
            let t = txl.clone();
            tokio::spawn(async move {
                let mut l = BufReader::new(e).lines();
                while let Ok(Some(x)) = l.next_line().await {
                    let _ = t.send(x);
                }
            });
        }
        drop(txl);
        let mut ultimas: Vec<String> = Vec::new();
        while let Some(l) = rxl.recv().await {
            let l = l.trim().to_string();
            if l.is_empty() {
                continue;
            }
            let _ = tx.send(Compila::Linea(l.chars().take(160).collect())).await;
            ultimas.push(l);
            if ultimas.len() > 6 {
                ultimas.remove(0);
            }
        }
        let fin = match hijo.wait().await {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => Err(ultimas.last().cloned().unwrap_or_else(|| "La compilación falló".into())),
            Err(e) => Err(e.to_string()),
        };
        let _ = tx.send(Compila::Fin(fin)).await;
    })
}

pub async fn mods_horizon(instancia: String) -> Result<Vec<ModHorizon>, String> {
    let v = json(vec!["mods".into(), "list".into(), "--instance".into(), instancia]).await?;
    Ok(v.as_array()
        .into_iter()
        .flatten()
        .map(|m| ModHorizon {
            proyecto: texto_o(&m["project"], &m["slug"]),
            titulo: m["title"].as_str().unwrap_or("?").to_string(),
            version: m["version"].as_str().unwrap_or_default().to_string(),
            fuente: m["source"].as_str().unwrap_or_default().to_string(),
            dependencia: m["dependency"].as_bool().unwrap_or(false),
            nativo: m["file"].as_str().is_some_and(|f| f.ends_with(".hnmod")),
        })
        .collect())
}

pub async fn quitar_mod(proyecto: String, instancia: String) -> Result<(), String> {
    json(vec!["mods".into(), "remove".into(), proyecto, "--instance".into(), instancia]).await.map(|_| ())
}

#[derive(Debug, Clone)]
pub struct Mundo {
    pub nombre: String,
    /// El icon.png del mundo (la foto que hace el juego), ya decodificado.
    pub icono: Option<iced::widget::image::Handle>,
    pub carpeta: PathBuf,
    pub modo: String,
    pub version: String,
    pub ultima_ms: Option<u64>,
    pub tamano: u64,
}

pub async fn mundos(instancia: String) -> Result<Vec<Mundo>, String> {
    let v = json(vec!["worlds".into(), "list".into(), "--instance".into(), instancia]).await?;
    let lista: Vec<Mundo> = v
        .as_array()
        .into_iter()
        .flatten()
        .map(|m| Mundo {
            icono: None,
            carpeta: PathBuf::from(m["path"].as_str().unwrap_or_default()),
            nombre: m["name"].as_str().unwrap_or("?").to_string(),
            modo: m["mode"].as_str().unwrap_or_default().to_string(),
            version: m["version"].as_str().unwrap_or_default().to_string(),
            ultima_ms: m["last_played_ms"].as_u64(),
            tamano: m["size"].as_u64().unwrap_or(0),
        })
        .collect();
    Ok(con_iconos(lista).await)
}

/// Pone a cada mundo su icon.png (64×64), leído en un hilo aparte.
pub async fn con_iconos(mut lista: Vec<Mundo>) -> Vec<Mundo> {
    let rutas: Vec<PathBuf> = lista.iter().map(|m| m.carpeta.join("icon.png")).filter(|r| r.is_file()).collect();
    let iconos = crate::miniatura::varias(rutas, 64, 64).await;
    for m in &mut lista {
        let r = m.carpeta.join("icon.png");
        m.icono = iconos.iter().find(|(p, _)| *p == r).map(|(_, h)| h.clone());
    }
    lista
}

// ---------- Ajustes de Horizon ----------

#[derive(Debug, Clone, Default)]
pub struct AjustesHorizon {
    pub perfil_nativo: String,
    pub cache_gib: u64,
    pub argumentos: String,
    pub instalar_nuevas: bool,
    pub version_elegida: String,
    pub proxy: String,
    pub podar: bool,
    pub guardar: u64,
}

pub async fn ajustes_horizon() -> Result<AjustesHorizon, String> {
    let v = json(args(&["settings"])).await?;
    Ok(AjustesHorizon {
        perfil_nativo: v["native_profile"].as_str().unwrap_or("performance").to_string(),
        cache_gib: v["native_cache_gib"].as_u64().unwrap_or(30),
        argumentos: v["game_args"].as_str().unwrap_or_default().to_string(),
        instalar_nuevas: v["install_new_versions"].as_bool().unwrap_or(true),
        version_elegida: v["selected_version"].as_str().unwrap_or_default().to_string(),
        proxy: v["proxy"].as_str().unwrap_or_default().to_string(),
        podar: v["prune_old_versions"].as_bool().unwrap_or(true),
        guardar: v["keep_versions"].as_u64().unwrap_or(2),
    })
}

pub async fn ajuste_horizon(clave: String, valor: String) -> Result<(), String> {
    json(vec!["settings".into(), clave, valor]).await.map(|_| ())
}

pub async fn argumentos_juego(valor: String) -> Result<(), String> {
    json(vec!["game-args".into(), valor]).await.map(|_| ())
}

pub async fn tamano_cache() -> Result<(u64, u64), String> {
    let v = json(args(&["native-cache"])).await?;
    let usado = v["size"].as_u64().or_else(|| v["bytes"].as_u64()).unwrap_or(0);
    let limite = v["limit"].as_u64().or_else(|| v["limit_bytes"].as_u64()).unwrap_or(0);
    Ok((usado, limite))
}

pub async fn instalar_version(tag: String) -> Result<(), String> {
    json(vec!["install".into(), tag]).await.map(|_| ())
}

/// La versión más nueva que no está instalada, si la hay (el chip de «Horizon tiene versión nueva»).
pub fn nueva(versiones: &[VersionHorizon]) -> Option<&VersionHorizon> {
    let instalada = versiones.iter().position(|v| v.installed)?;
    versiones[..instalada].iter().find(|v| !v.installed)
}

// ---------- Jugar ----------

/// Lo que va pasando desde que pulsas Jugar.
#[derive(Debug, Clone)]
pub enum Partida {
    /// La última línea que ha escrito Horizon (o Prism): «Descargando…», «Preparando…».
    Linea(String),
    /// El juego ya está corriendo (hay un proceso de Java o de Horizon debajo). Lleva el pid a vigilar.
    EnJuego(u32),
    /// Terminó: bien, o con el error que escribió.
    Fin(Result<(), String>),
}

fn orden_jugar(inst: &Instancia, extra: &[String]) -> Result<std::process::Command, String> {
    Ok(match inst.origen {
        Origen::Prism => crate::prism::orden_jugar(&inst.id).ok_or("No encuentro Prism Launcher")?,
        Origen::Horizon => {
            let bin = binario().ok_or("No encuentro horizon-launcher")?;
            let mut c = std::process::Command::new(bin);
            c.args(["play", "--instance", &inst.id]);
            for a in extra {
                c.args(["--arg", a]);
            }
            c
        }
    })
}

/// Procesos que cuelgan de `pid` (hijos, nietos…), con su nombre.
fn descendientes(pid: u32) -> Vec<(u32, String)> {
    let mut padres: Vec<(u32, u32, String)> = Vec::new();
    for e in std::fs::read_dir("/proc").into_iter().flatten().flatten() {
        let Some(n) = e.file_name().to_str().and_then(|s| s.parse::<u32>().ok()) else { continue };
        let Ok(stat) = std::fs::read_to_string(e.path().join("stat")) else { continue };
        // pid (nombre) estado ppid …
        let (Some(a), Some(b)) = (stat.find('('), stat.rfind(')')) else { continue };
        let nombre = stat[a + 1..b].to_string();
        let ppid = stat[b + 2..].split_whitespace().nth(1).and_then(|x| x.parse().ok()).unwrap_or(0);
        padres.push((n, ppid, nombre));
    }
    let mut res = Vec::new();
    let mut cola = vec![pid];
    while let Some(p) = cola.pop() {
        for (n, pp, nombre) in &padres {
            if *pp == p {
                res.push((*n, nombre.clone()));
                cola.push(*n);
            }
        }
    }
    res
}

fn es_juego(nombre: &str) -> bool {
    let n = nombre.to_lowercase();
    n.contains("java") || n.contains("horizon-client") || n.contains("minecraft")
}

/// Arranca la partida. La salida va a un archivo (no a una tubería), así el launcher puede cerrarse
/// sin romper nada; se va leyendo para enseñar qué hace Horizon mientras descarga o prepara.
pub fn partida(inst: Instancia, extra: Vec<String>) -> impl iced::futures::Stream<Item = Partida> {
    use iced::futures::SinkExt;
    iced::stream::channel(16, move |mut tx| async move {
        let dir = crate::ajustes::carpeta_datos().join("partidas");
        let _ = std::fs::create_dir_all(&dir);
        let nombre: String = inst.id.chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '_' }).collect();
        let ruta = dir.join(format!("{nombre}.log"));
        let preparar = || -> Result<std::process::Child, String> {
            let f = std::fs::File::create(&ruta).map_err(|e| e.to_string())?;
            let f2 = f.try_clone().map_err(|e| e.to_string())?;
            let mut c = orden_jugar(&inst, &extra)?;
            c.stdin(Stdio::null()).stdout(f).stderr(f2);
            #[cfg(unix)]
            {
                use std::os::unix::process::CommandExt;
                // Su propio grupo de procesos: si el launcher se cierra, el juego sigue.
                c.process_group(0);
            }
            c.spawn().map_err(|e| format!("No pude arrancar el juego: {e}"))
        };
        let mut hijo = match preparar() {
            Ok(h) => h,
            Err(e) => {
                let _ = tx.send(Partida::Fin(Err(e))).await;
                return;
            }
        };
        let pid = hijo.id();
        let mut leido = 0usize;
        let mut en_juego = false;
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            if let Ok(t) = std::fs::read_to_string(&ruta) {
                if t.len() > leido {
                    if let Some(l) = t[leido..].lines().rev().map(str::trim).find(|l| !l.is_empty()) {
                        let _ = tx.send(Partida::Linea(l.chars().take(160).collect())).await;
                    }
                    leido = t.len();
                }
            }
            if !en_juego && descendientes(pid).iter().any(|(_, n)| es_juego(n)) {
                en_juego = true;
                let _ = tx.send(Partida::EnJuego(pid)).await;
            }
            match hijo.try_wait() {
                Ok(Some(estado)) => {
                    let fin = if estado.success() {
                        Ok(())
                    } else {
                        let t = std::fs::read_to_string(&ruta).unwrap_or_default();
                        let ultimas: Vec<&str> = t.lines().map(str::trim).filter(|l| !l.is_empty()).collect();
                        let desde = ultimas.len().saturating_sub(3);
                        let msg = ultimas[desde..].join(" · ");
                        Err(if msg.is_empty() { "El juego terminó con un error".into() } else { msg.trim_start_matches("error: ").to_string() })
                    };
                    let _ = tx.send(Partida::Fin(fin)).await;
                    break;
                }
                Ok(None) => {}
                Err(e) => {
                    let _ = tx.send(Partida::Fin(Err(e.to_string()))).await;
                    break;
                }
            }
        }
    })
}

/// Cambia el icono de una instancia de Horizon: un bloque o ítem (`grass_block`), un color (`#3fa7d6`) o `none`.
pub async fn icono(id: String, icono: String) -> Result<(), String> {
    json(vec!["instance".into(), "icon".into(), id, icono]).await.map(|_| ())
}

/// Mete una instancia de Horizon en una carpeta («A/B») o la saca (None).
pub async fn grupo(id: String, grupo: Option<String>) -> Result<(), String> {
    let mut a = vec!["instance".into(), "group".into(), id];
    if let Some(g) = grupo {
        a.push(g);
    }
    json(a).await.map(|_| ())
}

/// Importa un archivo (.mrpack, zip de CurseForge o de una instancia) como Horizon decida.
pub async fn importar(ruta: String) -> Result<(), String> {
    json(vec!["instance".into(), "import".into(), ruta]).await.map(|_| ())
}

/// Copia una instancia de Horizon (mundos, opciones y paquetes) en una nueva.
pub async fn duplicar(id: String, nombre: String) -> Result<(), String> {
    json(vec!["instance".into(), "create".into(), nombre, "--copy".into(), id]).await.map(|_| ())
}

/// Acceso directo en el escritorio (y en el menú de aplicaciones) que juega la instancia.
pub async fn acceso_directo(id: String) -> Result<(), String> {
    json(vec!["instance".into(), "shortcut".into(), id]).await.map(|_| ())
}
