@echo off
setlocal EnableDelayedExpansion
:: Khata — one-shot setup + start (Windows)
:: Run from anywhere: setup.bat
:: Subsequent runs: same command — skips steps already done.

set "PROJECT=%~dp0"
:: Remove trailing backslash
if "%PROJECT:~-1%"=="\" set "PROJECT=%PROJECT:~0,-1%"

set "PGDATA=%PROJECT%\.pgdata"
set "PGPORT=5433"
set "PGLOG=%PGDATA%\pg.log"
set "ENV_FILE=%PROJECT%\.env"

echo.
echo =========================================
echo   Khata -- Finance Tracker
echo =========================================
echo.

:: ── 1. Prerequisites ────────────────────────────────────────────────────────────
echo [^>] Checking prerequisites...

where cargo    >nul 2>&1 || ( echo [X] cargo not found. Install Rust from https://rustup.rs & exit /b 1 )
where psql     >nul 2>&1 || ( echo [X] psql not found. Install PostgreSQL from https://www.postgresql.org/download/windows/ & exit /b 1 )
where pg_ctl   >nul 2>&1 || ( echo [X] pg_ctl not found. Ensure PostgreSQL bin directory is in PATH. & exit /b 1 )
where node     >nul 2>&1 || ( echo [X] node not found. Install Node.js from https://nodejs.org & exit /b 1 )
where npm      >nul 2>&1 || ( echo [X] npm not found. & exit /b 1 )
where sqlx     >nul 2>&1
if errorlevel 1 (
    echo [^>] Installing sqlx-cli...
    cargo install sqlx-cli --no-default-features --features rustls,postgres
    if errorlevel 1 ( echo [X] sqlx-cli install failed. & exit /b 1 )
)
where claude   >nul 2>&1 || echo [!] claude CLI not found -- chat Q^&A will not work. Install: npm i -g @anthropic-ai/claude-code

echo [OK] Prerequisites OK

:: ── 2. .env ─────────────────────────────────────────────────────────────────────
if not exist "%ENV_FILE%" (
    echo [^>] Creating .env from .env.example...
    copy "%PROJECT%\.env.example" "%ENV_FILE%" >nul
    echo [OK] .env created
) else (
    echo [OK] .env already exists
)

:: Load .env into environment
for /f "usebackq tokens=1,* delims==" %%A in ("%ENV_FILE%") do (
    set "line=%%A"
    if not "!line:~0,1!"=="#" if not "%%A"=="" (
        set "%%A=%%B"
    )
)

if "%DATABASE_URL%"=="" set "DATABASE_URL=postgresql://khata:khata@127.0.0.1:5433/khata"
if "%RO_DATABASE_URL%"=="" set "RO_DATABASE_URL=postgresql://khata_ro:khata_ro@127.0.0.1:5433/khata"
if "%JWT_SECRET%"=="" set "JWT_SECRET=change-me-in-.env"
if "%BIND_ADDR%"=="" set "BIND_ADDR=127.0.0.1:8090"
if "%RUST_LOG%"=="" set "RUST_LOG=info"

:: ── 3. Postgres init (first run only) ───────────────────────────────────────────
if not exist "%PGDATA%" (
    echo [^>] Initialising Postgres cluster at .pgdata...
    initdb -D "%PGDATA%" --no-locale --encoding=UTF8 -A trust
    if errorlevel 1 ( echo [X] initdb failed. & exit /b 1 )

    echo port = %PGPORT%              >> "%PGDATA%\postgresql.conf"
    echo listen_addresses = '127.0.0.1' >> "%PGDATA%\postgresql.conf"
    echo host    khata    khata      127.0.0.1/32    scram-sha-256 >> "%PGDATA%\pg_hba.conf"
    echo host    khata    khata_ro   127.0.0.1/32    scram-sha-256 >> "%PGDATA%\pg_hba.conf"

    pg_ctl -D "%PGDATA%" -l "%PGLOG%" start
    timeout /t 3 /nobreak >nul

    psql -p %PGPORT% -d postgres -c "CREATE ROLE khata    LOGIN PASSWORD 'khata';"
    psql -p %PGPORT% -d postgres -c "CREATE ROLE khata_ro LOGIN PASSWORD 'khata_ro';"
    psql -p %PGPORT% -d postgres -c "ALTER  ROLE khata    CREATEDB;"
    psql -p %PGPORT% -d postgres -c "CREATE DATABASE khata OWNER khata;"
    psql -p %PGPORT% -d khata    -c "CREATE EXTENSION IF NOT EXISTS citext;"

    pg_ctl -D "%PGDATA%" stop
    timeout /t 2 /nobreak >nul
    echo [OK] Postgres cluster initialised
) else (
    echo [OK] Postgres cluster already exists
)

:: ── 4. Start Postgres ────────────────────────────────────────────────────────────
pg_ctl -D "%PGDATA%" status >nul 2>&1
if errorlevel 1 (
    echo [^>] Starting Postgres...
    pg_ctl -D "%PGDATA%" -l "%PGLOG%" start
    timeout /t 2 /nobreak >nul
    echo [OK] Postgres started on port %PGPORT%
) else (
    echo [OK] Postgres already running on port %PGPORT%
)

:: ── 5. Migrations ────────────────────────────────────────────────────────────────
echo [^>] Running database migrations...
cd /d "%PROJECT%\backend"
sqlx migrate run
if errorlevel 1 ( echo [X] Migrations failed. & exit /b 1 )
echo [OK] Migrations up to date
cd /d "%PROJECT%"

:: ── 6. Frontend dependencies ─────────────────────────────────────────────────────
if not exist "%PROJECT%\frontend\node_modules" (
    echo [^>] Installing frontend dependencies...
    cd /d "%PROJECT%\frontend"
    npm install
    if errorlevel 1 ( echo [X] npm install failed. & exit /b 1 )
    cd /d "%PROJECT%"
    echo [OK] Frontend dependencies installed
) else (
    echo [OK] Frontend node_modules present
)

:: ── 7. Start backend + frontend in new windows ──────────────────────────────────
echo.
echo =========================================
echo   Starting Khata
echo =========================================
echo.

echo [^>] Starting backend...
start "Khata Backend" cmd /k "cd /d "%PROJECT%\backend" && set DATABASE_URL=%DATABASE_URL% && set RO_DATABASE_URL=%RO_DATABASE_URL% && set JWT_SECRET=%JWT_SECRET% && set BIND_ADDR=%BIND_ADDR% && set RUST_LOG=%RUST_LOG% && cargo run"

timeout /t 2 /nobreak >nul

echo [^>] Starting frontend...
start "Khata Frontend" cmd /k "cd /d "%PROJECT%\frontend" && npm run dev"

echo.
echo [OK] Khata is starting!
echo.
echo   Frontend: http://localhost:5173
echo   Backend:  http://%BIND_ADDR%
echo.
echo   Backend and frontend are running in separate windows.
echo   Close those windows (or Ctrl+C in each) to stop.
echo.
pause
endlocal
