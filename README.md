# bitbybeat

Конфигурируемый аудиоанализатор реального времени с GUI → OSC. Rust-порт TouchDesigner-прототипа
**Analysis 2.2**: раскладывает вход на полосы (low/mid/high), детектит kick/snare/rythm, считает
спектральные фичи (centroid, flux/fms/smsd) и адаптивно подстраивает гейны/пороги, а результат шлёт
по OSC (по умолчанию `127.0.0.1:7700`).

Полная документация тракта и решений — в каталоге [`md_plans/`](md_plans/) (00 — обзорная карта).

---

## Возможности

- **Вход:** аудиоустройства (ALSA/WASAPI через cpal) **или** захват системного звука (loopback/monitor)
  на Linux через PipeWire/PulseAudio (`parec`).
- **Выбор каналов** для многоканальных устройств (микшер/интерфейс): отметить нужные 1–2 канала.
- **Полосы** с биквад-фильтрами, RMS, сглаживанием; тумблеры включения/выключения.
- **Детекторы** kick/snare/rythm (rythm — onset по спектральному flux), счётчики долей 4/8/16.
- **Адаптивное управление**: RMS входа → lag → мапперы/сигмоиды → гейны полос и пороги.
- **GUI (egui):** живые метры, спектр, большие лампы сигналов, редактор сигмоид с превью и
  всплывающими окнами, подсказки к параметрам, пресеты (`.ron`).
- **OSC-выход** на порт 7700 (bundle или отдельные сообщения).

---

## Требования

- **Rust** (stable, edition 2021). Установка: <https://rustup.rs>
- Системные библиотеки (см. ниже по ОС).

### Linux (Debian/Ubuntu и производные)

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libasound2-dev
# для захвата системного звука (monitor) и списка источников:
sudo apt install -y pulseaudio-utils   # даёт parec/pactl (в PipeWire-системах — через pipewire-pulse)
```

> На других дистрибутивах: нужны заголовки ALSA (`alsa-lib-devel` / `alsa-lib`), `pkg-config`
> и утилиты PulseAudio (`pulseaudio-utils` / `pipewire-pulse`).

### Windows

Аудио идёт через **WASAPI**, системные пакеты не нужны. Захват системного звука — через WASAPI
loopback (устройства-мониторы видны прямо в списке cpal); `parec`/`pactl` там отсутствуют, pulse-
дропдаун будет пустым — это нормально.

Есть два тулчейна — **выбирай GNU, если не хочешь ставить Visual Studio:**

- **GNU (mingw-w64), без Visual Studio** — рекомендуется, если VS/Build Tools не нужны. См. раздел
  [«Сборка под Windows (GNU/mingw)»](#сборка-под-windows-gnumingw).
- **MSVC** — `rustup default stable-x86_64-pc-windows-msvc` + Visual Studio Build Tools (C++).
  Ставить только если специально хочешь MSVC.

---

## Сборка

```bash
git clone <repo> bitbybeat
cd bitbybeat
cargo build --release
```

Бинарь: `target/release/bitbybeat`.

## Запуск

```bash
cargo run --release
# или напрямую:
./target/release/bitbybeat
```

Диагностика устройств без окна (полезно, чтобы найти нужный вход/каналы):

```bash
./target/release/bitbybeat --list-devices
```

## Сборка под Windows (GNU/mingw)

Без Visual Studio — только mingw-w64 GCC. Два способа:

### Вариант А. Кросс-компиляция с Linux (собираем `.exe`, не заходя в винду)

В репозитории уже лежит `.cargo/config.toml` с линкером mingw и добавлен target — нужен лишь пакет
mingw-w64:

```bash
sudo apt install -y mingw-w64
rustup target add x86_64-pc-windows-gnu   # если ещё не добавлен
cargo build --release --target x86_64-pc-windows-gnu
```

Готовый бинарь: `target/x86_64-pc-windows-gnu/release/bitbybeat.exe` — копируешь на Windows и запускаешь.

### Вариант Б. Нативно на Windows через git bash + mingw

```bash
# ставим GNU-тулчейн Rust (несёт свой линкер, VS не нужен)
rustup toolchain install stable-x86_64-pc-windows-gnu
rustup default stable-x86_64-pc-windows-gnu
cargo build --release
```

Если линковка жалуется на отсутствие компилятора C — поставь mingw-w64 (например через MSYS2:
`pacman -S mingw-w64-x86_64-gcc`) и добавь его `bin` в `PATH`. Бинарь: `target\release\bitbybeat.exe`.

> Примечание: GNU-сборка тянет несколько DLL рантайма mingw. `rustup`-тулчейн `windows-gnu` обычно
> самодостаточен; при кросс-сборке с Linux всё статически линкуется штатно.

## Установка (Linux, опционально)

```bash
# положить бинарь в PATH пользователя
install -Dm755 target/release/bitbybeat ~/.local/bin/bitbybeat
```

При желании — ярлык `~/.local/share/applications/bitbybeat.desktop`:

```ini
[Desktop Entry]
Type=Application
Name=bitbybeat
Comment=Audio analyzer → OSC
Exec=bitbybeat
Terminal=false
Categories=AudioVideo;Audio;
```

---

## Использование

### Выбор входа (левая панель)

1. **pulse source (monitor)** — захват системного звука. Выбери `🔁 Monitor …` нужного выхода
   (например «Monitor of … Analog»); идёт через `parec`, стабильно и без паник ALSA. Это
   приоритетный путь на Linux/PipeWire.
2. **ALSA device** — конкретное железо (микрофон/интерфейс). Активен, когда pulse-источник не выбран.
   Для многоканальных устройств появляется ряд **тумблеров каналов** — отметь нужные 1–2 (пусто =
   моно-даунмикс всех). Число каналов показано в метке устройства `[Nch]`.
3. Кнопка **«применить (restart)»** переоткрывает вход, **«обновить списки»** перечитывает устройства.
4. Галочка **«предпочитать мониторы»** авто-выбирает первый monitor.

> **PipeWire-патчбей (qpwgraph и т.п.):** при захвате через pulse наша нода `bitbybeat` появляется в
> графе — можно вручную протянуть нужные capture-порты источника/микшера в её `input_FL/FR`. Это
> самый точный способ выбора каналов на PipeWire.

### Центральная панель

- **Полосы** (side-by-side): частота среза/rolloff/Q, pregain, threshold, gain, add, smooth. При
  включённом адаптиве `gain` полос неактивен (правится автоматом).
- **Детекторы**: порог + retrigger. При адаптиве пороги kick/snare правятся сигмоидами (неактивны),
  rythm-порог всегда ручной (шкала 0..1 по нормированному flux).
- **Адаптивное управление**: вкл/выкл, corr-gain, lag (stateful), мапперы гейнов и порогов + сигмоиды
  (кнопка «⧉ окно» открывает крупный график; голубая точка — вход, жёлтая — выход/порог).

### Пресеты

Внизу левой панели — путь к файлу (`preset.ron`) и кнопки сохранить/загрузить. При старте программа
подхватывает `preset.ron` из текущего каталога, если он есть.

---

## OSC-выход

По умолчанию UDP `127.0.0.1:7700`, один bundle. Адреса и значения (float):

| Адрес | Смысл |
|---|---|
| `/low` `/mid` `/high` | уровни полос |
| `/kick` | триггер кика |
| `/snare` | гейт снейра |
| `/rythm` | onset-триггер ритма |
| `/spectralCentroid` | спектральный центроид |
| `/fmsd` `/smsd` | спектральные меры энергии |
| `/trigger4k` `/trigger8k` `/trigger16k` | счётчики долей кика (4/8/16) |
| `/trigger4s` `/trigger8s` `/trigger16s` | счётчики долей снейра |
| `/dsprms` | RMS DSP-ветви (только если включён RMS-power) |

Хост/порт/режим bundle настраиваются в GUI (секция OSC).

---

## Кросс-платформенность и заметки

- **Windows без Visual Studio:** используй GNU-тулчейн (mingw-w64) — можно даже кросс-собирать `.exe`
  прямо с Linux (см. [«Сборка под Windows (GNU/mingw)»](#сборка-под-windows-gnumingw)). Код
  кросс-платформенный: на Windows аудио — WASAPI, а Linux-специфичные `parec`/`pactl` просто не
  вызываются (pulse-список будет пуст).
- **cpal 0.18+** обязателен: в 0.15 были паники ALSA-таймстампов (`get_htstamp`), из-за которых
  подвисал выбор устройств. Здесь уже 0.18.
- Подробности алгоритмов, отклонения от TD и калибровка — в `md_plans/11_implementation_notes.md`.

## Troubleshooting

- **Не вижу микрофон/интерфейс в списке** — проверь `--list-devices`. Онборд-вход часто называется
  по-ALSA-шному (напр. `HD-Audio Generic, … Analog`). Записи `[0ch]` — плагины/ресемплеры, не входы.
- **Нет monitor-источников** — на PipeWire убедись, что активен профиль с выходом; `pactl list sources`
  должен показывать `*.monitor`.
- **Тишина при захвате через patchbay** — вход ноды `bitbybeat` не подключён; протяни кабель от
  `output/monitor` источника в её `input_FL/FR`.
