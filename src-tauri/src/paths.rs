use std::path::{Path, PathBuf};

const APP_STATE_DIR: &str = ".claude-plus-plus";

pub fn home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(PathBuf::from))
}

pub fn app_state_dir() -> PathBuf {
    home_dir()
        .map(|home| app_state_dir_from_home(&home))
        .unwrap_or_else(|| PathBuf::from(APP_STATE_DIR))
}

pub fn app_state_dir_from_home(home: &Path) -> PathBuf {
    home.join(APP_STATE_DIR)
}

pub fn ccswitch_db_path() -> Option<PathBuf> {
    home_dir().map(|home| ccswitch_db_path_from_home(&home))
}

pub fn ccswitch_db_path_from_home(home: &Path) -> PathBuf {
    home.join(".cc-switch").join("cc-switch.db")
}

#[cfg(test)]
mod tests {
    use super::{app_state_dir_from_home, ccswitch_db_path_from_home};
    use std::path::Path;

    #[test]
    fn app_state_dir_uses_home_scoped_directory() {
        assert_eq!(
            app_state_dir_from_home(Path::new(r"C:\Users\Ada")),
            Path::new(r"C:\Users\Ada").join(".claude-plus-plus")
        );
    }

    #[test]
    fn ccswitch_db_path_uses_home_scoped_database() {
        assert_eq!(
            ccswitch_db_path_from_home(Path::new(r"C:\Users\Ada")),
            Path::new(r"C:\Users\Ada")
                .join(".cc-switch")
                .join("cc-switch.db")
        );
    }
}
