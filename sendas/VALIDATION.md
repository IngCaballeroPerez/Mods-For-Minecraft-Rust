# Validación

Revisado en Linux con Rust 1.97.0 el 2026-11-14, con los commits exactos de `compatibility.json` (Horizon
Nightly 2026-11-08, horizon-mixins 0.6.0) y el paquete de esta carpeta.

| Revisión | Resultado |
|---|---|
| `cargo fmt --all --check` | Pasa |
| `cargo clippy --workspace --all-targets --locked --offline -- -D warnings` | Pasa, sin avisos |
| `cargo test --workspace --locked --offline` | Pasan las 8 pruebas de las reglas |
| `python3 tools/horizon.py stage` | horizon-mixins acepta el paquete: encuentra los tres enganches y agrega el módulo |
| `python3 tools/horizon.py check` | El juego (`horizon-client`, con `--no-default-features`) compila con el mod, sin avisos |
| `cargo check -p horizon-client --bin horizon-client` sobre el código aplicado, con sonido y mandos activados | El juego compila con el mod también con las funciones que usa el launcher, sin avisos |
| `cargo check --profile test -p horizon-client --lib` sobre el código aplicado | Las pruebas del mod también compilan dentro del juego |
| `python3 tools/test_stage.py` | Solo cambió lo planeado (tres enganches y un módulo); aplicar dos veces da lo mismo; el clon de Horizon queda intacto |
| `python3 tools/package.py` | Genera `target/sendas-0.1.0.hnmod`, idéntico en cada corrida |
| `cargo horizon-mod check target/sendas-0.1.0.hnmod` | El revisor de paquetes de Horizon lo acepta como `native-mod` para 26.3 |

Además, un error puesto a propósito en un enganche hace fallar `check`, lo que confirma que la revisión sí
compila el código del mod.

## Lo que no se ha revisado

- **No se ha abierto el juego con el mod.** Faltan los recursos de Minecraft 26.3, que cada jugador prepara con
  su propia copia. Que `/senda` aparezca, responda y guarde dentro de un mundo es una prueba manual: los cuatro
  pasos de "Probarlo en tu juego" en el README.
- **No se ha construido el programa completo** (`tools/horizon.py build`) ni se han corrido las pruebas dentro
  del programa del juego (`tools/horizon.py test`); solo se comprobó que compilan.
- **No se ha construido con el launcher**, que usa su propio Rust 1.99.1. La revisión de aquí usó Rust 1.97.0.
- Solo se revisó el Horizon de `compatibility.json`. El mod declara que le basta la versión 163 de los puntos
  de enganche, pero no se probó en otras versiones de Horizon ni junto a otros mods.

## Notas sobre las herramientas

- `tools/audit` compara cada archivo ya asentado: impreso hasta que el texto deja de cambiar. Una sola impresión
  no basta, porque puede partir distinto una línea larga. También acepta un módulo en un archivo sin enganches.
- `tools/horizon.py test` corre las pruebas de la biblioteca del juego, que es donde vive el módulo del mod.
- `tools/package.py` escribe `game` y `hooks` en `horizon-native.json` e incluye la licencia del repositorio.
