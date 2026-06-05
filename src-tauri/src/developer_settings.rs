use std::path::PathBuf;

pub fn developer_settings_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    if let Some(appdata) = std::env::var_os("APPDATA").map(PathBuf::from) {
        candidates.push(appdata.join("Claude").join("developer_settings.json"));
        candidates.push(appdata.join("Claude-3p").join("developer_settings.json"));
    }
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        candidates.push(
            local_appdata
                .join("Packages")
                .join(crate::constants::CLAUDE_STORE_PACKAGE_NAME)
                .join("LocalCache")
                .join("Roaming")
                .join("Claude")
                .join("developer_settings.json"),
        );
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::developer_settings_candidates;

    #[test]
    fn developer_settings_candidates_keep_known_claude_locations() {
        let candidates = developer_settings_candidates();
        let rendered = candidates
            .iter()
            .map(|path| path.to_string_lossy())
            .collect::<Vec<_>>()
            .join("\n");

        assert!(rendered.contains("developer_settings.json"));
        assert!(
            rendered.contains("Claude") || rendered.contains("Claude-3p") || candidates.is_empty()
        );
    }
}
