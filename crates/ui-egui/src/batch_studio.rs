//! Batch Workflow Studio
//!
//! High-performance batch processing for 20+ images and PDFs simultaneously.
//! Supports 4 primary workflows:
//! - Merge: Combines mixed images and PDFs into a single unified document.
//! - Convert: Batch converts images and documents into standalone PDF files.
//! - Compress: Optimizes and reduces file sizes across multiple PDFs concurrently.
//! - Watermark: Stamps custom text watermarks across all pages of all queued documents.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use egui::{vec2, Align2, Color32, CornerRadius, Sense, Stroke};
use crate::theme::{self, Tokens};
use crate::{widgets, PdfCraftApp};

/// Supported file kinds in the batch queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchFileKind {
    Pdf,
    Image,
    Unsupported,
}

impl BatchFileKind {
    pub fn from_path(path: &Path) -> Self {
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
        match ext.as_str() {
            "pdf" => Self::Pdf,
            "png" | "jpg" | "jpeg" | "webp" | "bmp" | "tiff" | "tif" | "gif" => Self::Image,
            _ => Self::Unsupported,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pdf => "PDF",
            Self::Image => "IMAGE",
            Self::Unsupported => "OTHER",
        }
    }
}

/// Processing status for an item in the batch queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BatchItemStatus {
    Queued,
    Processing,
    Done,
    Failed(String),
}

/// An individual item in the batch processing queue.
#[derive(Clone, Debug)]
pub struct BatchItem {
    pub path: PathBuf,
    pub filename: String,
    pub kind: BatchFileKind,
    pub size_bytes: u64,
    pub status: BatchItemStatus,
}

/// The available batch operations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchOperation {
    Merge,
    Convert,
    Compress,
    Watermark,
}

impl BatchOperation {
    pub fn title(self) -> &'static str {
        match self {
            Self::Merge => "Merge Documents",
            Self::Convert => "Convert Images to PDF",
            Self::Compress => "Batch Compression",
            Self::Watermark => "Stamp Watermark",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Merge => "Combine all queued PDFs and images into a single unified master PDF document.",
            Self::Convert => "Convert all queued image files into individual vector PDF documents.",
            Self::Compress => "Optimize and downsample image streams to significantly reduce PDF file size.",
            Self::Watermark => "Apply a custom text watermark across every page of all queued documents simultaneously.",
        }
    }
}

/// Settings for batch watermarking.
#[derive(Clone, Debug)]
pub struct BatchWatermarkSettings {
    pub text: String,
    pub opacity: f32,
    pub font_size: f32,
    pub rotation: f32,
    pub color_rgb: [u8; 3],
}

impl Default for BatchWatermarkSettings {
    fn default() -> Self {
        Self {
            text: "CONFIDENTIAL".to_string(),
            opacity: 0.35,
            font_size: 48.0,
            rotation: 45.0,
            color_rgb: [120, 120, 120],
        }
    }
}

/// State for the Batch Workflow Studio modal/view.
#[derive(Clone, Debug)]
pub struct BatchStudioState {
    pub items: Vec<BatchItem>,
    pub operation: BatchOperation,
    pub watermark_settings: BatchWatermarkSettings,
    pub compression_level: usize, // 0 = Balanced, 1 = Maximum
    pub is_running: bool,
    pub current_index: usize,
    pub total_processed: usize,
    pub total_errors: usize,
    pub status_message: Option<String>,
}

impl Default for BatchStudioState {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            operation: BatchOperation::Merge,
            watermark_settings: BatchWatermarkSettings::default(),
            compression_level: 0,
            is_running: false,
            current_index: 0,
            total_processed: 0,
            total_errors: 0,
            status_message: None,
        }
    }
}

impl BatchStudioState {
    /// Add multiple file paths to the queue.
    pub fn add_paths(&mut self, paths: Vec<PathBuf>) {
        for path in paths {
            let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("document").to_string();
            let kind = BatchFileKind::from_path(&path);
            let size_bytes = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            
            // Avoid duplicate exact paths
            if !self.items.iter().any(|item| item.path == path) {
                self.items.push(BatchItem {
                    path,
                    filename,
                    kind,
                    size_bytes,
                    status: BatchItemStatus::Queued,
                });
            }
        }
        self.status_message = Some(format!("{} files in queue", self.items.len()));
    }

    /// Clear all files from queue.
    pub fn clear(&mut self) {
        self.items.clear();
        self.total_processed = 0;
        self.total_errors = 0;
        self.status_message = None;
    }
}

/// Render the Batch Workflow Studio dialog.
pub fn show(app: &mut PdfCraftApp, ctx: &egui::Context) {
    if app.batch_studio.is_none() {
        return;
    }

    let mut close = false;
    let mut run_workflow = false;
    let mut select_more_files = false;
    let mut clear_all = false;
    let mut remove_idx: Option<usize> = None;

    let t = Tokens::get(ctx);

    let modal = widgets::show_modal(ctx, "batch_workflow_studio", |ui| {
        ui.set_width(780.0);

        let state = match &mut app.batch_studio {
            Some(s) => s,
            None => return,
        };

        // Header
        ui.horizontal(|ui| {
            egui::Frame::NONE
                .fill(t.accent_soft)
                .corner_radius(CornerRadius::same(6))
                .inner_margin(egui::Margin::symmetric(8, 4))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new("HIGH-THROUGHPUT ENGINE")
                            .font(theme::semibold(10.0))
                            .color(t.accent_text)
                            .extra_letter_spacing(0.5),
                    );
                });
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new("Batch Workflow Studio")
                    .font(theme::semibold(18.0))
                    .color(t.text),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::pill_button(ui, "Close", false).clicked() {
                    close = true;
                }
            });
        });
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Drop 20+ images or PDFs to merge, convert, compress, or stamp watermarks across all files simultaneously.")
                .font(theme::regular(12.5))
                .color(t.text_muted),
        );
        ui.add_space(14.0);

        // Section 1: Workflow Selector (4 Tabs)
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            for op in [
                BatchOperation::Merge,
                BatchOperation::Convert,
                BatchOperation::Compress,
                BatchOperation::Watermark,
            ] {
                let is_sel = state.operation == op;
                if widgets::pill_button(ui, op.title(), is_sel).clicked() {
                    state.operation = op;
                }
            }
        });
        ui.add_space(6.0);
        ui.label(
            egui::RichText::new(state.operation.description())
                .font(theme::regular(11.5))
                .color(t.text_faint),
        );
        ui.add_space(12.0);

        // Config Drawer for specific operations
        match state.operation {
            BatchOperation::Watermark => {
                egui::Frame::NONE
                    .fill(t.card)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Watermark Text:").font(theme::medium(12.0)));
                            ui.add(egui::TextEdit::singleline(&mut state.watermark_settings.text).desired_width(180.0));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Opacity:").font(theme::medium(12.0)));
                            ui.add(egui::Slider::new(&mut state.watermark_settings.opacity, 0.05..=1.0).text(""));
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new("Rotation:").font(theme::medium(12.0)));
                            ui.add(egui::Slider::new(&mut state.watermark_settings.rotation, 0.0..=90.0).suffix("°"));
                        });
                    });
                ui.add_space(10.0);
            }
            BatchOperation::Compress => {
                egui::Frame::NONE
                    .fill(t.card)
                    .stroke(Stroke::new(1.0, t.border))
                    .corner_radius(CornerRadius::same(8))
                    .inner_margin(egui::Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new("Compression Level:").font(theme::medium(12.0)));
                            ui.add_space(8.0);
                            for (lvl, title) in [(0, "Balanced (150 DPI JPEG)"), (1, "Maximum (96 DPI JPEG)")] {
                                let is_active = state.compression_level == lvl;
                                if widgets::pill_button(ui, title, is_active).clicked() {
                                    state.compression_level = lvl;
                                }
                            }
                        });
                    });
                ui.add_space(10.0);
            }
            _ => {}
        }

        // Section 2: Queue Header & Actions
        ui.horizontal(|ui| {
            let total_count = state.items.len();
            let total_size: u64 = state.items.iter().map(|i| i.size_bytes).sum();
            let size_fmt = crate::panels::human_size(total_size as usize);
            ui.label(
                egui::RichText::new(format!("FILE QUEUE ({total_count} files, {size_fmt})"))
                    .font(theme::semibold(11.0))
                    .color(t.text_muted)
                    .extra_letter_spacing(0.5),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::pill_button(ui, "Clear Queue", false).clicked() {
                    clear_all = true;
                }
                ui.add_space(6.0);
                if widgets::pill_button(ui, "Add Files...", true).clicked() {
                    select_more_files = true;
                }
            });
        });
        ui.add_space(6.0);

        // Section 3: File Queue Scroll Area (Supports 20+ items smoothly)
        let queue_height = 240.0;
        egui::Frame::NONE
            .fill(t.pasteboard)
            .stroke(Stroke::new(1.0, t.border))
            .corner_radius(CornerRadius::same(8))
            .inner_margin(egui::Margin::symmetric(10, 8))
            .show(ui, |ui| {
                if state.items.is_empty() {
                    let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), queue_height - 16.0), Sense::click());
                    let press = theme::Press::track(ui, &resp);
                    let border_col = if resp.hovered() { t.accent } else { t.border };
                    ui.painter().rect(rect, CornerRadius::same(6), press.fill(t.card, t.hover, t.pressed), Stroke::new(1.0, border_col), egui::StrokeKind::Inside);
                    ui.painter().text(
                        rect.center() - vec2(0.0, 10.0),
                        Align2::CENTER_CENTER,
                        "Drop 20+ images or PDF files here",
                        theme::semibold(14.0),
                        t.text,
                    );
                    ui.painter().text(
                        rect.center() + vec2(0.0, 12.0),
                        Align2::CENTER_CENTER,
                        "or click anywhere in this area to select files from your computer",
                        theme::regular(12.0),
                        t.text_muted,
                    );
                    if theme::hand(resp).clicked() {
                        select_more_files = true;
                    }
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(queue_height)
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 4.0);
                            for (idx, item) in state.items.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.set_width(ui.available_width());

                                    // File kind badge
                                    let badge_color = match item.kind {
                                        BatchFileKind::Pdf => Color32::from_rgb(0xDC, 0x26, 0x26),
                                        BatchFileKind::Image => Color32::from_rgb(0x25, 0x63, 0xEB),
                                        BatchFileKind::Unsupported => t.text_muted,
                                    };
                                    egui::Frame::NONE
                                        .fill(badge_color.gamma_multiply(0.15))
                                        .corner_radius(CornerRadius::same(4))
                                        .inner_margin(egui::Margin::symmetric(6, 2))
                                        .show(ui, |ui| {
                                            ui.label(
                                                egui::RichText::new(item.kind.label())
                                                    .font(theme::semibold(9.0))
                                                    .color(badge_color),
                                            );
                                        });
                                    ui.add_space(4.0);

                                    // Filename
                                    let name_text = if item.filename.len() > 42 {
                                        format!("{}...{}", &item.filename[..22], &item.filename[item.filename.len() - 16..])
                                    } else {
                                        item.filename.clone()
                                    };
                                    ui.label(egui::RichText::new(name_text).font(theme::medium(12.0)).color(t.text));

                                    // File size
                                    ui.add_space(6.0);
                                    ui.label(
                                        egui::RichText::new(crate::panels::human_size(item.size_bytes as usize))
                                            .font(theme::regular(11.0))
                                            .color(t.text_faint),
                                    );

                                    // Status & Remove
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.small_button("×").clicked() {
                                            remove_idx = Some(idx);
                                        }
                                        ui.add_space(4.0);
                                        match &item.status {
                                            BatchItemStatus::Queued => {
                                                ui.label(egui::RichText::new("Queued").font(theme::regular(11.0)).color(t.text_muted));
                                            }
                                            BatchItemStatus::Processing => {
                                                ui.label(egui::RichText::new("Processing...").font(theme::semibold(11.0)).color(t.accent));
                                            }
                                            BatchItemStatus::Done => {
                                                ui.label(egui::RichText::new("Done").font(theme::semibold(11.0)).color(Color32::from_rgb(0x16, 0xA3, 0x4A)));
                                            }
                                            BatchItemStatus::Failed(err) => {
                                                ui.label(egui::RichText::new(format!("Error: {err}")).font(theme::regular(11.0)).color(Color32::from_rgb(0xDC, 0x26, 0x26)));
                                            }
                                        }
                                    });
                                });
                                ui.separator();
                            }
                        });
                }
            });
        ui.add_space(14.0);

        // Section 4: Footer Status & Execution Bar
        ui.horizontal(|ui| {
            if let Some(msg) = &state.status_message {
                ui.label(egui::RichText::new(msg).font(theme::medium(12.0)).color(t.text_muted));
            } else {
                ui.label(egui::RichText::new("Ready to process batch").font(theme::regular(12.0)).color(t.text_muted));
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let btn_text = format!("Run {} →", state.operation.title());
                let can_run = !state.items.is_empty() && !state.is_running;
                ui.add_enabled_ui(can_run, |ui| {
                    if widgets::pill_button(ui, &btn_text, true).clicked() {
                        run_workflow = true;
                    }
                });
                ui.add_space(8.0);
                if widgets::pill_button(ui, "Cancel", false).clicked() {
                    close = true;
                }
            });
        });
    });

    if modal.should_close() || close {
        app.batch_studio = None;
        return;
    }

    if let Some(state) = &mut app.batch_studio {
        if clear_all {
            state.clear();
        }
        if let Some(idx) = remove_idx {
            if idx < state.items.len() {
                state.items.remove(idx);
            }
        }
    }

    if select_more_files {
        app.pick_batch_files();
    }

    if run_workflow {
        app.execute_batch_studio_workflow();
    }
}

impl PdfCraftApp {
    /// Open the Batch Studio modal.
    pub fn open_batch_studio(&mut self) {
        if self.batch_studio.is_none() {
            self.batch_studio = Some(BatchStudioState::default());
        }
    }

    /// Prompt user to pick 20+ files for the batch queue.
    pub fn pick_batch_files(&mut self) {
        let files = rfd::FileDialog::new()
            .set_title("Select Files for Batch Workflow Studio")
            .add_filter("PDF and Images", &["pdf", "png", "jpg", "jpeg", "webp", "bmp", "tiff", "gif"])
            .add_filter("PDF Documents", &["pdf"])
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "bmp", "tiff", "gif"])
            .pick_files();

        if let Some(paths) = files {
            if let Some(state) = &mut self.batch_studio {
                state.add_paths(paths);
            }
        }
    }

    /// Execute the active batch workflow across all queued files.
    pub fn execute_batch_studio_workflow(&mut self) {
        let (operation, items, wm_settings, comp_level) = match &self.batch_studio {
            Some(s) => (s.operation, s.items.clone(), s.watermark_settings.clone(), s.compression_level),
            None => return,
        };

        if items.is_empty() {
            return;
        }

        match operation {
            BatchOperation::Merge => {
                self.execute_batch_merge(&items);
            }
            BatchOperation::Convert => {
                self.execute_batch_convert(&items);
            }
            BatchOperation::Compress => {
                self.execute_batch_compress(&items, comp_level);
            }
            BatchOperation::Watermark => {
                self.execute_batch_watermark(&items, &wm_settings);
            }
        }
    }

    /// Batch Merge: Combines mixed images and PDFs into a single unified master document.
    fn execute_batch_merge(&mut self, items: &[BatchItem]) {
        let mut sources: Vec<(String, Arc<Vec<u8>>)> = Vec::new();
        let mut err_count = 0;

        for item in items {
            let bytes = match std::fs::read(&item.path) {
                Ok(b) => b,
                Err(_) => {
                    err_count += 1;
                    continue;
                }
            };

            match item.kind {
                BatchFileKind::Pdf => {
                    sources.push((item.filename.clone(), Arc::new(bytes)));
                }
                BatchFileKind::Image => {
                    // Convert image to 1-page PDF in-memory
                    match self.session.create_from_images(&[(item.filename.clone(), bytes)]) {
                        Ok(pdf_bytes) => {
                            sources.push((format!("{}.pdf", item.filename), pdf_bytes));
                        }
                        Err(_) => {
                            err_count += 1;
                        }
                    }
                }
                BatchFileKind::Unsupported => {
                    err_count += 1;
                }
            }
        }

        if sources.is_empty() {
            if let Some(state) = &mut self.batch_studio {
                state.status_message = Some(format!("Failed: No valid files could be read for merging ({err_count} errors)"));
            }
            return;
        }

        match self.session.combine(&sources) {
            Ok(merged_bytes) => {
                let doc_name = format!("Merged_Batch_{}_Files.pdf", sources.len());
                let _ = self.open_bytes(&doc_name, None, (*merged_bytes).clone());
                self.batch_studio = None;
                let err_msg = if err_count > 0 { format!(" ({err_count} unreadable files skipped)") } else { "".to_string() };
                self.toast = Some((format!("Successfully merged {} files into a unified PDF!{err_msg}", sources.len()), 0.0));
            }
            Err(e) => {
                if let Some(state) = &mut self.batch_studio {
                    state.status_message = Some(format!("Merge failed: {e}"));
                }
            }
        }
    }

    /// Batch Convert: Converts all images to standalone PDFs and prompts destination folder.
    fn execute_batch_convert(&mut self, items: &[BatchItem]) {
        let target_dir = match rfd::FileDialog::new().set_title("Select Output Folder for Converted PDFs").pick_folder() {
            Some(d) => d,
            None => return,
        };

        let mut success = 0;
        let mut failed = 0;

        for item in items {
            if item.kind != BatchFileKind::Image {
                continue;
            }
            let bytes = match std::fs::read(&item.path) {
                Ok(b) => b,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            match self.session.create_from_images(&[(item.filename.clone(), bytes)]) {
                Ok(pdf_bytes) => {
                    let mut out_path = target_dir.clone();
                    let stem = item.path.file_stem().and_then(|s| s.to_str()).unwrap_or("converted");
                    out_path.push(format!("{stem}.pdf"));
                    if std::fs::write(out_path, &*pdf_bytes).is_ok() {
                        success += 1;
                    } else {
                        failed += 1;
                    }
                }
                Err(_) => {
                    failed += 1;
                }
            }
        }

        if let Some(state) = &mut self.batch_studio {
            state.total_processed = success;
            state.total_errors = failed;
            state.status_message = Some(format!("Batch conversion complete: {success} succeeded, {failed} failed."));
            for item in &mut state.items {
                if item.kind == BatchFileKind::Image {
                    item.status = BatchItemStatus::Done;
                }
            }
        }
        self.toast = Some((format!("Converted {success} image(s) to PDF in selected folder"), 0.0));
    }

    /// Batch Compress: Optimizes multiple PDFs and saves with `_compressed.pdf` suffix or folder.
    fn execute_batch_compress(&mut self, items: &[BatchItem], level: usize) {
        let target_dir = match rfd::FileDialog::new().set_title("Select Output Folder for Compressed PDFs").pick_folder() {
            Some(d) => d,
            None => return,
        };

        let settings = if level == 1 {
            // Maximum compression
            pdfcraft_optimize::Settings {
                color: pdfcraft_optimize::ImageSettings {
                    downsample: true,
                    target_ppi: 96.0,
                    above_ppi: 140.0,
                    compression: pdfcraft_optimize::Compression::Jpeg(45),
                },
                gray: pdfcraft_optimize::ImageSettings {
                    downsample: true,
                    target_ppi: 96.0,
                    above_ppi: 140.0,
                    compression: pdfcraft_optimize::Compression::Jpeg(45),
                },
                ..Default::default()
            }
        } else {
            // Balanced
            pdfcraft_optimize::Settings::default()
        };

        let mut success = 0;
        let mut failed = 0;
        let mut total_saved: i64 = 0;

        for item in items {
            if item.kind != BatchFileKind::Pdf {
                continue;
            }
            let bytes = match std::fs::read(&item.path) {
                Ok(b) => b,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };
            let orig_len = bytes.len();

            let mut doc = match pdfcraft_cos::Document::open(Arc::new(bytes)) {
                Ok(d) => d,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let _ = pdfcraft_optimize::optimize(&mut doc, &settings);
            let opt_bytes = match pdfcraft_cos::write_full(&doc, &pdfcraft_cos::SaveOptions::default()) {
                Ok(b) => b,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let new_len = opt_bytes.len();
            total_saved += (orig_len as i64) - (new_len as i64);

            let mut out_path = target_dir.clone();
            let stem = item.path.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
            out_path.push(format!("{stem}_compressed.pdf"));
            if std::fs::write(out_path, &opt_bytes).is_ok() {
                success += 1;
            } else {
                failed += 1;
            }
        }

        let saved_str = if total_saved > 0 {
            format!(" Saved {}", crate::panels::human_size(total_saved as usize))
        } else {
            "".to_string()
        };

        if let Some(state) = &mut self.batch_studio {
            state.total_processed = success;
            state.total_errors = failed;
            state.status_message = Some(format!("Compressed {success} PDFs.{saved_str}"));
            for item in &mut state.items {
                if item.kind == BatchFileKind::Pdf {
                    item.status = BatchItemStatus::Done;
                }
            }
        }
        self.toast = Some((format!("Compressed {success} file(s).{saved_str}"), 0.0));
    }

    /// Batch Watermark: Stamps watermarks on all pages across all queued PDFs simultaneously.
    fn execute_batch_watermark(&mut self, items: &[BatchItem], wm: &BatchWatermarkSettings) {
        let target_dir = match rfd::FileDialog::new().set_title("Select Output Folder for Watermarked PDFs").pick_folder() {
            Some(d) => d,
            None => return,
        };

        let wm_def = pdfcraft_edit::Watermark {
            text: wm.text.clone(),
            source: None,
            scale: 1.0,
            font_size: wm.font_size as f64,
            color: [
                wm.color_rgb[0] as f64 / 255.0,
                wm.color_rgb[1] as f64 / 255.0,
                wm.color_rgb[2] as f64 / 255.0,
            ],
            opacity: wm.opacity as f64,
            rotation: wm.rotation as f64,
            behind: false,
            offset: [0.0, 0.0],
        };

        let mut success = 0;
        let mut failed = 0;

        for item in items {
            if item.kind != BatchFileKind::Pdf {
                continue;
            }
            let bytes = match std::fs::read(&item.path) {
                Ok(b) => b,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let mut doc = match pdfcraft_cos::Document::open(Arc::new(bytes)) {
                Ok(d) => d,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let page_count = pdfcraft_model::pages(&doc).len();
            let pages: Vec<usize> = (0..page_count).collect();

            if pdfcraft_edit::add_watermark(&mut doc, &pages, &wm_def, false).is_err() {
                failed += 1;
                continue;
            }

            let wm_bytes = match pdfcraft_cos::write_full(&doc, &pdfcraft_cos::SaveOptions::default()) {
                Ok(b) => b,
                Err(_) => {
                    failed += 1;
                    continue;
                }
            };

            let mut out_path = target_dir.clone();
            let stem = item.path.file_stem().and_then(|s| s.to_str()).unwrap_or("document");
            out_path.push(format!("{stem}_watermarked.pdf"));
            if std::fs::write(out_path, &wm_bytes).is_ok() {
                success += 1;
            } else {
                failed += 1;
            }
        }

        if let Some(state) = &mut self.batch_studio {
            state.total_processed = success;
            state.total_errors = failed;
            state.status_message = Some(format!("Watermarked {success} PDFs ({failed} failed)."));
            for item in &mut state.items {
                if item.kind == BatchFileKind::Pdf {
                    item.status = BatchItemStatus::Done;
                }
            }
        }
        self.toast = Some((format!("Stamped watermark on {success} PDF document(s)"), 0.0));
    }
}
