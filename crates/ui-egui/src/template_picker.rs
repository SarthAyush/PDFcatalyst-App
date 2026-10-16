//! Template style picker modal: customizer for parametric PDF templates with color palettes,
//! layout presets, export capabilities, and personal library saving.

use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, vec2};

use crate::theme::{self, Tokens};
use crate::{PdfCraftApp, widgets};

pub struct TemplateStyleOption {
    pub name: &'static str,
    pub desc: &'static str,
    pub badge: &'static str,
}

/// A user-saved reusable parametric template in their personal library.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct CustomTemplate {
    pub id: String,
    pub title: String,
    pub description: String,
    pub category_key: String,
    pub default_filename: String,
    pub selected_style: usize,
    pub selected_color: usize,
    pub color_rgb: [u8; 3],
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemplatePickerState {
    pub category_key: &'static str,
    pub category_title: &'static str,
    pub default_filename: &'static str,
    pub selected_style: usize,
    pub selected_color: usize,
    pub custom_title: String,
    pub custom_desc: String,
    pub show_save_drawer: bool,
}

impl TemplatePickerState {
    pub fn new(category_key: &'static str, category_title: &'static str, default_filename: &'static str) -> Self {
        Self {
            category_key,
            category_title,
            default_filename,
            selected_style: 0,
            selected_color: 0,
            custom_title: String::new(),
            custom_desc: String::new(),
            show_save_drawer: false,
        }
    }
}

pub const INVOICE_STYLES: &[TemplateStyleOption] = &[
    TemplateStyleOption { name: "Corporate Minimalist", desc: "Clean hairline table dividers, crisp mono totals & compact terms", badge: "Clean" },
    TemplateStyleOption { name: "Executive Header Banner", desc: "Full-bleed solid accent top banner with reversed white branding", badge: "Bold" },
    TemplateStyleOption { name: "Tech Stripe Modern", desc: "Vertical accent bar framing the left margin with tag badges", badge: "Modern" },
    TemplateStyleOption { name: "Perimeter Box Framing", desc: "Structured formal perimeter frame with inset client cards", badge: "Classic" },
    TemplateStyleOption {
        name: "Split Ledger Dual-Tone",
        desc: "Two-tone alternating line items with bold accent total due callout",
        badge: "Detailed",
    },
    TemplateStyleOption { name: "Compact Agency", desc: "High-density itemization tailored for multi-deliverable design studios", badge: "Compact" },
    TemplateStyleOption { name: "Creative Studio", desc: "Prominent billed-to cards with rounded highlights and wire details", badge: "Studio" },
    TemplateStyleOption { name: "International Swiss", desc: "Minimalist typographic hierarchy with clear VAT and IBAN fields", badge: "Swiss" },
    TemplateStyleOption {
        name: "Consulting Statement",
        desc: "Milestone breakdown with consultant hours and hourly rate totals",
        badge: "Consulting",
    },
    TemplateStyleOption { name: "Freelance Simple", desc: "Quick streamlined invoice with prominent net-30 payment due banner", badge: "Simple" },
];

pub const RESUME_STYLES: &[TemplateStyleOption] = &[
    TemplateStyleOption { name: "Modern Split-Column", desc: "Left colored accent sidebar with technical skills & right timeline", badge: "Sidebar" },
    TemplateStyleOption { name: "Executive Serif Classic", desc: "Single-column traditional layout with elegant rule dividers", badge: "Classic" },
    TemplateStyleOption { name: "Silicon Valley Tech", desc: "Clean tags, condensed experience, and highlighted GitHub repos", badge: "Tech" },
    TemplateStyleOption {
        name: "Creative Director",
        desc: "Accent header badge, dual-tone typography, and skill proficiency chips",
        badge: "Creative",
    },
    TemplateStyleOption { name: "Academic CV", desc: "Comprehensive sections for publications, grants, and teaching tenure", badge: "Academic" },
    TemplateStyleOption { name: "Startup Generalist", desc: "Compact impact-driven layout emphasizing key business metrics", badge: "Metrics" },
    TemplateStyleOption {
        name: "Minimalist Scandinavian",
        desc: "Generous whitespace, subtle divider rules, and clean date alignment",
        badge: "Clean",
    },
    TemplateStyleOption {
        name: "Engineering Architect",
        desc: "Emphasis on system architecture, fault tolerance, and backend scale",
        badge: "Engineering",
    },
    TemplateStyleOption { name: "Data Scientist & AI", desc: "Model deployment highlights, tech stack pills, and research papers", badge: "Data/AI" },
    TemplateStyleOption { name: "Product Leadership", desc: "Focus on cross-functional roadmap execution and product KPIs", badge: "Leadership" },
];

pub const PROPOSAL_STYLES: &[TemplateStyleOption] = &[
    TemplateStyleOption {
        name: "Executive Presentation",
        desc: "Formal title cover page, scope of work, and executive sign-off blocks",
        badge: "Executive",
    },
    TemplateStyleOption { name: "Enterprise RFP Bid", desc: "Structured technical compliance matrix, SLAs, and architecture", badge: "Enterprise" },
    TemplateStyleOption {
        name: "Creative Agency Pitch",
        desc: "Vibrant cover card, visual milestone schedule, and asset deliverables",
        badge: "Creative",
    },
    TemplateStyleOption {
        name: "Software Architecture Scope",
        desc: "System design specifications, sprint breakdown, and cloud infrastructure",
        badge: "Software",
    },
    TemplateStyleOption {
        name: "Consulting Retainer",
        desc: "Strategic advisory roadmaps, monthly commitments, and rate schedule",
        badge: "Consulting",
    },
    TemplateStyleOption {
        name: "Government Grant Bid",
        desc: "Formal structured layout with compliance checkpoints and budget audit",
        badge: "Grant",
    },
    TemplateStyleOption { name: "Venture Memo", desc: "Problem statement, traction metrics, unit economics, and roadmap", badge: "Venture" },
    TemplateStyleOption { name: "SaaS Agreement", desc: "Tiered pricing table, uptime guarantees, and payment schedule", badge: "SaaS" },
    TemplateStyleOption {
        name: "Turnkey Implementation",
        desc: "Sprint breakdowns, resource allocation, and mutual signature block",
        badge: "Turnkey",
    },
    TemplateStyleOption { name: "Marketing Campaign", desc: "Campaign deliverables, target audience matrix, and projected ROI", badge: "Marketing" },
];

pub const NOTEBOOK_STYLES: &[TemplateStyleOption] = &[
    TemplateStyleOption {
        name: "5mm Dot Grid Matrix",
        desc: "Subtle dot matrix ideal for bullet journaling, UI sketches & diagrams",
        badge: "Dot Grid",
    },
    TemplateStyleOption {
        name: "Cornell Research Notes",
        desc: "Wide left cue column, lecture notes area, and bottom summary block",
        badge: "Cornell",
    },
    TemplateStyleOption {
        name: "Engineering Graph Paper",
        desc: "5mm precision isometric grid for calculations and technical drawings",
        badge: "Grid",
    },
    TemplateStyleOption { name: "College Ruled Clean", desc: "7.1mm standard horizontal ruling with red left margin baseline", badge: "Lined" },
    TemplateStyleOption { name: "Daily Time-Blocked Log", desc: "Hourly schedule agenda with prioritized task checkboxes", badge: "Planner" },
    TemplateStyleOption {
        name: "Laboratory Experiment Log",
        desc: "Formal hypothesis, apparatus notes, data table & witness signature",
        badge: "Lab",
    },
    TemplateStyleOption { name: "UI/UX Wireframe Book", desc: "Dual-viewport sketching frames with annotation lines underneath", badge: "Wireframe" },
    TemplateStyleOption { name: "Meeting Action Pad", desc: "Attendee roster, agenda points, decisions, and action owners", badge: "Meeting" },
    TemplateStyleOption { name: "Code Scratchpad", desc: "Dual-column monospace numbered rows for algorithms and pseudo-code", badge: "Code" },
    TemplateStyleOption { name: "Blank Artist Folio", desc: "Subtle corner crop marks, clean framing rule, and date register", badge: "Folio" },
];

pub const CERTIFICATE_STYLES: &[TemplateStyleOption] = &[
    TemplateStyleOption { name: "Ornate Dual-Border", desc: "Classic double guilloche-inspired border with corner brackets", badge: "Ornate" },
    TemplateStyleOption { name: "Header Banner Ribbon", desc: "Bold colored top ribbon with official commendation badge", badge: "Banner" },
    TemplateStyleOption { name: "Modern Minimalist Frame", desc: "Clean geometric perimeter lines with contemporary typography", badge: "Modern" },
    TemplateStyleOption {
        name: "Corporate Excellence Award",
        desc: "Distinguished corporate achievement award with gold seal block",
        badge: "Excellence",
    },
    TemplateStyleOption { name: "Academic Diploma", desc: "Formal Latinate typography with dual authority verification lines", badge: "Diploma" },
    TemplateStyleOption { name: "Technical Certification", desc: "Competency badge with official verification ID placeholder", badge: "Technical" },
    TemplateStyleOption {
        name: "Appreciation & Service",
        desc: "Warm appreciation border for dedicated volunteer or tenure service",
        badge: "Service",
    },
    TemplateStyleOption {
        name: "Leadership Fellowship",
        desc: "Executive fellowship recognition with distinguished crest emblem",
        badge: "Leadership",
    },
    TemplateStyleOption {
        name: "Course Completion",
        desc: "Accredited training completion certificate with hours and instructor signature",
        badge: "Course",
    },
    TemplateStyleOption { name: "Quarterly MVP Award", desc: "Radiant accent corners and star commendation for standout impact", badge: "MVP" },
];

pub const PALETTES: [(&str, [u8; 3], Color32); 6] = [
    ("Sapphire", [37, 99, 235], Color32::from_rgb(37, 99, 235)),
    ("Emerald", [16, 185, 129], Color32::from_rgb(16, 185, 129)),
    ("Violet", [124, 58, 237], Color32::from_rgb(124, 58, 237)),
    ("Amber", [217, 119, 6], Color32::from_rgb(217, 119, 6)),
    ("Crimson", [225, 29, 72], Color32::from_rgb(225, 29, 72)),
    ("Slate", [51, 65, 85], Color32::from_rgb(51, 65, 85)),
];

pub fn get_styles(category_key: &str) -> &'static [TemplateStyleOption] {
    match category_key {
        "invoice" => INVOICE_STYLES,
        "resume" => RESUME_STYLES,
        "proposal" => PROPOSAL_STYLES,
        "notebook" => NOTEBOOK_STYLES,
        "certificate" => CERTIFICATE_STYLES,
        _ => INVOICE_STYLES,
    }
}

pub fn show(app: &mut PdfCraftApp, ctx: &egui::Context) {
    let Some(picker) = &app.template_picker else { return };
    let category_key = picker.category_key;
    let category_title = picker.category_title;
    let default_filename = picker.default_filename;
    let mut selected_style = picker.selected_style;
    let mut selected_color = picker.selected_color;
    let mut custom_title = picker.custom_title.clone();
    let mut custom_desc = picker.custom_desc.clone();
    let mut show_save_drawer = picker.show_save_drawer;

    let t = Tokens::get(ctx);
    let styles = get_styles(category_key);
    if selected_style >= styles.len() {
        selected_style = 0;
    }
    if selected_color >= PALETTES.len() {
        selected_color = 0;
    }

    let mut close = false;
    let mut create = false;
    let mut export_pdf = false;
    let mut save_custom = false;

    let modal = widgets::show_modal(ctx, "template_style_picker", |ui| {
        ui.set_width(640.0);

        // Header Title
        ui.horizontal(|ui| {
            egui::Frame::NONE.fill(t.accent_soft).corner_radius(CornerRadius::same(6)).inner_margin(egui::Margin::symmetric(8, 4)).show(ui, |ui| {
                ui.label(egui::RichText::new("PARAMETRIC STUDIO").font(theme::semibold(10.0)).color(t.accent_text));
            });
            ui.add_space(4.0);
            ui.label(egui::RichText::new(format!("{category_title} Customizer")).font(theme::semibold(17.0)).color(t.text));
        });
        ui.add_space(2.0);
        ui.label(
            egui::RichText::new("Select a layout variation and accent color theme. Rendered directly in-memory using pure vector primitives.")
                .font(theme::regular(12.0))
                .color(t.text_muted),
        );
        ui.add_space(14.0);

        // Section 1: Accent Color Palettes
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("ACCENT PALETTE").font(theme::semibold(10.5)).color(t.text_muted).extra_letter_spacing(0.5));
            ui.add_space(6.0);
            let active_pal = PALETTES[selected_color].0;
            ui.label(egui::RichText::new(format!("({active_pal})")).font(theme::medium(11.0)).color(t.accent_text));
        });
        ui.add_space(6.0);

        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);
            for (idx, (pal_name, _, pal_color)) in PALETTES.iter().enumerate() {
                let is_sel = selected_color == idx;
                let (rect, resp) = ui.allocate_exact_size(vec2(88.0, 32.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, *pal_name));
                let press = theme::Press::track(ui, &resp);
                let fill = if is_sel { t.hover } else { press.fill(t.card, t.hover, t.pressed) };
                let border_stroke = if is_sel {
                    Stroke::new(1.8, *pal_color)
                } else if resp.hovered() {
                    Stroke::new(1.0, pal_color.gamma_multiply(0.7))
                } else {
                    Stroke::new(1.0, t.border)
                };
                ui.painter().rect(rect, CornerRadius::same(8), fill, border_stroke, egui::StrokeKind::Inside);

                let dot_center = rect.left_center() + vec2(14.0, 0.0);
                ui.painter().circle_filled(dot_center, 5.5, *pal_color);
                if is_sel {
                    ui.painter().circle_stroke(dot_center, 7.5, Stroke::new(1.2, *pal_color));
                }

                ui.painter().text(
                    rect.left_center() + vec2(26.0, 0.0),
                    Align2::LEFT_CENTER,
                    *pal_name,
                    if is_sel { theme::semibold(11.5) } else { theme::medium(11.5) },
                    if is_sel { t.text } else { t.text_muted },
                );

                if theme::hand(resp).clicked() {
                    selected_color = idx;
                }
            }
        });

        ui.add_space(14.0);

        // Section 2: Layout Variations List
        let active_color = PALETTES[selected_color].2;
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("LAYOUT VARIATION").font(theme::semibold(10.5)).color(t.text_muted).extra_letter_spacing(0.5));
            ui.add_space(6.0);
            ui.label(egui::RichText::new(format!("({} Available Presets)", styles.len())).font(theme::medium(11.0)).color(t.text_faint));
        });
        ui.add_space(6.0);

        egui::ScrollArea::vertical().max_height(240.0).auto_shrink([false, false]).show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 6.0);
            for (idx, opt) in styles.iter().enumerate() {
                let is_sel = selected_style == idx;
                let (rect, resp) = ui.allocate_exact_size(vec2(ui.available_width() - 8.0, 48.0), Sense::click());
                resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, opt.name));
                let press = theme::Press::track(ui, &resp);
                let fill = if is_sel { t.hover } else { press.fill(t.card, t.hover, t.pressed) };
                let border_stroke = if is_sel {
                    Stroke::new(1.6, active_color)
                } else if resp.hovered() {
                    Stroke::new(1.0, active_color.gamma_multiply(0.5))
                } else {
                    Stroke::new(1.0, t.border)
                };
                ui.painter().rect(rect, CornerRadius::same(8), fill, border_stroke, egui::StrokeKind::Inside);

                // Number badge
                let num_text = format!("{:02}", idx + 1);
                let num_rect = Rect::from_center_size(rect.left_center() + vec2(22.0, 0.0), vec2(24.0, 24.0));
                let num_bg = if is_sel { active_color.gamma_multiply(0.2) } else { t.pasteboard };
                let num_fg = if is_sel { active_color } else { t.text_muted };
                ui.painter().rect(num_rect, CornerRadius::same(6), num_bg, Stroke::NONE, egui::StrokeKind::Inside);
                ui.painter().text(num_rect.center(), Align2::CENTER_CENTER, num_text, theme::semibold(11.0), num_fg);

                // Title & Description
                let title_pos = rect.min + vec2(46.0, 9.0);
                ui.painter().text(title_pos, Align2::LEFT_TOP, opt.name, if is_sel { theme::semibold(12.5) } else { theme::medium(12.5) }, t.text);

                let desc_pos = rect.min + vec2(46.0, 26.0);
                let mut desc_job = egui::text::LayoutJob::simple_singleline(opt.desc.to_owned(), theme::regular(10.5), t.text_muted);
                desc_job.wrap = egui::text::TextWrapping::truncate_at_width(rect.width() - 150.0);
                let desc_galley = ui.fonts_mut(|f| f.layout_job(desc_job));
                ui.painter().galley(desc_pos, desc_galley, t.text_muted);

                // Badge tag on right
                let badge_rect = Rect::from_center_size(rect.right_center() - vec2(48.0, 0.0), vec2(70.0, 20.0));
                let badge_fill = if is_sel { active_color.gamma_multiply(0.18) } else { t.hover };
                let badge_color = if is_sel { active_color } else { t.text_muted };
                ui.painter().rect(badge_rect, CornerRadius::same(5), badge_fill, Stroke::NONE, egui::StrokeKind::Inside);
                ui.painter().text(badge_rect.center(), Align2::CENTER_CENTER, opt.badge, theme::semibold(9.5), badge_color);

                if theme::hand(resp).clicked() {
                    selected_style = idx;
                }
            }
        });

        ui.add_space(12.0);

        // Section 3: Save As Custom Template Drawer
        if show_save_drawer {
            egui::Frame::NONE
                .fill(t.card)
                .stroke(Stroke::new(1.0, t.accent.gamma_multiply(0.4)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(14, 10))
                .show(ui, |ui| {
                    ui.label(egui::RichText::new("Save To Personal Template Library").font(theme::semibold(12.5)).color(t.text));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("Name:").font(theme::medium(11.5)));
                        let default_name = format!("My {} ({})", styles[selected_style].name, PALETTES[selected_color].0);
                        if custom_title.is_empty() {
                            custom_title = default_name;
                        }
                        ui.add(egui::TextEdit::singleline(&mut custom_title).desired_width(220.0));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Description:").font(theme::medium(11.5)));
                        ui.add(egui::TextEdit::singleline(&mut custom_desc).desired_width(200.0));
                    });
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        if widgets::pill_button(ui, "Save Template Now", true).clicked() {
                            save_custom = true;
                        }
                        ui.add_space(6.0);
                        if widgets::pill_button(ui, "Dismiss", false).clicked() {
                            show_save_drawer = false;
                        }
                    });
                });
            ui.add_space(10.0);
        }

        // Section 4: Footer Actions
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Vector PDF generated in memory").font(theme::regular(11.0)).color(t.text_faint));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if widgets::pill_button(ui, "Create & Open Template →", true).clicked() {
                    create = true;
                }
                ui.add_space(6.0);
                if widgets::pill_button(ui, "Export PDF File", false).clicked() {
                    export_pdf = true;
                }
                ui.add_space(6.0);
                if widgets::pill_button(ui, if show_save_drawer { "Hide Save Form" } else { "Save As Custom..." }, false).clicked() {
                    show_save_drawer = !show_save_drawer;
                }
                ui.add_space(6.0);
                if widgets::pill_button(ui, tl!("Cancel"), false).clicked() {
                    close = true;
                }
            });
        });
    });

    if modal.should_close() || close {
        app.template_picker = None;
    } else if create {
        let (_, rgb, _) = PALETTES[selected_color.min(PALETTES.len() - 1)];
        app.template_picker = None;
        app.create_styled_template_document(default_filename, category_key, selected_style, rgb);
    } else if export_pdf {
        let (_, rgb, _) = PALETTES[selected_color.min(PALETTES.len() - 1)];
        let file_path =
            rfd::FileDialog::new().set_title("Export Template PDF").set_file_name(default_filename).add_filter("PDF Document", &["pdf"]).save_file();
        if let Some(path) = file_path {
            match app.session.create_styled_template(category_key, selected_style, rgb) {
                Ok(bytes) => {
                    if std::fs::write(&path, &*bytes).is_ok() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("template.pdf");
                        app.toast = Some((format!("Exported template PDF to {name}"), 0.0));
                    } else {
                        app.toast = Some(("Failed to write exported PDF to disk".to_string(), 0.0));
                    }
                }
                Err(e) => {
                    app.toast = Some((format!("Export failed: {e}"), 0.0));
                }
            }
        }
    } else if save_custom {
        let (_, rgb, _) = PALETTES[selected_color.min(PALETTES.len() - 1)];
        let title = if custom_title.trim().is_empty() { styles[selected_style].name.to_string() } else { custom_title.trim().to_string() };
        let desc = if custom_desc.trim().is_empty() { styles[selected_style].desc.to_string() } else { custom_desc.trim().to_string() };
        app.custom_templates.push(CustomTemplate {
            id: format!(
                "{}_{}",
                category_key,
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0)
            ),
            title: title.clone(),
            description: desc,
            category_key: category_key.to_string(),
            default_filename: default_filename.to_string(),
            selected_style,
            selected_color,
            color_rgb: rgb,
            created_at: "Personal Library".to_string(),
        });
        app.toast = Some((format!("Saved \"{title}\" to your personal template library!"), 0.0));
        if let Some(p) = &mut app.template_picker {
            p.show_save_drawer = false;
        }
    } else if let Some(p) = &mut app.template_picker {
        p.selected_style = selected_style;
        p.selected_color = selected_color;
        p.custom_title = custom_title;
        p.custom_desc = custom_desc;
        p.show_save_drawer = show_save_drawer;
    }
}
