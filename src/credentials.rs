use k8s_openapi::api::core::v1::Secret;

pub struct Credentials {
    pub password_hash: String,
    pub username: String,
}

impl Credentials {
    pub fn password_file_line(&self) -> String {
        format!("{}:{}\n", self.username, self.password_hash)
    }
}

fn get_data<'a>(secret: &'a Secret, key: &str) -> Option<&'a str> {
    if let Some(d) = secret
        .string_data
        .as_ref()
        .and_then(|data| data.get(key))
        .map(String::as_str)
    {
        return Some(d);
    }

    secret
        .data
        .as_ref()
        .and_then(|data| data.get(key))
        .and_then(|bytes| std::str::from_utf8(&bytes.0).ok())
}

impl TryFrom<&Secret> for Credentials {
    type Error = ();

    fn try_from(value: &Secret) -> Result<Self, Self::Error> {
        let hash = get_data(value, "hash");
        let username = get_data(value, "username");

        match (hash, username) {
            (Some(hash), Some(username)) => Ok(Credentials {
                password_hash: hash.to_string(),
                username: username.to_string(),
            }),
            _ => Err(()),
        }
    }
}
