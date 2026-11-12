//! Estado del launcher, mensajes y qué hace cada uno. Las pantallas están en `vistas/`.

use crate::ajustes::{Ajustes, Favorito, Orden, Perfil, Recolector, Sistema, Tamano, Vista};
use crate::datos::{Instancia, Motor, Origen};
use crate::iconos::{self, Glifo};
use crate::motor::{self, Acceso, AjustesHorizon, Cuenta, Encontrado, JavaInstancia, Lanzamiento, ModHorizon, Mundo};
use crate::skins::{self, Perfil as PerfilCuenta, Skin};
use crate::tema::{Paleta, Tema};
use iced::widget::image::Handle;
use iced::{Point, Size, Subscription, Task};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dlg {
    Nueva,
    Editar,
    Ajustes,
    Cuentas,
    Armario,
    Explorar,
    Icono,
    Servidores,
    Archivo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fuente {
    Horizon,
    Java,
    Prism,
    Modpack,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagE {
    Version,
    Mods,
    Recursos,
    Shaders,
    Mundos,
    Capturas,
    Ajustes,
    Skin,
    Registro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PagA {
    Rendimiento,
    Apariencia,
    Horizon,
    Red,
    Avanzado,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flag {
    Insignia,
    Filo,
    Hexagono,
    Etiqueta,
    Esquina,
    Texturas,
    VerPrism,
    Cerrar,
    GestionarJava,
    PaginasGrandes,
    Gamemode,
    Mangohud,
    Gpu,
    HilosNvidia,
    HiloServidor,
    Actualizaciones,
    Grafica,
    ServidoresSiempre,
    Reabrir,
}

/// Un archivo de mods/, resourcepacks/ o shaderpacks/.
#[derive(Debug, Clone)]
pub struct Archivo {
    pub nombre: String,
    pub ruta: PathBuf,
    pub activo: bool,
    pub tamano: u64,
    /// Proyecto de Modrinth y su nombre, si se ha reconocido el archivo.
    pub proyecto: Option<(String, String)>,
}

/// Lo que se sabe de una partida en marcha.
#[derive(Debug, Clone)]
pub struct EnMarcha {
    pub linea: String,
    pub en_juego: bool,
}

/// Atajos de teclado de la ventana principal.
#[derive(Debug, Clone, Copy)]
pub enum Atajo {
    Jugar,
    Borrar,
    Buscar,
    Nueva,
    Recargar,
}

/// Menú contextual abierto: dónde y de qué.
#[derive(Debug, Clone, PartialEq)]
pub enum QueMenu {
    Instancia(String),
    Carpeta(String),
    Fondo,
}

#[derive(Debug, Clone)]
pub struct Menu {
    pub pos: Point,
    pub que: QueMenu,
    /// Lista de carpetas desplegada para «Mover a…».
    pub mover: bool,
}

/// Pedir un nombre (carpeta nueva o renombrar).
#[derive(Debug, Clone)]
pub enum ParaNombre {
    NuevaCarpeta(String),
    RenombrarCarpeta(String),
}

#[derive(Debug, Clone)]
pub struct PideNombre {
    pub para: ParaNombre,
    pub valor: String,
}

/// Selector de icono de una instancia.
#[derive(Default)]
pub struct Selector {
    pub clave: String,
    /// Elegido para la instancia que se está creando (vuelve a «Nueva instancia»).
    pub para_nueva: bool,
    pub filtro: String,
    pub texturas: Vec<String>,
    pub previas: Vec<(String, Handle)>,
    pub propios: Vec<(String, Handle)>,
    pub ocupado: bool,
}

/// Opción de los desplegables de instancia.
#[derive(Debug, Clone, PartialEq)]
pub struct Opcion {
    pub clave: String,
    pub nombre: String,
}
impl std::fmt::Display for Opcion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.nombre)
    }
}

#[derive(Default)]
pub struct Nueva {
    pub fuente: Option<Fuente>,
    pub nombre: String,
    pub version_h: Option<String>,
    pub version_j: Option<String>,
    pub snapshots: bool,
    pub cargador: Option<&'static str>,
    pub java: Vec<motor::VersionJava>,
    pub prism: Option<String>,
    pub ocupado: bool,
    pub error: Option<String>,
    // Lo que se elige ya al crear
    pub icono: Option<String>,
    pub carpeta: String,
    pub memoria: Option<u32>,
    pub skin: Option<String>,
    // Paquete de mods
    pub texto_pack: String,
    pub packs: Vec<Encontrado>,
    pub pack: Option<usize>,
    pub archivo: Option<String>,
    /// Ids que había antes de crear, para saber cuál es la nueva.
    pub antes: Vec<String>,
}

#[derive(Default)]
pub struct Editar {
    pub clave: String,
    pub pagina: Option<PagE>,
    pub nombre: String,
    pub mods: Vec<Archivo>,
    pub mods_h: Vec<ModHorizon>,
    pub recursos: Vec<Archivo>,
    pub shaders: Vec<Archivo>,
    pub mundos: Vec<Mundo>,
    pub capturas: Vec<(PathBuf, Handle)>,
    pub sel: Option<usize>,
    pub registro: String,
    pub java: Option<JavaInstancia>,
    pub lanz: Option<Lanzamiento>,
    pub cargador_nuevo: Option<String>,
}

#[derive(Default)]
pub struct Acceder {
    pub activo: bool,
    pub codigo: Option<(String, String)>,
    pub lineas: Vec<String>,
}

#[derive(Default)]
pub struct Armario {
    pub guinada: f32,
    pub cabeceo: f32,
    pub girando: Option<Point>,
    pub figura: Option<Handle>,
    pub skins: Vec<Skin>,
    pub sel: Option<usize>,
    pub espalda: bool,
    pub perfil: Option<PerfilCuenta>,
    pub cargando: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoPedido {
    Esperando,
    Bajando,
    Hecho(String),
    Fallo(String),
}

/// Una descarga de la cola: qué, de qué tipo, de dónde y para qué instancia.
#[derive(Debug, Clone)]
pub struct Pedido {
    pub m: Encontrado,
    pub tipo: &'static str,
    pub fuente: &'static str,
    pub inst: String,
    pub estado: EstadoPedido,
}

#[derive(Default)]
pub struct Explorar {
    pub texto: String,
    pub fuente: &'static str,
    pub tipo: &'static str,
    pub instancia: Option<Opcion>,
    pub resultados: Vec<Encontrado>,
    pub sel: Option<usize>,
    pub buscando: bool,
    pub instalando: bool,
    pub error: Option<String>,
    /// Cada búsqueda lleva un número; se descartan las respuestas viejas.
    pub turno: u64,
    pub cache: HashMap<String, Vec<Encontrado>>,
    /// Marcados para instalar de una vez, cada uno con su tipo (se conservan al cambiar de búsqueda).
    pub marcados: Vec<(Encontrado, &'static str)>,
    /// La cola de descargas: sigue bajando aunque cambies de búsqueda, de tipo o de instancia.
    pub cola: Vec<Pedido>,
    pub ver_cola: bool,
    /// Proyectos que ya tiene la instancia elegida (ids de Modrinth y slugs), para marcar «Instalado».
    pub instalados: std::collections::HashSet<String>,
    /// Filtro de cargador (None = cualquiera) y si filtrar por la versión de la instancia.
    pub cargador: Option<&'static str>,
    pub por_version: bool,
    /// Si Modrinth tiene más resultados que los mostrados, y si se están pidiendo.
    pub hay_mas: bool,
    pub pidiendo_mas: bool,
    /// Cuánto funciona cada mod en Horizon (slug → «Full», «Partly»…), llega aparte de la búsqueda.
    pub soporte: HashMap<String, String>,
    /// Si no hubo nada para la versión de la instancia y se enseñan los de cualquiera: esa versión.
    pub relajado: Option<String>,
}

pub struct App {
    pub aj: Ajustes,
    pub sis: Sistema,
    pub p: Paleta,
    pub horizon: Vec<Instancia>,
    pub prism: Vec<Instancia>,
    pub error_horizon: Option<String>,
    pub cargando: bool,
    pub sel: Option<String>,
    pub filtro: String,
    pub iconos: HashMap<String, Handle>,
    cache: iconos::Cache,
    pub insignia_rust: Handle,
    pub insignia_java: Handle,
    pub hexagono: Handle,
    pub flecha_abierta: Handle,
    pub flecha_cerrada: Handle,
    pub mascota: Handle,
    pub cuentas: Vec<Cuenta>,
    pub versiones: Vec<motor::VersionHorizon>,
    pub ah: Option<AjustesHorizon>,
    pub ah_args: String,
    pub cache_nativa: Option<(u64, u64)>,
    pub dlg: Option<Dlg>,
    pub pag_a: PagA,
    pub nueva: Nueva,
    pub editar: Editar,
    pub acceder: Acceder,
    pub armario: Armario,
    pub explorar: Explorar,
    pub aviso: Option<(String, bool)>,
    pub memoria_mb: u64,
    pub borrar: Option<String>,
    ultimo_clic: Option<(String, Instant)>,
    /// Pantalla que abrir al arrancar (RUSTCRAFT_ABRIR), para probar y hacer capturas.
    abrir: Option<String>,
    /// Logos de mods por id de proyecto de Modrinth.
    pub logos: HashMap<String, Handle>,
    pub partidas: HashMap<String, EnMarcha>,
    /// Compilaciones de mods nativos en marcha (clave → última línea) y la que pide confirmación.
    pub compilando: HashMap<String, String>,
    pub confirmar_compilar: Option<String>,
    /// El resultado de la última compilación de cada instancia (para enseñarlo en su página de mods).
    pub compilado: HashMap<String, Result<(), String>>,
    pub selector: Selector,
    pub carpeta_actual: String,
    pub arrastre: Option<String>,
    pub arrastrando: bool,
    pub destino: Option<String>,
    pub menu: Option<Menu>,
    pub ventana: Size,
    pub borrar_texto: String,
    pub pide_nombre: Option<PideNombre>,
    pub cara: Option<Handle>,
    pub carpeta_ico: Handle,
    /// El logo de Jiyu para la barra.
    pub logo: Handle,
    /// Instancia bajo el ratón (para resaltarla).
    pub encima: Option<String>,
    /// El icono por defecto (tierra), para enseñarlo en «Nueva instancia».
    pub icono_defecto: Option<Handle>,
    pub estrella_llena: Handle,
    pub estrella_vacia: Handle,
    /// Iconos de la barra de arriba (y el archivador de abajo).
    pub barra: HashMap<&'static str, Handle>,
    /// Icono de carpeta de cada color (0 = el del tema).
    pub carpetas_ico: HashMap<u32, Handle>,
    /// Las mismas, a tamaño de fila (16–18 px en pantalla).
    pub carpetas_ico_p: HashMap<u32, Handle>,
    /// Modo Big Picture (pantalla completa, para sofá y mando) y qué tarjeta está elegida (estante, posición).
    pub grande: bool,
    pub ultimo_bp: Option<Instant>,
    /// El primer estante que se ve y, en cada estante, la primera tarjeta que se ve.
    pub bp_fila0: usize,
    pub bp_col0: HashMap<usize, usize>,
    pub bp: (usize, usize),
    pub gatos: HashMap<String, Handle>,
    /// Instancia sobre la que sueltas otra (para ordenarlas a mano).
    pub destino_inst: Option<String>,
    pub sv_estado: HashMap<String, Result<crate::servidores::Estado, String>>,
    pub sv_nombre: String,
    pub sv_dir: String,
    pub sincronizadas: std::collections::HashSet<String>,
    clic_mod: Option<(usize, Instant)>,
    pub proxy_texto: String,
}

#[derive(Debug, Clone)]
pub enum Msg {
    // Carga
    Recargar,
    Instancias(Result<Vec<Instancia>, String>),
    Prism(Vec<Instancia>),
    Cuentas(Result<Vec<Cuenta>, String>),
    Versiones(Result<Vec<motor::VersionHorizon>, String>),
    Tick,
    // Principal
    Elegir(String),
    Filtro(String),
    Grupo(String),
    Jugar(String),
    Arrancar(String),
    Abrir(Dlg),
    AbrirEditar(String, PagE),
    Cerrar,
    Carpeta(PathBuf),
    Url(String),
    Hecho(Result<String, String>),
    CopiarAHorizon(String),
    PedirBorrar(String),
    Borrar(String),
    CancelarBorrar,
    ActualizarHorizon(String),
    // Nueva instancia
    NFuente(Fuente),
    NNombre(String),
    NVersionH(Option<String>),
    NVersionJ(String),
    NSnapshots(bool),
    NCargador(Option<&'static str>),
    NJava(Result<Vec<motor::VersionJava>, String>),
    NPrism(String),
    NCrear,
    NCreada(Result<(), String>),
    // Editar instancia
    EPagina(PagE),
    ESel(usize),
    EActivar(usize, bool),
    EQuitar,
    EQuitarH(String),
    EModsH(Result<Vec<ModHorizon>, String>),
    EMundos(Result<Vec<Mundo>, String>),
    EJava(Result<JavaInstancia, String>),
    ELanz(Result<Lanzamiento, String>),
    ENombre(String),
    ERenombrar,
    EVersion(String),
    ECargador(String),
    EMem(u32),
    EArgs(String),
    EEnv(String),
    EEnvoltorio(String),
    EGamemode(bool),
    EMango(bool),
    EGpu(bool),
    EGuardar,
    ERegistro,
    // Ajustes
    APagina(PagA),
    ATema(Tema),
    AFlag(Flag, bool),
    ATamano(Tamano),
    APerfil(Perfil),
    AMem(u32),
    ARec(Recolector),
    AAplicarJava,
    AHorizon(Result<AjustesHorizon, String>),
    ACache(Result<(u64, u64), String>),
    AHPerfil(&'static str),
    AHCache(u64),
    AHNuevas(bool),
    AHArgs(String),
    AHGuardarArgs,
    // Cuentas
    CElegir(String),
    CSalir(String),
    CEntrar,
    CAcceso(Acceso),
    CCopiar(String),
    // Armario
    SSel(usize),
    SEspalda(bool),
    SDelgado(bool),
    SRecargar,
    SPerfil(Result<PerfilCuenta, String>),
    SBajar,
    SBajada(Result<PathBuf, String>),
    SPoner,
    SCapa(Option<String>),
    SHecho(Result<String, String>),
    // Explorar
    XTexto(String),
    XFuente(&'static str),
    XTipo(&'static str),
    XInstancia(Opcion),
    XBuscar,
    XResultados(Result<Vec<Encontrado>, String>),
    XSel(usize),
    XPausa(u64),
    XTurno(u64, Result<Vec<Encontrado>, String>),
    XSoporte(u64, Result<Vec<Encontrado>, String>),
    XMas,
    XDesplazado(f32),
    XMasR(u64, Result<Vec<Encontrado>, String>),
    Logos(Vec<(String, Handle)>),
    EIdentificados(HashMap<PathBuf, (String, String)>),
    ECapturas(Vec<(PathBuf, Handle)>),
    Partida(String, motor::Partida),
    PrismIconos(Result<usize, String>),
    Gatos(Vec<(String, Handle)>),
    Orden(Orden),
    Archivar(String, bool),
    ArchivarCarpeta(String, bool),
    ColorCarpeta(String, u32),
    Atajo(Atajo),
    Duplicar(String),
    AccesoDirecto(String),
    // Ajustes nuevos
    AMascota(String),
    AProxy(String),
    AGuardarProxy,
    AHPodar(bool),
    AHGuardarN(u64),
    VaciarCache,
    // Servidores
    SvNombre(String),
    SvDir(String),
    SvAnadir,
    SvQuitar(usize),
    SvRefrescar,
    SvEstado(String, Result<crate::servidores::Estado, String>),
    SvAplicar,
    Cara(Option<Handle>),
    // Vistas, carpetas, arrastre y menús
    Vista(Vista),
    EntrarCarpeta(String),
    Pulsar(String),
    Hover(Option<String>),
    Destino(Option<String>),
    Soltar,
    Mover(String, String),
    AbrirMenu(QueMenu, Point),
    MoverDesplegar,
    CerrarMenu,
    Ventana(Size),
    BorrarTexto(String),
    PedirNombre(ParaNombre),
    NombreTexto(String),
    NombreAceptar,
    BorrarCarpeta(String),
    NuevaEn(String),
    // Nueva instancia: lo que se elige al crear
    NIcono,
    NCarpeta(String),
    NMemoria(u32),
    NSkin(String),
    NPackTexto(String),
    NPackBuscar,
    NPacks(Result<Vec<Encontrado>, String>),
    NPack(usize),
    NPackArchivo,
    NPackArchivoElegido(Option<String>),
    NNuevas(Result<Vec<Instancia>, String>),
    // Explorar
    XMarcar(String),
    XFavorito(usize),
    XCargador(Option<&'static str>),
    XPorVersion(bool),
    XInstalarVarios,
    XInstalar(usize),
    XSiguiente,
    XHecho(usize, Result<String, String>),
    XVerCola,
    XLimpiarCola,
    XQuitarPedido(usize),
    XDesmarcarTodo,
    XInstalados(String, std::collections::HashSet<String>),
    ExplorarPara(String, &'static str),
    // Big Picture
    BPAlternar,
    BPMover(i32, i32),
    BPElegir(usize, usize),
    BPEditar,
    BPClic(usize, usize),
    BPRueda(f32, f32),
    // Mods nativos de Horizon
    PedirCompilar(Option<String>),
    Compilar(String),
    Compilando(String, motor::Compila),
    // Skin por instancia y armario 3D
    ESkin(String),
    SImportar,
    SImportada(Result<Option<PathBuf>, String>),
    SGirar(bool),
    SMover(Point),
    // Selector de icono
    AbrirIcono(String),
    IFiltro(String),
    IElegir(String),
    IHecho(Result<(), String>),
}

fn insignias(p: &Paleta) -> (Handle, Handle, Handle) {
    (
        iconos::insignia(Glifo::Engranaje, p.rust, p.on_badge, p.base),
        iconos::insignia(Glifo::Taza, p.java, p.on_badge, p.base),
        iconos::hexagono(64, p.rust, 0.26),
    )
}

impl App {
    pub fn nuevo() -> (Self, Task<Msg>) {
        let mut aj = Ajustes::carga();
        if let Ok(t) = std::env::var("RUSTCRAFT_TEMA") {
            if let Some(tema) = Tema::TODOS.iter().find(|x| x.nombre().to_lowercase().replace(' ', "-") == t) {
                aj.tema = *tema;
            }
        }
        let p = aj.tema.paleta();
        let (insignia_rust, insignia_java, hexagono) = insignias(&p);
        let lado_carpeta = (aj.tamano.px() * 2.0) as u32;
        let app = Self {
            sel: aj.seleccion.clone(),
            sis: Sistema::detecta(),
            p,
            aj,
            horizon: Vec::new(),
            prism: Vec::new(),
            error_horizon: None,
            cargando: true,
            filtro: String::new(),
            iconos: HashMap::new(),
            cache: iconos::Cache::default(),
            insignia_rust,
            insignia_java,
            hexagono,
            flecha_abierta: iconos::flecha(true, p.dim),
            flecha_cerrada: iconos::flecha(false, p.dim),
            mascota: Handle::from_bytes(include_bytes!("../assets/mascota-dormida.png").as_slice()),
            cuentas: Vec::new(),
            versiones: Vec::new(),
            ah: None,
            ah_args: String::new(),
            cache_nativa: None,
            dlg: None,
            pag_a: PagA::Rendimiento,
            nueva: Nueva::default(),
            editar: Editar::default(),
            acceder: Acceder::default(),
            armario: Armario::default(),
            explorar: Explorar { fuente: "modrinth", tipo: "mod", por_version: true, ..Default::default() },
            aviso: None,
            memoria_mb: crate::ajustes::memoria_propia_mb(),
            borrar: None,
            ultimo_clic: None,
            abrir: std::env::var("RUSTCRAFT_ABRIR").ok(),
            logos: HashMap::new(),
            partidas: HashMap::new(),
            compilando: HashMap::new(),
            confirmar_compilar: None,
            compilado: HashMap::new(),
            selector: Selector::default(),
            carpeta_actual: String::new(),
            arrastre: None,
            arrastrando: false,
            destino: None,
            menu: None,
            ventana: Size::new(1180.0, 760.0),
            borrar_texto: String::new(),
            pide_nombre: None,
            cara: None,
            carpeta_ico: iconos::carpeta(p.java, 32),
            logo: iconos::logo_jiyu(48),
            encima: None,
            icono_defecto: None,
            estrella_llena: iconos::estrella(true, p.rust),
            estrella_vacia: iconos::estrella(false, p.dim),
            barra: iconos_barra(&p),
            carpetas_ico: iconos_carpeta(&p, lado_carpeta),
            carpetas_ico_p: iconos_carpeta(&p, 32),
            grande: std::env::var("RUSTCRAFT_ABRIR").is_ok_and(|a| a == "grande"),
            bp: (0, 0),
            ultimo_bp: None,
            bp_fila0: 0,
            bp_col0: HashMap::new(),
            gatos: HashMap::new(),
            destino_inst: None,
            sv_estado: HashMap::new(),
            sv_nombre: String::new(),
            sv_dir: String::new(),
            sincronizadas: Default::default(),
            clic_mod: None,
            proxy_texto: String::new(),
        };
        let mut tareas = vec![app.carga()];
        tareas.push(Task::perform(
            async {
                let nombres = crate::prism_iconos::descargar_gatos().await;
                tokio::task::spawn_blocking(move || {
                    nombres
                        .into_iter()
                        .filter_map(|n| {
                            let f = crate::prism_iconos::gato(&n)?;
                            Some((n, crate::miniatura::reduce(&f, 400, 400, true)?))
                        })
                        .collect::<Vec<_>>()
                })
                .await
                .unwrap_or_default()
            },
            Msg::Gatos,
        ));
        if !crate::prism_iconos::descargados() {
            tareas.push(Task::perform(crate::prism_iconos::descargar(), Msg::PrismIconos));
        }
        if app.aj.buscar_actualizaciones {
            tareas.push(Task::perform(motor::versiones_horizon(), Msg::Versiones));
        }
        (app, Task::batch(tareas))
    }

    fn carga(&self) -> Task<Msg> {
        let mut t = vec![
            Task::perform(motor::instancias(), Msg::Instancias),
            Task::perform(motor::cuentas(), Msg::Cuentas),
        ];
        if self.aj.ver_prism {
            t.push(Task::perform(async { tokio::task::spawn_blocking(crate::prism::instancias).await.unwrap_or_default() }, Msg::Prism));
        }
        Task::batch(t)
    }

    pub fn titulo(&self) -> String {
        match self.dlg {
            Some(Dlg::Nueva) => "Nueva instancia — Jiyu Launcher".into(),
            Some(Dlg::Editar) => format!("{} — Jiyu Launcher", self.instancia(&self.editar.clave).map(|i| i.nombre.as_str()).unwrap_or("Editar")),
            Some(Dlg::Ajustes) => "Ajustes — Jiyu Launcher".into(),
            Some(Dlg::Cuentas) => "Cuentas — Jiyu Launcher".into(),
            Some(Dlg::Armario) => "Armario — Jiyu Launcher".into(),
            Some(Dlg::Explorar) => "Explorar mods — Jiyu Launcher".into(),
            Some(Dlg::Icono) => "Icono — Jiyu Launcher".into(),
            Some(Dlg::Servidores) => "Servidores — Jiyu Launcher".into(),
            Some(Dlg::Archivo) => "Archivadas — Jiyu Launcher".into(),
            None => "Jiyu Launcher".into(),
        }
    }

    pub fn theme(&self) -> iced::Theme {
        self.aj.tema.iced()
    }

    pub fn subscription(&self) -> Subscription<Msg> {
        // Un latido cada 5 s solo para el contador de memoria de la barra de estado. Nada más corre en reposo.
        // En Big Picture no: repintaría la pantalla entera cada 5 s para nada.
        let mut s = if self.grande { vec![] } else { vec![iced::time::every(Duration::from_secs(5)).map(|_| Msg::Tick)] };
        // Soltar (fin de un arrastre o del giro de la skin) y el tamaño de la ventana (para el menú).
        s.push(iced::event::listen_with(|e, _, _| match e {
            iced::Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => Some(Msg::Soltar),
            iced::Event::Window(iced::window::Event::Resized(t)) => Some(Msg::Ventana(t)),
            _ => None,
        }));
        Subscription::batch(s)
    }

    /// Los estantes de Big Picture: «Seguir jugando» y luego cada carpeta de arriba (con lo de sus
    /// subcarpetas) y las que no tienen carpeta.
    pub fn estantes(&self) -> Vec<(String, Vec<&Instancia>)> {
        let mut v: Vec<(String, Vec<&Instancia>)> = Vec::new();
        let mut recientes: Vec<&Instancia> = self.visibles().filter(|i| i.ultima_ms.is_some()).collect();
        recientes.sort_by_key(|i| std::cmp::Reverse(i.ultima_ms.unwrap_or(0)));
        recientes.truncate(10);
        if !recientes.is_empty() {
            v.push(("Seguir jugando".into(), recientes));
        }
        for c in self.subcarpetas("") {
            let pre = format!("{c}/");
            let mut dentro: Vec<&Instancia> = self.visibles().filter(|i| {
                let k = self.carpeta_de(i);
                k == c || k.starts_with(&pre)
            }).collect();
            self.ordena(&mut dentro);
            if !dentro.is_empty() {
                v.push((c, dentro));
            }
        }
        let sueltas = self.en_carpeta("");
        if !sueltas.is_empty() {
            v.push((if v.is_empty() { "Tus instancias".into() } else { "Sin carpeta".into() }, sueltas));
        }
        v
    }

    /// Pone la selección de Big Picture sobre la instancia elegida en la ventana normal.
    fn bp_desde_sel(&mut self) {
        let (pos, primera) = {
            let estantes = self.estantes();
            let pos = self.sel.as_ref().and_then(|s| {
                estantes.iter().enumerate().find_map(|(f, (_, l))| l.iter().position(|i| i.clave() == *s).map(|c| (f, c)))
            });
            (pos, estantes.first().and_then(|e| e.1.first()).map(|i| i.clave()))
        };
        self.bp = pos.unwrap_or((0, 0));
        if pos.is_none() {
            if let Some(c) = primera {
                self.sel = Some(c);
            }
        }
    }

    pub fn todas(&self) -> impl Iterator<Item = &Instancia> {
        self.horizon.iter().chain(self.prism.iter().filter(move |_| self.aj.ver_prism))
    }

    pub fn instancia(&self, clave: &str) -> Option<&Instancia> {
        self.todas().find(|i| i.clave() == clave)
    }

    pub fn cuenta_activa(&self) -> Option<&Cuenta> {
        self.cuentas.iter().find(|c| c.elegida).or(self.cuentas.first())
    }

    /// Todas las instancias para «Instalar en»: las de Horizon y las de Prism.
    pub fn opciones_instancias(&self) -> Vec<Opcion> {
        self.todas()
            .map(|i| Opcion {
                clave: i.clave(),
                nombre: format!("{} · {}", i.nombre, if i.origen == Origen::Prism { "Prism" } else { i.etiqueta_motor() }),
            })
            .collect()
    }

    fn destino_explorar(&self) -> Option<Instancia> {
        self.explorar.instancia.as_ref().and_then(|o| self.instancia(&o.clave)).cloned()
    }

    // ---------- Carpetas ----------

    /// La carpeta («A/B») de una instancia; "" es la raíz.
    pub fn carpeta_de(&self, i: &Instancia) -> String {
        match i.origen {
            Origen::Horizon => i.grupo.clone().unwrap_or_default(),
            Origen::Prism => self.aj.carpetas_prism.get(&i.clave()).cloned().or_else(|| i.grupo.clone()).unwrap_or_default(),
        }
    }

    /// Todas las carpetas que existen, con sus padres.
    pub fn todas_carpetas(&self) -> Vec<String> {
        let mut v: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        let mut mete = |c: &str| {
            let mut ruta = String::new();
            for parte in c.split('/').filter(|p| !p.is_empty()) {
                if !ruta.is_empty() {
                    ruta.push('/');
                }
                ruta.push_str(parte);
                v.insert(ruta.clone());
            }
        };
        for i in self.visibles() {
            mete(&self.carpeta_de(i));
        }
        for c in &self.aj.carpetas {
            mete(c);
        }
        v.into_iter().filter(|c| !self.carpeta_archivada(c)).collect()
    }

    /// ¿Está archivada esta carpeta, o alguna de las que la contienen?
    pub fn carpeta_archivada(&self, c: &str) -> bool {
        !c.is_empty() && self.aj.carpetas_archivadas.iter().any(|a| c == a || c.starts_with(&format!("{a}/")))
    }

    /// Instancias dentro de un grupo archivado (también en sus subcarpetas).
    pub fn en_grupo_archivado(&self, g: &str) -> Vec<&Instancia> {
        let pre = format!("{g}/");
        let mut v: Vec<&Instancia> = self.todas().filter(|i| {
            let k = self.carpeta_de(i);
            k == g || k.starts_with(&pre)
        }).collect();
        self.ordena(&mut v);
        v
    }

    /// Lo que cuenta el botón del archivador: instancias sueltas más grupos.
    pub fn n_archivo(&self) -> usize {
        self.archivadas().len() + self.aj.carpetas_archivadas.len()
    }

    pub fn subcarpetas(&self, padre: &str) -> Vec<String> {
        self.todas_carpetas().into_iter().filter(|c| padre_de(c) == padre).collect()
    }

    /// Las que se ven en la ventana (todas menos las archivadas).
    pub fn visibles(&self) -> impl Iterator<Item = &Instancia> {
        self.todas().filter(move |i| !self.aj.archivadas.contains(&i.clave()) && !self.carpeta_archivada(&self.carpeta_de(i)))
    }

    /// Archivadas una a una (las de un grupo archivado salen con su grupo).
    pub fn archivadas(&self) -> Vec<&Instancia> {
        self.todas()
            .filter(|i| self.aj.archivadas.contains(&i.clave()) && !self.carpeta_archivada(&self.carpeta_de(i)))
            .collect()
    }

    /// Ordena según lo elegido (nombre, última vez, más jugadas o a mano).
    pub fn ordena<'b>(&self, v: &mut Vec<&'b Instancia>) {
        match self.aj.orden {
            Orden::Nombre => v.sort_by_key(|i| i.nombre.to_lowercase()),
            Orden::Ultima => v.sort_by_key(|i| std::cmp::Reverse(i.ultima_ms.unwrap_or(0))),
            Orden::Jugado => v.sort_by_key(|i| std::cmp::Reverse(i.jugado_s)),
            Orden::Manual => {
                let pos = |i: &Instancia| self.aj.orden_manual.iter().position(|k| *k == i.clave()).unwrap_or(usize::MAX);
                v.sort_by(|a, b| pos(a).cmp(&pos(b)).then_with(|| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase())));
            }
        }
    }

    pub fn en_carpeta(&self, c: &str) -> Vec<&Instancia> {
        let f = self.filtro.to_lowercase();
        let mut v: Vec<&Instancia> =
            self.visibles().filter(|i| self.carpeta_de(i) == c && (f.is_empty() || i.nombre.to_lowercase().contains(&f))).collect();
        self.ordena(&mut v);
        v
    }

    /// Las que coinciden con la búsqueda, estén donde estén.
    pub fn buscadas(&self) -> Vec<&Instancia> {
        let f = self.filtro.to_lowercase();
        let mut v: Vec<&Instancia> = self.visibles().filter(|i| i.nombre.to_lowercase().contains(&f)).collect();
        self.ordena(&mut v);
        v
    }

    pub fn icono_carpeta(&self, c: &str) -> Handle {
        let color = self.aj.colores_carpeta.get(c).copied().unwrap_or(0);
        self.carpetas_ico.get(&color).cloned().unwrap_or_else(|| self.carpeta_ico.clone())
    }

    /// La carpeta a tamaño de fila.
    pub fn icono_carpeta_p(&self, c: &str) -> Handle {
        let color = self.aj.colores_carpeta.get(c).copied().unwrap_or(0);
        self.carpetas_ico_p.get(&color).cloned().unwrap_or_else(|| self.carpeta_ico.clone())
    }

    /// La mascota de la esquina según el tema y lo elegido.
    pub fn mascota(&self) -> Option<Handle> {
        if !self.aj.esquina {
            return None;
        }
        let elegida = match self.aj.mascota.as_str() {
            "auto" if self.p.jiyu => "jiyu",
            "auto" => "kitteh",
            otra => otra,
        };
        if elegida == "jiyu" {
            Some(self.mascota.clone())
        } else {
            self.gatos.get(elegida).cloned().or_else(|| Some(self.mascota.clone()))
        }
    }

    /// Cuántas instancias hay dentro (también en subcarpetas).
    pub fn cuantas_en(&self, c: &str) -> usize {
        let pre = format!("{c}/");
        self.visibles().filter(|i| {
            let k = self.carpeta_de(i);
            k == c || k.starts_with(&pre)
        }).count()
    }

    fn mover_tarea(&mut self, clave: &str, destino: &str) -> Task<Msg> {
        let Some(inst) = self.instancia(clave).cloned() else { return Task::none() };
        let origen = self.carpeta_de(&inst);
        if origen == destino {
            return Task::none();
        }
        // La carpeta de la que sale no desaparece aunque se quede vacía (como en Nautilus).
        if !origen.is_empty() && !self.aj.carpetas.contains(&origen) {
            self.aj.carpetas.push(origen);
        }
        match inst.origen {
            Origen::Prism => {
                self.aj.carpetas_prism.insert(inst.clave(), destino.to_string());
                self.guarda();
                Task::none()
            }
            Origen::Horizon => {
                if let Some(i) = self.horizon.iter_mut().find(|i| i.id == inst.id) {
                    i.grupo = (!destino.is_empty()).then(|| destino.to_string());
                }
                self.guarda();
                let g = (!destino.is_empty()).then(|| destino.to_string());
                Task::perform(motor::grupo(inst.id.clone(), g), |r| match r {
                    Ok(()) => Msg::Tick,
                    Err(e) => Msg::Hecho(Err(e)),
                })
            }
        }
    }

    /// La clave de la caché de búsquedas: todo lo que cambia el resultado.
    fn clave_explorar(&self) -> String {
        let e = &self.explorar;
        format!(
            "{}|{}|{}|{}|{:?}|{}",
            e.fuente,
            e.tipo,
            e.instancia.as_ref().map(|o| o.clave.clone()).unwrap_or_default(),
            e.texto.trim().to_lowercase(),
            e.cargador,
            e.por_version
        )
    }

    fn fuente_instalar(&self) -> &'static str {
        if self.explorar.fuente == "horizon" {
            "horizon"
        } else {
            "modrinth"
        }
    }

    /// Mete en la cola si no está ya esperando o bajando para esa instancia.
    fn encola(&mut self, m: Encontrado, tipo: &'static str, fuente: &'static str, inst: String) {
        let ya = self.explorar.cola.iter().any(|p| {
            p.m.id == m.id && p.inst == inst && matches!(p.estado, EstadoPedido::Esperando | EstadoPedido::Bajando)
        });
        if !ya {
            self.explorar.cola.push(Pedido { m, tipo, fuente, inst, estado: EstadoPedido::Esperando });
        }
    }

    /// El estado de un resultado en la instancia elegida: en la cola, bajando, ya instalado…
    pub fn estado_de(&self, m: &Encontrado) -> Option<EstadoPedido> {
        let inst = self.explorar.instancia.as_ref()?.clave.as_str();
        if let Some(p) = self.explorar.cola.iter().rev().find(|p| p.m.id == m.id && p.inst == inst) {
            if !matches!(&p.estado, EstadoPedido::Fallo(e) if e == "Quitada de la cola") {
                return Some(p.estado.clone());
            }
        }
        let i = &self.explorar.instalados;
        (i.contains(&m.id.to_lowercase()) || m.modrinth.as_ref().is_some_and(|x| i.contains(&x.to_lowercase())))
            .then(|| EstadoPedido::Hecho("Ya instalado".into()))
    }

    pub fn refigura(&mut self) {
        let a = &self.armario;
        self.armario.figura = a.sel.and_then(|i| a.skins.get(i)).and_then(|s| {
            skins::figura_3d(&s.ruta, self.aj.modelo_delgado, a.guinada, a.cabeceo, 260, 400)
        });
    }

    fn rehace_iconos(&mut self) {
        let mut mapa = HashMap::new();
        let insts: Vec<(String, crate::datos::Icono, String)> =
            self.todas().map(|i| (i.clave(), i.icono.clone(), i.nombre.clone())).collect();
        for (clave, icono, nombre) in insts {
            // El icono que elegiste en este launcher manda (las de Prism se guardan aquí, no en Prism).
            let icono = self.aj.iconos.get(&clave).map(|e| icono_de(e)).unwrap_or(icono);
            mapa.insert(clave, self.cache.icono(&icono, &nombre, self.aj.texturas_mc));
        }
        self.iconos = mapa;
        self.icono_defecto = Some(self.cache.icono(&crate::datos::Icono::Defecto, "default", self.aj.texturas_mc));
    }

    fn avisa(&mut self, texto: impl Into<String>, error: bool) {
        self.aviso = Some((texto.into(), error));
    }

    fn guarda(&mut self) {
        self.aj.guarda();
    }

    pub fn update(&mut self, msg: Msg) -> Task<Msg> {
        // Cualquier acción (salvo mover el ratón o desplegar «Mover a…») cierra el menú contextual.
        // Las respuestas que llegan solas (instancias, logos, cuentas…) no lo cierran: solo lo que haces tú.
        let de_fondo = matches!(
            msg,
            Msg::AbrirMenu(..)
                | Msg::MoverDesplegar
                | Msg::Tick
                | Msg::Ventana(_)
                | Msg::Hover(_)
                | Msg::Soltar
                | Msg::Destino(_)
                | Msg::Instancias(_)
                | Msg::Prism(_)
                | Msg::Cuentas(_)
                | Msg::Versiones(_)
                | Msg::Cara(_)
                | Msg::Gatos(_)
                | Msg::SvEstado(..)
                | Msg::PrismIconos(_)
                | Msg::Logos(_)
                | Msg::EModsH(_)
                | Msg::EMundos(_)
                | Msg::EJava(_)
                | Msg::ELanz(_)
                | Msg::EIdentificados(_)
                | Msg::ECapturas(_)
                | Msg::Partida(..)
                | Msg::XTurno(..)
                | Msg::XSoporte(..)
                | Msg::XMasR(..)
                | Msg::XResultados(_)
                | Msg::XPausa(_)
                | Msg::XHecho(..)
                | Msg::Compilando(..)
                | Msg::XInstalados(..)
                | Msg::NJava(_)
                | Msg::NPacks(_)
                | Msg::NNuevas(_)
                | Msg::NCreada(_)
                | Msg::SPerfil(_)
                | Msg::SBajada(_)
                | Msg::SHecho(_)
                | Msg::SImportada(_)
                | Msg::AHorizon(_)
                | Msg::ACache(_)
                | Msg::Hecho(_)
                | Msg::CAcceso(_)
                | Msg::IHecho(_)
        );
        if self.menu.is_some() && !de_fondo {
            self.menu = None;
            if matches!(msg, Msg::CerrarMenu | Msg::Cerrar) {
                return Task::none();
            }
        }
        match msg {
            // ---------- Carga ----------
            Msg::Recargar => {
                self.cargando = true;
                return self.carga();
            }
            Msg::Instancias(r) => {
                self.cargando = false;
                match r {
                    Ok(v) => {
                        self.horizon = v;
                        self.error_horizon = None;
                    }
                    Err(e) => self.error_horizon = Some(e),
                }
                if self.sel.as_ref().map_or(true, |s| self.instancia(s).is_none()) {
                    self.sel = self.horizon.iter().find(|i| i.elegida).map(|i| i.clave());
                }
                self.rehace_iconos();
                if let Some(a) = self.abrir.take() {
                    return self.abre_prueba(&a);
                }
            }
            Msg::Prism(v) => {
                self.prism = v;
                self.rehace_iconos();
            }
            Msg::Cuentas(r) => match r {
                Ok(v) => {
                    self.cuentas = v;
                    let mut t = Vec::new();
                    if let Some(uuid) = self.cuenta_activa().map(|c| c.uuid.clone()) {
                        // La cabeza de tu skin, al lado de tu nombre.
                        t.push(Task::perform(skins::cara_de_cuenta(uuid.clone()), Msg::Cara));
                        // Si el armario se abrió antes de saber la cuenta, ahora se piden sus capas.
                        if self.dlg == Some(Dlg::Armario) && self.armario.perfil.is_none() && !self.armario.cargando {
                            self.armario.cargando = true;
                            t.push(Task::perform(skins::perfil(uuid), Msg::SPerfil));
                        }
                    }
                    return Task::batch(t);
                }
                Err(e) => self.avisa(e, true),
            },
            Msg::Versiones(r) => {
                if let Ok(v) = r {
                    self.versiones = v;
                }
            }
            Msg::Tick => self.memoria_mb = crate::ajustes::memoria_propia_mb(),

            // ---------- Principal ----------
            Msg::Elegir(clave) => {
                // Doble clic (dos clics en menos de 400 ms sobre la misma) = jugar, como en Prism.
                let doble = matches!(&self.ultimo_clic, Some((c, t)) if *c == clave && t.elapsed() < Duration::from_millis(400));
                self.ultimo_clic = Some((clave.clone(), Instant::now()));
                self.sel = Some(clave.clone());
                self.aj.seleccion = Some(clave.clone());
                self.guarda();
                if doble {
                    self.ultimo_clic = None;
                    return self.update(Msg::Jugar(clave));
                }
            }
            Msg::Filtro(f) => self.filtro = f,
            Msg::Grupo(g) => {
                if let Some(i) = self.aj.grupos_cerrados.iter().position(|x| *x == g) {
                    self.aj.grupos_cerrados.remove(i);
                } else {
                    self.aj.grupos_cerrados.push(g);
                }
                self.guarda();
            }
            Msg::Jugar(clave) => {
                let Some(inst) = self.instancia(&clave).cloned() else { return Task::none() };
                // Tus servidores, en su lista Multijugador antes de jugar (solo la primera vez en esta sesión).
                if self.aj.servidores_siempre && !self.aj.servidores.is_empty() && !self.sincronizadas.contains(&clave) {
                    self.sincronizadas.insert(clave.clone());
                    if let Err(e) = crate::servidores::sincroniza(&inst.carpeta, &self.aj.servidores) {
                        self.avisa(format!("No añadí tus servidores: {e}"), true);
                    }
                }
                // Si esta instancia tiene su skin, se pone en la cuenta antes de jugar (Mojang la guarda en la
                // cuenta: es la que verán todos hasta que la cambies).
                if let (Some(ruta), Some(uuid)) =
                    (self.aj.skin_instancia.get(&clave).cloned(), self.cuenta_activa().map(|c| c.uuid.clone()))
                {
                    let delgado = self.aj.modelo_delgado;
                    let actual = format!("skin-puesta-{uuid}");
                    if self.aj.iconos.get(&actual) != Some(&ruta) {
                        self.aj.iconos.insert(actual, ruta.clone());
                        self.guarda();
                        self.avisa("Poniendo la skin de esta instancia…", false);
                        let c2 = clave.clone();
                        return Task::perform(skins::poner_skin(uuid, PathBuf::from(ruta), delgado), move |r| match r {
                            Ok(()) => Msg::Jugar(c2.clone()),
                            Err(e) => Msg::Hecho(Err(format!("No pude ponerte la skin: {e}"))),
                        });
                    }
                }
                // Antes de jugar una Java de Horizon, se le pasan la memoria y los argumentos del perfil.
                if inst.origen == Origen::Horizon && self.aj.gestionar_java {
                    let id = inst.id.clone();
                    let java = (inst.motor == Motor::Java)
                        .then(|| JavaInstancia { memoria_mib: self.aj.memoria_mib, argumentos: self.aj.argumentos_jvm() });
                    let aj = self.aj.clone();
                    let sis = self.sis.clone();
                    self.avisa(format!("Preparando {}…", inst.nombre), false);
                    return Task::perform(
                        async move {
                            if let Some(j) = java {
                                motor::guardar_java(id.clone(), j).await?;
                            }
                            let mut l = motor::lanzamiento(id.clone()).await.unwrap_or_default();
                            l.gamemode = aj.gamemode && sis.gamemode;
                            l.mangohud = aj.mangohud && sis.mangohud;
                            l.gpu_dedicada = aj.gpu_dedicada;
                            let propias: Vec<&str> = l
                                .entorno
                                .split_whitespace()
                                .filter(|v| !v.starts_with("__GL_THREADED_OPTIMIZATIONS") && !v.starts_with("__GL_SHADER_DISK_CACHE_SKIP_CLEANUP"))
                                .collect();
                            let mut entorno = propias.join(" ");
                            let nuestras = aj.entorno();
                            if !nuestras.is_empty() {
                                entorno = format!("{entorno} {nuestras}").trim().to_string();
                            }
                            l.entorno = entorno;
                            motor::guardar_lanzamiento(id, l).await
                        },
                        move |r| match r {
                            Ok(()) => Msg::Arrancar(clave.clone()),
                            Err(e) => Msg::Hecho(Err(format!("No pude preparar la instancia: {e}"))),
                        },
                    );
                }
                return self.update(Msg::Arrancar(clave));
            }
            Msg::Arrancar(clave) => {
                let Some(inst) = self.instancia(&clave).cloned() else { return Task::none() };
                if self.partidas.contains_key(&clave) {
                    return Task::none();
                }
                let extra = if inst.motor == Motor::Rust { self.aj.argumentos_rust() } else { vec![] };
                let primera = inst.motor == Motor::Java && inst.ultima_ms.is_none() && inst.origen == Origen::Horizon;
                let linea = if primera {
                    "Primera vez: Horizon descarga Minecraft y su Java…".to_string()
                } else {
                    "Arrancando…".to_string()
                };
                self.partidas.insert(clave.clone(), EnMarcha { linea, en_juego: false });
                self.avisa(format!("Arrancando {}", inst.nombre), false);
                let c = clave.clone();
                return Task::stream(motor::partida(inst, extra)).map(move |p| Msg::Partida(c.clone(), p));
            }
            Msg::Partida(clave, p) => {
                let nombre = self.instancia(&clave).map(|i| i.nombre.clone()).unwrap_or_default();
                let carpeta = self.instancia(&clave).map(|i| i.carpeta.clone()).unwrap_or_default();
                match p {
                    motor::Partida::Linea(l) => {
                        if let Some(e) = self.partidas.get_mut(&clave) {
                            if !e.en_juego {
                                e.linea = l;
                            }
                        }
                    }
                    motor::Partida::EnJuego(pid) => {
                        if let Some(e) = self.partidas.get_mut(&clave) {
                            e.en_juego = true;
                            e.linea = "En juego".into();
                        }
                        // Solo ahora, con el juego ya abierto, se cierra el launcher.
                        if self.aj.cerrar_al_jugar {
                            if let (true, Ok(yo)) = (self.aj.reabrir, std::env::current_exe()) {
                                let _ = std::process::Command::new(yo)
                                    .arg("--vigilar")
                                    .arg(pid.to_string())
                                    .arg(&carpeta)
                                    .stdin(std::process::Stdio::null())
                                    .stdout(std::process::Stdio::null())
                                    .stderr(std::process::Stdio::null())
                                    .spawn();
                            }
                            return iced::exit();
                        }
                        self.avisa(format!("Jugando a {nombre}"), false);
                    }
                    motor::Partida::Fin(r) => {
                        self.partidas.remove(&clave);
                        match r {
                            Ok(()) => self.avisa(format!("{nombre}: partida terminada"), false),
                            Err(e) => self.avisa(format!("{nombre} no arrancó: {e}"), true),
                        }
                        return self.carga();
                    }
                }
            }
            Msg::Abrir(d) => return self.abre(d),
            Msg::AbrirEditar(clave, pag) => {
                self.editar = Editar { clave: clave.clone(), ..Default::default() };
                self.editar.nombre = self.instancia(&clave).map(|i| i.nombre.clone()).unwrap_or_default();
                self.dlg = Some(Dlg::Editar);
                return self.update(Msg::EPagina(pag));
            }
            Msg::Cerrar => {
                if self.pide_nombre.take().is_some() || self.borrar.take().is_some() {
                    return Task::none();
                }
                if self.dlg == Some(Dlg::Icono) && self.selector.para_nueva {
                    self.selector.para_nueva = false;
                    self.dlg = Some(Dlg::Nueva);
                    return Task::none();
                }
                if self.dlg.take().is_none() && self.menu.is_none() && self.grande {
                    return self.update(Msg::BPAlternar);
                }
            }
            Msg::BPAlternar => {
                if self.dlg.is_some() {
                    return Task::none();
                }
                self.grande = !self.grande;
                self.menu = None;
                if self.grande {
                    self.bp_fila0 = 0;
                    self.bp_col0.clear();
                    self.bp_desde_sel();
                }
                let modo = if self.grande { iced::window::Mode::Fullscreen } else { iced::window::Mode::Windowed };
                let ventana = iced::window::get_latest().and_then(move |id| iced::window::change_mode(id, modo));
                if self.grande {
                    let (f, c) = self.bp;
                    return Task::batch([ventana, self.update(Msg::BPElegir(f, c))]);
                }
                return ventana;
            }
            Msg::BPMover(dx, dy) => {
                if !self.grande || self.dlg.is_some() {
                    return Task::none();
                }
                let estantes = self.estantes();
                if estantes.is_empty() {
                    return Task::none();
                }
                let (mut f, mut c) = self.bp;
                f = (f as i32 + dy).clamp(0, estantes.len() as i32 - 1) as usize;
                if dy != 0 {
                    c = c.min(estantes[f].1.len().saturating_sub(1));
                }
                c = (c as i32 + dx).clamp(0, estantes[f].1.len() as i32 - 1) as usize;
                if (f, c) == self.bp {
                    return Task::none();
                }
                return self.update(Msg::BPElegir(f, c));
            }
            Msg::BPClic(f, c) => {
                // Con el ratón: un clic elige, y otro clic rápido sobre la misma juega (como el doble clic).
                // Las flechas nunca juegan: para eso está Intro.
                let doble = self.bp == (f, c) && self.ultimo_bp.is_some_and(|t| t.elapsed() < Duration::from_millis(450));
                self.ultimo_bp = if doble { None } else { Some(Instant::now()) };
                if doble {
                    let clave = self.estantes().get(f).and_then(|e| e.1.get(c)).map(|i| i.clave());
                    if let Some(k) = clave.filter(|k| !self.partidas.contains_key(k)) {
                        return self.update(Msg::Jugar(k));
                    }
                    return Task::none();
                }
                return self.update(Msg::BPElegir(f, c));
            }
            Msg::BPElegir(f, c) => {
                let (clave, largo) = {
                    let estantes = self.estantes();
                    let Some(k) = estantes.get(f).and_then(|e| e.1.get(c)).map(|i| i.clave()) else { return Task::none() };
                    (k, estantes.len())
                };
                let _ = largo;
                self.bp = (f, c);
                self.sel = Some(clave);
                // Solo se mueve la «ventana» de estantes y tarjetas cuando la elegida se sale de ella.
                let (filas, cols) = crate::vistas::grande::cabe(self.ventana);
                if f < self.bp_fila0 {
                    self.bp_fila0 = f;
                } else if f >= self.bp_fila0 + filas {
                    self.bp_fila0 = f + 1 - filas;
                }
                let c0 = self.bp_col0.entry(f).or_insert(0);
                if c < *c0 {
                    *c0 = c;
                } else if c >= *c0 + cols {
                    *c0 = c + 1 - cols;
                }
                let _ = largo;
                return Task::none();
            }
            Msg::PedirCompilar(c) => self.confirmar_compilar = c,
            Msg::Compilar(clave) => {
                self.confirmar_compilar = None;
                let Some(inst) = self.instancia(&clave).filter(|i| i.origen == Origen::Horizon).cloned() else { return Task::none() };
                if self.compilando.contains_key(&clave) {
                    return Task::none();
                }
                self.compilado.remove(&clave);
                self.compilando.insert(clave.clone(), "Empezando…".into());
                return Task::stream(motor::compilar(inst.id)).map(move |c| Msg::Compilando(clave.clone(), c));
            }
            Msg::Compilando(clave, c) => match c {
                motor::Compila::Linea(l) => {
                    self.compilando.insert(clave, l);
                }
                motor::Compila::Fin(r) => {
                    self.compilando.remove(&clave);
                    let nombre = self.instancia(&clave).map(|i| i.nombre.clone()).unwrap_or_default();
                    match &r {
                        Ok(()) => self.avisa(format!("{nombre}: mods compilados, ya puedes jugar"), false),
                        Err(e) => self.avisa(format!("{nombre}: no se pudo compilar: {e}"), true),
                    }
                    self.compilado.insert(clave, r);
                }
            },
            Msg::BPRueda(dx, dy) => {
                // La rueda del ratón: arriba y abajo cambian de estante; de lado, de tarjeta.
                if dy.abs() >= dx.abs() && dy != 0.0 {
                    return self.update(Msg::BPMover(0, if dy > 0.0 { -1 } else { 1 }));
                }
                if dx != 0.0 {
                    return self.update(Msg::BPMover(if dx > 0.0 { -1 } else { 1 }, 0));
                }
            }
            Msg::BPEditar => {
                if self.grande && self.dlg.is_none() {
                    if let Some(c) = self.sel.clone() {
                        return self.update(Msg::AbrirEditar(c, PagE::Version));
                    }
                }
            }
            Msg::Carpeta(ruta) => {
                let _ = std::fs::create_dir_all(&ruta);
                abre_externo(ruta.to_string_lossy().as_ref());
            }
            Msg::Url(u) => abre_externo(&u),
            Msg::Hecho(r) => {
                match r {
                    Ok(t) => self.avisa(t, false),
                    Err(e) => self.avisa(e, true),
                }
                return self.carga();
            }
            Msg::CopiarAHorizon(clave) => {
                if let Some(i) = self.instancia(&clave) {
                    let nombre = i.nombre.clone();
                    self.avisa(format!("Copiando {nombre} a Horizon…"), false);
                    let como = format!("{nombre} (Horizon)");
                    return Task::perform(motor::copiar_de_prism(nombre.clone(), como.clone()), move |r| {
                        Msg::Hecho(r.map(|_| format!("Copiada como «{como}». La de Prism sigue igual.")))
                    });
                }
            }
            Msg::PedirBorrar(clave) => {
                self.borrar_texto.clear();
                self.borrar = Some(clave);
            }
            Msg::BorrarTexto(t) => self.borrar_texto = t,
            Msg::CancelarBorrar => self.borrar = None,
            Msg::Borrar(clave) => {
                if self.borrar_texto.trim() != "BORRAR" {
                    return Task::none();
                }
                self.borrar = None;
                self.borrar_texto.clear();
                if let Some(i) = self.instancia(&clave).cloned() {
                    let nombre = i.nombre.clone();
                    if self.sel.as_deref() == Some(clave.as_str()) {
                        self.sel = None;
                    }
                    match i.origen {
                        Origen::Horizon => {
                            return Task::perform(motor::borrar(i.id.clone()), move |r| {
                                Msg::Hecho(r.map(|_| format!("{nombre} está en la papelera; puedes recuperarla desde allí.")))
                            });
                        }
                        Origen::Prism => {
                            // La carpeta entera de la instancia de Prism, a la papelera.
                            let dir = crate::prism::raiz().map(|(r, _)| r.join("instances").join(&i.id));
                            let ok = dir.is_some_and(|d| {
                                std::process::Command::new("gio").arg("trash").arg(&d).status().is_ok_and(|s| s.success())
                            });
                            return self.update(Msg::Hecho(if ok {
                                Ok(format!("{nombre} está en la papelera; Prism dejará de verla."))
                            } else {
                                Err("No pude moverla a la papelera".into())
                            }));
                        }
                    }
                }
            }
            Msg::ActualizarHorizon(tag) => {
                self.avisa(format!("Instalando Horizon {tag}…"), false);
                return Task::perform(motor::instalar_version(tag.clone()), move |r| {
                    Msg::Hecho(r.map(|_| format!("Horizon {tag} instalado.")))
                })
                .chain(Task::perform(motor::versiones_horizon(), Msg::Versiones));
            }

            // ---------- Nueva instancia ----------
            Msg::NFuente(f) => {
                self.nueva.fuente = Some(f);
                self.nueva.error = None;
                if f == Fuente::Java && self.nueva.java.is_empty() {
                    return Task::perform(motor::versiones_java(self.nueva.snapshots), Msg::NJava);
                }
                if f == Fuente::Modpack && self.nueva.packs.is_empty() {
                    return self.update(Msg::NPackBuscar);
                }
                if f == Fuente::Horizon && self.versiones.is_empty() {
                    return Task::perform(motor::versiones_horizon(), Msg::Versiones);
                }
            }
            Msg::NNombre(n) => self.nueva.nombre = n,
            Msg::NVersionH(v) => self.nueva.version_h = v,
            Msg::NVersionJ(v) => {
                if self.nueva.nombre.is_empty() || self.nueva.java.iter().any(|j| j.id == self.nueva.nombre) {
                    self.nueva.nombre = v.clone();
                }
                self.nueva.version_j = Some(v);
            }
            Msg::NSnapshots(s) => {
                self.nueva.snapshots = s;
                return Task::perform(motor::versiones_java(s), Msg::NJava);
            }
            Msg::NCargador(c) => self.nueva.cargador = c,
            Msg::NJava(r) => match r {
                Ok(v) => {
                    if self.nueva.version_j.is_none() {
                        self.nueva.version_j = v.iter().find(|x| x.kind == "release").map(|x| x.id.clone());
                    }
                    self.nueva.java = v;
                }
                Err(e) => self.nueva.error = Some(e),
            },
            Msg::NPrism(n) => {
                self.nueva.nombre = format!("{n} (Horizon)");
                self.nueva.prism = Some(n);
            }
            Msg::NCrear => {
                let nombre = self.nueva.nombre.trim().to_string();
                if nombre.is_empty() {
                    self.nueva.error = Some("Ponle un nombre".into());
                    return Task::none();
                }
                self.nueva.ocupado = true;
                self.nueva.error = None;
                self.nueva.antes = self.horizon.iter().map(|i| i.id.clone()).collect();
                let tarea = match self.nueva.fuente {
                    Some(Fuente::Modpack) => {
                        if let Some(f) = self.nueva.archivo.clone() {
                            Task::perform(motor::importar(f), Msg::NCreada)
                        } else if let Some(m) = self.nueva.pack.and_then(|i| self.nueva.packs.get(i)) {
                            let id = m.id.clone();
                            Task::perform(
                                async move {
                                    motor::json(vec!["mods".into(), "install".into(), id, "--source".into(), "modrinth".into()]).await.map(|_| ())
                                },
                                Msg::NCreada,
                            )
                        } else {
                            self.nueva.ocupado = false;
                            self.nueva.error = Some("Elige un paquete de mods o un archivo".into());
                            return Task::none();
                        }
                    }
                    Some(Fuente::Horizon) => {
                        let tag = self.nueva.version_h.clone();
                        let instalar = tag
                            .as_ref()
                            .and_then(|t| self.versiones.iter().find(|v| &v.tag == t))
                            .is_some_and(|v| !v.installed);
                        Task::perform(
                            async move {
                                if instalar {
                                    motor::instalar_version(tag.clone().unwrap()).await?;
                                }
                                motor::crear_horizon(nombre, tag).await
                            },
                            Msg::NCreada,
                        )
                    }
                    Some(Fuente::Java) => {
                        let Some(v) = self.nueva.version_j.clone() else {
                            self.nueva.ocupado = false;
                            self.nueva.error = Some("Elige una versión".into());
                            return Task::none();
                        };
                        Task::perform(motor::crear_java(nombre, v, self.nueva.cargador.map(str::to_string)), Msg::NCreada)
                    }
                    Some(Fuente::Prism) => {
                        let Some(origen) = self.nueva.prism.clone() else {
                            self.nueva.ocupado = false;
                            self.nueva.error = Some("Elige una instancia de Prism".into());
                            return Task::none();
                        };
                        Task::perform(motor::copiar_de_prism(origen, nombre), Msg::NCreada)
                    }
                    None => Task::none(),
                };
                return tarea;
            }
            Msg::NCreada(r) => match r {
                // Creada: se mira cuál es la nueva para ponerle icono, carpeta, memoria y skin.
                Ok(()) => return Task::perform(motor::instancias(), Msg::NNuevas),
                Err(e) => {
                    self.nueva.ocupado = false;
                    self.nueva.error = Some(e);
                }
            },
            Msg::NNuevas(r) => {
                self.nueva.ocupado = false;
                let nuevas: Vec<Instancia> = r
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|i| !self.nueva.antes.contains(&i.id))
                    .collect();
                let n = std::mem::take(&mut self.nueva);
                self.dlg = None;
                let mut t = Vec::new();
                for i in &nuevas {
                    let id = i.id.clone();
                    if let Some(spec) = &n.icono {
                        if let Some(v) = spec.strip_prefix("tex:").map(str::to_string).or_else(|| spec.starts_with('#').then(|| spec.clone())) {
                            t.push(Task::perform(motor::icono(id.clone(), v), |_| Msg::Tick));
                        } else {
                            self.aj.iconos.insert(i.clave(), spec.clone());
                        }
                    }
                    if !n.carpeta.is_empty() {
                        t.push(Task::perform(motor::grupo(id.clone(), Some(n.carpeta.clone())), |_| Msg::Tick));
                    }
                    if let (Some(m), Motor::Java) = (n.memoria, i.motor) {
                        let j = JavaInstancia { memoria_mib: m, argumentos: self.aj.argumentos_jvm() };
                        t.push(Task::perform(motor::guardar_java(id.clone(), j), |_| Msg::Tick));
                    }
                    if let Some(sk) = &n.skin {
                        self.aj.skin_instancia.insert(i.clave(), sk.clone());
                    }
                    self.sel = Some(i.clave());
                }
                self.guarda();
                let nombre = nuevas.first().map(|i| i.nombre.clone()).unwrap_or(n.nombre);
                self.avisa(format!("«{nombre}» creada"), false);
                return Task::batch(t).chain(self.carga());
            }

            // ---------- Editar ----------
            Msg::EPagina(pag) => {
                self.editar.pagina = Some(pag);
                self.editar.sel = None;
                return self.carga_pagina(pag);
            }
            Msg::ESel(i) => self.editar.sel = Some(i),
            Msg::EActivar(i, on) => {
                let lista = match self.editar.pagina {
                    Some(PagE::Recursos) => &mut self.editar.recursos,
                    Some(PagE::Shaders) => &mut self.editar.shaders,
                    _ => &mut self.editar.mods,
                };
                if let Some(a) = lista.get_mut(i) {
                    match activar(a, on) {
                        Ok(()) => {}
                        Err(e) => self.aviso = Some((e, true)),
                    }
                }
            }
            Msg::EQuitar => {
                let pag = self.editar.pagina;
                let lista = match pag {
                    Some(PagE::Recursos) => &mut self.editar.recursos,
                    Some(PagE::Shaders) => &mut self.editar.shaders,
                    _ => &mut self.editar.mods,
                };
                let mut aviso = None;
                if let Some(i) = self.editar.sel.take() {
                    if i < lista.len() {
                        let a = lista.remove(i);
                        // A la papelera, para poder recuperarlo.
                        let ok = std::process::Command::new("gio").arg("trash").arg(&a.ruta).status().is_ok_and(|s| s.success());
                        if ok {
                            aviso = Some((format!("{} está en la papelera", a.nombre), false));
                        } else {
                            lista.insert(i, a);
                            aviso = Some(("No pude moverlo a la papelera".to_string(), true));
                        }
                    }
                }
                self.aviso = aviso.or(self.aviso.take());
            }
            Msg::EQuitarH(proyecto) => {
                if let Some(i) = self.instancia(&self.editar.clave) {
                    let id = i.id.clone();
                    return Task::perform(motor::quitar_mod(proyecto, id.clone()), move |_| Msg::EPagina(PagE::Mods));
                }
            }
            Msg::EModsH(r) => match r {
                Ok(v) => self.editar.mods_h = v,
                Err(e) => self.avisa(e, true),
            },
            Msg::EMundos(r) => match r {
                Ok(v) => self.editar.mundos = v,
                Err(e) => self.avisa(e, true),
            },
            Msg::EJava(r) => self.editar.java = r.ok(),
            Msg::ELanz(r) => self.editar.lanz = r.ok(),
            Msg::ENombre(n) => self.editar.nombre = n,
            Msg::ERenombrar => {
                if let Some(i) = self.instancia(&self.editar.clave) {
                    let (id, n) = (i.id.clone(), self.editar.nombre.trim().to_string());
                    if !n.is_empty() {
                        return Task::perform(motor::renombrar(id, n.clone()), move |r| Msg::Hecho(r.map(|_| format!("Ahora se llama «{n}»"))));
                    }
                }
            }
            Msg::EVersion(v) => {
                if let Some(i) = self.instancia(&self.editar.clave) {
                    let id = i.id.clone();
                    let tag = if v == "latest" { "latest".to_string() } else { v.clone() };
                    let instalar = self.versiones.iter().any(|x| x.tag == tag && !x.installed);
                    return Task::perform(
                        async move {
                            if instalar {
                                motor::instalar_version(tag.clone()).await?;
                            }
                            motor::fijar_version(id, tag).await
                        },
                        |r| Msg::Hecho(r.map(|_| "Versión cambiada".to_string())),
                    );
                }
            }
            Msg::ECargador(c) => {
                if let Some(i) = self.instancia(&self.editar.clave) {
                    let id = i.id.clone();
                    self.editar.cargador_nuevo = Some(c.clone());
                    let valor = if c == "Ninguno" { "none".to_string() } else { c.to_lowercase() };
                    return Task::perform(
                        async move { motor::json(vec!["instance".into(), "loader".into(), id, valor]).await },
                        |r| Msg::Hecho(r.map(|_| "Cargador cambiado".to_string())),
                    );
                }
            }
            Msg::EMem(m) => {
                if let Some(j) = &mut self.editar.java {
                    j.memoria_mib = m;
                }
            }
            Msg::EArgs(a) => {
                if let Some(j) = &mut self.editar.java {
                    j.argumentos = a;
                }
            }
            Msg::EEnv(e) => {
                if let Some(l) = &mut self.editar.lanz {
                    l.entorno = e;
                }
            }
            Msg::EEnvoltorio(e) => {
                if let Some(l) = &mut self.editar.lanz {
                    l.envoltorio = e;
                }
            }
            Msg::EGamemode(b) => {
                if let Some(l) = &mut self.editar.lanz {
                    l.gamemode = b;
                }
            }
            Msg::EMango(b) => {
                if let Some(l) = &mut self.editar.lanz {
                    l.mangohud = b;
                }
            }
            Msg::EGpu(b) => {
                if let Some(l) = &mut self.editar.lanz {
                    l.gpu_dedicada = b;
                }
            }
            Msg::EGuardar => {
                if let Some(i) = self.instancia(&self.editar.clave) {
                    let id = i.id.clone();
                    let java = self.editar.java.clone();
                    let lanz = self.editar.lanz.clone();
                    // Si tocas una instancia a mano, el perfil global deja de pisar su Java al jugar.
                    if java.is_some() && self.aj.gestionar_java {
                        self.aj.gestionar_java = false;
                        self.guarda();
                    }
                    return Task::perform(
                        async move {
                            if let Some(j) = java {
                                motor::guardar_java(id.clone(), j).await?;
                            }
                            if let Some(l) = lanz {
                                motor::guardar_lanzamiento(id, l).await?;
                            }
                            Ok("Ajustes de la instancia guardados".to_string())
                        },
                        Msg::Hecho,
                    );
                }
            }
            Msg::ERegistro => return self.carga_pagina(PagE::Registro),

            // ---------- Ajustes ----------
            Msg::APagina(p) => {
                self.pag_a = p;
                self.proxy_texto = self.aj.proxy.clone();
                if p == PagA::Horizon || p == PagA::Red {
                    return Task::batch([
                        Task::perform(motor::ajustes_horizon(), Msg::AHorizon),
                        Task::perform(motor::tamano_cache(), Msg::ACache),
                    ]);
                }
            }
            Msg::ATema(t) => {
                self.aj.tema = t;
                self.p = t.paleta();
                let (a, b, c) = insignias(&self.p);
                self.insignia_rust = a;
                self.insignia_java = b;
                self.hexagono = c;
                self.flecha_abierta = iconos::flecha(true, self.p.dim);
                self.carpeta_ico = iconos::carpeta(self.p.java, 32);
                self.carpetas_ico_p = iconos_carpeta(&self.p, 32);
                self.barra = iconos_barra(&self.p);
                self.carpetas_ico = iconos_carpeta(&self.p, (self.aj.tamano.px() * 2.0) as u32);
                self.estrella_llena = iconos::estrella(true, self.p.rust);
                self.estrella_vacia = iconos::estrella(false, self.p.dim);
                self.flecha_cerrada = iconos::flecha(false, self.p.dim);
                self.guarda();
            }
            Msg::AFlag(f, v) => {
                let aj = &mut self.aj;
                let perfil = matches!(
                    f,
                    Flag::Cerrar | Flag::PaginasGrandes | Flag::Gamemode | Flag::Mangohud | Flag::Gpu | Flag::HilosNvidia | Flag::HiloServidor
                );
                match f {
                    Flag::Insignia => aj.insignia = v,
                    Flag::Filo => aj.filo = v,
                    Flag::Hexagono => aj.hexagono = v,
                    Flag::Etiqueta => aj.etiqueta = v,
                    Flag::Esquina => aj.esquina = v,
                    Flag::Texturas => aj.texturas_mc = v,
                    Flag::VerPrism => aj.ver_prism = v,
                    Flag::Cerrar => aj.cerrar_al_jugar = v,
                    Flag::GestionarJava => aj.gestionar_java = v,
                    Flag::PaginasGrandes => aj.paginas_grandes = v,
                    Flag::Gamemode => aj.gamemode = v,
                    Flag::Mangohud => aj.mangohud = v,
                    Flag::Gpu => aj.gpu_dedicada = v,
                    Flag::HilosNvidia => aj.hilos_nvidia = v,
                    Flag::HiloServidor => aj.hilo_servidor = v,
                    Flag::Actualizaciones => aj.buscar_actualizaciones = v,
                    Flag::Grafica => aj.grafica = v,
                    Flag::ServidoresSiempre => aj.servidores_siempre = v,
                    Flag::Reabrir => aj.reabrir = v,
                }
                if perfil {
                    aj.perfil = Perfil::Propio;
                }
                self.guarda();
                if f == Flag::Texturas {
                    self.rehace_iconos();
                }
                if f == Flag::VerPrism {
                    return self.carga();
                }
            }
            Msg::ATamano(t) => {
                // Las carpetas se pintan al doble del tamaño en pantalla: si cambia, se repintan.
                self.carpetas_ico = iconos_carpeta(&self.p, (t.px() * 2.0) as u32);
                self.aj.tamano = t;
                self.guarda();
            }
            Msg::APerfil(p) => {
                let sis = self.sis.clone();
                self.aj.aplica_perfil(p, &sis);
                self.guarda();
            }
            Msg::AMem(m) => {
                self.aj.memoria_mib = m;
                self.aj.perfil = Perfil::Propio;
                self.guarda();
            }
            Msg::ARec(r) => {
                self.aj.recolector = r;
                self.aj.perfil = Perfil::Propio;
                self.guarda();
            }
            Msg::AAplicarJava => {
                let ids: Vec<String> = self.horizon.iter().filter(|i| i.motor == Motor::Java).map(|i| i.id.clone()).collect();
                let n = ids.len();
                let j = JavaInstancia { memoria_mib: self.aj.memoria_mib, argumentos: self.aj.argumentos_jvm() };
                self.aj.gestionar_java = true;
                self.guarda();
                return Task::perform(
                    async move {
                        for id in ids {
                            motor::guardar_java(id, j.clone()).await?;
                        }
                        Ok(format!("Aplicado a {n} instancias Java de Horizon"))
                    },
                    Msg::Hecho,
                );
            }
            Msg::AHorizon(r) => match r {
                Ok(a) => {
                    if self.proxy_texto.is_empty() && !a.proxy.is_empty() {
                        self.proxy_texto = a.proxy.clone();
                    }
                    self.ah_args = a.argumentos.clone();
                    self.ah = Some(a);
                }
                Err(e) => self.avisa(e, true),
            },
            Msg::ACache(r) => self.cache_nativa = r.ok(),
            Msg::AHPerfil(p) => {
                if let Some(a) = &mut self.ah {
                    a.perfil_nativo = p.to_string();
                }
                return Task::perform(motor::ajuste_horizon("native-profile".into(), p.into()), |r| {
                    Msg::Hecho(r.map(|_| "Perfil de mods nativos cambiado".into()))
                });
            }
            Msg::AHCache(g) => {
                if let Some(a) = &mut self.ah {
                    a.cache_gib = g;
                }
                return Task::perform(motor::ajuste_horizon("native-cache-gib".into(), g.to_string()), |r| {
                    Msg::Hecho(r.map(|_| "Límite de caché cambiado".into()))
                });
            }
            Msg::AHNuevas(b) => {
                if let Some(a) = &mut self.ah {
                    a.instalar_nuevas = b;
                }
                let v = if b { "on" } else { "off" };
                return Task::perform(motor::ajuste_horizon("install-new-versions".into(), v.into()), |r| {
                    Msg::Hecho(r.map(|_| "Guardado".into()))
                });
            }
            Msg::AHArgs(a) => self.ah_args = a,
            Msg::AHGuardarArgs => {
                return Task::perform(motor::argumentos_juego(self.ah_args.clone()), |r| {
                    Msg::Hecho(r.map(|_| "Argumentos del juego guardados".into()))
                });
            }

            // ---------- Cuentas ----------
            Msg::CElegir(u) => {
                return Task::perform(motor::elegir_cuenta(u), |r| Msg::Hecho(r.map(|_| "Cuenta elegida".into())));
            }
            Msg::CSalir(u) => {
                return Task::perform(motor::cerrar_sesion(u), |r| Msg::Hecho(r.map(|_| "Sesión cerrada".into())));
            }
            Msg::CEntrar => {
                self.acceder = Acceder { activo: true, ..Default::default() };
                return Task::stream(motor::iniciar_sesion()).map(Msg::CAcceso);
            }
            Msg::CAcceso(a) => match a {
                Acceso::Codigo { codigo, enlace } => {
                    let abrir = enlace.clone();
                    self.acceder.codigo = Some((codigo, enlace));
                    abre_externo(&abrir);
                }
                Acceso::Linea(l) => {
                    if !l.is_empty() {
                        self.acceder.lineas.push(l);
                        if self.acceder.lineas.len() > 6 {
                            self.acceder.lineas.remove(0);
                        }
                    }
                }
                Acceso::Hecho(r) => {
                    self.acceder = Acceder::default();
                    return self.update(Msg::Hecho(r.map(|_| "Sesión iniciada".into())));
                }
            },
            Msg::CCopiar(t) => return iced::clipboard::write(t),

            // ---------- Armario ----------
            Msg::SSel(i) => {
                self.armario.sel = Some(i);
                self.refigura();
            }
            Msg::SEspalda(b) => {
                self.armario.espalda = b;
                self.armario.guinada = if b { std::f32::consts::PI } else { -0.5 };
                self.armario.cabeceo = 0.15;
                self.refigura();
            }
            Msg::SGirar(on) => {
                self.armario.girando = on.then(|| Point::new(f32::NAN, f32::NAN));
            }
            Msg::SMover(p) => {
                if let Some(prev) = self.armario.girando {
                    if !prev.x.is_nan() {
                        self.armario.guinada += (p.x - prev.x) * 0.012;
                        self.armario.cabeceo = (self.armario.cabeceo + (p.y - prev.y) * 0.008).clamp(-0.7, 0.7);
                        self.refigura();
                    }
                    self.armario.girando = Some(p);
                }
            }
            Msg::SImportar => return Task::perform(skins::importar(), Msg::SImportada),
            Msg::SImportada(r) => match r {
                Ok(Some(ruta)) => {
                    self.armario.skins = skins::lista(self.aj.modelo_delgado);
                    if self.dlg == Some(Dlg::Editar) {
                        self.aj.skin_instancia.insert(self.editar.clave.clone(), ruta.to_string_lossy().to_string());
                        self.guarda();
                    }
                    self.armario.sel = self.armario.skins.iter().position(|s| s.ruta == ruta);
                    self.refigura();
                    self.avisa("Skin importada al armario", false);
                }
                Ok(None) => {}
                Err(e) => self.avisa(e, true),
            },
            Msg::ESkin(ruta) => {
                if ruta.is_empty() {
                    self.aj.skin_instancia.remove(&self.editar.clave);
                } else {
                    self.aj.skin_instancia.insert(self.editar.clave.clone(), ruta);
                }
                self.guarda();
            }
            Msg::SDelgado(b) => {
                self.aj.modelo_delgado = b;
                self.guarda();
                let t = self.update(Msg::SRecargar);
                self.refigura();
                return t;
            }
            Msg::SRecargar => {
                let d = self.aj.modelo_delgado;
                self.armario.skins = skins::lista(d);
            }
            Msg::SPerfil(r) => {
                self.armario.cargando = false;
                match r {
                    Ok(p) => {
                        self.armario.error = None;
                        if self.armario.perfil.is_none() && p.delgado != self.aj.modelo_delgado {
                            self.aj.modelo_delgado = p.delgado;
                            self.armario.skins = skins::lista(p.delgado);
                        }
                        self.armario.perfil = Some(p);
                    }
                    Err(e) => self.armario.error = Some(e),
                }
            }
            Msg::SBajar => {
                if let Some(c) = self.cuenta_activa() {
                    return Task::perform(skins::bajar_de_cuenta(c.uuid.clone(), c.nombre.clone()), Msg::SBajada);
                }
            }
            Msg::SBajada(r) => match r {
                Ok(_) => {
                    self.armario.skins = skins::lista(self.aj.modelo_delgado);
                    self.avisa("Tu skin actual está en el armario", false);
                }
                Err(e) => self.avisa(e, true),
            },
            Msg::SPoner => {
                if let Some(u) = self.cuenta_activa().map(|c| c.uuid.clone()) {
                    self.aj.iconos.remove(&format!("skin-puesta-{u}"));
                }
                let (Some(i), Some(c)) = (self.armario.sel, self.cuenta_activa()) else { return Task::none() };
                if let Some(s) = self.armario.skins.get(i) {
                    let (cuenta, ruta, d, nombre) = (c.uuid.clone(), s.ruta.clone(), self.aj.modelo_delgado, s.nombre.clone());
                    self.armario.cargando = true;
                    return Task::perform(skins::poner_skin(cuenta, ruta, d), move |r| {
                        Msg::SHecho(r.map(|_| format!("Ahora llevas «{nombre}». Se verá al volver a entrar al juego.")))
                    });
                }
            }
            Msg::SCapa(id) => {
                let Some(uuid) = self.cuenta_activa().map(|c| c.uuid.clone()) else { return Task::none() };
                self.armario.cargando = true;
                let quitar = id.is_none();
                return Task::perform(skins::poner_capa(uuid, id), move |r| {
                    Msg::SHecho(r.map(|_| if quitar { "Capa quitada".to_string() } else { "Capa puesta".to_string() }))
                });
            }
            Msg::SHecho(r) => {
                self.armario.cargando = false;
                match r {
                    Ok(t) => {
                        self.avisa(t, false);
                        if let Some(uuid) = self.cuenta_activa().map(|c| c.uuid.clone()) {
                            self.armario.cargando = true;
                            return Task::perform(skins::perfil(uuid), Msg::SPerfil);
                        }
                    }
                    Err(e) => self.armario.error = Some(e),
                }
            }

            // ---------- Explorar ----------
            Msg::XTexto(t) => {
                self.explorar.texto = t;
                self.explorar.buscando = true;
                // Busca sola cuando dejas de escribir 300 ms.
                self.explorar.turno += 1;
                let turno = self.explorar.turno;
                return Task::perform(tokio::time::sleep(Duration::from_millis(300)), move |_| Msg::XPausa(turno));
            }
            Msg::XPausa(turno) => {
                if turno == self.explorar.turno {
                    return self.update(Msg::XBuscar);
                }
                if turno == self.explorar.turno + 1_000_000 {
                    return self.update(Msg::NPackBuscar);
                }
            }
            Msg::XFuente(f) => {
                self.explorar.fuente = f;
                if f == "modrinth" && self.explorar.tipo == "shader" {
                    self.explorar.tipo = "mod";
                }
                return self.update(Msg::XBuscar);
            }
            Msg::XTipo(t) => {
                self.explorar.tipo = t;
                return self.update(Msg::XBuscar);
            }
            Msg::XInstancia(o) => {
                self.explorar.instancia = Some(o);
                // El filtro de cargador sigue al de la instancia elegida.
                self.explorar.cargador = self.destino_explorar().and_then(|i| i.cargador).and_then(|(c, _)| match c.as_str() {
                    "Fabric" => Some("fabric"),
                    "Forge" => Some("forge"),
                    "NeoForge" => Some("neoforge"),
                    "Quilt" => Some("quilt"),
                    _ => None,
                });
                self.explorar.instalados.clear();
                let deteccion = match self.destino_explorar() {
                    Some(inst) => {
                        let clave = inst.clave();
                        Task::perform(instalados_en(inst), move |v| Msg::XInstalados(clave.clone(), v))
                    }
                    None => Task::none(),
                };
                return Task::batch([deteccion, self.update(Msg::XBuscar)]);
            }
            Msg::XCargador(c) => {
                self.explorar.cargador = c;
                return self.update(Msg::XBuscar);
            }
            Msg::XPorVersion(b) => {
                self.explorar.por_version = b;
                return self.update(Msg::XBuscar);
            }
            Msg::XMarcar(id) => {
                if let Some(i) = self.explorar.marcados.iter().position(|(m, _)| m.id == id) {
                    self.explorar.marcados.remove(i);
                } else if let Some(m) = self.explorar.resultados.iter().find(|m| m.id == id).cloned() {
                    self.explorar.marcados.push((m, self.explorar.tipo));
                }
            }
            Msg::XDesmarcarTodo => self.explorar.marcados.clear(),
            Msg::XFavorito(i) => {
                if let Some(m) = self.explorar.resultados.get(i).cloned() {
                    if let Some(k) = self.aj.favoritos.iter().position(|f| f.id == m.id) {
                        self.aj.favoritos.remove(k);
                        if self.explorar.fuente == "favoritos" {
                            self.explorar.resultados.remove(i);
                        }
                    } else {
                        self.aj.favoritos.push(Favorito {
                            id: m.id.clone(),
                            modrinth: m.modrinth.clone(),
                            titulo: m.titulo.clone(),
                            autor: m.autor.clone(),
                            tipo: self.explorar.tipo.to_string(),
                            icono_url: m.icono_url.clone(),
                        });
                    }
                    self.guarda();
                }
            }
            Msg::XInstalarVarios => {
                let Some(inst) = self.explorar.instancia.as_ref().map(|o| o.clave.clone()) else {
                    self.explorar.error = Some("Elige en qué instancia instalar".into());
                    return Task::none();
                };
                let marcados = std::mem::take(&mut self.explorar.marcados);
                let fuente = self.fuente_instalar();
                for (m, tipo) in marcados {
                    self.encola(m, tipo, fuente, inst.clone());
                }
                return self.update(Msg::XSiguiente);
            }
            Msg::XInstalar(i) => {
                let Some(inst) = self.explorar.instancia.as_ref().map(|o| o.clave.clone()) else {
                    self.explorar.error = Some("Elige en qué instancia instalar".into());
                    return Task::none();
                };
                if let Some(m) = self.explorar.resultados.get(i).cloned() {
                    let (tipo, fuente) = (self.explorar.tipo, self.fuente_instalar());
                    // Si estaba marcado, ya no hace falta.
                    self.explorar.marcados.retain(|(x, _)| x.id != m.id);
                    self.encola(m, tipo, fuente, inst);
                }
                return self.update(Msg::XSiguiente);
            }
            Msg::XSiguiente => {
                // De una en una: Horizon no quiere dos instalaciones a la vez en la misma instancia.
                if self.explorar.cola.iter().any(|p| p.estado == EstadoPedido::Bajando) {
                    return Task::none();
                }
                let Some(i) = self.explorar.cola.iter().position(|p| p.estado == EstadoPedido::Esperando) else {
                    self.explorar.instalando = false;
                    return Task::none();
                };
                self.explorar.instalando = true;
                self.explorar.error = None;
                let pedido = self.explorar.cola[i].clone();
                let Some(inst) = self.instancia(&pedido.inst).cloned() else {
                    self.explorar.cola[i].estado = EstadoPedido::Bajando;
                    return self.update(Msg::XHecho(i, Err("Esa instancia ya no existe".into())));
                };
                self.explorar.cola[i].estado = EstadoPedido::Bajando;
                return Task::perform(tarea_pedido(pedido, inst), move |r| Msg::XHecho(i, r));
            }
            Msg::XHecho(i, r) => {
                if let Some(p) = self.explorar.cola.get_mut(i) {
                    match r {
                        Ok(t) => {
                            p.estado = EstadoPedido::Hecho(t);
                            let actual = self.explorar.instancia.as_ref().map(|o| o.clave.as_str());
                            if actual == Some(p.inst.as_str()) {
                                self.explorar.instalados.insert(p.m.id.to_lowercase());
                                if let Some(id) = &p.m.modrinth {
                                    self.explorar.instalados.insert(id.to_lowercase());
                                }
                            }
                        }
                        Err(e) => {
                            p.estado = EstadoPedido::Fallo(e);
                            self.explorar.ver_cola = true;
                        }
                    }
                }
                let quedan = self.explorar.cola.iter().any(|p| matches!(p.estado, EstadoPedido::Esperando | EstadoPedido::Bajando));
                if !quedan {
                    self.explorar.instalando = false;
                    let bien = self.explorar.cola.iter().filter(|p| matches!(p.estado, EstadoPedido::Hecho(_))).count();
                    let mal = self.explorar.cola.iter().filter(|p| matches!(&p.estado, EstadoPedido::Fallo(e) if e != "Quitada de la cola")).count();
                    let texto = match (bien, mal) {
                        (1, 0) => "Descarga lista".to_string(),
                        (b, 0) => format!("{b} descargas listas"),
                        (0, m) => format!("{m} descargas fallaron: mira la cola"),
                        (b, m) => format!("{b} descargas listas, {m} fallaron"),
                    };
                    self.avisa(texto, mal > 0);
                    return self.carga();
                }
                return self.update(Msg::XSiguiente);
            }
            Msg::XVerCola => self.explorar.ver_cola = !self.explorar.ver_cola,
            Msg::XLimpiarCola => {
                // Solo cuando no baja nada: así los índices de las tareas en marcha no cambian.
                if !self.explorar.instalando {
                    self.explorar.cola.clear();
                    self.explorar.ver_cola = false;
                }
            }
            Msg::XQuitarPedido(i) => {
                if self.explorar.cola.get(i).is_some_and(|p| p.estado == EstadoPedido::Esperando) {
                    // Se marca en vez de quitarla, para no mover los índices de la que está bajando.
                    self.explorar.cola[i].estado = EstadoPedido::Fallo("Quitada de la cola".into());
                }
            }
            Msg::XInstalados(clave, v) => {
                if self.explorar.instancia.as_ref().is_some_and(|o| o.clave == clave) {
                    self.explorar.instalados = v;
                }
            }
            Msg::ExplorarPara(clave, tipo) => {
                self.explorar.tipo = tipo;
                if self.explorar.fuente == "favoritos" || (self.explorar.fuente == "horizon" && tipo == "shader") {
                    self.explorar.fuente = "modrinth";
                }
                self.dlg = Some(Dlg::Explorar);
                if let Some(o) = self.opciones_instancias().into_iter().find(|o| o.clave == clave) {
                    return self.update(Msg::XInstancia(o));
                }
                return self.update(Msg::XBuscar);
            }
            Msg::XBuscar => {
                self.explorar.turno += 1;
                let turno = self.explorar.turno;
                self.explorar.error = None;
                if self.explorar.fuente == "favoritos" {
                    let tipo = self.explorar.tipo;
                    let v: Vec<Encontrado> = self
                        .aj
                        .favoritos
                        .iter()
                        .filter(|f| f.tipo == tipo)
                        .map(|f| Encontrado {
                            id: f.id.clone(),
                            modrinth: f.modrinth.clone(),
                            icono_url: f.icono_url.clone(),
                                            titulo: f.titulo.clone(),
                            resumen: String::new(),
                            autor: f.autor.clone(),
                            descargas: 0,
                            soporte: String::new(),
                            pagina: format!("https://modrinth.com/project/{}", f.id),
                        })
                        .collect();
                    return self.update(Msg::XResultados(Ok(v)));
                }
                self.explorar.buscando = true;
                self.explorar.hay_mas = false;
                self.explorar.pidiendo_mas = false;
                self.explorar.soporte.clear();
                self.explorar.relajado = None;
                let destino = self.destino_explorar();
                let clave = self.clave_explorar();
                let e = &self.explorar;
                // Lo ya buscado sale al momento; no se vacía la lista mientras llega lo nuevo.
                if let Some(v) = e.cache.get(&clave).cloned() {
                    let mas = e.fuente == "modrinth" && v.len() >= crate::modrinth::POR_PAGINA;
                    self.explorar.buscando = false;
                    self.explorar.hay_mas = mas;
                    return self.update(Msg::XResultados(Ok(v)));
                }
                let kind_cli = if e.tipo == "resourcepack" { "resource".to_string() } else { e.tipo.to_string() };
                let rust = destino.as_ref().is_some_and(|i| i.motor == Motor::Rust);
                if e.fuente == "horizon" {
                    let inst = destino.filter(|i| i.origen == Origen::Horizon).map(|i| i.id);
                    let b = motor::buscar(e.texto.clone(), kind_cli, "horizon".into(), inst);
                    return Task::perform(b, move |r| Msg::XTurno(turno, r));
                }
                // Siempre Modrinth directo (medio segundo, con logos y páginas). Para una instancia Rust se
                // pregunta además a Horizon, a la vez, cuánto funciona cada mod: llega después y se rellena.
                let version = destino.as_ref().and_then(|i| i.version.clone().filter(|_| i.motor == Motor::Java)).filter(|_| e.por_version);
                let cargador = if rust { None } else { e.cargador.map(str::to_string) };
                let b = crate::modrinth::buscar(e.texto.clone(), e.tipo.to_string(), cargador, version, 0);
                let mut tareas = vec![Task::perform(b, move |r| Msg::XTurno(turno, r))];
                if rust && e.tipo == "mod" {
                    let inst = destino.filter(|i| i.origen == Origen::Horizon).map(|i| i.id);
                    let s = motor::buscar(e.texto.clone(), kind_cli, "modrinth".into(), inst);
                    tareas.push(Task::perform(s, move |r| Msg::XSoporte(turno, r)));
                }
                return Task::batch(tareas);
            }
            Msg::XTurno(turno, r) => {
                if turno != self.explorar.turno {
                    return Task::none();
                }
                // Nada para esta versión (pasa mucho con snapshots): se busca en todas y se avisa.
                if matches!(&r, Ok(v) if v.is_empty()) && self.explorar.relajado.is_none() && self.explorar.fuente == "modrinth" {
                    let destino = self.destino_explorar();
                    let e = &self.explorar;
                    if let Some(v) = destino.as_ref().and_then(|i| i.version.clone().filter(|_| i.motor == Motor::Java)).filter(|_| e.por_version) {
                        let b = crate::modrinth::buscar(e.texto.clone(), e.tipo.to_string(), e.cargador.map(str::to_string), None, 0);
                        self.explorar.relajado = Some(v);
                        return Task::perform(b, move |r| Msg::XTurno(turno, r));
                    }
                }
                let mut r = r;
                if let Ok(v) = &mut r {
                    self.explorar.hay_mas = self.explorar.fuente == "modrinth" && v.len() >= crate::modrinth::POR_PAGINA;
                    pon_soporte(v, &self.explorar.soporte);
                    let clave = self.clave_explorar();
                    self.explorar.cache.insert(clave, v.clone());
                }
                return self.update(Msg::XResultados(r));
            }
            Msg::XSoporte(turno, r) => {
                if turno != self.explorar.turno {
                    return Task::none();
                }
                // Si Horizon falla, la lista sigue igual, solo sin «funciona / en parte».
                if let Ok(v) = r {
                    self.explorar.soporte = v.into_iter().filter(|m| !m.soporte.is_empty()).map(|m| (m.id, m.soporte)).collect();
                    pon_soporte(&mut self.explorar.resultados, &self.explorar.soporte);
                    let clave = self.clave_explorar();
                    if let Some(c) = self.explorar.cache.get_mut(&clave) {
                        pon_soporte(c, &self.explorar.soporte);
                    }
                }
            }
            Msg::XDesplazado(y) => {
                if y > 0.9 && self.explorar.hay_mas && !self.explorar.pidiendo_mas {
                    return self.update(Msg::XMas);
                }
            }
            Msg::XMas => {
                if self.explorar.pidiendo_mas || !self.explorar.hay_mas {
                    return Task::none();
                }
                self.explorar.pidiendo_mas = true;
                let turno = self.explorar.turno;
                let destino = self.destino_explorar();
                let e = &self.explorar;
                let rust = destino.as_ref().is_some_and(|i| i.motor == Motor::Rust);
                let version = destino
                    .as_ref()
                    .and_then(|i| i.version.clone().filter(|_| i.motor == Motor::Java))
                    .filter(|_| e.por_version && e.relajado.is_none());
                let cargador = if rust { None } else { e.cargador.map(str::to_string) };
                let b = crate::modrinth::buscar(e.texto.clone(), e.tipo.to_string(), cargador, version, e.resultados.len());
                return Task::perform(b, move |r| Msg::XMasR(turno, r));
            }
            Msg::XMasR(turno, r) => {
                if turno != self.explorar.turno {
                    return Task::none();
                }
                self.explorar.pidiendo_mas = false;
                match r {
                    Ok(mut v) => {
                        self.explorar.hay_mas = v.len() >= crate::modrinth::POR_PAGINA;
                        pon_soporte(&mut v, &self.explorar.soporte);
                        v.retain(|m| !self.explorar.resultados.iter().any(|x| x.id == m.id));
                        let con_url: Vec<(String, String)> = v
                            .iter()
                            .filter_map(|m| Some((m.modrinth.clone()?, m.icono_url.clone()?)))
                            .filter(|(id, _)| !self.logos.contains_key(id))
                            .collect();
                        self.explorar.resultados.extend(v);
                        let clave = self.clave_explorar();
                        self.explorar.cache.insert(clave, self.explorar.resultados.clone());
                        return logos_por_tandas(con_url);
                    }
                    Err(e) => self.explorar.error = Some(e),
                }
            }
            Msg::XResultados(r) => {
                self.explorar.buscando = false;
                match r {
                    Ok(v) => {
                        let con_url: Vec<(String, String)> = v
                            .iter()
                            .filter_map(|m| Some((m.modrinth.clone()?, m.icono_url.clone()?)))
                            .filter(|(id, _)| !self.logos.contains_key(id))
                            .collect();
                        let faltan: Vec<String> = v
                            .iter()
                            .filter(|m| m.icono_url.is_none())
                            .filter_map(|m| m.modrinth.clone())
                            .filter(|id| !self.logos.contains_key(id))
                            .collect();
                        self.explorar.resultados = v;
                        self.explorar.sel = std::env::var("RUSTCRAFT_ELEGIR").ok().and_then(|n| n.parse().ok());
                        let mut tareas = vec![logos_por_tandas(con_url)];
                        if !faltan.is_empty() {
                            tareas.push(Task::perform(crate::modrinth::logos(faltan), Msg::Logos));
                        }
                        // Pruebas sin ratón: pulsa Instalar sola una vez en el elegido.
                        if let Some(i) = self.explorar.sel.filter(|_| std::env::var_os("RUSTCRAFT_INSTALAR").is_some()) {
                            std::env::remove_var("RUSTCRAFT_INSTALAR");
                            tareas.push(self.update(Msg::XInstalar(i)));
                        }
                        return Task::batch(tareas);
                    }
                    Err(e) => self.explorar.error = Some(e),
                }
            }
            Msg::Logos(v) => self.logos.extend(v),
            Msg::ECapturas(v) => self.editar.capturas = v,
            Msg::EIdentificados(mapa) => {
                let mut faltan = Vec::new();
                for a in self.editar.mods.iter_mut() {
                    let base = if a.activo { a.ruta.clone() } else { a.ruta.clone() };
                    if let Some((id, nombre)) = mapa.get(&base) {
                        if !self.logos.contains_key(id) {
                            faltan.push(id.clone());
                        }
                        a.proyecto = Some((id.clone(), nombre.clone()));
                    }
                }
                faltan.sort();
                faltan.dedup();
                if !faltan.is_empty() {
                    return Task::perform(crate::modrinth::logos(faltan), Msg::Logos);
                }
            }
            // ---------- Iconos de Prism y cara de la cuenta ----------
            Msg::PrismIconos(r) => {
                if r.is_ok() {
                    self.cache.limpia();
                    self.rehace_iconos();
                }
            }
            Msg::Cara(h) => self.cara = h,
            Msg::Gatos(v) => self.gatos.extend(v),
            Msg::Orden(o) => {
                self.aj.orden = o;
                self.guarda();
            }
            Msg::Archivar(clave, si) => {
                let nombre = self.instancia(&clave).map(|i| i.nombre.clone()).unwrap_or_default();
                if si {
                    if !self.aj.archivadas.contains(&clave) {
                        self.aj.archivadas.push(clave.clone());
                    }
                    if self.sel.as_deref() == Some(clave.as_str()) {
                        self.sel = None;
                    }
                    self.avisa(format!("{nombre} archivada: está en el archivador de abajo a la derecha"), false);
                } else {
                    self.aj.archivadas.retain(|k| *k != clave);
                    self.sel = Some(clave);
                    self.avisa(format!("{nombre} vuelve a la ventana"), false);
                }
                self.guarda();
            }
            Msg::ArchivarCarpeta(c, si) => {
                let nombre = c.replace('/', " › ");
                if si {
                    if !self.aj.carpetas_archivadas.contains(&c) {
                        self.aj.carpetas_archivadas.push(c.clone());
                    }
                    // Que no quede como carpeta vacía en la ventana.
                    if !self.aj.carpetas.contains(&c) {
                        self.aj.carpetas.push(c.clone());
                    }
                    if self.sel.as_ref().and_then(|s| self.instancia(s)).is_some_and(|i| self.carpeta_archivada(&self.carpeta_de(i))) {
                        self.sel = None;
                    }
                    if self.carpeta_archivada(&self.carpeta_actual) {
                        self.carpeta_actual = padre_de(&c).to_string();
                    }
                    let n = self.en_grupo_archivado(&c).len();
                    self.avisa(format!("Grupo «{nombre}» archivado con {n} instancias: está en el archivador"), false);
                } else {
                    self.aj.carpetas_archivadas.retain(|k| *k != c);
                    self.avisa(format!("El grupo «{nombre}» vuelve a la ventana"), false);
                }
                self.guarda();
            }
            Msg::ColorCarpeta(c, color) => {
                if color == 0 {
                    self.aj.colores_carpeta.remove(&c);
                } else {
                    self.aj.colores_carpeta.insert(c, color);
                }
                self.guarda();
            }
            Msg::Atajo(a) => {
                if self.dlg.is_some() || self.borrar.is_some() || self.pide_nombre.is_some() {
                    return Task::none();
                }
                match a {
                    Atajo::Jugar => {
                        let en_bp = self.grande.then(|| self.estantes().get(self.bp.0).and_then(|e| e.1.get(self.bp.1)).map(|i| i.clave())).flatten();
                        if let Some(c) = en_bp.or_else(|| self.sel.clone()) {
                            return self.update(Msg::Jugar(c));
                        }
                    }
                    Atajo::Borrar => {
                        if let Some(c) = self.sel.clone().filter(|c| c != "h:horizon") {
                            return self.update(Msg::PedirBorrar(c));
                        }
                    }
                    Atajo::Buscar => return iced::widget::text_input::focus(iced::widget::text_input::Id::new("buscar")),
                    Atajo::Nueva => return self.update(Msg::NuevaEn(String::new())),
                    Atajo::Recargar => return self.update(Msg::Recargar),
                }
            }
            Msg::Duplicar(clave) => {
                if let Some(i) = self.instancia(&clave) {
                    let (id, nombre) = (i.id.clone(), format!("{} (copia)", i.nombre));
                    let n2 = nombre.clone();
                    return Task::perform(motor::duplicar(id, nombre), move |r| Msg::Hecho(r.map(|_| format!("Creada «{n2}»"))));
                }
            }
            Msg::AccesoDirecto(clave) => {
                if let Some(i) = self.instancia(&clave) {
                    let nombre = i.nombre.clone();
                    return Task::perform(motor::acceso_directo(i.id.clone()), move |r| {
                        Msg::Hecho(r.map(|_| format!("Acceso directo a «{nombre}» en el escritorio y en el menú")))
                    });
                }
            }
            Msg::AMascota(m) => {
                self.aj.mascota = m;
                self.aj.esquina = true;
                self.guarda();
            }
            Msg::AProxy(t) => self.proxy_texto = t,
            Msg::AGuardarProxy => {
                let t = self.proxy_texto.trim().to_string();
                if !t.is_empty() && !(t.starts_with("http://") || t.starts_with("https://") || t.starts_with("socks5://")) {
                    self.avisa("El proxy empieza por http://, https:// o socks5://", true);
                    return Task::none();
                }
                self.aj.proxy = t.clone();
                self.guarda();
                for v in ["ALL_PROXY", "HTTPS_PROXY", "HTTP_PROXY"] {
                    if t.is_empty() {
                        std::env::remove_var(v);
                    } else {
                        std::env::set_var(v, &t);
                    }
                }
                let valor = if t.is_empty() { "default".to_string() } else { t };
                return Task::perform(motor::ajuste_horizon("proxy".into(), valor), |r| Msg::Hecho(r.map(|_| "Proxy guardado".into())));
            }
            Msg::AHPodar(b) => {
                if let Some(a) = &mut self.ah {
                    a.podar = b;
                }
                return Task::perform(motor::ajuste_horizon("prune-old-versions".into(), if b { "on" } else { "off" }.into()), |r| {
                    Msg::Hecho(r.map(|_| "Guardado".into()))
                });
            }
            Msg::AHGuardarN(n) => {
                if let Some(a) = &mut self.ah {
                    a.guardar = n;
                }
                return Task::perform(motor::ajuste_horizon("keep-versions".into(), n.to_string()), |r| Msg::Hecho(r.map(|_| "Guardado".into())));
            }
            Msg::VaciarCache => {
                let dir = crate::modrinth::carpeta_cache();
                let _ = std::fs::remove_dir_all(&dir);
                self.logos.clear();
                self.explorar.cache.clear();
                self.avisa("Caché de logos vaciada (los iconos de Prism se quedan)", false);
            }
            // ---------- Servidores ----------
            Msg::SvNombre(t) => self.sv_nombre = t,
            Msg::SvDir(t) => self.sv_dir = t.trim().to_string(),
            Msg::SvAnadir => {
                let dir = self.sv_dir.trim().to_string();
                if dir.is_empty() {
                    return Task::none();
                }
                let nombre = if self.sv_nombre.trim().is_empty() { dir.clone() } else { self.sv_nombre.trim().to_string() };
                if !self.aj.servidores.iter().any(|s| s.direccion.eq_ignore_ascii_case(&dir)) {
                    self.aj.servidores.push(crate::servidores::Servidor { nombre, direccion: dir.clone() });
                    self.sincronizadas.clear();
                    self.guarda();
                }
                self.sv_nombre.clear();
                self.sv_dir.clear();
                let d2 = dir.clone();
                return Task::perform(crate::servidores::estado(dir), move |r| Msg::SvEstado(d2.clone(), r));
            }
            Msg::SvQuitar(i) => {
                if i < self.aj.servidores.len() {
                    let s = self.aj.servidores.remove(i);
                    self.guarda();
                    self.avisa(format!("{} quitado de tu lista (las instancias que ya lo tenían lo conservan)", s.nombre), false);
                }
            }
            Msg::SvRefrescar => {
                let t: Vec<Task<Msg>> = self
                    .aj
                    .servidores
                    .iter()
                    .map(|s| {
                        let d = s.direccion.clone();
                        Task::perform(crate::servidores::estado(d.clone()), move |r| Msg::SvEstado(d.clone(), r))
                    })
                    .collect();
                return Task::batch(t);
            }
            Msg::SvEstado(d, r) => {
                self.sv_estado.insert(d, r);
            }
            Msg::SvAplicar => {
                let lista = self.aj.servidores.clone();
                let mut total = 0;
                let mut fallos = 0;
                let carpetas: Vec<(String, PathBuf)> = self.todas().map(|i| (i.clave(), i.carpeta.clone())).collect();
                for (k, c) in carpetas {
                    match crate::servidores::sincroniza(&c, &lista) {
                        Ok(n) => {
                            total += n;
                            self.sincronizadas.insert(k);
                        }
                        Err(_) => fallos += 1,
                    }
                }
                let msg = if fallos > 0 {
                    format!("Añadidos {total} servidores; {fallos} instancias tienen el servers.dat dañado y no se han tocado")
                } else {
                    format!("Listo: {total} servidores añadidos a tus instancias")
                };
                self.avisa(msg, fallos > 0);
            }

            // ---------- Vistas, carpetas, arrastre y menús ----------
            Msg::Vista(v) => {
                self.aj.vista = v;
                self.guarda();
            }
            Msg::EntrarCarpeta(c) => self.carpeta_actual = c,
            Msg::Pulsar(clave) => {
                self.arrastre = Some(clave.clone());
                self.arrastrando = false;
                self.destino = None;
                return self.update(Msg::Elegir(clave));
            }
            Msg::Hover(h) => {
                self.encima = h.clone();
                if self.arrastre.is_some() {
                    if h.is_some() && h != self.arrastre {
                        self.arrastrando = true;
                    }
                    self.destino_inst = h.filter(|x| Some(x) != self.arrastre.as_ref());
                }
            }
            Msg::Destino(d) => {
                if self.arrastre.is_some() {
                    if d.is_some() {
                        self.arrastrando = true;
                    }
                    self.destino = d;
                }
            }
            Msg::Soltar => {
                self.armario.girando = None;
                let (c, d, moviendo) = (self.arrastre.take(), self.destino.take(), std::mem::take(&mut self.arrastrando));
                let sobre = self.destino_inst.take();
                // Soltar encima de otra instancia: se pone delante de ella (orden a mano) y en su carpeta.
                if let (Some(c), Some(t), true) = (c.clone(), sobre, moviendo) {
                    let carpeta = self.instancia(&t).map(|i| self.carpeta_de(i)).unwrap_or_default();
                    let mut orden: Vec<String> = self.aj.orden_manual.clone();
                    let mut todas: Vec<&Instancia> = self.visibles().collect();
                    self.ordena(&mut todas);
                    for i in todas {
                        if !orden.contains(&i.clave()) {
                            orden.push(i.clave());
                        }
                    }
                    orden.retain(|k| *k != c);
                    let pos = orden.iter().position(|k| *k == t).unwrap_or(orden.len());
                    orden.insert(pos, c.clone());
                    self.aj.orden_manual = orden;
                    self.aj.orden = Orden::Manual;
                    self.guarda();
                    return self.mover_tarea(&c, &carpeta);
                }
                if let (Some(c), Some(d), true) = (c, d, moviendo) {
                    return self.update(Msg::Mover(c, d));
                }
            }
            Msg::Mover(clave, destino) => {
                let nombre = self.instancia(&clave).map(|i| i.nombre.clone()).unwrap_or_default();
                let t = self.mover_tarea(&clave, &destino);
                let donde = if destino.is_empty() { "la raíz".to_string() } else { format!("«{}»", destino.replace('/', " › ")) };
                self.avisa(format!("{nombre} movida a {donde}"), false);
                return t;
            }
            Msg::AbrirMenu(que, pos) => self.menu = Some(Menu { pos, que, mover: false }),
            Msg::MoverDesplegar => {
                if let Some(m) = &mut self.menu {
                    m.mover = !m.mover;
                }
            }
            Msg::CerrarMenu => self.menu = None,
            Msg::Ventana(sz) => {
                self.ventana = sz;
                if self.grande {
                    let (f, c) = self.bp;
                    return self.update(Msg::BPElegir(f, c));
                }
            }
            Msg::PedirNombre(para) => {
                let valor = match &para {
                    ParaNombre::RenombrarCarpeta(c) => c.rsplit('/').next().unwrap_or("").to_string(),
                    ParaNombre::NuevaCarpeta(_) => String::new(),
                };
                self.pide_nombre = Some(PideNombre { para, valor });
            }
            Msg::NombreTexto(t) => {
                if let Some(p) = &mut self.pide_nombre {
                    p.valor = t.replace('/', "-");
                }
            }
            Msg::NombreAceptar => {
                let Some(p) = self.pide_nombre.take() else { return Task::none() };
                let nombre = p.valor.trim().to_string();
                if nombre.is_empty() {
                    return Task::none();
                }
                match p.para {
                    ParaNombre::NuevaCarpeta(padre) => {
                        let ruta = une(&padre, &nombre);
                        if !self.aj.carpetas.contains(&ruta) {
                            self.aj.carpetas.push(ruta);
                        }
                        self.guarda();
                    }
                    ParaNombre::RenombrarCarpeta(vieja) => {
                        let nueva = une(padre_de(&vieja), &nombre);
                        let pre = format!("{vieja}/");
                        let cambia = |c: &str| -> Option<String> {
                            if c == vieja {
                                Some(nueva.clone())
                            } else {
                                c.strip_prefix(&pre).map(|resto| format!("{nueva}/{resto}"))
                            }
                        };
                        let movidas: Vec<(String, String)> = self
                            .todas()
                            .filter_map(|i| cambia(&self.carpeta_de(i)).map(|d| (i.clave(), d)))
                            .collect();
                        for c in self.aj.carpetas.iter_mut() {
                            if let Some(d) = cambia(c) {
                                *c = d;
                            }
                        }
                        if let Some(d) = cambia(&self.carpeta_actual) {
                            self.carpeta_actual = d;
                        }
                        let t: Vec<Task<Msg>> = movidas.iter().map(|(c, d)| self.mover_tarea(c, d)).collect();
                        self.aj.carpetas.retain(|c| *c != vieja);
                        self.guarda();
                        return Task::batch(t);
                    }
                }
            }
            Msg::BorrarCarpeta(c) => {
                // Lo que había dentro sube a la carpeta de arriba; nada se borra.
                let padre = padre_de(&c).to_string();
                let pre = format!("{c}/");
                let dentro: Vec<String> = self
                    .todas()
                    .filter(|i| {
                        let k = self.carpeta_de(i);
                        k == c || k.starts_with(&pre)
                    })
                    .map(|i| i.clave())
                    .collect();
                let t: Vec<Task<Msg>> = dentro.iter().map(|k| self.mover_tarea(k, &padre)).collect();
                self.aj.carpetas.retain(|x| *x != c && !x.starts_with(&pre));
                if self.carpeta_actual == c || self.carpeta_actual.starts_with(&pre) {
                    self.carpeta_actual = padre;
                }
                self.guarda();
                self.avisa(format!("Carpeta «{}» quitada; lo que tenía está en la de arriba", c.replace('/', " › ")), false);
                return Task::batch(t);
            }
            Msg::NuevaEn(c) => {
                self.nueva = Nueva { carpeta: c, ..Default::default() };
                return self.update(Msg::Abrir(Dlg::Nueva));
            }

            // ---------- Nueva instancia: lo que se elige al crear ----------
            Msg::NIcono => return self.update(Msg::AbrirIcono(String::new())),
            Msg::NCarpeta(c) => self.nueva.carpeta = if c == RAIZ { String::new() } else { c },
            Msg::NMemoria(m) => self.nueva.memoria = Some(m),
            Msg::NSkin(sk) => self.nueva.skin = (!sk.is_empty()).then_some(sk),
            Msg::NPackTexto(t) => {
                self.nueva.texto_pack = t;
                self.explorar.turno += 1;
                let turno = self.explorar.turno;
                return Task::perform(tokio::time::sleep(Duration::from_millis(300)), move |_| Msg::XPausa(turno + 1_000_000));
            }
            Msg::NPackBuscar => {
                let b = crate::modrinth::buscar(self.nueva.texto_pack.clone(), "modpack".into(), None, None, 0);
                return Task::perform(b, Msg::NPacks);
            }
            Msg::NPacks(r) => match r {
                Ok(v) => {
                    let con_url: Vec<(String, String)> = v
                        .iter()
                        .filter_map(|m| Some((m.modrinth.clone()?, m.icono_url.clone()?)))
                        .filter(|(id, _)| !self.logos.contains_key(id))
                        .collect();
                    self.nueva.packs = v;
                    self.nueva.pack = None;
                    return logos_por_tandas(con_url);
                }
                Err(e) => self.nueva.error = Some(e),
            },
            Msg::NPack(i) => {
                self.nueva.pack = Some(i);
                self.nueva.archivo = None;
                if let Some(m) = self.nueva.packs.get(i) {
                    self.nueva.nombre = m.titulo.clone();
                }
            }
            Msg::NPackArchivo => {
                return Task::perform(
                    async {
                        let out = tokio::process::Command::new("zenity")
                            .args(["--file-selection", "--title=Paquete de mods", "--file-filter=Paquetes | *.mrpack *.zip"])
                            .output()
                            .await
                            .ok()?;
                        let r = String::from_utf8_lossy(&out.stdout).trim().to_string();
                        (!r.is_empty()).then_some(r)
                    },
                    Msg::NPackArchivoElegido,
                );
            }
            Msg::NPackArchivoElegido(f) => {
                if let Some(f) = f {
                    self.nueva.nombre = std::path::Path::new(&f).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    self.nueva.archivo = Some(f);
                    self.nueva.pack = None;
                }
            }

            // ---------- Selector de icono ----------
            Msg::AbrirIcono(clave) => {
                self.selector.para_nueva = clave.is_empty();
                if self.selector.texturas.is_empty() {
                    self.selector.texturas = self.cache.nombres_texturas();
                }
                let mut propios = Vec::new();
                if let Some(dir) = crate::prism::carpeta_iconos() {
                    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                        let f = e.path();
                        if f.extension().is_some_and(|x| ["png", "jpg", "jpeg", "webp"].contains(&x.to_string_lossy().to_lowercase().as_str())) {
                            if let Some(stem) = f.file_stem() {
                                if let Some(h) = crate::miniatura::reduce(&f, 64, 64, true) {
                                    propios.push((format!("prism:{}", stem.to_string_lossy()), h));
                                }
                            }
                        }
                    }
                }
                // Los de serie de Prism (bajados a la caché), detrás de los tuyos.
                for k in crate::prism_iconos::claves() {
                    if let Some(h) = crate::prism_iconos::archivo(&k).and_then(|f| crate::miniatura::reduce(&f, 64, 64, false)) {
                        propios.push((format!("prism:{k}"), h));
                    }
                }
                self.selector.clave = clave;
                self.selector.propios = propios;
                self.selector.ocupado = false;
                self.dlg = Some(Dlg::Icono);
                return self.update(Msg::IFiltro(self.selector.filtro.clone()));
            }
            Msg::IFiltro(f) => {
                let q = f.trim().to_lowercase().replace(' ', "_");
                let nombres: Vec<String> = if q.is_empty() {
                    SUGERIDOS.iter().map(|s| s.to_string()).filter(|n| self.selector.texturas.contains(n)).collect()
                } else {
                    self.selector.texturas.iter().filter(|n| n.contains(&q)).take(120).cloned().collect()
                };
                // Solo se preparan las que se ven (como mucho 120).
                self.selector.previas = nombres
                    .into_iter()
                    .filter_map(|n| self.cache.textura(&n).map(|h| (format!("tex:{n}"), h)))
                    .collect();
                self.selector.filtro = f;
            }
            Msg::IElegir(spec) => {
                if self.selector.para_nueva {
                    self.nueva.icono = (spec != "none").then_some(spec);
                    self.selector.para_nueva = false;
                    self.dlg = Some(Dlg::Nueva);
                    return Task::none();
                }
                let clave = self.selector.clave.clone();
                let Some(inst) = self.instancia(&clave).cloned() else { return Task::none() };
                // Horizon guarda bloques, colores y «none» él mismo: así se ve igual en su launcher.
                let a_horizon = match spec.as_str() {
                    s if inst.origen != Origen::Horizon || s.starts_with("prism:") => None,
                    "none" => Some("none".to_string()),
                    s if s.starts_with('#') => Some(s.to_string()),
                    s => s.strip_prefix("tex:").map(str::to_string),
                };
                match a_horizon {
                    Some(valor) => {
                        self.aj.iconos.remove(&clave);
                        self.guarda();
                        self.selector.ocupado = true;
                        return Task::perform(motor::icono(inst.id.clone(), valor), Msg::IHecho);
                    }
                    None => {
                        if spec == "none" {
                            self.aj.iconos.remove(&clave);
                        } else {
                            self.aj.iconos.insert(clave, spec);
                        }
                        self.guarda();
                        self.rehace_iconos();
                        self.dlg = None;
                    }
                }
            }
            Msg::IHecho(r) => {
                self.selector.ocupado = false;
                self.dlg = None;
                return self.update(Msg::Hecho(r.map(|_| "Icono cambiado".to_string())));
            }
            Msg::XSel(i) => {
                // Doble clic en un resultado: lo marca (o desmarca) para instalar.
                let doble = matches!(self.clic_mod, Some((j, t)) if j == i && t.elapsed() < Duration::from_millis(400));
                self.clic_mod = if doble { None } else { Some((i, Instant::now())) };
                self.explorar.sel = Some(i);
                if doble {
                    if let Some(id) = self.explorar.resultados.get(i).map(|m| m.id.clone()) {
                        return self.update(Msg::XMarcar(id));
                    }
                }
            }
        }
        Task::none()
    }

    fn abre_prueba(&mut self, que: &str) -> Task<Msg> {
        let (pantalla, pagina) = que.split_once(':').unwrap_or((que, ""));
        let clave = self.sel.clone().or_else(|| self.horizon.first().map(|i| i.clave())).unwrap_or_default();
        match pantalla {
            "nueva" => {
                let t = self.abre(Dlg::Nueva);
                match pagina {
                    "java" => Task::batch([t, self.update(Msg::NFuente(Fuente::Java))]),
                    "prism" => Task::batch([t, self.update(Msg::NFuente(Fuente::Prism))]),
                    "modpack" => Task::batch([t, self.update(Msg::NFuente(Fuente::Modpack))]),
                    _ => t,
                }
            }
            "editar" => {
                let pag = match pagina {
                    "mods" => PagE::Mods,
                    "mundos" => PagE::Mundos,
                    "ajustes" => PagE::Ajustes,
                    "registro" => PagE::Registro,
                    "capturas" => PagE::Capturas,
                    "skin" => PagE::Skin,
                    _ => PagE::Version,
                };
                let clave = std::env::var("RUSTCRAFT_INSTANCIA").unwrap_or(clave);
                self.sel = Some(clave.clone());
                self.update(Msg::AbrirEditar(clave, pag))
            }
            "ajustes" => {
                self.pag_a = match pagina {
                    "apariencia" => PagA::Apariencia,
                    "horizon" => PagA::Horizon,
                    "avanzado" => PagA::Avanzado,
                    _ => PagA::Rendimiento,
                };
                self.abre(Dlg::Ajustes)
            }
            "cuentas" => self.abre(Dlg::Cuentas),
            "servidores" => self.abre(Dlg::Servidores),
            "archivo" => self.abre(Dlg::Archivo),
            "vista" => {
                self.aj.vista = match pagina {
                    "carpetas" => Vista::Carpetas,
                    "lista" => Vista::Lista,
                    _ => Vista::Grupos,
                };
                self.carpeta_actual = std::env::var("RUSTCRAFT_CARPETA").unwrap_or_default();
                Task::none()
            }
            "menu" => {
                let que = match pagina {
                    "carpeta" => QueMenu::Carpeta(self.todas_carpetas().first().cloned().unwrap_or_default()),
                    "fondo" => QueMenu::Fondo,
                    _ => QueMenu::Instancia(clave),
                };
                self.menu = Some(Menu { pos: Point::new(420.0, 160.0), que, mover: pagina == "mover" });
                if pagina == "mover" {
                    if let Some(m) = &mut self.menu {
                        m.que = QueMenu::Instancia(self.sel.clone().unwrap_or_default());
                    }
                }
                Task::none()
            }
            "borrar" => {
                let clave = std::env::var("RUSTCRAFT_INSTANCIA").unwrap_or(clave);
                let t = self.update(Msg::PedirBorrar(clave));
                self.borrar_texto = pagina.to_string();
                t
            }
            "icono" => {
                let clave = std::env::var("RUSTCRAFT_INSTANCIA").unwrap_or(clave);
                self.selector.filtro = pagina.to_string();
                self.update(Msg::AbrirIcono(clave))
            }
            "armario" => self.abre(Dlg::Armario),
            "grande" => {
                // RUSTCRAFT_BP=estante,posición: elige esa tarjeta (para probar el desplazamiento).
                let bp = std::env::var("RUSTCRAFT_BP").ok().and_then(|v| {
                    let (f, c) = v.split_once(',')?;
                    Some((f.parse().ok()?, c.parse().ok()?))
                });
                match bp {
                    Some((f, c)) => self.update(Msg::BPElegir(f, c)),
                    None => Task::none(),
                }
            }
            "explorar" => {
                if let Ok(c) = std::env::var("RUSTCRAFT_INSTANCIA") {
                    if !c.is_empty() {
                        self.sel = Some(c);
                    }
                }
                self.explorar.texto = pagina.to_string();
                self.abre(Dlg::Explorar)
            }
            _ => Task::none(),
        }
    }

    fn abre(&mut self, d: Dlg) -> Task<Msg> {
        self.dlg = Some(d);
        match d {
            Dlg::Nueva => {
                self.armario.skins = skins::lista(self.aj.modelo_delgado);
                if self.nueva.fuente.is_none() {
                    let carpeta = std::mem::take(&mut self.nueva.carpeta);
                    self.nueva = Nueva { carpeta, ..Default::default() };
                    return self.update(Msg::NFuente(Fuente::Horizon));
                }
            }
            Dlg::Ajustes => return self.update(Msg::APagina(self.pag_a)),
            Dlg::Cuentas => return Task::perform(motor::cuentas(), Msg::Cuentas),
            Dlg::Servidores => return self.update(Msg::SvRefrescar),
            Dlg::Archivo => {}
            Dlg::Armario => {
                self.armario.skins = skins::lista(self.aj.modelo_delgado);
                if let Some(n) = std::env::var("RUSTCRAFT_ELEGIR").ok().and_then(|n| n.parse().ok()) {
                    self.armario.sel = Some(n);
                }
                if self.armario.figura.is_none() {
                    self.armario.guinada = -0.5;
                    self.armario.cabeceo = 0.15;
                }
                self.refigura();
                if let Some(uuid) = self.cuenta_activa().map(|c| c.uuid.clone()) {
                    self.armario.cargando = true;
                    return Task::perform(skins::perfil(uuid), Msg::SPerfil);
                }
            }
            Dlg::Explorar => {
                if self.explorar.instancia.is_none() {
                    let sel = self.sel.as_ref().and_then(|s| self.instancia(s)).or(self.horizon.first()).map(|i| i.clave());
                    if let Some(o) = sel.and_then(|c| self.opciones_instancias().into_iter().find(|o| o.clave == c)) {
                        return self.update(Msg::XInstancia(o));
                    }
                }
                if self.explorar.resultados.is_empty() {
                    return self.update(Msg::XBuscar);
                }
            }
            Dlg::Editar | Dlg::Icono => {}
        }
        Task::none()
    }

    fn carga_pagina(&mut self, pag: PagE) -> Task<Msg> {
        let Some(inst) = self.instancia(&self.editar.clave).cloned() else { return Task::none() };
        let dir = inst.carpeta.clone();
        match pag {
            PagE::Mods => {
                self.editar.mods = archivos(&dir.join("mods"), &["jar", "zip"]);
                self.editar.sel = std::env::var("RUSTCRAFT_ELEGIR").ok().and_then(|n| n.parse().ok());
                let jars: Vec<PathBuf> = self.editar.mods.iter().filter(|a| a.ruta.is_file()).map(|a| a.ruta.clone()).collect();
                let ident = Task::perform(crate::modrinth::identifica(jars), Msg::EIdentificados);
                if inst.origen == Origen::Horizon {
                    return Task::batch([ident, Task::perform(motor::mods_horizon(inst.id.clone()), Msg::EModsH)]);
                }
                return ident;
            }
            PagE::Recursos => self.editar.recursos = archivos(&dir.join("resourcepacks"), &["zip"]),
            PagE::Shaders => self.editar.shaders = archivos(&dir.join("shaderpacks"), &["zip"]),
            PagE::Mundos => {
                if inst.origen == Origen::Horizon {
                    return Task::perform(motor::mundos(inst.id.clone()), Msg::EMundos);
                }
                let saves = dir.join("saves");
                return Task::perform(
                    async move { Ok(motor::con_iconos(tokio::task::spawn_blocking(move || mundos_de_carpeta(&saves)).await.unwrap_or_default()).await) },
                    Msg::EMundos,
                );
            }
            PagE::Capturas => {
                let mut v: Vec<(std::time::SystemTime, PathBuf)> = std::fs::read_dir(dir.join("screenshots"))
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|e| e.path().extension().is_some_and(|x| x == "png"))
                    .map(|e| (e.metadata().and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), e.path()))
                    .collect();
                v.sort_by(|a, b| b.0.cmp(&a.0));
                // Las 24 más recientes, reducidas a miniatura en otro hilo.
                let rutas: Vec<PathBuf> = v.into_iter().take(24).map(|(_, p)| p).collect();
                self.editar.capturas.clear();
                return Task::perform(crate::miniatura::varias(rutas, 400, 225), Msg::ECapturas);
            }
            PagE::Ajustes => {
                if inst.origen == Origen::Horizon {
                    let id = inst.id.clone();
                    let mut t = vec![Task::perform(motor::lanzamiento(id.clone()), Msg::ELanz)];
                    if inst.motor == Motor::Java {
                        t.push(Task::perform(motor::java_de(id), Msg::EJava));
                    }
                    return Task::batch(t);
                }
            }
            PagE::Skin => {
                self.armario.skins = skins::lista(self.aj.modelo_delgado);
            }
            PagE::Registro => {
                let log = dir.join("logs/latest.log");
                self.editar.registro = match std::fs::read_to_string(&log) {
                    Ok(t) => {
                        let lineas: Vec<&str> = t.lines().collect();
                        let desde = lineas.len().saturating_sub(400);
                        lineas[desde..].join("\n")
                    }
                    Err(_) => "Todavía no hay registro: aparece al jugar esta instancia.".into(),
                };
            }
            PagE::Version => {
                if inst.motor == Motor::Rust && self.versiones.is_empty() {
                    return Task::perform(motor::versiones_horizon(), Msg::Versiones);
                }
            }
        }
        Task::none()
    }
}

fn archivos(dir: &std::path::Path, ext: &[&str]) -> Vec<Archivo> {
    let mut v: Vec<Archivo> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let ruta = e.path();
            let nombre = e.file_name().to_string_lossy().to_string();
            if nombre.starts_with('.') {
                return None;
            }
            let (base, activo) = match nombre.strip_suffix(".disabled") {
                Some(b) => (b.to_string(), false),
                None => (nombre.clone(), true),
            };
            let es_carpeta = ruta.is_dir();
            if !es_carpeta && !ext.iter().any(|x| base.to_lowercase().ends_with(&format!(".{x}"))) {
                return None;
            }
            Some(Archivo { nombre: base, ruta, activo, tamano: e.metadata().map(|m| m.len()).unwrap_or(0), proyecto: None })
        })
        .collect();
    v.sort_by(|a, b| a.nombre.to_lowercase().cmp(&b.nombre.to_lowercase()));
    v
}

/// Activa o desactiva un archivo como hace Prism: añadiendo o quitando `.disabled`.
fn activar(a: &mut Archivo, on: bool) -> Result<(), String> {
    if a.activo == on {
        return Ok(());
    }
    let nueva = if on {
        a.ruta.with_file_name(&a.nombre)
    } else {
        a.ruta.with_file_name(format!("{}.disabled", a.nombre))
    };
    std::fs::rename(&a.ruta, &nueva).map_err(|e| format!("No pude cambiarlo: {e}"))?;
    a.ruta = nueva;
    a.activo = on;
    Ok(())
}

fn mundos_de_carpeta(dir: &std::path::Path) -> Vec<Mundo> {
    let mut v: Vec<Mundo> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("level.dat").is_file())
        .map(|e| Mundo {
            icono: None,
            carpeta: e.path(),
            nombre: e.file_name().to_string_lossy().to_string(),
            modo: String::new(),
            version: String::new(),
            ultima_ms: e
                .path()
                .join("level.dat")
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_millis() as u64),
            tamano: 0,
        })
        .collect();
    v.sort_by(|a, b| b.ultima_ms.cmp(&a.ultima_ms));
    v
}

pub fn abre_externo(destino: &str) {
    let _ = std::process::Command::new("xdg-open")
        .arg(destino)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

/// Bloques e ítems que salen en el selector antes de buscar.
const SUGERIDOS: &[&str] = &[
    "grass_block_side", "dirt", "stone", "cobblestone", "oak_planks", "crafting_table_front", "furnace_front", "bookshelf",
    "tnt_side", "copper_block", "purpur_block", "amethyst_block", "diamond_block", "emerald_block", "gold_block", "netherrack",
    "end_stone", "obsidian", "glowstone", "sculk", "cherry_leaves", "diamond", "emerald", "ender_pearl", "ender_eye",
    "blaze_powder", "golden_apple", "apple", "cooked_beef", "bread", "cake", "diamond_sword", "diamond_pickaxe", "bow",
    "compass_00", "clock_00", "map", "book", "enchanted_book", "totem_of_undying", "nether_star", "heart_of_the_sea",
    "string", "slime_ball", "redstone", "lapis_lazuli", "music_disc_cat", "spyglass",
];

/// Convierte lo guardado («tex:grass_block», «#3fa7d6», «prism:viltrum») en un icono.
fn icono_de(spec: &str) -> crate::datos::Icono {
    use crate::datos::Icono;
    if let Some(t) = spec.strip_prefix("tex:") {
        Icono::Textura(t.to_string())
    } else if let Some(c) = spec.strip_prefix('#') {
        u32::from_str_radix(c, 16).map(Icono::Color).unwrap_or(Icono::Defecto)
    } else if let Some(k) = spec.strip_prefix("prism:") {
        Icono::Prism(k.to_string())
    } else {
        Icono::Defecto
    }
}

/// Nombre de la opción «sin carpeta» en los desplegables.
pub const RAIZ: &str = "(sin carpeta)";

pub fn padre_de(c: &str) -> &str {
    c.rsplit_once('/').map(|(p, _)| p).unwrap_or("")
}

pub fn une(padre: &str, nombre: &str) -> String {
    if padre.is_empty() {
        nombre.to_string()
    } else {
        format!("{padre}/{nombre}")
    }
}

fn iconos_barra(p: &Paleta) -> HashMap<&'static str, Handle> {
    ["anadir", "carpeta", "ajustes", "explorar", "recargar", "servidores", "armario", "gato", "archivo", "tele"]
        .into_iter()
        .map(|n| (n, iconos::barra(n, p.text, p.hl)))
        .collect()
}

fn iconos_carpeta(p: &Paleta, lado: u32) -> HashMap<u32, Handle> {
    iconos::COLORES_CARPETA
        .iter()
        .map(|c| (*c, iconos::carpeta(if *c == 0 { p.java } else { crate::tema::hex(*c) }, lado)))
        .collect()
}

/// Pone a cada resultado lo que dijo Horizon de él (si funciona entero o en parte).
fn pon_soporte(v: &mut [Encontrado], soporte: &HashMap<String, String>) {
    if soporte.is_empty() {
        return;
    }
    for m in v.iter_mut() {
        if let Some(s) = soporte.get(&m.id) {
            m.soporte = s.clone();
        }
    }
}

/// Logos en tandas de 6: los primeros salen enseguida, sin esperar al más lento de los 30.
fn logos_por_tandas(pares: Vec<(String, String)>) -> Task<Msg> {
    Task::batch(pares.chunks(6).map(|t| Task::perform(crate::modrinth::logos_de(t.to_vec()), Msg::Logos)))
}

/// Instala un pedido de la cola. En las instancias de Horizon lo hace Horizon (porta los mods Java a
/// las Rust y comprueba las sumas); en las de Prism, y los shaders, se baja directo de Modrinth con
/// sus dependencias obligatorias.
async fn tarea_pedido(p: Pedido, inst: Instancia) -> Result<String, String> {
    let (tipo, fuente) = (p.tipo, p.fuente.to_string());
    let id = p.m.id.clone();
    let modrinth_id = p.m.modrinth.clone().unwrap_or_else(|| p.m.id.clone());
    let mc = inst.version.clone().filter(|_| inst.motor == Motor::Java);
    let cargador = inst.cargador.as_ref().map(|c| c.0.to_lowercase());
    let por_horizon = inst.origen == Origen::Horizon && tipo != "shader";
    if tipo == "modpack" {
        motor::json(vec!["mods".into(), "install".into(), id, "--source".into(), fuente]).await?;
        return Ok("Creado como instancia nueva".into());
    }
    if !por_horizon && tipo == "mod" && cargador.as_deref().is_none_or(|c| c == "vanilla") {
        return Err("Esta instancia no tiene cargador de mods (Fabric, Forge…): el mod no haría nada".into());
    }
    if tipo == "datapack" && !por_horizon {
        return Err("Los data packs van dentro de un mundo: mételo en la carpeta datapacks del mundo".into());
    }
    if por_horizon {
        let r = motor::json(vec!["mods".into(), "install".into(), id, "--source".into(), fuente, "--instance".into(), inst.id.clone()]).await?;
        let n = r.as_array().map(|a| a.len()).unwrap_or(1);
        return Ok(if n > 1 { format!("Instalado, con {} dependencias", n - 1) } else { "Instalado".into() });
    }
    let archivos = crate::modrinth::instalar(modrinth_id, tipo.to_string(), inst.carpeta.clone(), mc, cargador).await?;
    Ok(if archivos.len() > 1 { format!("Instalado, con {} dependencias", archivos.len() - 1) } else { "Instalado".into() })
}

/// Qué proyectos tiene ya una instancia: los .jar/.zip de mods, recursos y shaders identificados en
/// Modrinth por su huella, más los que tiene Horizon apuntados.
async fn instalados_en(inst: Instancia) -> std::collections::HashSet<String> {
    let mut res = std::collections::HashSet::new();
    if inst.origen == Origen::Horizon {
        if let Ok(v) = motor::mods_horizon(inst.id.clone()).await {
            res.extend(v.into_iter().map(|m| m.proyecto.to_lowercase()));
        }
    }
    let mut archivos = Vec::new();
    for sub in ["mods", "resourcepacks", "shaderpacks"] {
        for e in std::fs::read_dir(inst.carpeta.join(sub)).into_iter().flatten().flatten() {
            let f = e.path();
            let ext = f.extension().and_then(|x| x.to_str()).unwrap_or("");
            if f.is_file() && matches!(ext, "jar" | "zip" | "disabled") {
                archivos.push(f);
            }
        }
    }
    for (_, (id, _)) in crate::modrinth::identifica(archivos).await {
        res.insert(id.to_lowercase());
    }
    res
}
