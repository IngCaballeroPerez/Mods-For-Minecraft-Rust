// Un comando de un jugador en el mundo: si es /senda, lo atiende Sendas.
fn sendas_perform_command(
    game: &mut crate::game::Game<'_>,
    player: &crate::game::native_server::NativeServerPlayer,
    command: &str,
    ran: &mut bool,
) {
    use crate::game::sendas;
    use horizon_client::text::{self, Text};
    // Apagado en Opciones > Mods: no hace nada hasta que la siguiente
    // construcción lo deje fuera.
    if *ran || !sendas::is_ours(command) || !game.native_on(sendas::ID) {
        return;
    }
    *ran = true;
    let Some(world) = game.native_world_dir() else {
        game.native_feedback(Text::literal("Sendas: no hay un mundo abierto.").with_color(text::RED));
        return;
    };
    if let Some(reply) = sendas::run(&world, player.uuid, command) {
        let line = Text::literal(reply.text);
        game.native_feedback(if reply.error { line.with_color(text::RED) } else { line });
    }
}
