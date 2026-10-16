//! Parametric vector template generator: creates structured PDF documents with vector styling,
//! custom color palettes, and professional typographic hierarchy without external files.

use crate::{CreateError, add_page, set_title};
use pdfcraft_cos::{Dict, Document, Object};

/// Standard paper sizes in points.
const A4: (f64, f64) = (595.28, 841.89);
const LETTER: (f64, f64) = (612.0, 792.0);
const LETTER_LANDSCAPE: (f64, f64) = (792.0, 612.0);

fn build_fonts(doc: &mut Document) -> Dict {
    let mut f_regular = Dict::new();
    f_regular.set(b"Type".to_vec(), Object::name("Font"));
    f_regular.set(b"Subtype".to_vec(), Object::name("Type1"));
    f_regular.set(b"BaseFont".to_vec(), Object::name("Helvetica"));
    f_regular.set(b"Encoding".to_vec(), Object::name("WinAnsiEncoding"));
    let r1 = doc.add(f_regular);

    let mut f_bold = Dict::new();
    f_bold.set(b"Type".to_vec(), Object::name("Font"));
    f_bold.set(b"Subtype".to_vec(), Object::name("Type1"));
    f_bold.set(b"BaseFont".to_vec(), Object::name("Helvetica-Bold"));
    f_bold.set(b"Encoding".to_vec(), Object::name("WinAnsiEncoding"));
    let r2 = doc.add(f_bold);

    let mut fonts = Dict::new();
    fonts.set(b"F1".to_vec(), Object::Ref(r1));
    fonts.set(b"F2".to_vec(), Object::Ref(r2));

    let mut res = Dict::new();
    res.set(b"Font".to_vec(), Object::Dict(fonts));
    res
}

fn rgb_f(rgb: [u8; 3]) -> (f64, f64, f64) {
    (rgb[0] as f64 / 255.0, rgb[1] as f64 / 255.0, rgb[2] as f64 / 255.0)
}

fn fill_rect(c: &mut String, x: f64, y: f64, w: f64, h: f64, rgb: (f64, f64, f64)) {
    c.push_str(&format!("{:.3} {:.3} {:.3} rg {:.2} {:.2} {:.2} {:.2} re f\n", rgb.0, rgb.1, rgb.2, x, y, w, h));
}

fn stroke_rect(c: &mut String, x: f64, y: f64, w: f64, h: f64, line_w: f64, rgb: (f64, f64, f64)) {
    c.push_str(&format!("{:.3} {:.3} {:.3} RG {:.2} w {:.2} {:.2} {:.2} {:.2} re s\n", rgb.0, rgb.1, rgb.2, line_w, x, y, w, h));
}

fn draw_line(c: &mut String, x1: f64, y1: f64, x2: f64, y2: f64, line_w: f64, rgb: (f64, f64, f64)) {
    c.push_str(&format!("{:.3} {:.3} {:.3} RG {:.2} w {:.2} {:.2} m {:.2} {:.2} l s\n", rgb.0, rgb.1, rgb.2, line_w, x1, y1, x2, y2));
}

fn draw_text(c: &mut String, font: &str, size: f64, x: f64, y: f64, rgb: (f64, f64, f64), text: &str) {
    let escaped = text.replace('\\', "\\\\").replace('(', "\\(").replace(')', "\\)");
    c.push_str(&format!("BT /{font} {size:.1} Tf {:.3} {:.3} {:.3} rg 1 0 0 1 {x:.2} {y:.2} Tm ({escaped}) Tj ET\n", rgb.0, rgb.1, rgb.2));
}

/// Generates a styled, parametric vector PDF template.
pub fn generate_template(category: &str, style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    match category {
        "invoice" => generate_invoice(style, color),
        "resume" => generate_resume(style, color),
        "proposal" => generate_proposal(style, color),
        "notebook" => generate_notebook(style, color),
        "certificate" => generate_certificate(style, color),
        _ => generate_invoice(style, color),
    }
}

// ── Invoices ─────────────────────────────────────────────────────────────────

fn generate_invoice(style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    let mut doc = Document::new_empty();
    let res = build_fonts(&mut doc);
    let (w, h) = A4;
    let accent = rgb_f(color);
    let text_dark = (0.12, 0.12, 0.15);
    let text_muted = (0.42, 0.44, 0.50);
    let border_color = (0.85, 0.86, 0.90);

    let mut c = String::new();

    // Style-specific header treatments
    match style % 4 {
        0 => {
            // Minimalist Top Rule
            draw_line(&mut c, 44.0, h - 44.0, w - 44.0, h - 44.0, 4.0, accent);
        }
        1 => {
            // Full Header Accent Banner
            fill_rect(&mut c, 0.0, h - 100.0, w, 100.0, accent);
            draw_text(&mut c, "F2", 24.0, 44.0, h - 55.0, (1.0, 1.0, 1.0), "INVOICE");
            draw_text(&mut c, "F1", 11.0, 44.0, h - 75.0, (0.9, 0.9, 0.9), "INV-2026-0042");
            draw_text(&mut c, "F2", 15.0, w - 180.0, h - 55.0, (1.0, 1.0, 1.0), "ACME STUDIO LLC");
            draw_text(&mut c, "F1", 10.0, w - 180.0, h - 72.0, (0.9, 0.9, 0.9), "billing@acmestudio.com");
        }
        2 => {
            // Left Accent Strip
            fill_rect(&mut c, 0.0, 0.0, 12.0, h, accent);
        }
        _ => {
            // Clean Box Framing
            stroke_rect(&mut c, 34.0, 34.0, w - 68.0, h - 68.0, 1.5, accent);
        }
    }

    if style % 4 != 1 {
        // Standard header title
        draw_text(&mut c, "F2", 26.0, 44.0, h - 80.0, text_dark, "INVOICE");
        draw_text(&mut c, "F1", 11.0, 44.0, h - 98.0, text_muted, "Invoice Number: INV-2026-0042");
        draw_text(&mut c, "F1", 11.0, 44.0, h - 114.0, text_muted, "Issue Date: October 14, 2026");
        draw_text(&mut c, "F1", 11.0, 44.0, h - 130.0, text_muted, "Payment Due: Net 30 Days");

        draw_text(&mut c, "F2", 16.0, w - 210.0, h - 80.0, accent, "ACME STUDIO LLC");
        draw_text(&mut c, "F1", 10.5, w - 210.0, h - 96.0, text_dark, "100 Innovation Blvd, Suite 400");
        draw_text(&mut c, "F1", 10.5, w - 210.0, h - 110.0, text_dark, "San Francisco, CA 94107");
        draw_text(&mut c, "F1", 10.5, w - 210.0, h - 124.0, text_muted, "contact@acmestudio.com");
    }

    // Bill To & Ship To Cards
    let bill_y = h - 210.0;
    fill_rect(&mut c, 44.0, bill_y - 50.0, 230.0, 68.0, (0.97, 0.97, 0.98));
    stroke_rect(&mut c, 44.0, bill_y - 50.0, 230.0, 68.0, 1.0, border_color);
    draw_text(&mut c, "F2", 10.5, 54.0, bill_y + 4.0, accent, "BILLED TO:");
    draw_text(&mut c, "F2", 12.0, 54.0, bill_y - 12.0, text_dark, "Global Enterprise Systems Inc.");
    draw_text(&mut c, "F1", 10.0, 54.0, bill_y - 27.0, text_muted, "Attn: Financial Accounting");
    draw_text(&mut c, "F1", 10.0, 54.0, bill_y - 41.0, text_muted, "accounting@globalenterprise.com");

    fill_rect(&mut c, w - 254.0, bill_y - 50.0, 210.0, 68.0, (0.97, 0.97, 0.98));
    stroke_rect(&mut c, w - 254.0, bill_y - 50.0, 210.0, 68.0, 1.0, border_color);
    draw_text(&mut c, "F2", 10.5, w - 244.0, bill_y + 4.0, accent, "PROJECT DETAILS:");
    draw_text(&mut c, "F2", 11.5, w - 244.0, bill_y - 12.0, text_dark, "Platform Migration Phase 2");
    draw_text(&mut c, "F1", 10.0, w - 244.0, bill_y - 27.0, text_muted, "PO Reference: PO-88392");
    draw_text(&mut c, "F1", 10.0, w - 244.0, bill_y - 41.0, text_muted, "Status: Completed Deliverable");

    // Line Items Table Header
    let table_top = h - 310.0;
    fill_rect(&mut c, 44.0, table_top, w - 88.0, 26.0, accent);
    draw_text(&mut c, "F2", 11.0, 54.0, table_top + 8.0, (1.0, 1.0, 1.0), "DESCRIPTION");
    draw_text(&mut c, "F2", 11.0, w - 250.0, table_top + 8.0, (1.0, 1.0, 1.0), "QTY");
    draw_text(&mut c, "F2", 11.0, w - 180.0, table_top + 8.0, (1.0, 1.0, 1.0), "UNIT PRICE");
    draw_text(&mut c, "F2", 11.0, w - 100.0, table_top + 8.0, (1.0, 1.0, 1.0), "TOTAL");

    // Table Rows
    let items = [
        ("Cloud Infrastructure Architecture & Provisioning", "1", "$4,200.00", "$4,200.00"),
        ("Database Sharding & Data Migration Protocol", "1", "$3,500.00", "$3,500.00"),
        ("Security Audit, Penetration Testing & Fixes", "1", "$2,800.00", "$2,800.00"),
        ("Automated CI/CD Pipeline & GitHub Integration", "1", "$1,950.00", "$1,950.00"),
        ("Performance Tuning, Benchmark & Load Tests", "1", "$1,400.00", "$1,400.00"),
    ];

    let mut row_y = table_top - 26.0;
    for (i, (desc, qty, unit, total)) in items.iter().enumerate() {
        if i % 2 == 1 {
            fill_rect(&mut c, 44.0, row_y, w - 88.0, 26.0, (0.97, 0.97, 0.98));
        }
        draw_line(&mut c, 44.0, row_y, w - 44.0, row_y, 0.75, border_color);
        draw_text(&mut c, "F1", 10.5, 54.0, row_y + 8.0, text_dark, desc);
        draw_text(&mut c, "F1", 10.5, w - 245.0, row_y + 8.0, text_dark, qty);
        draw_text(&mut c, "F1", 10.5, w - 180.0, row_y + 8.0, text_dark, unit);
        draw_text(&mut c, "F2", 10.5, w - 100.0, row_y + 8.0, text_dark, total);
        row_y -= 26.0;
    }
    draw_line(&mut c, 44.0, row_y, w - 44.0, row_y, 1.5, border_color);

    // Totals Section
    let total_box_top = row_y - 20.0;
    draw_text(&mut c, "F1", 11.0, w - 220.0, total_box_top, text_muted, "Subtotal:");
    draw_text(&mut c, "F1", 11.0, w - 100.0, total_box_top, text_dark, "$13,850.00");

    draw_text(&mut c, "F1", 11.0, w - 220.0, total_box_top - 18.0, text_muted, "Tax (8.25%):");
    draw_text(&mut c, "F1", 11.0, w - 100.0, total_box_top - 18.0, text_dark, "$1,142.63");

    fill_rect(&mut c, w - 230.0, total_box_top - 54.0, 186.0, 28.0, accent);
    draw_text(&mut c, "F2", 12.0, w - 220.0, total_box_top - 44.0, (1.0, 1.0, 1.0), "TOTAL DUE:");
    draw_text(&mut c, "F2", 13.0, w - 105.0, total_box_top - 44.0, (1.0, 1.0, 1.0), "$14,992.63");

    // Payment & Notes
    let note_y = total_box_top - 30.0;
    draw_text(&mut c, "F2", 11.0, 44.0, note_y, accent, "PAYMENT INSTRUCTIONS:");
    draw_text(&mut c, "F1", 9.5, 44.0, note_y - 15.0, text_muted, "Bank: First National Tech Bank · Routing: 121000358 · Account: 9948201948");
    draw_text(&mut c, "F1", 9.5, 44.0, note_y - 28.0, text_muted, "Please include invoice number INV-2026-0042 on your remittance reference.");

    // Footer
    draw_line(&mut c, 44.0, 45.0, w - 44.0, 45.0, 1.0, border_color);
    draw_text(&mut c, "F1", 9.0, 44.0, 32.0, text_muted, "Thank you for your business! Crafted with CraftPDF Personal Studio.");

    add_page(&mut doc, w, h, res, Some(c.into_bytes()))?;
    set_title(&mut doc, "Modern Invoice Template");
    Ok(doc)
}

// ── Resumes / CVs ────────────────────────────────────────────────────────────

fn generate_resume(style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    let mut doc = Document::new_empty();
    let res = build_fonts(&mut doc);
    let (w, h) = LETTER;
    let accent = rgb_f(color);
    let text_dark = (0.12, 0.12, 0.15);
    let text_muted = (0.38, 0.40, 0.46);
    let border_color = (0.86, 0.88, 0.91);

    let mut c = String::new();

    // Top Header
    fill_rect(&mut c, 40.0, h - 42.0, 6.0, 32.0, accent);
    draw_text(&mut c, "F2", 24.0, 54.0, h - 30.0, text_dark, "ALEX MORGAN");
    draw_text(&mut c, "F2", 12.0, 54.0, h - 44.0, accent, "LEAD SOFTWARE ARCHITECT & RUST SPECIALIST");

    // Contact line
    let contact_text = "San Francisco, CA  ·  alex.morgan@email.com  ·  +1 (555) 234-5678  ·  github.com/alexmorgan";
    draw_text(&mut c, "F1", 9.5, 54.0, h - 60.0, text_muted, contact_text);
    draw_line(&mut c, 40.0, h - 70.0, w - 40.0, h - 70.0, 1.0, border_color);

    let two_col = style % 2 == 1;
    let sidebar_w = 170.0;
    let main_x = if two_col { 40.0 + sidebar_w + 20.0 } else { 40.0 };
    let main_w = if two_col { w - main_x - 40.0 } else { w - 80.0 };

    if two_col {
        // Sidebar Background
        fill_rect(&mut c, 40.0, 40.0, sidebar_w, h - 120.0, (0.97, 0.97, 0.98));
        stroke_rect(&mut c, 40.0, 40.0, sidebar_w, h - 120.0, 1.0, border_color);

        // Sidebar Content
        let mut sy = h - 100.0;
        draw_text(&mut c, "F2", 11.5, 52.0, sy, accent, "TECHNICAL SKILLS");
        sy -= 16.0;
        for skill in &[
            "Rust / Tokio / async",
            "C / C++ (C++20)",
            "Systems Architecture",
            "Distributed Systems",
            "WebAssembly / egui",
            "Linux Kernel / eBPF",
            "Docker & Kubernetes",
        ] {
            draw_text(&mut c, "F1", 9.5, 52.0, sy, text_dark, &format!("• {skill}"));
            sy -= 14.0;
        }

        sy -= 14.0;
        draw_text(&mut c, "F2", 11.5, 52.0, sy, accent, "EDUCATION");
        sy -= 16.0;
        draw_text(&mut c, "F2", 10.0, 52.0, sy, text_dark, "B.S. Computer Science");
        sy -= 12.0;
        draw_text(&mut c, "F1", 9.0, 52.0, sy, text_muted, "Univ. of California, Berkeley");
        sy -= 12.0;
        draw_text(&mut c, "F1", 8.5, 52.0, sy, text_muted, "Magna Cum Laude (3.92 GPA)");

        sy -= 18.0;
        draw_text(&mut c, "F2", 11.5, 52.0, sy, accent, "CERTIFICATIONS");
        sy -= 16.0;
        draw_text(&mut c, "F1", 9.0, 52.0, sy, text_dark, "AWS Solutions Architect Pro");
        sy -= 13.0;
        draw_text(&mut c, "F1", 9.0, 52.0, sy, text_dark, "Linux Foundation CKA");
    }

    // Main Column
    let mut my = h - 100.0;

    // Summary
    draw_text(&mut c, "F2", 12.5, main_x, my, accent, "PROFESSIONAL SUMMARY");
    draw_line(&mut c, main_x, my - 4.0, main_x + main_w, my - 4.0, 0.75, accent);
    my -= 16.0;
    draw_text(
        &mut c,
        "F1",
        9.5,
        main_x,
        my,
        text_dark,
        "High-velocity systems engineer with 10+ years specializing in low-latency infrastructure, memory-safe",
    );
    my -= 13.0;
    draw_text(
        &mut c,
        "F1",
        9.5,
        main_x,
        my,
        text_dark,
        "Rust programming, and scalable backend platforms. Proven track record scaling workloads to 10M+ req/sec.",
    );

    my -= 24.0;

    // Experience
    draw_text(&mut c, "F2", 12.5, main_x, my, accent, "EXPERIENCE");
    draw_line(&mut c, main_x, my - 4.0, main_x + main_w, my - 4.0, 0.75, accent);
    my -= 18.0;

    let experiences = [
        (
            "Principal Systems Architect",
            "Apex Distributed Systems",
            "2022 - Present",
            &[
                "Architected high-throughput consensus engine in Rust reducing tail latencies by 64%.",
                "Led engineering team of 18 distributed systems and platform reliability engineers.",
                "Designed automated fault injection and fuzz testing suite catching 12 critical concurrency bugs.",
            ],
        ),
        (
            "Senior Software Engineer",
            "HyperScale Cloud Infrastructure",
            "2018 - 2022",
            &[
                "Re-engineered telemetry pipeline processing 4.2TB daily log streams using SIMD-accelerated parsers.",
                "Implemented cross-platform rendering engine deployed across Linux, macOS, and Windows clients.",
                "Mentored junior engineers and spearheaded cross-team memory safety and concurrency workshops.",
            ],
        ),
        (
            "Software Engineer",
            "NextGen Networks Inc.",
            "2015 - 2018",
            &[
                "Developed low-overhead network packet inspection filters utilizing eBPF and XDP kernel hooks.",
                "Decreased CPU memory footprint by 40% through zero-copy buffer pools and custom allocators.",
                "Collaborated with firmware engineers on automated continuous integration testing pipelines.",
            ],
        ),
    ];

    for (role, company, period, bullets) in &experiences {
        draw_text(&mut c, "F2", 11.0, main_x, my, text_dark, role);
        draw_text(&mut c, "F2", 10.0, main_x + main_w - 90.0, my, accent, period);
        my -= 14.0;
        draw_text(&mut c, "F1", 10.0, main_x, my, text_muted, company);
        my -= 14.0;
        for b in *bullets {
            draw_text(&mut c, "F1", 9.2, main_x + 8.0, my, text_dark, &format!("•  {b}"));
            my -= 13.0;
        }
        my -= 8.0;
    }

    add_page(&mut doc, w, h, res, Some(c.into_bytes()))?;
    set_title(&mut doc, "Professional Resume Template");
    Ok(doc)
}

// ── Proposals (3 Pages) ──────────────────────────────────────────────────────

fn generate_proposal(style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    let mut doc = Document::new_empty();
    let res = build_fonts(&mut doc);
    let (w, h) = LETTER;
    let accent = rgb_f(color);
    let text_dark = (0.12, 0.12, 0.15);
    let text_muted = (0.40, 0.42, 0.48);
    let border_color = (0.86, 0.88, 0.91);

    // Page 1: Cover Page
    let mut p1 = String::new();
    fill_rect(&mut p1, 0.0, h - 220.0, w, 220.0, accent);
    draw_text(&mut p1, "F2", 34.0, 50.0, h - 90.0, (1.0, 1.0, 1.0), "PROJECT PROPOSAL");
    draw_text(&mut p1, "F1", 16.0, 50.0, h - 125.0, (0.92, 0.92, 0.95), "Architecture, Deliverables & Implementation Roadmap");

    let status_badge = match style % 3 {
        0 => "CONFIDENTIAL · COMMERCIAL IN CONFIDENCE",
        1 => "TECHNICAL PROPOSAL · VERSION 2.0",
        _ => "ENTERPRISE ARCHITECTURE SCOPE",
    };
    draw_text(&mut p1, "F2", 10.5, 50.0, h - 170.0, (1.0, 1.0, 1.0), status_badge);

    // Cover Meta Box
    fill_rect(&mut p1, 50.0, 160.0, w - 100.0, 140.0, (0.97, 0.97, 0.98));
    stroke_rect(&mut p1, 50.0, 160.0, w - 100.0, 140.0, 1.5, accent);
    draw_text(&mut p1, "F2", 12.0, 70.0, 275.0, accent, "PREPARED FOR:");
    draw_text(&mut p1, "F1", 11.0, 70.0, 258.0, text_dark, "Global Infrastructure Enterprises Corp.");
    draw_text(&mut p1, "F1", 10.0, 70.0, 242.0, text_muted, "Attn: Chief Technology Officer & Architecture Board");

    draw_text(&mut p1, "F2", 12.0, 70.0, 215.0, accent, "PREPARED BY:");
    draw_text(&mut p1, "F1", 11.0, 70.0, 198.0, text_dark, "Apex Solutions & Architecture Group");
    draw_text(&mut p1, "F1", 10.0, 70.0, 182.0, text_muted, "October 2026 · Release 1.0");

    add_page(&mut doc, w, h, res.clone(), Some(p1.into_bytes()))?;

    // Page 2: Scope & Technical Approach
    let mut p2 = String::new();
    draw_line(&mut p2, 50.0, h - 45.0, w - 50.0, h - 45.0, 2.0, accent);
    draw_text(&mut p2, "F2", 18.0, 50.0, h - 70.0, text_dark, "1. Executive Summary & Project Scope");
    draw_text(
        &mut p2,
        "F1",
        10.5,
        50.0,
        h - 92.0,
        text_dark,
        "This document outlines the strategic migration of legacy database and API services to a zero-copy,",
    );
    draw_text(&mut p2, "F1", 10.5, 50.0, h - 106.0, text_dark, "high-throughput distributed microservice architecture written in Rust.");

    draw_text(&mut p2, "F2", 14.0, 50.0, h - 140.0, accent, "Core Technical Objectives");
    let objectives = [
        "Eliminate 99.9% of memory vulnerabilities via memory-safe native Rust implementations.",
        "Attain sustained throughput exceeding 100,000 queries per second per cluster node.",
        "Implement end-to-end audit logging with cryptographic verification and zero telemetry.",
        "Maintain backwards compatibility with existing REST and gRPC client interfaces.",
    ];
    let mut oy = h - 162.0;
    for obj in &objectives {
        draw_text(&mut p2, "F1", 10.0, 60.0, oy, text_dark, &format!("✓  {obj}"));
        oy -= 16.0;
    }

    draw_text(&mut p2, "F2", 14.0, 50.0, oy - 20.0, accent, "Architecture Diagram & Flow");
    fill_rect(&mut p2, 50.0, oy - 150.0, w - 100.0, 120.0, (0.96, 0.97, 0.98));
    stroke_rect(&mut p2, 50.0, oy - 150.0, w - 100.0, 120.0, 1.0, border_color);
    draw_text(&mut p2, "F2", 12.0, 70.0, oy - 70.0, text_dark, "[Edge Gateway]  →  [Rust Tokio Cluster]  →  [Distributed Storage]");
    draw_text(&mut p2, "F1", 10.0, 70.0, oy - 95.0, text_muted, "Zero-copy serialization via Cap'n Proto over encrypted TLS 1.3 tunnels.");

    add_page(&mut doc, w, h, res.clone(), Some(p2.into_bytes()))?;

    // Page 3: Milestones & Investment
    let mut p3 = String::new();
    draw_line(&mut p3, 50.0, h - 45.0, w - 50.0, h - 45.0, 2.0, accent);
    draw_text(&mut p3, "F2", 18.0, 50.0, h - 70.0, text_dark, "2. Milestones & Investment Schedule");

    // Table
    let table_y = h - 110.0;
    fill_rect(&mut p3, 50.0, table_y, w - 100.0, 24.0, accent);
    draw_text(&mut p3, "F2", 10.5, 60.0, table_y + 7.0, (1.0, 1.0, 1.0), "PHASE / MILESTONE");
    draw_text(&mut p3, "F2", 10.5, w - 240.0, table_y + 7.0, (1.0, 1.0, 1.0), "DURATION");
    draw_text(&mut p3, "F2", 10.5, w - 120.0, table_y + 7.0, (1.0, 1.0, 1.0), "INVESTMENT");

    let phases = [
        ("Phase 1: Architecture Blueprint & Proof of Concept", "4 Weeks", "$28,000"),
        ("Phase 2: Core Engine Implementation & Storage Driver", "8 Weeks", "$56,000"),
        ("Phase 3: Security Hardening & Concurrency Fuzzing", "3 Weeks", "$21,000"),
        ("Phase 4: Staging Deployment & Multi-Region Soak Tests", "3 Weeks", "$18,500"),
    ];

    let mut py = table_y - 26.0;
    for (name, dur, inv) in &phases {
        draw_line(&mut p3, 50.0, py, w - 50.0, py, 0.75, border_color);
        draw_text(&mut p3, "F1", 10.0, 60.0, py + 8.0, text_dark, name);
        draw_text(&mut p3, "F1", 10.0, w - 240.0, py + 8.0, text_dark, dur);
        draw_text(&mut p3, "F2", 10.0, w - 120.0, py + 8.0, text_dark, inv);
        py -= 26.0;
    }

    // Sign-off
    let sign_y = 100.0;
    draw_line(&mut p3, 60.0, sign_y, 240.0, sign_y, 1.0, text_dark);
    draw_text(&mut p3, "F1", 9.5, 60.0, sign_y - 14.0, text_muted, "Client Authorized Signature & Date");

    draw_line(&mut p3, w - 240.0, sign_y, w - 60.0, sign_y, 1.0, text_dark);
    draw_text(&mut p3, "F1", 9.5, w - 240.0, sign_y - 14.0, text_muted, "Apex Solutions Executive Signature");

    add_page(&mut doc, w, h, res, Some(p3.into_bytes()))?;
    set_title(&mut doc, "Executive Project Proposal");
    Ok(doc)
}

// ── Notebooks / Engineering Grids (2 Pages) ──────────────────────────────────

fn generate_notebook(style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    let mut doc = Document::new_empty();
    let res = build_fonts(&mut doc);
    let (w, h) = A4;
    let accent = rgb_f(color);
    let text_dark = (0.15, 0.15, 0.18);
    let text_muted = (0.50, 0.52, 0.58);
    let grid_line = (0.88, 0.90, 0.94);

    for page_num in 1..=2 {
        let mut c = String::new();

        // Header block
        fill_rect(&mut c, 40.0, h - 65.0, w - 80.0, 35.0, (0.97, 0.97, 0.98));
        stroke_rect(&mut c, 40.0, h - 65.0, w - 80.0, 35.0, 1.2, accent);
        draw_text(&mut c, "F2", 11.0, 50.0, h - 45.0, accent, "PROJECT / TOPIC:");
        draw_text(&mut c, "F1", 10.0, 170.0, h - 45.0, text_dark, "___________________________________");

        draw_text(&mut c, "F2", 11.0, w - 220.0, h - 45.0, accent, "DATE:");
        draw_text(&mut c, "F1", 10.0, w - 180.0, h - 45.0, text_dark, "___ / ___ / 2026");

        draw_text(&mut c, "F2", 11.0, w - 90.0, h - 45.0, accent, &format!("PAGE {page_num}"));

        match style % 4 {
            0 => {
                // 5mm Dot Grid (dots are small rectangles)
                let dot_spacing = 14.17; // 5mm
                let mut x = 44.0;
                while x <= w - 44.0 {
                    let mut y = 50.0;
                    while y <= h - 80.0 {
                        fill_rect(&mut c, x, y, 1.0, 1.0, (0.75, 0.77, 0.82));
                        y += dot_spacing;
                    }
                    x += dot_spacing;
                }
            }
            1 => {
                // 5mm Graph Paper Grid lines
                let grid_spacing = 14.17;
                let mut x = 40.0;
                while x <= w - 40.0 {
                    draw_line(&mut c, x, 50.0, x, h - 75.0, 0.4, grid_line);
                    x += grid_spacing;
                }
                let mut y = 50.0;
                while y <= h - 75.0 {
                    draw_line(&mut c, 40.0, y, w - 40.0, y, 0.4, grid_line);
                    y += grid_spacing;
                }
            }
            2 => {
                // Cornell Notes Layout
                let cue_w = 140.0;
                let summary_h = 100.0;
                draw_line(&mut c, 40.0 + cue_w, 50.0 + summary_h, 40.0 + cue_w, h - 75.0, 1.2, accent);
                draw_line(&mut c, 40.0, 50.0 + summary_h, w - 40.0, 50.0 + summary_h, 1.2, accent);
                draw_text(&mut c, "F2", 11.0, 50.0, h - 90.0, accent, "CUES / QUESTIONS");
                draw_text(&mut c, "F2", 11.0, 40.0 + cue_w + 14.0, h - 90.0, accent, "LECTURE / NOTE TAKING AREA");
                draw_text(&mut c, "F2", 11.0, 50.0, 50.0 + summary_h - 18.0, accent, "SUMMARY BLOCK");

                // Ruled lines in note area
                let mut y = 50.0 + summary_h + 20.0;
                while y < h - 100.0 {
                    draw_line(&mut c, 40.0 + cue_w, y, w - 40.0, y, 0.4, grid_line);
                    y += 18.0;
                }
            }
            _ => {
                // Lined College Ruled with Red Margin Line
                draw_line(&mut c, 100.0, 50.0, 100.0, h - 75.0, 1.0, (0.92, 0.45, 0.45));
                let mut y = 50.0;
                while y <= h - 80.0 {
                    draw_line(&mut c, 40.0, y, w - 40.0, y, 0.5, grid_line);
                    y += 20.0;
                }
            }
        }

        // Footer
        draw_text(&mut c, "F1", 8.5, 40.0, 30.0, text_muted, "CraftPDF Engineering Journal · All intellectual property rights reserved.");
        add_page(&mut doc, w, h, res.clone(), Some(c.into_bytes()))?;
    }

    set_title(&mut doc, "Engineering Notebook Template");
    Ok(doc)
}

// ── Certificates (Landscape) ─────────────────────────────────────────────────

fn generate_certificate(style: usize, color: [u8; 3]) -> Result<Document, CreateError> {
    let mut doc = Document::new_empty();
    let res = build_fonts(&mut doc);
    let (w, h) = LETTER_LANDSCAPE;
    let accent = rgb_f(color);
    let text_dark = (0.12, 0.12, 0.16);
    let text_muted = (0.42, 0.44, 0.50);

    let mut c = String::new();

    // Outer and Inner Ornate Borders
    stroke_rect(&mut c, 26.0, 26.0, w - 52.0, h - 52.0, 3.0, accent);
    stroke_rect(&mut c, 32.0, 32.0, w - 64.0, h - 64.0, 1.0, (0.80, 0.82, 0.86));

    match style % 3 {
        0 => {
            // Corner Bracket Details
            fill_rect(&mut c, 22.0, h - 38.0, 16.0, 16.0, accent);
            fill_rect(&mut c, w - 38.0, h - 38.0, 16.0, 16.0, accent);
            fill_rect(&mut c, 22.0, 22.0, 16.0, 16.0, accent);
            fill_rect(&mut c, w - 38.0, 22.0, 16.0, 16.0, accent);
        }
        1 => {
            // Header Top Band
            fill_rect(&mut c, 32.0, h - 70.0, w - 64.0, 38.0, accent);
            draw_text(&mut c, "F2", 12.0, w / 2.0 - 90.0, h - 50.0, (1.0, 1.0, 1.0), "HONOR & EXCELLENCE AWARD");
        }
        _ => {}
    }

    // Title
    let title_text = "CERTIFICATE OF ACHIEVEMENT";
    draw_text(&mut c, "F2", 28.0, w / 2.0 - 200.0, h - 130.0, accent, title_text);

    draw_text(&mut c, "F1", 12.0, w / 2.0 - 130.0, h - 170.0, text_muted, "THIS CERTIFICATE IS PROUDLY PRESENTED TO");

    // Recipient Name Line
    let name = "ALEXANDER HAMILTON";
    draw_text(&mut c, "F2", 26.0, w / 2.0 - 150.0, h - 230.0, text_dark, name);
    draw_line(&mut c, w / 2.0 - 220.0, h - 240.0, w / 2.0 + 220.0, h - 240.0, 1.5, accent);

    // Citation
    let line1 = "In recognition of outstanding dedication, leadership, and exemplary contributions";
    let line2 = "to the development and deployment of high-impact mission critical software systems.";
    draw_text(&mut c, "F1", 11.5, w / 2.0 - 240.0, h - 280.0, text_dark, line1);
    draw_text(&mut c, "F1", 11.5, w / 2.0 - 240.0, h - 300.0, text_dark, line2);

    // Seal Placeholder
    fill_rect(&mut c, w / 2.0 - 35.0, 80.0, 70.0, 70.0, (0.96, 0.96, 0.98));
    stroke_rect(&mut c, w / 2.0 - 35.0, 80.0, 70.0, 70.0, 2.0, accent);
    draw_text(&mut c, "F2", 11.0, w / 2.0 - 24.0, 118.0, accent, "OFFICIAL");
    draw_text(&mut c, "F2", 11.0, w / 2.0 - 16.0, 102.0, accent, "SEAL");

    // Signature Lines
    draw_line(&mut c, 90.0, 100.0, 260.0, 100.0, 1.2, text_dark);
    draw_text(&mut c, "F2", 10.5, 90.0, 85.0, text_dark, "SARTHAK SAXENA");
    draw_text(&mut c, "F1", 9.5, 90.0, 72.0, text_muted, "Chief Executive & Lead Architect");

    draw_line(&mut c, w - 260.0, 100.0, w - 90.0, 100.0, 1.2, text_dark);
    draw_text(&mut c, "F2", 10.5, w - 260.0, 85.0, text_dark, "OCTOBER 14, 2026");
    draw_text(&mut c, "F1", 9.5, w - 260.0, 72.0, text_muted, "Award Date & Verification Record");

    add_page(&mut doc, w, h, res, Some(c.into_bytes()))?;
    set_title(&mut doc, "Award Certificate of Achievement");
    Ok(doc)
}
