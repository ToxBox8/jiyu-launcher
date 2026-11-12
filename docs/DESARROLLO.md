# Jiyu Launcher · notas de desarrollo

Launcher de Minecraft con la estructura de Prism Launcher. Usa `horizon-launcher` como motor para las instancias de Horizon (Rust) y las de Java, y lee las instancias de Prism tal como están en su carpeta.

## Abrirlo

```sh
cargo build --release
./target/release/jiyu-launcher
```

Si `horizon-launcher` no está en el PATH ni en `~/Applications/horizon-launcher-linux-x86_64/`, indica su ruta con `JIYU_HORIZON=/ruta`.

## Qué hace cada parte

| Archivo | Qué hace |
|---|---|
| `src/motor.rs` | Llama a `horizon-launcher … --json`: instancias, versiones, cuentas, mods, mundos, ajustes y jugar |
| `src/prism.rs` | Lee `instance.cfg`, `mmc-pack.json` e `instgroups.json` de Prism; juega con `prism --launch` |
| `src/skins.rs` | El armario: skins locales, vista de frente y espalda, y skin y capas de la cuenta vía la API de Minecraft |
| `src/iconos.rs` | Iconos: texturas de tus archivos preparados de Horizon, dibujos originales para las claves de Prism, insignia y hexágono |
| `src/tema.rs` | Los temas Matugen, Prism Dark, Prism Bright y Jiyu Pulse, y el estilo de cada pieza |
| `src/ajustes.rs` | Preferencias (`~/.config/jiyu-launcher/ajustes.json`), perfiles de rendimiento y detección del PC |
| `src/app.rs` | Estado y mensajes |
| `src/vistas/` | Pantallas: principal, nueva instancia, editar, ajustes, cuentas, armario y explorar |

## Rendimiento

- Se dibuja por software (tiny-skia), sin abrir la GPU. Así no reserva memoria de la gráfica ni compite con el juego.
- Medido en reposo: unos 46 MB de memoria (25 MB son bibliotecas compartidas) y 0 ms de CPU en 10 s.
- Al jugar, el launcher se cierra. Un vigilante de unos 4 MB lo vuelve a abrir cuando cierras el juego (`jiyu-launcher --vigilar <pid> <carpeta>`).
- Las llamadas a Horizon y la lectura de disco se hacen fuera del hilo de la interfaz.

## Cuenta y armario

- El inicio de sesión lo hace Horizon (`account sign-in`) con su app, ya aprobada por Mojang.
- Para cambiar skin o capa, el armario pide a Horizon una sesión de un solo uso (`account session`), lee el token, borra el archivo al momento y se lo pasa a `curl` por la entrada estándar. El token no se guarda en ningún sitio y no aparece en la lista de procesos.

## Probar pantallas sin ratón

Sirve para hacer capturas:

```
RUSTCRAFT_TEMA=jiyu-pulse|matugen|prism-dark|prism-bright
RUSTCRAFT_ABRIR=nueva[:java|:prism|:modpack] | editar:mods|mundos|ajustes|skin|registro|capturas | ajustes:apariencia|horizon|red|avanzado | cuentas | armario | explorar:<texto> | servidores | archivo | grande | vista:grupos|carpetas|lista
RUSTCRAFT_INSTANCIA=h:mi-horizon | p:<carpeta de Prism>
RUSTCRAFT_ELEGIR=<fila que sale seleccionada>
RUSTCRAFT_INSTALAR=1   (en Explorar: pulsa «Instalar» sola en la fila elegida)
RUSTCRAFT_BP=<estante>,<tarjeta>   (con RUSTCRAFT_ABRIR=grande: elige esa tarjeta de Big Picture)
```

## Pendiente

- La app propia de Azure, cuando Mojang la apruebe.
- La capa del launcher: aparcada.
- No hay tipografía CJK incluida: los nombres de mods en chino salen con cuadros.
