use darling::FromMeta;

#[derive(Debug, Default, FromMeta)]
pub struct Attributes {
    #[darling(default = "default_cmd")]
    pub cmd: String,
    #[darling(default)]
    pub no_panic: bool,
}

fn default_cmd() -> String {
    "bash -c".to_string()
}
