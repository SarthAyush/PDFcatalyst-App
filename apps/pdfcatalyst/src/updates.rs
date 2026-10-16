//! Asks GitHub for the latest PdfCraft release and downloads updates (Help ▸ Check for updates, issue #28).

use std::time::Duration;

use pdfcraft_ui_egui::updates::{RELEASES_PAGE, Release};

const LATEST: &str = "https://api.github.com/repos/SarthAyush/PDFcatalyst-App/releases/latest";

/// The latest release. The answer is untrusted: its size is capped, and only a page under
/// [`RELEASES_PAGE`] is ever offered for download (anything else falls back to that list).
pub fn latest_release() -> Result<Release, String> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(10)))
        .tls_config(ureq::tls::TlsConfig::builder().root_certs(os_roots()?).build())
        .build()
        .new_agent();
    let mut response = agent
        .get(LATEST)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", concat!("PdfCraft/", env!("CARGO_PKG_VERSION")))
        // Read error statuses ourselves: a 403 from GitHub usually means its anonymous
        // rate limit ran out, and the retry time rides along in a response header.
        .config()
        .http_status_as_error(false)
        .build()
        .call()
        .map_err(|e| format!("couldn't reach GitHub ({e})"))?;
    let status = response.status().as_u16();
    if status == 403 {
        let reset = response.headers().get("x-ratelimit-reset").and_then(|v| v.to_str().ok());
        return Err(rate_limited(reset, now_unix()));
    }
    if !(200..300).contains(&status) {
        return Err(format!("couldn't reach GitHub (http status: {status})"));
    }
    let body = response.body_mut().with_config().limit(1 << 20).read_to_string().map_err(|e| format!("unreadable answer ({e})"))?;
    parse(&body)
}

/// Downloads the release update installer into a temporary folder, reporting progress.
pub fn download_update(release: Release, tx: std::sync::mpsc::Sender<pdfcraft_ui_egui::updates::DownloadProgress>, ctx: Option<egui::Context>) {
    let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Connecting);
    if let Some(ctx) = &ctx {
        ctx.request_repaint();
    }

    let download_url = match release.download_url {
        Some(url) => url,
        None => {
            let ver = release.version.trim_start_matches(['v', 'V']);
            format!("https://github.com/SarthAyush/PDFcatalyst-App/releases/download/{}/PDFCatalyst-Setup-v{}-x64.exe", release.version, ver)
        }
    };

    let filename = release.asset_name.unwrap_or_else(|| {
        let ver = release.version.trim_start_matches(['v', 'V']);
        format!("PDFCatalyst-Setup-v{}-x64.exe", ver)
    });

    let roots = match os_roots() {
        Ok(r) => r,
        Err(e) => {
            let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(e));
            return;
        }
    };

    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(600)))
        .tls_config(ureq::tls::TlsConfig::builder().root_certs(roots).build())
        .build()
        .new_agent();

    let mut response = match agent
        .get(&download_url)
        .header("Accept", "application/octet-stream")
        .header("User-Agent", concat!("PdfCraft/", env!("CARGO_PKG_VERSION")))
        .call()
    {
        Ok(resp) => resp,
        Err(e) => {
            let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Download failed ({e})")));
            return;
        }
    };

    let status = response.status().as_u16();
    if !(200..300).contains(&status) {
        let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Download failed (HTTP {status})")));
        return;
    }

    let total: Option<u64> = response.headers().get("content-length").and_then(|v| v.to_str().ok()).and_then(|s| s.parse::<u64>().ok());

    let temp_dir = std::env::temp_dir().join("CraftPDF_Updates");
    if let Err(e) = std::fs::create_dir_all(&temp_dir) {
        let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Couldn't create directory ({e})")));
        return;
    }
    let dest_path = temp_dir.join(&filename);

    let mut file = match std::fs::File::create(&dest_path) {
        Ok(f) => f,
        Err(e) => {
            let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Couldn't save update ({e})")));
            return;
        }
    };

    use std::io::{Read, Write};
    let mut reader = response.body_mut().as_reader();
    let mut buf = [0u8; 64 * 1024];
    let mut downloaded: u64 = 0;
    let mut last_report = std::time::Instant::now();

    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if let Err(e) = file.write_all(&buf[..n]) {
                    let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Disk write error ({e})")));
                    return;
                }
                downloaded += n as u64;
                if last_report.elapsed() > Duration::from_millis(80) {
                    let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Downloading { downloaded, total });
                    if let Some(ctx) = &ctx {
                        ctx.request_repaint();
                    }
                    last_report = std::time::Instant::now();
                }
            }
            Err(e) => {
                let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Connection interrupted ({e})")));
                return;
            }
        }
    }

    if let Err(e) = file.flush() {
        let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Failed(format!("Failed to finalize file ({e})")));
        return;
    }

    let _ = tx.send(pdfcraft_ui_egui::updates::DownloadProgress::Finished { path: dest_path });
    if let Some(ctx) = &ctx {
        ctx.request_repaint();
    }
}

/// The certificate authorities the operating system trusts.
fn os_roots() -> Result<ureq::tls::RootCerts, String> {
    let found = rustls_native_certs::load_native_certs();
    let certs: Vec<ureq::tls::Certificate<'static>> = found.certs.iter().map(|c| ureq::tls::Certificate::from_der(c.as_ref()).to_owned()).collect();
    if certs.is_empty() {
        return Err("no trusted certificates found on this system".into());
    }
    Ok(ureq::tls::RootCerts::new_with_certs(&certs))
}

/// The current Unix time in seconds (0 when the clock gives nothing usable).
fn now_unix() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// What to tell the user when GitHub answers 403: its anonymous quota (60 checks an hour,
/// shared by everyone behind the same address) ran out. `reset` is the `x-ratelimit-reset`
/// header, the Unix time the quota refills; `now` is the current Unix time. Anything
/// surprising (no header, not a number, already past) falls back to asking for patience.
fn rate_limited(reset: Option<&str>, now: u64) -> String {
    const FALLBACK: &str = "GitHub is limiting update checks for now (shared hourly quota); try again later";
    let wait = reset.and_then(|r| r.trim().parse::<u64>().ok()).and_then(|reset| reset.checked_sub(now)).filter(|&wait| wait > 0);
    match wait {
        Some(wait) => {
            let minutes = wait.div_ceil(60).max(1);
            let plural = if minutes == 1 { "" } else { "s" };
            format!("GitHub is limiting update checks for now (shared hourly quota); try again in about {minutes} minute{plural}")
        }
        None => FALLBACK.into(),
    }
}

fn parse(body: &str) -> Result<Release, String> {
    let v: serde_json::Value = serde_json::from_str(body).map_err(|e| format!("unreadable answer ({e})"))?;
    let version = v["tag_name"].as_str().filter(|t| !t.is_empty() && t.len() <= 64).ok_or("no release found")?.to_string();
    let url = v["html_url"]
        .as_str()
        .filter(|u| u.strip_prefix(RELEASES_PAGE).is_some_and(|rest| rest.starts_with('/') && !rest.contains(['?', '#', '\\'])))
        .unwrap_or(RELEASES_PAGE)
        .to_string();

    let mut download_url = None;
    let mut asset_name = None;

    if let Some(assets) = v["assets"].as_array() {
        let chosen = assets
            .iter()
            .find(|a| {
                let name = a["name"].as_str().unwrap_or("").to_lowercase();
                name.ends_with(".exe") && (name.contains("setup") || name.contains("install"))
            })
            .or_else(|| assets.iter().find(|a| a["name"].as_str().unwrap_or("").to_lowercase().ends_with(".exe")))
            .or_else(|| assets.first());

        if let Some(asset) = chosen {
            if let Some(dl) = asset["browser_download_url"].as_str() {
                download_url = Some(dl.to_string());
            }
            if let Some(name) = asset["name"].as_str() {
                asset_name = Some(name.to_string());
            }
        }
    }

    Ok(Release { version, url, download_url, asset_name })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_are_read_and_only_our_release_pages_are_offered() {
        let r = parse(r#"{"tag_name":"v0.2.0","html_url":"https://github.com/SarthAyush/PDFcatalyst-App/releases/tag/v0.2.0"}"#).unwrap();
        assert_eq!(r, Release::new("v0.2.0", "https://github.com/SarthAyush/PDFcatalyst-App/releases/tag/v0.2.0"));
        for elsewhere in ["https://example.com/pdfcraft.exe", "https://github.com/SarthAyush/PDFcatalyst-App/releases.evil/x", "javascript:alert(1)"]
        {
            let r = parse(&format!(r#"{{"tag_name":"v9.9.9","html_url":"{elsewhere}"}}"#)).unwrap();
            assert_eq!(r.url, RELEASES_PAGE, "{elsewhere}");
        }
        assert!(parse(r#"{"message":"Not Found"}"#).is_err());
        assert!(parse("<html>").is_err());
    }

    #[test]
    fn rate_limit_messages_name_the_wait() {
        let limited = "GitHub is limiting update checks for now (shared hourly quota); try again in about 20 minutes";
        assert_eq!(rate_limited(Some("1800001200"), 1800000000), limited);
        // Rounds up: 61 seconds out reads as 2 minutes.
        assert_eq!(
            rate_limited(Some("1800000061"), 1800000000),
            "GitHub is limiting update checks for now (shared hourly quota); try again in about 2 minutes"
        );
        assert_eq!(
            rate_limited(Some("1800000060"), 1800000000),
            "GitHub is limiting update checks for now (shared hourly quota); try again in about 1 minute"
        );
        // No header, not a number, or already past: ask for patience instead of a time.
        let later = "GitHub is limiting update checks for now (shared hourly quota); try again later";
        for bad in [None, Some(""), Some("soon"), Some("1799999999"), Some("1800000000")] {
            assert_eq!(rate_limited(bad, 1800000000), later);
        }
    }

    /// Live: asks GitHub over TLS with the OS's roots (`cargo test -p pdfcraft -- --ignored`).
    #[test]
    #[ignore = "needs network access"]
    fn github_answers_with_the_latest_release() {
        let r = latest_release().unwrap();
        assert!(pdfcraft_ui_egui::updates::is_newer(&r.version, "0.0.0"), "{r:?}");
        assert!(r.url.starts_with(RELEASES_PAGE), "{r:?}");
    }
}
