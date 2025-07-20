#!/bin/bash

# Настройки переменных окружения
HOST_ADDRESS="0.0.0.0"
HOST_PORT="4000"
SERVER_ADDRESS="192.168.0.228"
SERVER_PORT="8000"

# Путь к бинарю
BINARY="./client"

# Запуск с переменными (локально в рамках этой команды)
HOST_ADDRESS="$HOST_ADDRESS" \
HOST_PORT="$HOST_PORT" \
SERVER_ADDRESS="$SERVER_ADDRESS" \
SERVER_PORT="$SERVER_PORT" \
"$BINARY"