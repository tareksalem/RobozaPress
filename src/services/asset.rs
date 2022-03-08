use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/"]
#[prefix = "assets/"]
pub struct Asset;
