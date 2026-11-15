// La fila de Sendas en Opciones > Mods.
fn sendas_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    use crate::game::sendas;
    mods.push(crate::game::native_hooks::NativeMod {
        id: sendas::ID.into(),
        title: format!("Sendas {}", sendas::VERSION),
        description: sendas::DESCRIPTION.into(),
        keys: Vec::new(),
    });
}
