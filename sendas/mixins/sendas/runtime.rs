// Sendas: tres sendas (clases) para Horizon: Guerrero, Mago y Arquero.
//
// Este archivo es a la vez la biblioteca de Cargo del mod (con sus pruebas) y
// el módulo que el paquete agrega al juego como `game::sendas`. Solo usa la
// biblioteca estándar de Rust, así que sus reglas se prueban sin abrir el juego.
//
// Aquí viven las reglas: qué sendas hay, qué pide cada comando, qué se guarda
// y cómo. Los archivos pequeños de al lado (`list.rs`, `register.rs`,
// `command.rs`) solo conectan estas reglas con los puntos de enganche.
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// El id del mod: su fila en Opciones > Mods y su carpeta de datos.
pub const ID: &str = "sendas";
pub const VERSION: &str = "0.1.0";
pub const DESCRIPTION: &str =
    "Elige una senda (Guerrero, Mago o Arquero) con /senda. Cada una guarda su propio nivel.";
/// La primera palabra de los comandos del mod.
pub const COMMAND: &str = "senda";
/// Los usos que el servidor registra, para que Tab los complete.
pub const USAGES: [&str; 4] = ["senda", "senda guerrero", "senda mago", "senda arquero"];
/// La carpeta del mod dentro de la carpeta del mundo, y su archivo.
pub const DIR: &str = "sendas";
pub const FILE: &str = "jugadores.txt";
/// La primera línea del archivo: su formato.
const HEADER: &str = "sendas 1";
/// El nivel más alto de una senda y las mejoras que admite cada casilla.
pub const MAX_LEVEL: u8 = 10;
pub const MAX_RANK: u8 = 3;

/// Una senda: la clase que el jugador sigue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Senda {
    Guerrero,
    Mago,
    Arquero,
}

impl Senda {
    pub const ALL: [Senda; 3] = [Senda::Guerrero, Senda::Mago, Senda::Arquero];

    /// Como se escribe en el comando y en el archivo.
    pub fn word(self) -> &'static str {
        match self {
            Senda::Guerrero => "guerrero",
            Senda::Mago => "mago",
            Senda::Arquero => "arquero",
        }
    }

    /// Como se le muestra al jugador.
    pub fn name(self) -> &'static str {
        match self {
            Senda::Guerrero => "Guerrero",
            Senda::Mago => "Mago",
            Senda::Arquero => "Arquero",
        }
    }

    /// La senda que nombra `word`, sin distinguir mayúsculas.
    pub fn from_word(word: &str) -> Option<Self> {
        let word = word.to_lowercase();
        Self::ALL.into_iter().find(|s| s.word() == word)
    }

    fn index(self) -> usize {
        match self {
            Senda::Guerrero => 0,
            Senda::Mago => 1,
            Senda::Arquero => 2,
        }
    }
}

/// El avance de un jugador en una senda.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Progress {
    /// Del 1 al `MAX_LEVEL`.
    pub level: u8,
    /// Experiencia hacia el siguiente nivel.
    pub xp: u32,
    /// Mejoras puestas en la ventaja fija y en las dos habilidades, de 0 a
    /// `MAX_RANK` cada una.
    pub ranks: [u8; 3],
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            ranks: [0; 3],
        }
    }
}

/// Lo que el mod guarda de un jugador: la senda que sigue y su avance en cada
/// una, así que cambiar de senda no pierde nada.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Player {
    pub active: Option<Senda>,
    pub progress: [Progress; 3],
}

/// Todos los jugadores de un mundo, por su identificador.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Roster {
    players: BTreeMap<u128, Player>,
}

impl Roster {
    /// Lee el texto del archivo. Un texto vacío es un mundo sin jugadores
    /// anotados. Cualquier línea que no se entienda es un error, con su
    /// número: así nunca se pierde un jugador en silencio.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut lines = text
            .lines()
            .enumerate()
            .map(|(i, line)| (i + 1, line.trim()))
            .filter(|(_, line)| !line.is_empty());
        let Some((_, header)) = lines.next() else {
            return Ok(Self::default());
        };
        if header != HEADER {
            return Err(format!(
                "el archivo de Sendas es de otra versión (empieza con \"{header}\")"
            ));
        }
        let mut players = BTreeMap::new();
        for (number, line) in lines {
            let (uuid, player) = parse_player(line)
                .ok_or_else(|| format!("no se entiende la línea {number} del archivo de Sendas"))?;
            players.insert(uuid, player);
        }
        Ok(Self { players })
    }

    /// El texto del archivo: el encabezado y una línea por jugador.
    pub fn to_text(&self) -> String {
        let mut text = format!("{HEADER}\n");
        for (uuid, player) in &self.players {
            let active = player.active.map_or("-", Senda::word);
            text.push_str(&format!("{uuid:032x} {active}"));
            for p in &player.progress {
                let [a, b, c] = p.ranks;
                text.push_str(&format!(" {}:{}:{a}:{b}:{c}", p.level, p.xp));
            }
            text.push('\n');
        }
        text
    }

    /// Lo guardado de un jugador; uno nuevo todavía no sigue ninguna senda.
    pub fn player(&self, uuid: u128) -> Player {
        self.players.get(&uuid).cloned().unwrap_or_default()
    }
}

/// Una línea de jugador: su identificador, su senda (o `-`) y tres avances.
fn parse_player(line: &str) -> Option<(u128, Player)> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let [uuid, active, guerrero, mago, arquero] = fields[..] else {
        return None;
    };
    if uuid.len() != 32 {
        return None;
    }
    let uuid = u128::from_str_radix(uuid, 16).ok()?;
    let active = match active {
        "-" => None,
        word => Some(Senda::from_word(word)?),
    };
    let progress = [
        parse_progress(guerrero)?,
        parse_progress(mago)?,
        parse_progress(arquero)?,
    ];
    Some((uuid, Player { active, progress }))
}

/// Un avance: `nivel:experiencia:mejora:mejora:mejora`.
fn parse_progress(field: &str) -> Option<Progress> {
    let parts: Vec<&str> = field.split(':').collect();
    let [level, xp, a, b, c] = parts[..] else {
        return None;
    };
    let level: u8 = level.parse().ok()?;
    let ranks = [a.parse().ok()?, b.parse().ok()?, c.parse().ok()?];
    if !(1..=MAX_LEVEL).contains(&level) || ranks.iter().any(|&r: &u8| r > MAX_RANK) {
        return None;
    }
    Some(Progress {
        level,
        xp: xp.parse().ok()?,
        ranks,
    })
}

/// Lo que pide un comando del mod.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Request {
    /// `/senda`: la senda que sigo.
    Status,
    /// `/senda mago`: seguir esa senda.
    Choose(Senda),
    /// `/senda` con algo que no es una senda.
    Unknown(String),
}

/// Si la línea (sin la barra) es un comando de Sendas.
pub fn is_ours(command: &str) -> bool {
    command.split_whitespace().next() == Some(COMMAND)
}

/// Lo que pide la línea (sin la barra); `None` cuando el comando es de otro.
pub fn parse(command: &str) -> Option<Request> {
    let mut words = command.split_whitespace();
    if words.next() != Some(COMMAND) {
        return None;
    }
    let rest: Vec<&str> = words.collect();
    Some(match rest[..] {
        [] => Request::Status,
        [word] => Senda::from_word(word)
            .map_or_else(|| Request::Unknown(word.to_owned()), Request::Choose),
        _ => Request::Unknown(rest.join(" ")),
    })
}

/// Una línea de respuesta al jugador; las de error van en rojo.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reply {
    pub text: String,
    pub error: bool,
}

impl Reply {
    fn said(text: String) -> Self {
        Self { text, error: false }
    }
    fn failed(text: String) -> Self {
        Self { text, error: true }
    }
}

/// Aplica lo pedido por el jugador `uuid`. Devuelve la respuesta y si hay algo
/// nuevo que guardar.
pub fn apply(roster: &mut Roster, uuid: u128, request: Request) -> (Reply, bool) {
    let mut player = roster.player(uuid);
    match request {
        Request::Status => {
            let reply = match player.active {
                Some(senda) => format!(
                    "Tu senda: {}, nivel {}.",
                    senda.name(),
                    player.progress[senda.index()].level
                ),
                None => {
                    "Aún no eliges senda. Escribe /senda guerrero, /senda mago o /senda arquero."
                        .to_owned()
                }
            };
            (Reply::said(reply), false)
        }
        Request::Choose(senda) => {
            let level = player.progress[senda.index()].level;
            if player.active == Some(senda) {
                let reply = format!("Ya sigues la senda del {} (nivel {level}).", senda.name());
                return (Reply::said(reply), false);
            }
            player.active = Some(senda);
            roster.players.insert(uuid, player);
            let reply = format!(
                "Ahora sigues la senda del {} (nivel {level}).",
                senda.name()
            );
            (Reply::said(reply), true)
        }
        Request::Unknown(word) => (
            Reply::failed(format!(
                "No existe la senda \"{word}\". Opciones: guerrero, mago, arquero."
            )),
            false,
        ),
    }
}

/// El archivo del mod en la carpeta del mundo `world`.
pub fn file_path(world: &Path) -> PathBuf {
    world.join(DIR).join(FILE)
}

/// Lee el archivo; si aún no existe, nadie ha elegido senda en este mundo.
pub fn load(file: &Path) -> Result<Roster, String> {
    match fs::read_to_string(file) {
        Ok(text) => Roster::parse(&text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Roster::default()),
        Err(error) => Err(format!("no se pudo leer el archivo de Sendas: {error}")),
    }
}

/// Escribe el archivo completo en uno temporal y lo pone en su lugar de un
/// solo paso, para que un corte a medias no deje el archivo roto.
pub fn save(file: &Path, roster: &Roster) -> Result<(), String> {
    let failed =
        |error: std::io::Error| format!("no se pudo guardar el archivo de Sendas: {error}");
    if let Some(dir) = file.parent() {
        fs::create_dir_all(dir).map_err(failed)?;
    }
    let temporary = file.with_extension("tmp");
    fs::write(&temporary, roster.to_text()).map_err(failed)?;
    fs::rename(&temporary, file).map_err(failed)
}

/// Un comando del jugador `uuid` en el mundo de la carpeta `world`: lee lo
/// guardado, aplica lo pedido y guarda si cambió. Si no se pudo guardar, la
/// respuesta es el error, nunca un "listo" que no ocurrió. Sin respuesta
/// cuando el comando es de otro.
pub fn run(world: &Path, uuid: u128, command: &str) -> Option<Reply> {
    let request = parse(command)?;
    let file = file_path(world);
    let mut roster = match load(&file) {
        Ok(roster) => roster,
        Err(error) => return Some(Reply::failed(format!("Sendas: {error}."))),
    };
    let (reply, changed) = apply(&mut roster, uuid, request);
    if changed && let Err(error) = save(&file, &roster) {
        return Some(Reply::failed(format!("Sendas: {error}.")));
    }
    Some(reply)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ANA: u128 = 0x0123_4567_89ab_cdef_0123_4567_89ab_cdef;
    const BETO: u128 = 7;

    /// Una carpeta de mundo vacía y propia de la prueba `name`.
    fn world(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sendas-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn names_the_three_sendas() {
        assert_eq!(Senda::from_word("guerrero"), Some(Senda::Guerrero));
        assert_eq!(Senda::from_word("MAGO"), Some(Senda::Mago));
        assert_eq!(Senda::from_word("Arquero"), Some(Senda::Arquero));
        assert_eq!(Senda::from_word("paladin"), None);
        for senda in Senda::ALL {
            assert_eq!(Senda::from_word(senda.word()), Some(senda));
            assert!(USAGES.contains(&format!("{COMMAND} {}", senda.word()).as_str()));
        }
    }

    #[test]
    fn reads_what_a_command_asks() {
        assert_eq!(parse("senda"), Some(Request::Status));
        assert_eq!(parse("senda mago"), Some(Request::Choose(Senda::Mago)));
        assert_eq!(
            parse("  senda   arquero "),
            Some(Request::Choose(Senda::Arquero))
        );
        assert_eq!(parse("senda brujo"), Some(Request::Unknown("brujo".into())));
        assert_eq!(
            parse("senda mago ya"),
            Some(Request::Unknown("mago ya".into()))
        );
        assert_eq!(parse("sendas"), None);
        assert_eq!(parse("say senda"), None);
        assert!(is_ours("senda mago") && !is_ours("sendas") && !is_ours(""));
    }

    #[test]
    fn choosing_is_saved_and_repeating_changes_nothing() {
        let mut roster = Roster::default();
        let (reply, changed) = apply(&mut roster, ANA, Request::Status);
        assert!(!changed && !reply.error && reply.text.starts_with("Aún no eliges senda"));
        let (reply, changed) = apply(&mut roster, ANA, Request::Choose(Senda::Guerrero));
        assert!(changed);
        assert_eq!(reply.text, "Ahora sigues la senda del Guerrero (nivel 1).");
        let (reply, changed) = apply(&mut roster, ANA, Request::Choose(Senda::Guerrero));
        assert!(!changed);
        assert_eq!(reply.text, "Ya sigues la senda del Guerrero (nivel 1).");
        let (reply, changed) = apply(&mut roster, ANA, Request::Status);
        assert!(!changed);
        assert_eq!(reply.text, "Tu senda: Guerrero, nivel 1.");
        // Otro jugador no hereda nada.
        assert_eq!(roster.player(BETO), Player::default());
        let (reply, changed) = apply(&mut roster, ANA, Request::Unknown("brujo".into()));
        assert!(reply.error && !changed && reply.text.contains("brujo"));
    }

    #[test]
    fn each_senda_keeps_its_own_progress() {
        let mut roster = Roster::default();
        apply(&mut roster, ANA, Request::Choose(Senda::Mago));
        roster.players.get_mut(&ANA).unwrap().progress[Senda::Mago.index()] = Progress {
            level: 4,
            xp: 250,
            ranks: [1, 2, 0],
        };
        let (reply, _) = apply(&mut roster, ANA, Request::Choose(Senda::Arquero));
        assert_eq!(reply.text, "Ahora sigues la senda del Arquero (nivel 1).");
        let (reply, _) = apply(&mut roster, ANA, Request::Choose(Senda::Mago));
        assert_eq!(reply.text, "Ahora sigues la senda del Mago (nivel 4).");
    }

    #[test]
    fn the_file_round_trips() {
        let mut roster = Roster::default();
        apply(&mut roster, ANA, Request::Choose(Senda::Mago));
        apply(&mut roster, BETO, Request::Choose(Senda::Arquero));
        roster.players.get_mut(&ANA).unwrap().progress[1] = Progress {
            level: 10,
            xp: 4500,
            ranks: [3, 3, 3],
        };
        let text = roster.to_text();
        assert_eq!(
            text,
            "sendas 1\n\
             00000000000000000000000000000007 arquero 1:0:0:0:0 1:0:0:0:0 1:0:0:0:0\n\
             0123456789abcdef0123456789abcdef mago 1:0:0:0:0 10:4500:3:3:3 1:0:0:0:0\n"
        );
        assert_eq!(Roster::parse(&text), Ok(roster));
        assert_eq!(Roster::parse(""), Ok(Roster::default()));
        assert_eq!(Roster::parse("\n  \n"), Ok(Roster::default()));
    }

    #[test]
    fn a_file_it_does_not_understand_is_an_error_not_a_loss() {
        assert!(
            Roster::parse("sendas 2\n")
                .unwrap_err()
                .contains("otra versión")
        );
        let line = "0123456789abcdef0123456789abcdef mago 1:0:0:0:0 1:0:0:0:0 1:0:0:0:0";
        assert!(Roster::parse(&format!("sendas 1\n{line}\n")).is_ok());
        for broken in [
            "123 mago 1:0:0:0:0 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef brujo 1:0:0:0:0 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef mago 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef mago 0:0:0:0:0 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef mago 11:0:0:0:0 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef mago 1:0:4:0:0 1:0:0:0:0 1:0:0:0:0",
            "0123456789abcdef0123456789abcdef mago 1:x:0:0:0 1:0:0:0:0 1:0:0:0:0",
        ] {
            let error = Roster::parse(&format!("sendas 1\n{line}\n{broken}\n")).unwrap_err();
            assert!(error.contains("línea 3"), "{broken}: {error}");
        }
    }

    #[test]
    fn the_senda_is_still_there_after_leaving_the_world() {
        let world = world("stays");
        assert_eq!(run(&world, ANA, "say hola"), None);
        let reply = run(&world, ANA, "senda").unwrap();
        assert!(reply.text.starts_with("Aún no eliges senda"));
        assert!(!file_path(&world).exists(), "nothing to save yet");
        let reply = run(&world, ANA, "senda mago").unwrap();
        assert_eq!(reply.text, "Ahora sigues la senda del Mago (nivel 1).");
        // Cada comando lee el archivo de nuevo, como al volver al mundo.
        let reply = run(&world, ANA, "senda").unwrap();
        assert_eq!(reply.text, "Tu senda: Mago, nivel 1.");
        assert!(
            run(&world, BETO, "senda")
                .unwrap()
                .text
                .starts_with("Aún no eliges senda")
        );
        assert!(!file_path(&world).with_extension("tmp").exists());
        fs::remove_dir_all(&world).unwrap();
    }

    #[test]
    fn a_broken_file_is_reported_and_left_alone() {
        let world = world("broken");
        fs::create_dir_all(world.join(DIR)).unwrap();
        fs::write(file_path(&world), "sendas 9\n").unwrap();
        let reply = run(&world, ANA, "senda mago").unwrap();
        assert!(
            reply.error && reply.text.contains("otra versión"),
            "{}",
            reply.text
        );
        assert_eq!(fs::read_to_string(file_path(&world)).unwrap(), "sendas 9\n");
        fs::remove_dir_all(&world).unwrap();
    }
}
