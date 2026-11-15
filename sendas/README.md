# Sendas

Mod nativo para [Horizon](https://github.com/horizon-rust/horizon): eliges una de tres sendas (Guerrero, Mago o
Arquero) y cada una guarda su propio avance. El nombre es provisional.

## Qué hace

- Aparece en **Opciones > Mods** como "Sendas 0.1.0". Ahí se enciende y se apaga.
- `/senda` dice qué senda sigues y tu nivel.
- `/senda guerrero`, `/senda mago` y `/senda arquero` eligen senda. El chat los completa con Tab.
- Cambiar de senda es libre y no pierde nada: cada senda guarda su nivel.
- La elección se guarda con el mundo, en `sendas/jugadores.txt` dentro de la carpeta del mundo, así que sigue ahí
  al volver a entrar y viaja con los respaldos del mundo.

Las habilidades, la energía, la barra en pantalla y los niveles llegan en las etapas siguientes del diseño.

## Probarlo en tu juego

Los mods nativos se construyen dentro del juego: el launcher de Horizon descarga su propio Rust y arma una copia
del juego con el mod adentro. La primera construcción tarda de 20 a 30 minutos; las siguientes son más rápidas.

En Ubuntu hacen falta estos paquetes una sola vez:

```sh
sudo apt install build-essential pkg-config libasound2-dev libudev-dev
```

Luego, con este repositorio descargado en tu equipo:

```sh
horizon-launcher native-build /ruta/a/Mods-For-Minecraft-Rust/sendas/mixins/sendas --trust-native-code
```

`--trust-native-code` es tu confirmación de que confías en el código: un mod nativo corre con los mismos permisos
que el juego.

Para comprobar que funciona:

1. Entra a un mundo y abre Opciones > Mods: debe aparecer "Sendas 0.1.0".
2. Escribe `/senda`: responde que aún no eliges senda.
3. Escribe `/senda mago`: responde "Ahora sigues la senda del Mago (nivel 1)."
4. Sal del mundo, vuelve a entrar y escribe `/senda`: responde "Tu senda: Mago, nivel 1."

## Cómo está armado

| Archivo | Qué es |
|---|---|
| `mixins/sendas/horizon-mixin.toml` | La lista de instrucciones: un módulo y tres enganches |
| `mixins/sendas/runtime.rs` | Las reglas del mod y sus pruebas. Se agrega al juego como `game::sendas` |
| `mixins/sendas/list.rs` | La fila en Opciones > Mods |
| `mixins/sendas/register.rs` | Registra `/senda` para que el chat lo complete |
| `mixins/sendas/command.rs` | Atiende `/senda` cuando un jugador lo escribe |
| `compatibility.json` | El Horizon en que se probó y la versión de puntos de enganche que necesita |
| `tools/` | Aplicar el mod sobre Horizon, revisar que compile, empaquetar |

El mod solo usa los puntos de enganche nativos de Horizon (`native_mods`, `native_register_commands`,
`native_perform_command`), que conservan su forma entre versiones. `compatibility.json` nombra la versión que
necesita (`"hooks": 163`), así que el launcher lo construye en cualquier Horizon que la tenga;
`horizon_revision` es el Horizon en que se probó (Nightly 2026-11-08).

## Revisar y empaquetar

Requisitos: Linux, Rust 1.95 o posterior, Python 3.12 o posterior y Git. Los comandos se corren dentro de esta
carpeta (`sendas/`).

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked          # las reglas, sin abrir el juego
```

Para revisar el mod sobre el código real de Horizon hacen falta dos clones junto a la carpeta del repositorio
(al lado de `Mods-For-Minecraft-Rust/`, no adentro): Horizon, con el commit de `compatibility.json`, y
horizon-mixins en su commit.

```sh
git clone https://github.com/horizon-rust/horizon ../../horizon
git clone https://github.com/horizon-rust/horizon-mixins ../../horizon-mixins
git -C ../../horizon-mixins checkout "$(python3 -c 'import json; print(json.load(open("compatibility.json"))["horizon_mixins_revision"])')"
cargo fetch --locked --manifest-path ../../horizon-mixins/Cargo.toml

python3 tools/horizon.py stage      # aplica el mod sobre una copia del código
cargo fetch --locked --manifest-path target/mixins/modded/Cargo.toml
python3 tools/horizon.py check      # revisa que el juego compile con el mod
python3 tools/horizon.py test       # corre las pruebas del mod dentro del juego
python3 tools/test_stage.py         # confirma que solo cambió lo planeado
python3 tools/package.py            # target/sendas-0.1.0.hnmod
```

Si los clones están en otro lugar, `--horizon RUTA` y la variable `HORIZON_MIXINS` dicen dónde. Estas
herramientas nunca tocan el clon de Horizon: trabajan sobre una copia en `target/`. Lo que se revisó y con qué
resultado está en [VALIDATION.md](VALIDATION.md).

## Origen y licencia

Las reglas, los nombres y los textos de Sendas son propios. No incluye código ni recursos de Minecraft ni de
otros mods. El mod se distribuye con la licencia del repositorio, [Apache-2.0](../LICENSE).

Las herramientas de `tools/` vienen de la plantilla oficial
[horizon-native-mod-template](https://github.com/horizon-rust/horizon-native-mod-template) y conservan su
licencia MIT ([tools/LICENSE](tools/LICENSE)).
