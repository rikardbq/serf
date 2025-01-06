@echo off & setlocal

:: INSTALL
echo [RUNNING::INSTALL]
echo [[ %~f0 ]]

set SERF_ROOT_DIR=%APPDATA%\.serf
call :ResolvePath TARGET_DIR %~dpn0\..\..\..\target\release

for /F "tokens=1-9 delims= " %%a in ("%*") do (
    if %%a==-rd (
        set SERF_ROOT_DIR=%%~fb
    )
)

echo [RUNNING::BUILD] ROOT=%SERF_ROOT_DIR%
echo [BUILD_TARGET] ARTIFACTS=%TARGET_DIR%

cargo build --release

echo [COMPLETE::BUILD]
echo [RUNNING::COPY_EXECUTABLES] ROOT=%SERF_ROOT_DIR%

:: check dir existence and make if not exist
dir /A:D %SERF_ROOT_DIR% >nul 2>&1 & if ERRORLEVEL 1 (
    mkdir %SERF_ROOT_DIR%
)

copy %TARGET_DIR%\*.exe %SERF_ROOT_DIR%

echo [COMPLETE::COPY_EXECUTABLES]
echo [COMPLETE::INSTALL]

exit /b

:: FUNCTIONS
:ResolvePath
    set %1=%~f2
    exit /b
