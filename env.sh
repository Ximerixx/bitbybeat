#!/usr/bin/env bash
# Окружение для сборки под Windows из Git Bash, без Visual Studio.
#
#   source ./env.sh
#   cargo build --release
#
# Штатный тулчейн Rust на Windows — MSVC, ему нужен link.exe из Visual Studio.
# Здесь вместо него используется GNU-тулчейн с линкером из MinGW-w64.
#
# Путь к MinGW можно задать заранее:  BITBYBEAT_MINGW=/c/msys64/mingw64 source ./env.sh

TOOLCHAIN="stable-x86_64-pc-windows-gnu"

_bbb_die() {
    echo "env.sh: $1" >&2
    return 1
}

_bbb_setup() {
    case "$(uname -s)" in
        MINGW*|MSYS*|CYGWIN*) ;;
        *)
            echo "env.sh: не Windows — MSVC/MinGW не нужны, используйте cargo напрямую"
            return 0
            ;;
    esac

    # cargo/rustup ставятся в ~/.cargo/bin, которого в PATH у Git Bash обычно нет.
    if [ -d "$HOME/.cargo/bin" ]; then
        case ":$PATH:" in
            *":$HOME/.cargo/bin:"*) ;;
            *) PATH="$HOME/.cargo/bin:$PATH" ;;
        esac
    fi

    command -v cargo >/dev/null 2>&1 || _bbb_die "cargo не найден, установите Rust: https://rustup.rs" || return 1

    local candidates=()
    [ -n "$BITBYBEAT_MINGW" ] && candidates+=("$BITBYBEAT_MINGW/bin" "$BITBYBEAT_MINGW")
    candidates+=(
        /d/Qt/Tools/mingw1310_64/bin
        /c/Qt/Tools/mingw1310_64/bin
        /c/msys64/mingw64/bin
        /c/mingw64/bin
        /c/TDM-GCC-64/bin
    )

    local mingw_bin=""
    local dir
    for dir in "${candidates[@]}"; do
        if [ -x "$dir/x86_64-w64-mingw32-gcc.exe" ] || [ -x "$dir/gcc.exe" ]; then
            mingw_bin="$dir"
            break
        fi
    done

    # Уже в PATH (например, установлен системно) — этого достаточно.
    if [ -z "$mingw_bin" ] && command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        mingw_bin="already-in-path"
    fi

    if [ -z "$mingw_bin" ]; then
        _bbb_die "MinGW-w64 не найден. Укажите его так:  BITBYBEAT_MINGW=/путь/к/mingw64 source ./env.sh"
        return 1
    fi

    if [ "$mingw_bin" != "already-in-path" ]; then
        case ":$PATH:" in
            *":$mingw_bin:"*) ;;
            *) PATH="$mingw_bin:$PATH" ;;
        esac
    fi

    if ! rustup toolchain list 2>/dev/null | grep -q "^$TOOLCHAIN"; then
        echo "env.sh: ставлю тулчейн $TOOLCHAIN (один раз)..."
        rustup toolchain install "$TOOLCHAIN" || return 1
    fi

    export PATH
    export RUSTUP_TOOLCHAIN="$TOOLCHAIN"

    echo "env.sh: тулчейн  $TOOLCHAIN"
    echo "env.sh: линкер   $(command -v x86_64-w64-mingw32-gcc || command -v gcc)"
    echo "env.sh: готово — cargo build / cargo test работают в этой сессии"
}

_bbb_setup
unset -f _bbb_setup _bbb_die
