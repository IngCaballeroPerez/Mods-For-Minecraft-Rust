// Los comandos de Sendas en el árbol de comandos del mundo, para que el chat
// de cada jugador los complete con Tab.
fn sendas_register_commands(commands: &mut Vec<crate::game::native_server::NativeCommand>) {
    for usage in crate::game::sendas::USAGES {
        commands.push(crate::game::native_server::NativeCommand::new(usage));
    }
}
