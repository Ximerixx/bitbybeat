# bitbybeat

## чё это

аудиоанализ в реальном времени. слушает вход (микро / карта / монитор системы), режет на полосы  low/mid/high, вычисляет kick/snare/rythm, считает спектр, шлёт OSC хрень.

это порт моей хуйни из touchdesigner, тока на расте и с гуём. TD жрал пк как не в себя — вот это вместо него.

## нахуя

чтоб другой софт (qlc+, визуалы, что угодно с osc) жрал **ивенты**, а не сырой звук.

по умолчанию на `127.0.0.1:7700`. адреса те же по смыслу что в том патче: `/low` `/mid` `/high` `/kick` `/snare` `/rythm` `/spectralCentroid` `/fmsd` `/smsd` плюс доли `/trigger4k` `/trigger8k` … и т.д. хост/порт/какие каналы слать — в гуе.

нормального маленького «битдетектор → osc» как продукта всё равно нет, так што это оно, епт.

## как работает (коротко)

звук → кольцо → раз в N герц обсчёт (не на аудиорейте, по этому не сдыхает).

1. опц. компрессор (если ratio < 1 — он пики **раздувает**, не давит, это не баг крутилки а формула из дампа)
2. три фильтра-полосы, rms, порог, гейн, add
3. детекторы по порогу (кик с low, снейр с high, rythm с flux спектра)
4. адаптив: громкость зала(читай сигнала) → инерция → крутит гейны/пороги само
5. последний снимок пакуется в osc bundle

пресет `preset.ron` в текущей папке подхватывается при старте. лупа (иконка) — график вход/выход по блоку. схема тракта — кнопка «схема».

## сборка

нужен rust: [https://rustup.rs](https://rustup.rs)

### linux

```bash
sudo apt install -y build-essential pkg-config libasound2-dev pulseaudio-utils
cargo build --release
./target/release/bitbybeat
```

`pulseaudio-utils` — это `parec`/`pactl`, без них монитор выхода не схватишь (на pipewire тот же пакет через pipewire-pulse). другие дистры: заголовки alsa + pkg-config.

устройства без гуя:

```bash
./target/release/bitbybeat --list-devices
```

вход: pulse monitor (системный звук) или alsa-железо. многоканальная карта — галки каналов, пусто = всё в моно. после смены входа жмите **применить (restart)** или будешь крутить мёртвые крутилки и материться.

### windows

аудио через wasapi, либы ставить не надо. loopback виден как monitor-устройство в списке. `parec` там нет — pulse-список пустой, забейте.

без visual studio (норм путь):

```bash
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build --release
```

exe: `target\release\bitbybeat.exe`. если линкер орёт — поставь mingw gcc и его `bin` в PATH.

кросс с линуха в `.exe`:

```bash
sudo apt install -y mingw-w64
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu
```

файл: `target/x86_64-pc-windows-gnu/release/bitbybeat.exe`.

msvc тоже можно (`windows-msvc` + build tools), если любишь страдать. cpal ниже 0.18 на линухе паниковал на alsa — тут уже нормальный.