use rand::{Rng, distributions::Alphanumeric};

/// Генерирует валидную строку email (до @)
pub fn generate_email_local_part() -> String {
    let mut rng = rand::thread_rng();
    let len = rng.gen_range(8..=16);

    let mut s: String = (0..len)
        .map(|_| {
            let c = rng.sample(Alphanumeric) as char;
            c.to_ascii_lowercase()
        })
        .collect();

    s = s.trim_matches('.').to_string();
    while s.contains("..") {
        s = s.replace("..", ".");
    }

    let specials = ['.', '_', '-', '+'];
    if len > 6 && rng.gen_bool(0.3) {
        let i = rng.gen_range(1..s.len() - 1);
        let ch = specials[rng.gen_range(0..specials.len())];
        s.insert(i, ch);
    }

    s
}
