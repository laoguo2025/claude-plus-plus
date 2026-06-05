use std::time::Duration;

const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/laoguo2025/claude-plus-plus/releases/latest";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ReleaseInfo {
    pub tag_name: String,
    pub version: String,
    pub name: String,
    pub url: String,
    pub published_at: Option<String>,
    pub body: String,
    pub assets: Vec<ReleaseAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub url: String,
    pub size: u64,
    pub kind: ReleaseAssetKind,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReleaseAssetKind {
    Windows,
    MacosArm64,
    MacosX64,
    Other,
}

#[derive(serde::Deserialize)]
struct GithubReleasePayload {
    tag_name: String,
    name: Option<String>,
    html_url: String,
    published_at: Option<String>,
    body: Option<String>,
    assets: Vec<GithubReleaseAsset>,
}

#[derive(serde::Deserialize)]
struct GithubReleaseAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

pub async fn fetch_latest_release() -> Result<ReleaseInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(format!("Claude++/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| format!("创建 GitHub 请求客户端失败: {error}"))?;

    let response = client
        .get(LATEST_RELEASE_API)
        .header(reqwest::header::ACCEPT, "application/vnd.github+json")
        .send()
        .await
        .map_err(|error| format!("请求 GitHub Release 失败: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        return Err(format!("GitHub Release 请求失败: HTTP {status}"));
    }

    let text = response
        .text()
        .await
        .map_err(|error| format!("读取 GitHub Release 响应失败: {error}"))?;
    parse_latest_release(&text)
}

pub fn parse_latest_release(payload: &str) -> Result<ReleaseInfo, String> {
    let payload: GithubReleasePayload = serde_json::from_str(payload)
        .map_err(|error| format!("解析 GitHub Release 响应失败: {error}"))?;
    let assets = payload
        .assets
        .into_iter()
        .filter_map(|asset| {
            let kind = classify_asset(&asset.name);
            if matches!(kind, ReleaseAssetKind::Other) {
                return None;
            }
            Some(ReleaseAsset {
                name: asset.name,
                url: asset.browser_download_url,
                size: asset.size,
                kind,
            })
        })
        .collect();

    Ok(ReleaseInfo {
        version: payload.tag_name.trim_start_matches('v').to_string(),
        name: payload.name.unwrap_or_else(|| payload.tag_name.clone()),
        tag_name: payload.tag_name,
        url: payload.html_url,
        published_at: payload.published_at,
        body: payload.body.unwrap_or_default(),
        assets,
    })
}

fn classify_asset(name: &str) -> ReleaseAssetKind {
    let lower = name.to_ascii_lowercase();
    if lower.ends_with(".exe") {
        ReleaseAssetKind::Windows
    } else if lower.ends_with(".dmg") && (lower.contains("aarch64") || lower.contains("arm64")) {
        ReleaseAssetKind::MacosArm64
    } else if lower.ends_with(".dmg") && lower.contains("x64") {
        ReleaseAssetKind::MacosX64
    } else {
        ReleaseAssetKind::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_latest_release_payload_and_keeps_installer_assets() {
        let payload = r###"{
          "tag_name": "v1.0.1",
          "name": "Claude++ v1.0.1",
          "html_url": "https://github.com/laoguo2025/claude-plus-plus/releases/tag/v1.0.1",
          "published_at": "2026-06-05T04:31:20Z",
          "body": "## 更新内容\n\n- 新增 Win 虚拟机平台自动开启。",
          "assets": [
            {
              "name": "Claude++_1.0.1_x64-setup.exe",
              "browser_download_url": "https://example.test/win.exe",
              "size": 5336337
            },
            {
              "name": "Source code (zip)",
              "browser_download_url": "https://example.test/source.zip",
              "size": 12
            },
            {
              "name": "Claude++_1.0.1_aarch64.dmg",
              "browser_download_url": "https://example.test/arm64.dmg",
              "size": 8594604
            }
          ]
        }"###;

        let release = parse_latest_release(payload).expect("parse release");

        assert_eq!(release.tag_name, "v1.0.1");
        assert_eq!(release.version, "1.0.1");
        assert_eq!(release.assets.len(), 2);
        assert_eq!(release.assets[0].name, "Claude++_1.0.1_x64-setup.exe");
        assert_eq!(release.assets[1].name, "Claude++_1.0.1_aarch64.dmg");
    }
}
