// Encima de todo (juego, menús y pantalla de título), salvo con F1 o si se apagó en Opciones > Mods.
fn jiyu_logo_draw(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch, gui_hidden: bool) {
    use crate::game::jiyu_logo as m;
    if gui_hidden || !game.native_on(m::ID) {
        return;
    }
    let [x, y] = m::sitio(batch.width, batch.height);
    game.native_fill(batch, [x, y, m::LADO, m::LADO], m::FONDO);
    for (tx, ty, largo) in m::tramos() {
        game.native_fill(batch, [x + tx as f32, y + ty as f32, largo as f32, 1.0], m::TINTA);
    }
}
