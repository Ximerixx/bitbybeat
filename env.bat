@echo off
rem Окружение для сборки под Windows из cmd, без Visual Studio (см. env.sh для Git Bash).
rem
rem   env.bat
rem   cargo build --release
rem
rem Штатный тулчейн Rust на Windows — MSVC, ему нужен link.exe из Visual Studio.
rem Здесь вместо него GNU-тулчейн с линкером из MinGW-w64.
rem Свой путь к MinGW:  set BITBYBEAT_MINGW=C:\msys64\mingw64

set "TOOLCHAIN=stable-x86_64-pc-windows-gnu"

if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"

where cargo >nul 2>&1
if errorlevel 1 (
    echo env.bat: cargo не найден, установите Rust: https://rustup.rs
    exit /b 1
)

set "MINGW_BIN="
if defined BITBYBEAT_MINGW call :try "%BITBYBEAT_MINGW%\bin"
if defined BITBYBEAT_MINGW call :try "%BITBYBEAT_MINGW%"
call :try "D:\Qt\Tools\mingw1310_64\bin"
call :try "C:\Qt\Tools\mingw1310_64\bin"
call :try "C:\msys64\mingw64\bin"
call :try "C:\mingw64\bin"
call :try "C:\TDM-GCC-64\bin"

if not defined MINGW_BIN (
    where x86_64-w64-mingw32-gcc >nul 2>&1
    if errorlevel 1 (
        echo env.bat: MinGW-w64 не найден. Укажите путь: set BITBYBEAT_MINGW=C:\путь\к\mingw64
        exit /b 1
    )
) else (
    set "PATH=%MINGW_BIN%;%PATH%"
)

rustup toolchain list | findstr /b /c:"%TOOLCHAIN%" >nul
if errorlevel 1 (
    echo env.bat: ставлю тулчейн %TOOLCHAIN% ^(один раз^)...
    rustup toolchain install %TOOLCHAIN% || exit /b 1
)

set "RUSTUP_TOOLCHAIN=%TOOLCHAIN%"
echo env.bat: тулчейн %TOOLCHAIN%, готово — cargo build / cargo test работают в этой сессии
exit /b 0

:try
if defined MINGW_BIN exit /b 0
if exist "%~1\gcc.exe" set "MINGW_BIN=%~1"
exit /b 0
