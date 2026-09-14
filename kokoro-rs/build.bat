@echo off
chcp 65001 >nul
color 0B
echo ========================================================
echo        KOKORO TTS VIETNAMESE - BUILD SCRIPT
echo ========================================================
echo.

echo [1/3] Dang build Kokoro TTS Engine (kokoro-rs)...
cargo build --release
if %errorlevel% neq 0 (
    color 0C
    echo [Loi] Xay dung TTS Engine that bai!
    exit /b %errorlevel%
)

echo.
echo [2/3] Dang build Kokoro API Server (api-server)...
cd api-server
cargo build --release
if %errorlevel% neq 0 (
    color 0C
    echo [Loi] Xay dung API Server that bai!
    cd ..
    exit /b %errorlevel%
)
cd ..

echo.
echo [3/3] Dang copy cac file thuc thi vao thu muc dist...
if not exist "dist" mkdir dist
copy /y "target\release\kokoro-rs.exe" "dist\" >nul
copy /y "api-server\target\release\kokoro-api.exe" "dist\" >nul

:: Kiem tra va copy cac file tai nguyen vao dist (neu chua co)
if not exist "dist\kokoro_vi.onnx" (
    echo [*] Luu y: Ban can copy model 'kokoro_vi.onnx' vao thu muc dist.
)
if not exist "dist\config.json" (
    if exist "config.json" copy /y "config.json" "dist\" >nul
)
if not exist "dist\voicepacks_npy" (
    if exist "voicepacks_npy" xcopy /E /I /Y "voicepacks_npy" "dist\voicepacks_npy" >nul
)

echo.
color 0A
echo ========================================================
echo [THANH CONG] Toan bo project da duoc build va dong goi!
echo Ban co the mang thu muc 'dist' di bat ky dau de chay.
echo ========================================================
echo.
pause
