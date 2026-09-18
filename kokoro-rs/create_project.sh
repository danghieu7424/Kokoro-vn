#!/bin/bash

if [ -z "$1" ]; then
    echo "========================================================"
    echo "[Loi] Vui long nhap ten du an!"
    echo "Su dung: ./create_project.sh <ten_du_an>"
    echo "Vi du:   ./create_project.sh my_new_api"
    echo "========================================================"
    exit 1
fi

PROJECT_NAME=$1
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
SOURCE_DIR="$SCRIPT_DIR/init-template"
TARGET_DIR="$SCRIPT_DIR/../$PROJECT_NAME"

echo "========================================================"
echo "[*] KHOI TAO DU AN MOI: $PROJECT_NAME"
echo "========================================================"

if [ -d "$TARGET_DIR" ]; then
    echo "[Loi] Thu muc $TARGET_DIR da ton tai! Vui long chon ten khac."
    exit 1
fi

echo "[1/3] Copying template tu init-template..."
cp -r "$SOURCE_DIR" "$TARGET_DIR"

echo "[2/3] Don dep du lieu sinh ra trong qua trinh test..."
rm -rf "$TARGET_DIR/target" 2>/dev/null
rm -rf "$TARGET_DIR/logs" 2>/dev/null
rm -rf "$TARGET_DIR/storages" 2>/dev/null

echo "[3/3] Cap nhat ten Package trong Cargo.toml..."
# Ho tro ca macOS va Linux cho lenh sed
if [[ "$OSTYPE" == "darwin"* ]]; then
    sed -i '' "s/name = \"axum-daemon-template\"/name = \"$PROJECT_NAME\"/g" "$TARGET_DIR/Cargo.toml"
    sed -i '' "s/default-run = \"axum-daemon-template\"/default-run = \"$PROJECT_NAME\"/g" "$TARGET_DIR/Cargo.toml"
else
    sed -i "s/name = \"axum-daemon-template\"/name = \"$PROJECT_NAME\"/g" "$TARGET_DIR/Cargo.toml"
    sed -i "s/default-run = \"axum-daemon-template\"/default-run = \"$PROJECT_NAME\"/g" "$TARGET_DIR/Cargo.toml"
fi

echo ""
echo "========================================================"
echo "[HOAN TAT] Du an $PROJECT_NAME da san sang!"
echo "========================================================"
echo "De bat dau phat trien:"
echo "  1. cd ../$PROJECT_NAME"
echo "  2. cargo run"
echo ""
