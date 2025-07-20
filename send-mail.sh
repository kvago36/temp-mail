#!/bin/bash

# Конфигурация
SMTP_SERVER_IP="192.168.0.118"       # IP SMTP-сервера
TO_EMAIL="nhjqnlj6c1i_wqy@domain6.local"          # Кому отправляем письмо
FROM_EMAIL="sender@example.com"      # От кого
SUBJECT="Тестовое письмо с swaks"
PORT="4000"
BODY="Привет! Это письмо отправлено с помощью swaks через bash-скрипт."

# Отправка письма
swaks --to "$TO_EMAIL" \
      --ehlo domain3.local \
      --from "$FROM_EMAIL" \
      --server "$SMTP_SERVER_IP" \
      --header "Subject: $SUBJECT" \
      --body "$BODY" \
      --port "$PORT"