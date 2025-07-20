#!/bin/bash

# Путь к бинарю
BINARY_PATH="target/aarch64-unknown-linux-gnu/debug/client"

# Хост назначения
HOST="admin@192.168.0.118"

# Имя файла (последняя часть пути)
BINARY_NAME=$(basename "$BINARY_PATH")

# Проверка, существует ли файл
if [ ! -f "$BINARY_PATH" ]; then
  echo "❌ Бинарный файл не найден: $BINARY_PATH"
  exit 1
fi

echo "🚀 Отправка $BINARY_NAME на $HOST:~"
scp "$BINARY_PATH" "$HOST:~"

if [ $? -eq 0 ]; then
  echo "✅ Успешно отправлено!"
else
  echo "❌ Ошибка при отправке!"
  exit 1
fi
