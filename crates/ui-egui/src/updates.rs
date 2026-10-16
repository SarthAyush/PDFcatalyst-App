//! Help ▸ Check for updates: check for the latest release, download directly in-app, and install.

use std::sync::Arc;

use egui::{Align, Color32, CornerRadius, Layout, Stroke};

use crate::{PdfCraftApp, theme, widgets};

/// Where every CraftPDF release is listed.
pub const RELEASES_PAGE: &str = "https://github.com/SarthAyush/PDFix-App/releases";

/// The latest published release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Release {
    /// Its version tag, such as `v0.2.0`.
    pub version: String,
    /// Its page on [`RELEASES_PAGE`], where the downloads are.
    pub url: String,
    /// Direct asset download URL (e.g. for installer .exe).
    pub download_url: Option<String>,
    /// Filename of the asset (e.g. CraftPDF-Setup-v0.8.1-x64.exe).
    pub asset_name: Option<String>,
}

impl Release {
    pub fn new(version: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            url: url.into(),
            download_url: None,
            asset_name: None,
        }
    }
}

/// Asks for the latest release (blocking; it runs on its own thread).
pub type UpdateSource = Arc<dyn Fn() -> Result<Release, String> + Send + Sync>;

/// Asynchronously downloads a release asset with progress updates.
pub type UpdateDownloader = Arc<
    dyn Fn(Release, std::sync::mpsc::Sender<DownloadProgress>, Option<egui::Context>)
        + Send
        + Sync,
>;

/// Progress reports sent from the background download thread.
#[derive(Clone, Debug, PartialEq)]
pub enum DownloadProgress {
    Connecting,
    Downloading { downloaded: u64, total: Option<u64> },
    Finished { path: std::path::PathBuf },
    Failed(String),
}

/// Current state of the in-app download and installer.
#[derive(Default)]
pub enum DownloadState {
    #[default]
    Idle,
    Downloading {
        #[cfg(not(target_arch = "wasm32"))]
        rx: std::sync::mpsc::Receiver<DownloadProgress>,
        downloaded: u64,
        total: Option<u64>,
    },
    ReadyToInstall {
        path: std::path::PathBuf,
    },
    Installing,
    Failed(String),
}

/// Whether release `latest` (a tag such as `v0.2.0`) is newer than version `current` (`0.1.1`).
/// Pre-release and build suffixes are ignored; a version that doesn't parse is never newer.
pub fn is_newer(latest: &str, current: &str) -> bool {
    matches!((parse(latest), parse(current)), (Some(l), Some(c)) if l > c)
}

fn parse(v: &str) -> Option<(u64, u64, u64)> {
    let v = v.trim().trim_start_matches(['v', 'V']);
    let core = v.split(['-', '+']).next()?;
    let mut parts = core.split('.');
    let mut next = |required: bool| match parts.next() {
        Some(p) => p.parse::<u64>().ok(),
        None if required => None,
        None => Some(0),
    };
    let version = (next(true)?, next(false)?, next(false)?);
    parts.next().is_none().then_some(version)
}

/// Where a check is.
#[derive(Default)]
pub(crate) enum Check {
    #[default]
    Idle,
    #[cfg(not(target_arch = "wasm32"))]
    Running(std::sync::mpsc::Receiver<Result<Release, String>>),
    Done(Result<Release, String>),
}

#[derive(Default)]
pub(crate) struct Updates {
    pub(crate) check: Check,
    /// The Updates dialog is showing.
    pub(crate) open: bool,
    /// Whether this check was initiated silently in the background on startup.
    pub(crate) silent: bool,
    /// Current download / installer state.
    pub(crate) download_state: DownloadState,
}

impl PdfCraftApp {
    /// Help ▸ Check for updates: ask for the latest release and show the outcome.
    pub fn check_for_updates(&mut self) {
        let Some(source) = self.update_source.clone() else {
            self.open_url(RELEASES_PAGE);
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.updates.open = true;
            self.updates.silent = false;
            if matches!(self.updates.check, Check::Running(_)) {
                return;
            }
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.ctx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(source());
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
            self.updates.check = Check::Running(rx);
        }
        #[cfg(target_arch = "wasm32")]
        {
            let _ = source;
            self.open_url(RELEASES_PAGE);
        }
    }

    /// Check for updates silently on startup; notifies the user if a newer version is available.
    pub fn check_for_updates_silently(&mut self) {
        let Some(source) = self.update_source.clone() else {
            return;
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            if matches!(self.updates.check, Check::Running(_)) {
                return;
            }
            self.updates.silent = true;
            let (tx, rx) = std::sync::mpsc::channel();
            let ctx = self.ctx.clone();
            std::thread::spawn(move || {
                let _ = tx.send(source());
                if let Some(ctx) = ctx {
                    ctx.request_repaint();
                }
            });
            self.updates.check = Check::Running(rx);
        }
    }

    /// Starts downloading the release installer directly in-app.
    pub fn start_update_download(&mut self, release: Release) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if matches!(self.updates.download_state, DownloadState::Downloading { .. }) {
                return;
            }
            if let Some(downloader) = self.update_downloader.clone() {
                let (tx, rx) = std::sync::mpsc::channel();
                let ctx = self.ctx.clone();
                std::thread::spawn(move || {
                    downloader(release, tx, ctx);
                });
                self.updates.download_state = DownloadState::Downloading {
                    rx,
                    downloaded: 0,
                    total: None,
                };
            } else {
                self.open_url(&release.url);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.open_url(&release.url);
        }
    }

    /// Launches the downloaded installer executable.
    pub fn install_update(&mut self, path: &std::path::Path) {
        #[cfg(windows)]
        {
            match std::process::Command::new(path).spawn() {
                Ok(_) => {
                    self.toast = Some(("Installer launched! Follow setup prompts to complete the update.".into(), 0.0));
                    self.updates.download_state = DownloadState::Installing;
                    self.updates.open = false;
                    self.notification_viewer_open = false;
                }
                Err(e) => {
                    self.updates.download_state = DownloadState::Failed(format!("Failed to start installer: {e}"));
                }
            }
        }
        #[cfg(not(windows))]
        {
            let _ = path;
            self.toast = Some(("Update package ready.".into(), 0.0));
        }
    }

    /// Pick up finished check or download progress (each frame).
    pub(crate) fn poll_updates(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            if let Check::Running(rx) = &self.updates.check {
                let result = match rx.try_recv() {
                    Ok(r) => r,
                    Err(std::sync::mpsc::TryRecvError::Empty) => return,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        self.updates.check = Check::Done(Err("the update check stopped unexpectedly".into()));
                        return;
                    }
                };
                if let Ok(ref release) = result {
                    if is_newer(&release.version, env!("CARGO_PKG_VERSION")) {
                        self.updates.open = true;
                    }
                }
                self.updates.check = Check::Done(result);
            }

            if let DownloadState::Downloading { rx, downloaded, total } = &mut self.updates.download_state {
                while let Ok(msg) = rx.try_recv() {
                    match msg {
                        DownloadProgress::Connecting => {}
                        DownloadProgress::Downloading { downloaded: d, total: t } => {
                            *downloaded = d;
                            *total = t;
                        }
                        DownloadProgress::Finished { path } => {
                            self.updates.download_state = DownloadState::ReadyToInstall { path };
                            break;
                        }
                        DownloadProgress::Failed(err) => {
                            self.updates.download_state = DownloadState::Failed(err);
                            break;
                        }
                    }
                }
            }
        }
    }
}

/// The Updates dialog.
pub(crate) fn dialog(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if !app.updates.open {
        return;
    }
    let t = theme::Tokens::get(ctx);
    let current = env!("CARGO_PKG_VERSION");
    let mut close = false;

    let modal = crate::widgets::show_modal(ctx, "updates", |ui| {
        ui.set_width(440.0);
        ui.horizontal(|ui| {
            ui.add(crate::icons::image("cloud", 22.0, t.accent));
            ui.label(egui::RichText::new(tl!("Check for updates")).font(theme::semibold(16.0)));
        });
        ui.add_space(8.0);
        match &app.updates.check {
            Check::Idle => {
                ui.label(tl!("No check has run yet."));
            }
            #[cfg(not(target_arch = "wasm32"))]
            Check::Running(_) => {
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(tl!("Checking for a newer version…"));
                });
            }
            Check::Done(Ok(r)) if is_newer(&r.version, current) => {
                let version = r.version.trim_start_matches(['v', 'V']);
                ui.label(egui::RichText::new(format!("PDFix {version} is available.")).strong());
                ui.label(
                    egui::RichText::new(format!("Current version installed: {current}"))
                        .color(t.text_muted),
                );
                ui.add_space(6.0);

                match &app.updates.download_state {
                    DownloadState::Idle => {
                        ui.label(
                            egui::RichText::new("Click Download to automatically fetch the update installer in the background.")
                                .color(t.text_muted)
                                .small(),
                        );
                    }
                    DownloadState::Downloading { downloaded, total, .. } => {
                        let mb_down = *downloaded as f64 / 1_048_576.0;
                        if let Some(tot) = *total {
                            let mb_tot = tot as f64 / 1_048_576.0;
                            let fraction = (*downloaded as f32 / tot as f32).clamp(0.0, 1.0);
                            ui.label(egui::RichText::new(format!("Downloading update… {:.1} MB / {:.1} MB ({:.0}%)", mb_down, mb_tot, fraction * 100.0)).font(theme::medium(12.5)).color(t.text));
                            ui.add_space(4.0);
                            ui.add(egui::ProgressBar::new(fraction).animate(true));
                        } else {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label(egui::RichText::new(format!("Downloading update… {:.1} MB", mb_down)).font(theme::medium(12.5)).color(t.text));
                            });
                        }
                    }
                    DownloadState::ReadyToInstall { .. } => {
                        egui::Frame::NONE
                            .fill(t.accent_soft)
                            .stroke(Stroke::new(1.0, t.accent))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(10, 8))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("Verified:").font(theme::semibold(12.0)).color(t.accent_text));
                                    ui.label(egui::RichText::new("Update package ready. Click Install to run setup.").font(theme::medium(12.0)).color(t.text));
                                });
                            });
                    }
                    DownloadState::Installing => {
                        ui.label(egui::RichText::new("Setup wizard launched. Please follow prompts.").font(theme::medium(12.0)).color(t.text));
                    }
                    DownloadState::Failed(err) => {
                        ui.label(egui::RichText::new(format!("Download failed: {err}")).font(theme::regular(12.0)).color(Color32::from_rgb(0xe0, 0x40, 0x40)));
                    }
                }
            }
            Check::Done(Ok(_)) => {
                ui.label(egui::RichText::new(crate::i18n::fmt(tl!("PDFix {c} is up to date."), &[("c", current)])).font(theme::semibold(14.0)).color(t.text));
            }
            Check::Done(Err(e)) => {
                ui.label(crate::i18n::fmt(tl!("Couldn't check for updates: {e}"), &[("e", &e.to_string())]));
                ui.label(
                    egui::RichText::new(crate::i18n::fmt(
                        tl!("You have version {c}. All releases are listed at {page}."),
                        &[("c", current), ("page", RELEASES_PAGE)],
                    ))
                    .color(t.text_muted),
                );
            }
        }
        ui.add_space(10.0);
        ui.label(
            egui::RichText::new("PDFix updates seamlessly without external redirects.")
                .color(t.text_muted)
                .small(),
        );
        ui.add_space(12.0);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if let Check::Done(Ok(r)) = &app.updates.check {
                if is_newer(&r.version, current) {
                    match &app.updates.download_state {
                        DownloadState::Idle | DownloadState::Failed(_) => {
                            let rel = r.clone();
                            if widgets::pill_button(ui, tl!("Download"), true).clicked() {
                                app.start_update_download(rel);
                            }
                            if widgets::pill_button(ui, tl!("Later"), false).clicked() {
                                close = true;
                            }
                        }
                        DownloadState::Downloading { .. } => {
                            if widgets::pill_button(ui, tl!("Later"), false).clicked() {
                                close = true;
                            }
                        }
                        DownloadState::ReadyToInstall { path } => {
                            let p = path.clone();
                            if widgets::pill_button(ui, "Install", true).clicked() {
                                app.install_update(&p);
                            }
                            if widgets::pill_button(ui, tl!("Later"), false).clicked() {
                                close = true;
                            }
                        }
                        DownloadState::Installing => {
                            if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                                close = true;
                            }
                        }
                    }
                } else if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                    close = true;
                }
            } else if widgets::pill_button(ui, tl!("Close"), true).clicked() {
                close = true;
            }
        });
    });

    if modal.should_close() || close {
        app.updates.open = false;
    }
}
