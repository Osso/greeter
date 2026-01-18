use pwd::Passwd;

const UID_MIN: u32 = 1000;
const UID_MAX: u32 = 60000;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct User {
    pub username: String,
    pub display_name: String,
}

pub fn get_users() -> Vec<User> {
    Passwd::iter()
        .filter(|p| p.uid >= UID_MIN && p.uid <= UID_MAX)
        .map(|p| {
            let display_name = p
                .gecos
                .as_ref()
                .filter(|g| !g.is_empty())
                .map(|g| g.split(',').next().unwrap_or(g).to_string())
                .unwrap_or_else(|| p.name.clone());

            User {
                username: p.name.clone(),
                display_name,
            }
        })
        .collect()
}
