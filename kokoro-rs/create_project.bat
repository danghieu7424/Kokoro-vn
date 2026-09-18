@echo off
setlocal enabledelayedexpansion

if "%~1"=="" (
    echo ========================================================
    echo [Loi] Vui long nhap ten du an!
    echo Su dung: create_project.bat ^<ten_du_an^>
    echo Vi du:   create_project.bat my_new_api
    echo ========================================================
    exit /b 1
)

set PROJECT_NAME=%~1
set SOURCE_DIR=%~dp0init-template
set TARGET_DIR=%~dp0..\%PROJECT_NAME%

echo ========================================================
echo [*] KHOI TAO DU AN MOI: %PROJECT_NAME%
echo ========================================================

if exist "%TARGET_DIR%" (
    echo [Loi] Thu muc %TARGET_DIR% da ton tai! Vui long chon ten khac.
    exit /b 1
)

echo [1/3] Copying template tu init-template...
xcopy /E /I /Q "%SOURCE_DIR%" "%TARGET_DIR%"

echo [2/3] Don dep du lieu sinh ra trong qua trinh test...
rmdir /S /Q "%TARGET_DIR%\target" 2>nul
rmdir /S /Q "%TARGET_DIR%\logs" 2>nul
rmdir /S /Q "%TARGET_DIR%\storages" 2>nul

echo [3/3] Cap nhat ten Package trong Cargo.toml...
powershell -Command "(Get-Content '%TARGET_DIR%\Cargo.toml') -replace 'name = \"axum-daemon-template\"', 'name = \"%PROJECT_NAME%\"' -replace 'default-run = \"axum-daemon-template\"', 'default-run = \"%PROJECT_NAME%\"' | Set-Content '%TARGET_DIR%\Cargo.toml'"

echo.
echo ========================================================
echo [HOAN TAT] Du an %PROJECT_NAME% da san sang!
echo ========================================================
echo De bat dau phat trien:
echo   1. cd ..\%PROJECT_NAME%
echo   2. cargo run
echo.
