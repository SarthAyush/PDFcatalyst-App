//! Home tab: recommended tools, open card, pinned folders and recent files (local only, never
//! another app's list).

use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, vec2};
use pdfcraft_engine::catalog;

use crate::theme::{self, Tokens};
use crate::{Mode, PdfCraftApp, icons, panels::human_size, widgets};

const RECOMMENDED: [&str; 5] = ["organize", "comment", "form", "edit", "protect"];

/// A soft shadow that rises on hover and settles when the card is pressed.
fn card_lift(ui: &egui::Ui, t: &Tokens, rect: Rect, press: theme::Press) {
    let lift = press.hover * (1.0 - press.down);
    if lift > 0.04 {
        let shadow = egui::Shadow { offset: [0, 3], blur: (14.0 * lift) as u8, spread: 0, color: t.page_shadow.gamma_multiply(0.85 * lift) };
        ui.painter().add(shadow.as_shape(rect, CornerRadius::same(10)));
    }
}

struct DesignTemplate {
    category: usize,
    category_key: &'static str,
    badge: &'static str,
    title: &'static str,
    desc: &'static str,
    dim_label: &'static str,
    icon: &'static str,
    hue: Color32,
    default_filename: &'static str,
    width: f64,
    height: f64,
    pages: usize,
}

const TEMPLATES: [DesignTemplate; 6] = [
    DesignTemplate {
        category: 1,
        category_key: "blank",
        badge: "Standard",
        title: "Blank Canvas",
        desc: "Clean white letter-sized sheet, ready for adding text, vectors, annotations and signatures.",
        dim_label: "Letter · 8.5\" × 11\"",
        icon: "file-plus-2",
        hue: Color32::from_rgb(0x3B, 0x82, 0xF6),
        default_filename: "Blank Document.pdf",
        width: 612.0,
        height: 792.0,
        pages: 1,
    },
    DesignTemplate {
        category: 3,
        category_key: "invoice",
        badge: "Finance",
        title: "Modern Invoice",
        desc: "Professional layout tailored for client invoicing, itemized billing tables, and bank details.",
        dim_label: "A4 · 210 × 297 mm",
        icon: "file-spreadsheet",
        hue: Color32::from_rgb(0x10, 0xB9, 0x81),
        default_filename: "Invoice Template.pdf",
        width: 595.28,
        height: 841.89,
        pages: 1,
    },
    DesignTemplate {
        category: 3,
        category_key: "proposal",
        badge: "3 Pages",
        title: "Project Proposal",
        desc: "Complete executive proposal featuring Title Cover, Scope of Work, and Project Timeline.",
        dim_label: "Letter · 3 Pages",
        icon: "file-text",
        hue: Color32::from_rgb(0x8B, 0x5C, 0xF6),
        default_filename: "Project Proposal.pdf",
        width: 612.0,
        height: 792.0,
        pages: 3,
    },
    DesignTemplate {
        category: 1,
        category_key: "resume",
        badge: "Career",
        title: "Professional Resume",
        desc: "Clean two-column CV template highlighting experience, technical skills, and achievements.",
        dim_label: "Letter · 8.5\" × 11\"",
        icon: "file-check",
        hue: Color32::from_rgb(0xEC, 0x48, 0x99),
        default_filename: "Resume Template.pdf",
        width: 612.0,
        height: 792.0,
        pages: 1,
    },
    DesignTemplate {
        category: 2,
        category_key: "notebook",
        badge: "Technical",
        title: "Engineering Notebook",
        desc: "Multi-page technical journal with standard margins and 5mm grid structure for formulas and sketches.",
        dim_label: "A4 · 2 Pages",
        icon: "grid-3x3",
        hue: Color32::from_rgb(0xF5, 0x9E, 0x0B),
        default_filename: "Engineering Notebook.pdf",
        width: 595.28,
        height: 841.89,
        pages: 2,
    },
    DesignTemplate {
        category: 2,
        category_key: "certificate",
        badge: "Landscape",
        title: "Award Certificate",
        desc: "Distinguished certificate template featuring perimeter framing, presentation header, and signature lines.",
        dim_label: "11\" × 8.5\" · Landscape",
        icon: "badge-check",
        hue: Color32::from_rgb(0x06, 0xB6, 0xD4),
        default_filename: "Certificate of Completion.pdf",
        width: 792.0,
        height: 612.0,
        pages: 1,
    },
];

fn template_card(
    ui: &mut egui::Ui,
    t: &Tokens,
    tmpl: &DesignTemplate,
    width: f32,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 130.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tmpl.title));
    let press = theme::Press::track(ui, &resp);
    card_lift(ui, t, rect, press);
    let fill = press.fill(t.card, t.hover, t.pressed);
    let border_color = if resp.hovered() { tmpl.hue.gamma_multiply(0.8) } else { t.border };
    ui.painter().rect(rect, CornerRadius::same(12), fill, Stroke::new(1.2, border_color), egui::StrokeKind::Inside);
    let body = rect.translate(press.offset());

    icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(20.0, 20.0)), tmpl.icon, 19.0, tmpl.hue);

    let badge_job = egui::text::LayoutJob::simple_singleline(tmpl.badge.to_uppercase(), theme::semibold(9.5), tmpl.hue);
    let badge_galley = ui.fonts_mut(|f| f.layout_job(badge_job));
    ui.painter().galley(body.min + vec2(40.0, 16.0), badge_galley, tmpl.hue);

    let dim_job = egui::text::LayoutJob::simple_singleline(tmpl.dim_label.to_owned(), theme::medium(10.0), t.text_faint);
    let dim_galley = ui.fonts_mut(|f| f.layout_job(dim_job));
    ui.painter().galley(body.right_top() + vec2(-dim_galley.size().x - 14.0, 16.0), dim_galley, t.text_faint);

    let mut title_job = egui::text::LayoutJob::simple_singleline(tmpl.title.to_owned(), theme::semibold(13.5), t.text);
    title_job.wrap = egui::text::TextWrapping::truncate_at_width((body.width() - 28.0).max(10.0));
    let title_galley = ui.fonts_mut(|f| f.layout_job(title_job));
    ui.painter().galley(body.min + vec2(14.0, 42.0), title_galley, t.text);

    let mut desc_job = egui::text::LayoutJob::simple(tmpl.desc.to_owned(), theme::regular(11.0), t.text_muted, (body.width() - 28.0).max(10.0));
    desc_job.wrap.max_rows = 2;
    let desc_galley = ui.fonts_mut(|f| f.layout_job(desc_job));
    ui.painter().galley(body.min + vec2(14.0, 62.0), desc_galley, t.text_muted);

    let action_text = if tmpl.category_key == "blank" {
        if resp.hovered() { "Create Blank Now →" } else { "Create Blank →" }
    } else {
        if resp.hovered() { "Customize Styles →" } else { "Choose Style →" }
    };
    ui.painter().text(
        body.left_bottom() + vec2(14.0, -14.0),
        Align2::LEFT_CENTER,
        action_text,
        theme::semibold(11.5),
        if resp.hovered() { tmpl.hue } else { t.accent_text },
    );

    theme::hand(resp)
}

fn workflow_card(
    ui: &mut egui::Ui,
    t: &Tokens,
    icon: &str,
    color: Color32,
    badge: &str,
    title: &str,
    desc: &str,
    action_label: &str,
    width: f32,
) -> egui::Response {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 114.0), Sense::click());
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, title));
    let press = theme::Press::track(ui, &resp);
    card_lift(ui, t, rect, press);
    let fill = press.fill(t.card, t.hover, t.pressed);
    let border_color = if resp.hovered() { color.gamma_multiply(0.7) } else { t.border };
    ui.painter().rect(rect, CornerRadius::same(12), fill, Stroke::new(1.0, border_color), egui::StrokeKind::Inside);
    let body = rect.translate(press.offset());

    icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(20.0, 20.0)), icon, 18.0, color);

    let badge_job = egui::text::LayoutJob::simple_singleline(badge.to_uppercase(), theme::semibold(10.0), color);
    let badge_galley = ui.fonts_mut(|f| f.layout_job(badge_job));
    ui.painter().galley(body.min + vec2(40.0, 16.0), badge_galley, color);

    let mut title_job = egui::text::LayoutJob::simple_singleline(title.to_owned(), theme::semibold(13.5), t.text);
    title_job.wrap = egui::text::TextWrapping::truncate_at_width((body.width() - 28.0).max(10.0));
    let title_galley = ui.fonts_mut(|f| f.layout_job(title_job));
    ui.painter().galley(body.min + vec2(14.0, 40.0), title_galley, t.text);

    let mut desc_job = egui::text::LayoutJob::simple(desc.to_owned(), theme::regular(11.0), t.text_muted, (body.width() - 28.0).max(10.0));
    desc_job.wrap.max_rows = 2;
    let desc_galley = ui.fonts_mut(|f| f.layout_job(desc_job));
    ui.painter().galley(body.min + vec2(14.0, 60.0), desc_galley, t.text_muted);

    ui.painter().text(
        body.left_bottom() + vec2(14.0, -13.0),
        Align2::LEFT_CENTER,
        format!("{} →", action_label),
        theme::semibold(11.5),
        color,
    );

    theme::hand(resp)
}

pub fn show(app: &mut PdfCraftApp, ui: &mut egui::Ui) {
    let t = Tokens::get(ui.ctx());
    egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        egui::Frame::NONE.inner_margin(egui::Margin { left: 36, right: 36, top: 28, bottom: 28 }).show(ui, |ui| {
            // Hero Studio Header Card
            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.5, t.accent.gamma_multiply(0.4)))
                .corner_radius(CornerRadius::same(16))
                .inner_margin(egui::Margin::symmetric(24, 20))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        egui::Frame::NONE
                            .fill(t.accent_soft)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(10, 4))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("PDFIX STUDIO 2.0")
                                        .font(theme::semibold(11.0))
                                        .color(t.accent_text)
                                        .extra_letter_spacing(0.6),
                                );
                            });
                        ui.add_space(4.0);
                        egui::Frame::NONE
                            .fill(t.hover)
                            .stroke(Stroke::new(1.0, t.border))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new(format!("v{}", env!("CARGO_PKG_VERSION")))
                                        .font(theme::medium(11.0))
                                        .color(t.text_muted),
                                );
                            });
                        ui.add_space(4.0);
                        egui::Frame::NONE
                            .fill(t.hover)
                            .stroke(Stroke::new(1.0, t.border))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("MEMORY-SAFE RUST")
                                        .font(theme::semibold(11.0))
                                        .color(t.accent_text),
                                );
                            });
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("PDFix").font(theme::semibold(28.0)).color(t.text));
                        ui.add_space(8.0);
                        egui::Frame::NONE
                            .fill(t.accent_soft)
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.label(
                                    egui::RichText::new("Configured by Sarthak")
                                        .font(theme::semibold(11.5))
                                        .color(t.accent_text),
                                );
                            });
                    });
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("High-performance personal PDF engineering workbench — local, private, and blazing fast.")
                            .color(t.text_muted)
                            .font(theme::regular(14.0)),
                    );
                    ui.add_space(4.0);
                    ui.label(
                        egui::RichText::new(crate::tech_jokes::dynamic_greeting())
                            .color(t.accent_text)
                            .font(theme::medium(13.0)),
                    );
                    ui.add_space(8.0);
                    egui::Frame::NONE
                        .fill(t.pasteboard)
                        .stroke(Stroke::new(1.0, t.border))
                        .corner_radius(CornerRadius::same(8))
                        .inner_margin(egui::Margin::symmetric(12, 7))
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new("Tech Byte:").font(theme::semibold(12.0)).color(t.accent));
                                ui.add_space(4.0);
                                ui.label(egui::RichText::new(crate::tech_jokes::current_joke()).font(theme::regular(12.0)).color(t.text));
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.small_button("Next Joke").clicked() {
                                        crate::tech_jokes::next_joke();
                                    }
                                });
                            });
                        });
                    ui.add_space(10.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing = vec2(6.0, 2.0);
                        ui.label(egui::RichText::new("Pure Rust Engine").font(theme::medium(12.0)).color(t.text_muted));
                        ui.label(egui::RichText::new("·").color(t.text_faint));
                        ui.label(egui::RichText::new("100% Offline & Private").font(theme::medium(12.0)).color(t.text_muted));
                        ui.label(egui::RichText::new("·").color(t.text_faint));
                        ui.label(egui::RichText::new("Zero Telemetry").font(theme::medium(12.0)).color(t.text_muted));
                        ui.label(egui::RichText::new("·").color(t.text_faint));
                        ui.label(egui::RichText::new("GPU Accelerated").font(theme::medium(12.0)).color(t.text_muted));
                    });
                    ui.add_space(14.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(10.0, 8.0);
                        if widgets::pill_button(ui, "Open PDF Document", true).clicked() {
                            app.open_dialog();
                        }
                        if widgets::pill_button(ui, "Combine Files", false).clicked() {
                            app.open_combine_tab();
                        }
                        if widgets::pill_button(ui, "Batch Studio", false).clicked() {
                            app.open_batch_studio();
                        }
                        if widgets::pill_button(ui, "All Tools Suite", false).clicked() {
                            app.switch_mode(Mode::AllTools, true);
                        }
                        if widgets::icon_pill(ui, "github", "GitHub Repository", false).clicked() {
                            app.open_url("https://github.com/SarthAyush/PDFix-App");
                        }
                    });
                });
            ui.add_space(18.0);

            // Interactive Drag & Drop Box
            let (drop_rect, drop_resp) = ui.allocate_exact_size(vec2(ui.available_width(), 72.0), Sense::click());
            let drop_press = theme::Press::track(ui, &drop_resp);
            card_lift(ui, &t, drop_rect, drop_press);
            let drop_fill = drop_press.fill(t.pasteboard, t.hover, t.pressed);
            let drop_border = if drop_resp.hovered() { t.accent } else { t.border };
            ui.painter().rect(drop_rect, CornerRadius::same(12), drop_fill, Stroke::new(1.5, drop_border), egui::StrokeKind::Inside);
            let drop_body = drop_rect.translate(drop_press.offset());
            icons::paint(ui, Rect::from_center_size(drop_body.left_center() + vec2(38.0, 0.0), vec2(30.0, 30.0)), "folder-open", 26.0, t.accent);
            ui.painter().text(
                drop_body.left_center() + vec2(72.0, -9.0),
                Align2::LEFT_CENTER,
                tl!("Open file"),
                theme::semibold(14.5),
                t.text,
            );
            ui.painter().text(
                drop_body.left_center() + vec2(72.0, 11.0),
                Align2::LEFT_CENTER,
                "Drop any PDF document here to start editing immediately, or click to browse files",
                theme::regular(12.0),
                t.text_muted,
            );
            let browse_box = Rect::from_center_size(drop_body.right_center() - vec2(76.0, 0.0), vec2(110.0, 32.0));
            ui.painter().rect(browse_box, CornerRadius::same(8), t.card, Stroke::new(1.0, t.border), egui::StrokeKind::Inside);
            ui.painter().text(browse_box.center(), Align2::CENTER_CENTER, "Browse File", theme::semibold(12.5), t.accent_text);
            if theme::hand(drop_resp).clicked() {
                app.open_dialog();
            }
            ui.add_space(20.0);

            // Interactive Design Templates Section
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Interactive Design Templates").font(theme::semibold(16.0)).color(t.text));
                ui.add_space(8.0);
                egui::Frame::NONE
                    .fill(t.accent_soft)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(8, 2))
                    .show(ui, |ui| {
                        ui.label(egui::RichText::new("NEW").font(theme::semibold(10.0)).color(t.accent_text));
                    });
            });
            ui.add_space(2.0);
            ui.label(
                egui::RichText::new("Jump-start documents with instant pre-configured formats — zero setup required.")
                    .font(theme::regular(12.5))
                    .color(t.text_muted),
            );
            ui.add_space(10.0);

            // Category Filter Tabs
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
                for (name, id) in [
                    ("All Templates", 0),
                    ("Documents", 1),
                    ("Engineering", 2),
                    ("Business", 3),
                    ("Custom Library", 4),
                ] {
                    let active = app.home_template_tab == id;
                    if widgets::pill_button(ui, name, active).clicked() {
                        app.home_template_tab = id;
                    }
                }
            });
            ui.add_space(12.0);

            // Templates Grid (3 columns on wide screens)
            let mut create_blank: Option<(&'static str, f64, f64, usize)> = None;
            let mut open_custom: Option<(String, String, usize, [u8; 3])> = None;
            let mut export_custom: Option<(String, String, usize, [u8; 3])> = None;
            let mut delete_custom_id: Option<String> = None;

            if app.home_template_tab == 4 {
                // Personal Library tab
                if app.custom_templates.is_empty() {
                    egui::Frame::NONE
                        .fill(t.card)
                        .stroke(Stroke::new(1.0, t.border))
                        .corner_radius(CornerRadius::same(12))
                        .inner_margin(egui::Margin::symmetric(24, 20))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.label(egui::RichText::new("Your Personal Template Library is Empty").font(theme::semibold(14.5)).color(t.text));
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("Customize any parametric template from the tabs above (Documents, Engineering, Business) and click \"Save As Custom...\" to store reusable templates here.")
                                    .font(theme::regular(12.0))
                                    .color(t.text_muted),
                            );
                        });
                } else {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                        let w = ((ui.available_width() - 24.0) / 3.0).max(210.0);
                        for ct in &app.custom_templates {
                            egui::Frame::NONE
                                .fill(t.card)
                                .stroke(Stroke::new(1.0, t.border))
                                .corner_radius(CornerRadius::same(12))
                                .inner_margin(egui::Margin::same(14))
                                .show(ui, |ui| {
                                    ui.set_width(w);
                                    ui.horizontal(|ui| {
                                        let tag_col = Color32::from_rgb(ct.color_rgb[0], ct.color_rgb[1], ct.color_rgb[2]);
                                        ui.painter().circle_filled(ui.cursor().min + vec2(6.0, 6.0), 5.0, tag_col);
                                        ui.add_space(16.0);
                                        ui.label(
                                            egui::RichText::new(ct.category_key.to_uppercase())
                                                .font(theme::semibold(9.5))
                                                .color(t.accent_text),
                                        );
                                    });
                                    ui.add_space(4.0);
                                    ui.label(egui::RichText::new(&ct.title).font(theme::semibold(13.5)).color(t.text));
                                    ui.label(egui::RichText::new(&ct.description).font(theme::regular(11.0)).color(t.text_muted));
                                    ui.add_space(10.0);
                                    ui.horizontal(|ui| {
                                        if widgets::pill_button(ui, "Use Template", true).clicked() {
                                            open_custom = Some((ct.default_filename.clone(), ct.category_key.clone(), ct.selected_style, ct.color_rgb));
                                        }
                                        ui.add_space(4.0);
                                        if widgets::pill_button(ui, "Export", false).clicked() {
                                            export_custom = Some((ct.default_filename.clone(), ct.category_key.clone(), ct.selected_style, ct.color_rgb));
                                        }
                                        ui.add_space(4.0);
                                        if ui.small_button("×").on_hover_text("Delete from personal library").clicked() {
                                            delete_custom_id = Some(ct.id.clone());
                                        }
                                    });
                                });
                        }
                    });
                }
            } else {
                ui.horizontal_wrapped(|ui| {
                    ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                    let w = ((ui.available_width() - 24.0) / 3.0).max(210.0);
                    for tmpl in &TEMPLATES {
                        if app.home_template_tab != 0 && tmpl.category != app.home_template_tab {
                            continue;
                        }
                        if template_card(ui, &t, tmpl, w).clicked() {
                            if tmpl.category_key == "blank" {
                                create_blank = Some((tmpl.default_filename, tmpl.width, tmpl.height, tmpl.pages));
                            } else {
                                app.template_picker = Some(crate::TemplatePickerState::new(
                                    tmpl.category_key,
                                    tmpl.title,
                                    tmpl.default_filename,
                                ));
                            }
                        }
                    }
                });
            }

            if let Some((name, w, h, pages)) = create_blank {
                app.create_template_document(name, w, h, pages);
            }
            if let Some((name, cat, style, color)) = open_custom {
                app.create_styled_template_document(&name, &cat, style, color);
            }
            if let Some((name, cat, style, color)) = export_custom {
                let file_path = rfd::FileDialog::new()
                    .set_title("Export Custom Template PDF")
                    .set_file_name(&name)
                    .add_filter("PDF Document", &["pdf"])
                    .save_file();
                if let Some(path) = file_path {
                    match app.session.create_styled_template(&cat, style, color) {
                        Ok(bytes) => {
                            if std::fs::write(&path, &*bytes).is_ok() {
                                app.toast = Some((format!("Exported template PDF to {}", path.file_name().and_then(|n| n.to_str()).unwrap_or("file.pdf")), 0.0));
                            } else {
                                app.toast = Some(("Failed to write exported PDF to disk".to_string(), 0.0));
                            }
                        }
                        Err(e) => {
                            app.toast = Some((format!("Export failed: {e}"), 0.0));
                        }
                    }
                }
            }
            if let Some(del_id) = delete_custom_id {
                app.custom_templates.retain(|c| c.id != del_id);
                app.toast = Some(("Removed template from personal library".to_string(), 0.0));
            }
            ui.add_space(22.0);

            // Capabilities Grid (5 Quick Launch Cards)
            ui.label(egui::RichText::new("Studio Capabilities").font(theme::semibold(15.0)));
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(12.0, 12.0);
                let w = ((ui.available_width() - 48.0) / 5.0).max(160.0);
                
                // 1. Reader
                if workflow_card(
                    ui, &t, "book-open", egui::Color32::from_rgb(0x3B, 0x82, 0xF6),
                    "Viewer", "Document Reader",
                    "High-DPI rendering, search, bookmarks & dual-page view.",
                    "Open Reader", w
                ).clicked() {
                    app.switch_mode(Mode::Read, true);
                }

                // 2. Organizer
                if workflow_card(
                    ui, &t, "layers", egui::Color32::from_rgb(0x8B, 0x5C, 0xF6),
                    "Pages", "Page Organizer",
                    "Reorder, extract, rotate, split & merge pages easily.",
                    "Organize", w
                ).clicked() {
                    app.select_tool("organize");
                }

                // 3. Forms & Annotations
                if workflow_card(
                    ui, &t, "pencil", egui::Color32::from_rgb(0xEC, 0x48, 0x99),
                    "Forms", "Interactive Forms",
                    "Fill and save PDF forms, add notes, highlights & stamps.",
                    "Fill Forms", w
                ).clicked() {
                    app.select_tool("form");
                }

                // 4. Security & Privacy
                if workflow_card(
                    ui, &t, "lock", egui::Color32::from_rgb(0x10, 0xB9, 0x81),
                    "Security", "Protect & Sign",
                    "Digital signatures, password encryption & sanitize metadata.",
                    "Protect", w
                ).clicked() {
                    app.select_tool("protect");
                }

                // 5. Batch Workflow Studio
                if workflow_card(
                    ui, &t, "folder-plus", egui::Color32::from_rgb(0xF5, 0x9E, 0x0B),
                    "Batch", "Batch Studio",
                    "Drop 20+ images/PDFs to merge, convert, compress or watermark.",
                    "Open Studio", w
                ).clicked() {
                    app.open_batch_studio();
                }
            });
            ui.add_space(22.0);

            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::same(18))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.label(egui::RichText::new(tl!("Recommended tools")).font(theme::semibold(15.0)));
                    ui.add_space(10.0);
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(14.0, 14.0);
                        for id in RECOMMENDED {
                            let Some(g) = catalog::group(id) else { continue };
                            let (rect, resp) = ui.allocate_exact_size(vec2(190.0, 104.0), Sense::click());
                            resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!(g.label)));
                            let press = theme::Press::track(ui, &resp);
                            card_lift(ui, &t, rect, press);
                            let fill = press.fill(t.card, t.hover, t.pressed);
                            ui.painter().rect(rect, CornerRadius::same(10), fill, Stroke::new(1.0, t.divider), egui::StrokeKind::Inside);
                            let body = rect.translate(press.offset());
                            let color = egui::Color32::from_rgb(g.hue[0], g.hue[1], g.hue[2]);
                            icons::paint(ui, Rect::from_min_size(body.min + vec2(14.0, 14.0), vec2(22.0, 22.0)), g.icon, 21.0, color);
                            let mut title_job = egui::text::LayoutJob::simple_singleline(tl!(g.label).to_owned(), theme::semibold(13.5), t.text);
                            title_job.wrap =
                                egui::text::TextWrapping { max_width: body.width() - 58.0, max_rows: 1, break_anywhere: true, ..Default::default() };
                            let title = ui.fonts_mut(|f| f.layout_job(title_job));
                            let title_elided = title.elided;
                            ui.painter().galley(body.min + vec2(44.0, 25.0 - title.size().y * 0.5), title, t.text);
                            let blurb = g
                                .sections
                                .first()
                                .map(|s| s.items.iter().take(3).map(|i| tl!(i.label)).collect::<Vec<_>>().join(" · "))
                                .unwrap_or_default();
                            // Fixed-height cards must leave room for their action in every language.
                            let mut job = egui::text::LayoutJob::simple(blurb.clone(), theme::regular(11.5), t.text_muted, body.width() - 28.0);
                            job.wrap.max_rows = 2;
                            let galley = ui.fonts_mut(|f| f.layout_job(job));
                            let blurb_elided = galley.elided;
                            ui.painter().galley(body.min + vec2(14.0, 46.0), galley, t.text_muted);
                            ui.painter().text(
                                body.left_bottom() + vec2(14.0, -14.0),
                                Align2::LEFT_CENTER,
                                tl!("Use now"),
                                theme::medium(12.0),
                                t.accent_text,
                            );
                            let resp = if title_elided || blurb_elided { resp.on_hover_text(format!("{}\n{blurb}", tl!(g.label))) } else { resp };
                            if theme::hand(resp).clicked() {
                                app.select_tool(g.id);
                            }
                        }
                        let (rect, resp) = ui.allocate_exact_size(vec2(170.0, 104.0), Sense::click());
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Open file")));
                        let press = theme::Press::track(ui, &resp);
                        card_lift(ui, &t, rect, press);
                        ui.painter().rect(
                            rect,
                            CornerRadius::same(10),
                            press.fill(t.pasteboard, t.hover, t.pressed),
                            Stroke::new(1.0, t.divider),
                            egui::StrokeKind::Inside,
                        );
                        let body = rect.translate(press.offset());
                        icons::paint(ui, Rect::from_center_size(body.center() - vec2(0.0, 16.0), vec2(28.0, 28.0)), "folder-open", 26.0, t.icon);
                        ui.painter().text(body.center() + vec2(0.0, 22.0), Align2::CENTER_CENTER, tl!("Open file"), theme::semibold(13.0), t.text);
                        if theme::hand(resp).clicked() {
                            app.open_dialog();
                        }
                    });
                });

            #[cfg(not(target_arch = "wasm32"))]
            {
                ui.add_space(26.0);
                crate::folders_ui::section(app, ui);
            }

            ui.add_space(26.0);
            let mut clear = false;
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(tl!("Recent")).font(theme::semibold(17.0)));
                if !app.recent.is_empty() {
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let resp = widgets::ghost_button(ui, "trash-2", tl!("Clear"));
                        resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, tl!("Clear Recent Files")));
                        clear = resp.on_hover_text(tl!("Clear Recent Files")).clicked();
                    });
                }
            });
            ui.add_space(8.0);
            if app.recent.is_empty() {
                ui.label(egui::RichText::new("Files you open in PDFix appear here. Drop a PDF anywhere to open it.").color(t.text_muted));
            }
            let mut open = None;
            let mut remove = None;
            for r in &app.recent {
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width(), 46.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &r.name));
                let press = theme::Press::track(ui, &resp);
                // The row, not its response: the pointer stays "in" the row over its remove button.
                let hovered = ui.rect_contains_pointer(rect);
                if hovered && press.hover < 0.04 && press.down < 0.04 {
                    // The remove button sits in the row, so the row response may not be hovered.
                    ui.painter().rect_filled(rect, CornerRadius::same(8), t.hover);
                } else {
                    press.wash(ui, rect, 8, false);
                }
                if hovered {
                    // Remove just this file from the list (#430); the file itself is untouched.
                    let x_rect = Rect::from_center_size(rect.right_center() - vec2(26.0, 0.0), vec2(28.0, 28.0));
                    let x = ui.interact(x_rect, ui.id().with(("remove-recent", &r.path)), Sense::click());
                    let label = crate::i18n::fmt(tl!("Remove {name} from Recent"), &[("name", &r.name)]);
                    x.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, &label));
                    if x.hovered() {
                        ui.painter().rect_filled(x_rect, CornerRadius::same(6), t.pressed);
                    }
                    icons::paint(ui, x_rect.shrink(6.0), "x", 16.0, t.icon);
                    if x.on_hover_text(tl!("Remove from Recent")).clicked() {
                        remove = Some(r.path.clone());
                    }
                }
                let body = rect.translate(press.offset());
                icons::paint(
                    ui,
                    Rect::from_min_size(body.min + vec2(10.0, 11.0), vec2(24.0, 24.0)),
                    "file-text",
                    22.0,
                    egui::Color32::from_rgb(0xE0, 0x3E, 0x3E),
                );
                // Room on the right for the remove button.
                let detail = ui.painter().text(
                    body.right_center() - vec2(48.0, 0.0),
                    Align2::RIGHT_CENTER,
                    format!("{}  ·  {}", crate::i18n::trn(crate::i18n::current(), r.pages as u64, "{n} page", "{n} pages"), human_size(r.size)),
                    theme::regular(12.0),
                    t.text_muted,
                );
                // The name and path stop short of the page count; hovering shows the whole path.
                let width = detail.left() - 16.0 - (body.left() + 46.0);
                widgets::row_text(ui, body.min + vec2(46.0, 15.0), crate::bidi::visual(&r.name), theme::medium(13.5), t.text, width);
                widgets::row_text(ui, body.min + vec2(46.0, 32.0), crate::bidi::visual(&r.path), theme::regular(11.0), t.text_faint, width);
                if theme::hand(resp).on_hover_text(&r.path).clicked() {
                    open = Some(r.path.clone());
                }
            }
            if let Some(p) = open {
                app.open_recent(&p);
            }
            if let Some(p) = remove {
                app.recent.retain(|r| r.path != p);
            }
            if clear {
                app.execute("file.clear_recent");
            }
            ui.add_space(24.0);
            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.border))
                .corner_radius(CornerRadius::same(12))
                .inner_margin(egui::Margin::symmetric(20, 16))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        let (rect, _) = ui.allocate_exact_size(vec2(32.0, 32.0), Sense::hover());
                        icons::paint(ui, rect, "shield-check", 28.0, egui::Color32::from_rgb(0x10, 0xB9, 0x81));
                        ui.add_space(8.0);
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new("100% Offline & Private by Architecture").font(theme::semibold(14.0)).color(t.text));
                            ui.label(
                                egui::RichText::new("PDFix works completely offline. No telemetry, no remote analytics, no accounts, and zero cloud processing.")
                                    .font(theme::regular(12.0))
                                    .color(t.text_muted),
                            );
                        });
                    });
                });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translated_card_description_leaves_room_for_the_action() {
        let de = crate::i18n::Lang::from_code("de").unwrap();
        crate::i18n::set_current(de);
        let ctx = egui::Context::default();
        ctx.set_fonts(theme::font_definitions());
        let mut app = PdfCraftApp::new();
        let blurb = catalog::group("form")
            .unwrap()
            .sections
            .first()
            .unwrap()
            .items
            .iter()
            .take(3)
            .map(|item| crate::i18n::tr(de, item.label))
            .collect::<Vec<_>>()
            .join(" · ");
        let output = ctx
            .run_ui(egui::RawInput { screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, vec2(1440.0, 900.0))), ..Default::default() }, |ui| {
                show(&mut app, ui)
            });
        crate::i18n::set_current(crate::i18n::Lang::EN);
        let text_shapes: Vec<_> =
            output.shapes.iter().filter_map(|shape| if let egui::Shape::Text(text) = &shape.shape { Some(text) } else { None }).collect();
        let description = text_shapes.iter().find(|text| text.galley.job.text == blurb).unwrap();
        let action = text_shapes
            .iter()
            .find(|text| text.galley.job.text == crate::i18n::tr(de, "Use now") && (text.pos.x - description.pos.x).abs() < 0.1)
            .unwrap();
        assert!(description.galley.rows.len() <= 2);
        assert!(
            description.pos.y + description.galley.size().y + 4.0 <= action.pos.y,
            "the translated description must leave a visible gap before its action"
        );
        output.drop_without_applying_deltas();
    }
}
