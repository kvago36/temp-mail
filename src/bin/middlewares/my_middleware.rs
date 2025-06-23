use actix_web::{
    App, Error,
    body::MessageBody,
    cookie::Cookie,
    dev::{ServiceRequest, ServiceResponse},
    middleware::{Next, from_fn},
};
use log::info;
use uuid::Uuid;

pub(crate) async fn my_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // Проверяем существующую сессию
    let has_session = req.cookie("session_id").is_some();

    let mut res = next.call(req).await?;

    // Если нет сессии - создаем новую
    if !has_session {
        let session_id = Uuid::new_v4().to_string();

        info!("Created new session for {}", session_id);

        let session_cookie = Cookie::build("session_id", session_id)
            .path("/")
            .max_age(cookie::time::Duration::hours(24))
            .http_only(true)
            .secure(false) // В продакшене true с HTTPS
            .same_site(actix_web::cookie::SameSite::Lax)
            .finish();

        res.response_mut().add_cookie(&session_cookie)?;
    }

    Ok(res)
}
