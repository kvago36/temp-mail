import smtplib
from email.message import EmailMessage
from email.utils import make_msgid
import base64

# === Настройки ===
SMTP_SERVER = "localhost"
SMTP_PORT = 4000
SENDER = "sender@example.com"
RECIPIENT = "email16@test.com"

# === HTML с inline-изображением по cid ===
image_cid = make_msgid(domain="example.com")[1:-1]  # без < >
html = f"""
<html>
  <body style="font-family: Arial, sans-serif; background-color: #f4f4f4; padding: 20px;">
    <div style="max-width: 600px; margin: auto; background: white; padding: 20px; border-radius: 10px;">
      <h2 style="color: #333;">Добро пожаловать!</h2>
      <p>Вы успешно подписались на рассылку.</p>
      <img src="cid:{image_cid}" alt="Баннер" style="width: 100%; border-radius: 10px;">
      <p>Нажмите кнопку ниже, чтобы узнать больше:</p>
      <a href="https://example.com"
         style="display: inline-block; padding: 12px 20px; margin-top: 10px;
                background-color: #007bff; color: white; text-decoration: none;
                border-radius: 5px;">
        Перейти на сайт
      </a>
    </div>
  </body>
</html>
"""

# === Создание письма ===
msg = EmailMessage()
msg["Subject"] = "Email with Attachment"
msg["From"] = SENDER
msg["To"] = RECIPIENT
msg.set_content("Ваш email-клиент не поддерживает HTML.")
msg.add_alternative(html, subtype="html")

# === Присоединить изображение как inline ===
with open("guru.png", "rb") as img:
    img_data = img.read()
    msg.get_payload()[1].add_related(
        img_data,
        maintype="image",
        subtype="png",
        cid=f"<{image_cid}>",
        filename="guru.png"
    )

# === Отправка ===
with smtplib.SMTP(SMTP_SERVER, SMTP_PORT) as smtp:
    smtp.set_debuglevel(1)
    smtp.send_message(msg)

print("Письмо отправлено.")