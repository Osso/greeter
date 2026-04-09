use pwd::Passwd;

const UID_MIN: u32 = 1000;
const UID_MAX: u32 = 60000;

pub fn get_usernames() -> Vec<String> {
    Passwd::iter()
        .filter(|p| p.uid >= UID_MIN && p.uid <= UID_MAX)
        .map(|p| p.name.clone())
        .collect()
}
