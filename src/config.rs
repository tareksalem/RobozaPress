use std::{path::{PathBuf, Path}, env, str::FromStr};

pub const L_INUX_UBUNTU_BOOKMARKS_PATH: &str = "google-chrome/Default/Bookmarks";
pub const WINDOWS_BOOKMARKS_PATH: &str = "Google/Chrome/User Data/Default/Bookmarks";
pub const CACHE_MAIN_DIR: &str = "aan";
pub const CACHE_FILE_PATH: &str = "aan/Aan.json";
pub const CACHE_IMG_PATH: &str = "aan/images";
pub const DEFAULT_IMG_PATH: &str = "default.png";
pub const ASSETS_DIR_PATH: &str = "assets";
pub const LOADER_ICON_PATH: &str = "loader-icon.svg";
pub fn get_full_img_cache_path() -> PathBuf {
    Path::new(dirs::cache_dir().unwrap().as_path()).join(CACHE_IMG_PATH)
}

pub fn get_default_image_path() -> PathBuf {
    Path::new(ASSETS_DIR_PATH).join(DEFAULT_IMG_PATH)
}

pub fn get_loader_icon_path() -> PathBuf {
    Path::new(ASSETS_DIR_PATH).join(LOADER_ICON_PATH)
}
pub fn get_os_config_path() -> PathBuf {
    let current_os: String = detect_os().to_lowercase();
    match current_os.as_str() {
        "windows" => Path::new(dirs::cache_dir().unwrap().as_path()).join(WINDOWS_BOOKMARKS_PATH),
        "linux" => Path::new(dirs::config_dir().unwrap().as_path()).join(L_INUX_UBUNTU_BOOKMARKS_PATH),
        _ => panic!("not supported os"),
    }
}

fn detect_os() -> String {
    env::consts::OS.to_string()
}
pub fn get_bookmarks_path() -> String {
    get_os_config_path().display().to_string()
}

pub fn get_cache_file_path() -> PathBuf {
    Path::new(dirs::cache_dir().unwrap().as_path()).join(Path::new(CACHE_FILE_PATH))
}