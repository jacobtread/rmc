use rust_embed::EmbeddedFile;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "resources"]
pub struct Resources;

impl Resources {
    pub fn get_utf8(identifier: &Identifier) -> Option<String> {
        let file = identifier.load()?;
        let value = String::from_utf8(file.data.to_vec())
            .expect("invalid utf8 file unable t read");
        Some(value)
    }
}


// namespace, path
pub struct Identifier<'a> {
    pub namespace: &'a str,
    pub path: &'a str,
}

impl<'a> From<&'a str> for Identifier<'a> {
    fn from(value: &'a str) -> Self {
        let parts = Identifier::split(value);
        Identifier::new(parts[0], parts[1])
    }
}

impl<'a> Identifier<'a> {
    pub fn new(namespace: &'a str, path: &'a str) -> Identifier<'a> {
        Identifier { namespace, path }
    }

    pub fn load(&self) -> Option<EmbeddedFile> {
        let path = format!("assets/{}/{}", self.namespace, self.path);
        Resources::get(path.as_str())
    }

    pub fn split(value: &str) -> Vec<&str> {
        let mut parts = vec!["minecraft", value];
        let index = value.find(':');
        if index.is_some() {
            let index = index.unwrap();
            let len = value.len();
            parts[1] = &value[(index + 1)..len];
            if index >= 1 {
                parts[0] = &value[0..index];
            }
        }
        parts
    }
}