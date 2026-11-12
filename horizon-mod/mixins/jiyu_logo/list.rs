// Su fila en Opciones > Mods (desde ahí se puede apagar).
fn jiyu_logo_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::jiyu_logo as m;
    mods.push(crate::game::native_hooks::NativeMod {
        id: m::ID.into(),
        title: format!("{} {}", m::NAME, m::VERSION),
        description: m::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
