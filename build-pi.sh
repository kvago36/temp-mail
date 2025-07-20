#!/bin/bash

# Настройки
TARGET="aarch64-unknown-linux-gnu"  # Для RPi 2/3/4
PI_HOST="admin@192.168.0.118"
PI_PATH="/home/pi/myapp"
BINARY_NAME="client"

export HOST_ADDRESS=0.0.0.0
export HOST_PORT=4000
export SERVER_ADDRESS=192.168.0.228
export SERVER_PORT=8000

# Запустить сборку
cargo zigbuild --bin client --target aarch64-unknown-linux-gnu
