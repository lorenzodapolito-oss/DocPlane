//! DocPlane: the launcher of the suite. One window with the three programs, each with a
//! thumbnail, its name and what it is for; a click starts it. Files can be opened from here (the
//! right program is picked by the file type), dropped on the window, or chosen from the recent list.

#![cfg_attr(windows, windows_subsystem = "windows")]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unreachable,
    clippy::todo,
    clippy::unimplemented
)]

mod apps;
mod previews;
mod recent;

use apps::Kind;
use egui::{
    Align2, Color32, CornerRadius, FontId, Margin, Rect, RichText, Sense, Stroke, StrokeKind, pos2,
    vec2,
};
use std::path::{Path, PathBuf};

const OWNER: &str = "Lorenzo D'Apolito";

fn app_title() -> String {
    format!("DocPlane di {OWNER}")
}

// The soft palette: ivory, with powder blue, sage and apricot for the three programs.
const BG: Color32 = Color32::from_rgb(0xF4, 0xF0, 0xE6);
const CARD: Color32 = Color32::from_rgb(0xFB, 0xF9, 0xF4);
const BORDER: Color32 = Color32::from_rgb(0xDD, 0xD7, 0xC8);
const TEXT: Color32 = Color32::from_rgb(0x2C, 0x31, 0x38);
const DIM: Color32 = Color32::from_rgb(0x66, 0x6B, 0x73);

fn accent(kind: Kind) -> Color32 {
    match kind {
        Kind::Word => Color32::from_rgb(0x6F, 0x9B, 0xC4),
        Kind::Sheet => Color32::from_rgb(0x7F, 0xAE, 0x8A),
        Kind::Slides => Color32::from_rgb(0xE0, 0x96, 0x6F),
    }
}

/// A darker shade of the accent, readable as text on the card.
fn accent_text(kind: Kind) -> Color32 {
    match kind {
        Kind::Word => Color32::from_rgb(0x2F, 0x5F, 0x8C),
        Kind::Sheet => Color32::from_rgb(0x2E, 0x6B, 0x40),
        Kind::Slides => Color32::from_rgb(0x9A, 0x4F, 0x22),
    }
}

/// The folder where settings live: `docplane-data` beside the program when it exists (portable),
/// else the user's settings folder.
fn data_dir(exe_dir: &Path) -> Option<PathBuf> {
    let portable = exe_dir.join("docplane-data");
    if portable.is_dir() {
        return Some(portable);
    }
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(PathBuf::from))
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
    Some(base.join("DocPlane"))
}

struct Launcher {
    exe_dir: PathBuf,
    recent: Vec<recent::Recent>,
    message: Option<String>,
}

impl Launcher {
    fn new() -> Self {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(Path::to_path_buf))
            .unwrap_or_else(|| PathBuf::from("."));
        let exe_dir = std::env::var_os("DOCPLANE_APPS")
            .map(PathBuf::from)
            .unwrap_or(exe_dir);
        let recent = data_dir(&exe_dir)
            .and_then(|d| std::fs::read_to_string(d.join("recenti.txt")).ok())
            .map(|t| recent::parse(&t))
            .unwrap_or_default();
        Launcher {
            exe_dir,
            recent,
            message: None,
        }
    }

    fn save_recent(&self) {
        let Some(dir) = data_dir(&self.exe_dir) else {
            return;
        };
        if std::fs::create_dir_all(&dir).is_ok() {
            let _ = std::fs::write(dir.join("recenti.txt"), recent::serialize(&self.recent));
        }
    }

    /// Start a program, with a file or on a blank document, and say so when it can't.
    fn open(&mut self, kind: Kind, file: Option<&Path>) {
        let Some(exe) = apps::find_exe(kind, &self.exe_dir) else {
            self.message = Some(format!(
                "{} non è stato trovato: mettilo nella stessa cartella di DocPlane.",
                kind.name()
            ));
            return;
        };
        match apps::launch(&exe, file) {
            Ok(()) => {
                self.message = None;
                if let Some(f) = file {
                    recent::add(&mut self.recent, kind, f);
                    self.save_recent();
                }
            }
            Err(e) => self.message = Some(e),
        }
    }

    fn open_file(&mut self, path: &Path) {
        match Kind::for_path(path) {
            Some(kind) => self.open(kind, Some(path)),
            None => {
                self.message = Some(format!(
                    "Nessun programma del pacchetto apre «{}».",
                    path.display()
                ))
            }
        }
    }

    fn pick_and_open(&mut self, kind: Option<Kind>) {
        let mut dialog = rfd::FileDialog::new();
        match kind {
            Some(k) => dialog = dialog.add_filter(k.role(), k.extensions()),
            None => {
                let all: Vec<&str> = Kind::ALL
                    .into_iter()
                    .flat_map(|k| k.extensions().iter().copied())
                    .collect();
                dialog = dialog.add_filter("Tutti i documenti", &all);
            }
        }
        if let Some(path) = dialog.pick_file() {
            match kind {
                Some(k) => self.open(k, Some(&path)),
                None => self.open_file(&path),
            }
        }
    }

    fn header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("DocPlane")
                        .font(FontId::proportional(34.0))
                        .strong()
                        .color(TEXT),
                );
                ui.label(
                    RichText::new(format!("di {OWNER}"))
                        .font(FontId::proportional(14.0))
                        .color(DIM),
                );
            });
        });
    }

    fn card(&mut self, ui: &mut egui::Ui, kind: Kind, size: egui::Vec2) {
        let found = apps::find_exe(kind, &self.exe_dir).is_some();
        let button_h = 30.0;
        let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
        let click_zone = Rect::from_min_max(
            rect.min,
            pos2(rect.right(), rect.bottom() - button_h - 16.0),
        );
        let resp = ui.interact(
            click_zone,
            ui.id().with(("card", kind.name())),
            Sense::click(),
        );
        let hovered = resp.hovered();
        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        let p = ui.painter();
        let a = accent(kind);
        p.rect_filled(
            rect.translate(vec2(0.0, 2.0)),
            CornerRadius::same(14),
            Color32::from_black_alpha(if hovered { 30 } else { 16 }),
        );
        p.rect_filled(rect, CornerRadius::same(14), CARD);
        p.rect_stroke(
            rect,
            CornerRadius::same(14),
            Stroke::new(
                if hovered { 2.0 } else { 1.0 },
                if hovered { a } else { BORDER },
            ),
            StrokeKind::Inside,
        );
        let pad = 12.0;
        let preview = Rect::from_min_size(
            rect.min + vec2(pad, pad),
            vec2(size.x - 2.0 * pad, (size.x - 2.0 * pad) * 0.64),
        );
        previews::draw(&p.with_clip_rect(preview), preview, kind, a);
        let text_x = rect.left() + pad + 4.0;
        let mut y = preview.bottom() + 14.0;
        p.text(
            pos2(text_x, y),
            Align2::LEFT_TOP,
            kind.name(),
            FontId::proportional(21.0),
            TEXT,
        );
        y += 29.0;
        p.text(
            pos2(text_x, y),
            Align2::LEFT_TOP,
            kind.role(),
            FontId::proportional(15.0),
            accent_text(kind),
        );
        if !found {
            p.text(
                pos2(rect.right() - pad - 4.0, y),
                Align2::RIGHT_TOP,
                "non trovato",
                FontId::proportional(12.0),
                DIM,
            );
        }
        let button = Rect::from_min_size(
            pos2(rect.left() + pad, rect.bottom() - button_h - 12.0),
            vec2(size.x - 2.0 * pad, button_h),
        );
        if ui
            .put(
                button,
                egui::Button::new(RichText::new("Apri file…").color(TEXT))
                    .fill(a.gamma_multiply(0.28)),
            )
            .clicked()
        {
            self.pick_and_open(Some(kind));
        }
        if resp.clicked() {
            self.open(kind, None);
        }
    }

    fn recent_list(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("File recenti")
                    .font(FontId::proportional(18.0))
                    .strong()
                    .color(TEXT),
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Apri un altro file…").clicked() {
                    self.pick_and_open(None);
                }
            });
        });
        ui.add_space(6.0);
        if self.recent.is_empty() {
            ui.label(RichText::new("I file aperti da qui compaiono in questo elenco. Puoi anche trascinarne uno sulla finestra.").color(DIM));
            return;
        }
        let mut chosen: Option<recent::Recent> = None;
        egui::ScrollArea::vertical()
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for item in &self.recent {
                    let name = item
                        .path
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| item.path.to_string_lossy().to_string());
                    let folder = item
                        .path
                        .parent()
                        .map(|d| d.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let (rect, resp) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 38.0), Sense::click());
                    let p = ui.painter();
                    if resp.hovered() {
                        p.rect_filled(rect, CornerRadius::same(8), CARD);
                        p.rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            Stroke::new(1.0, BORDER),
                            StrokeKind::Inside,
                        );
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    p.circle_filled(
                        pos2(rect.left() + 16.0, rect.center().y),
                        6.0,
                        accent(item.kind),
                    );
                    p.text(
                        pos2(rect.left() + 34.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        &name,
                        FontId::proportional(15.0),
                        TEXT,
                    );
                    let name_w = p
                        .layout_no_wrap(name.clone(), FontId::proportional(15.0), TEXT)
                        .size()
                        .x;
                    let room = (rect.width() - 34.0 - name_w - 24.0 - 130.0).max(0.0);
                    let shown: String = if p
                        .layout_no_wrap(folder.clone(), FontId::proportional(12.5), DIM)
                        .size()
                        .x
                        <= room
                    {
                        folder
                    } else {
                        String::new()
                    };
                    p.text(
                        pos2(rect.left() + 34.0 + name_w + 14.0, rect.center().y),
                        Align2::LEFT_CENTER,
                        shown,
                        FontId::proportional(12.5),
                        DIM,
                    );
                    p.text(
                        pos2(rect.right() - 12.0, rect.center().y),
                        Align2::RIGHT_CENTER,
                        item.kind.name(),
                        FontId::proportional(12.5),
                        accent_text(item.kind),
                    );
                    if resp.clicked() {
                        chosen = Some(item.clone());
                    }
                }
            });
        if let Some(r) = chosen {
            if r.path.is_file() {
                self.open(r.kind, Some(&r.path));
            } else {
                self.message = Some(format!("Il file «{}» non esiste più.", r.path.display()));
                self.recent.retain(|x| x.path != r.path);
                self.save_recent();
            }
        }
    }
}

impl eframe::App for Launcher {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let dropped: Vec<PathBuf> = ui.ctx().input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_path_buf())
                .collect()
        });
        for f in dropped {
            self.open_file(&f);
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(BG).inner_margin(Margin::same(26)))
            .show(ui, |ui| {
                self.header(ui);
                ui.add_space(18.0);
                let gap = 20.0;
                let w = ((ui.available_width() - 2.0 * gap) / 3.0).clamp(200.0, 380.0);
                let h = w * 0.64 + 24.0 + 14.0 + 29.0 + 22.0 + 12.0 + 30.0 + 12.0;
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = gap;
                    for kind in Kind::ALL {
                        self.card(ui, kind, vec2(w, h));
                    }
                });
                ui.add_space(20.0);
                if let Some(m) = &self.message {
                    ui.label(RichText::new(m).color(Color32::from_rgb(0x9A, 0x3B, 0x2E)));
                    ui.add_space(8.0);
                }
                self.recent_list(ui);
            });
    }
}

fn icon() -> Option<egui::IconData> {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/docplane-256.png")).ok()
}

fn main() -> eframe::Result {
    let mut viewport = egui::ViewportBuilder::default()
        .with_title(app_title())
        .with_inner_size([1040.0, 680.0])
        .with_min_inner_size([760.0, 560.0])
        .with_drag_and_drop(true)
        .with_app_id("it.docplane.app");
    if let Some(icon) = icon() {
        viewport = viewport.with_icon(icon);
    }
    let options = eframe::NativeOptions {
        viewport,
        ..Default::default()
    };
    eframe::run_native(
        "DocPlane",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::ThemePreference::Light);
            cc.egui_ctx.global_style_mut(|s| {
                s.visuals.override_text_color = Some(TEXT);
                s.visuals.widgets.inactive.corner_radius = CornerRadius::same(8);
                s.visuals.widgets.hovered.corner_radius = CornerRadius::same(8);
                s.visuals.widgets.active.corner_radius = CornerRadius::same(8);
            });
            Ok(Box::new(Launcher::new()))
        }),
    )
}
