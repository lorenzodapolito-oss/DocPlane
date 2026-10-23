//! The thumbnails on the three cards, drawn in code (no image files): a page of text, a grid
//! with a chart, a slide with its strip of thumbnails. Original drawings.

use crate::apps::Kind;
use egui::{Color32, CornerRadius, Painter, Pos2, Rect, Stroke, StrokeKind, pos2, vec2};

const PAPER: Color32 = Color32::from_rgb(0xFF, 0xFE, 0xFB);
const LINE: Color32 = Color32::from_rgb(0xD3, 0xD0, 0xC6);
const SOFT_LINE: Color32 = Color32::from_rgb(0xE6, 0xE3, 0xDA);

fn radius(r: f32) -> CornerRadius {
    CornerRadius::same(r.clamp(0.0, 40.0) as u8)
}

fn paper(p: &Painter, r: Rect) {
    p.rect_filled(
        r.translate(vec2(0.0, 3.0)).expand(1.0),
        radius(4.0),
        Color32::from_black_alpha(26),
    );
    p.rect_filled(r, radius(3.0), PAPER);
    p.rect_stroke(
        r,
        radius(3.0),
        Stroke::new(1.0, SOFT_LINE),
        StrokeKind::Inside,
    );
}

fn bar(p: &Painter, from: Pos2, w: f32, h: f32, color: Color32) {
    p.rect_filled(
        Rect::from_min_size(from, vec2(w.max(0.0), h)),
        radius(h / 2.0),
        color,
    );
}

/// Draw the thumbnail of `kind` filling `r`, with `accent` as the program's colour.
pub fn draw(p: &Painter, r: Rect, kind: Kind, accent: Color32) {
    p.rect_filled(r, radius(10.0), accent.gamma_multiply(0.22));
    let inner = r.shrink(r.height() * 0.09);
    match kind {
        Kind::Word => word(p, inner, accent),
        Kind::Sheet => sheet(p, inner, accent),
        Kind::Slides => slides(p, inner, accent),
    }
}

fn word(p: &Painter, r: Rect, accent: Color32) {
    let w = (r.height() * 0.74).min(r.width());
    let page = Rect::from_center_size(r.center(), vec2(w, r.height()));
    paper(p, page);
    let m = page.width() * 0.12;
    let x = page.left() + m;
    let full = page.width() - 2.0 * m;
    let h = (page.height() * 0.032).max(2.0);
    let mut y = page.top() + page.height() * 0.1;
    bar(p, pos2(x, y), full * 0.62, h * 2.2, accent);
    y += h * 5.0;
    let widths = [1.0, 0.96, 0.99, 0.88, 1.0, 0.7];
    for w in widths {
        bar(p, pos2(x, y), full * w, h, LINE);
        y += h * 2.1;
    }
    y += h;
    let img = Rect::from_min_size(pos2(x, y), vec2(full * 0.42, page.height() * 0.2));
    p.rect_filled(img, radius(3.0), accent.gamma_multiply(0.45));
    let tx = img.right() + full * 0.07;
    let mut ty = y;
    for w in [0.5, 0.46, 0.5, 0.4] {
        bar(p, pos2(tx, ty), full * w, h, LINE);
        ty += h * 2.1;
    }
    y = img.bottom() + h * 2.0;
    for w in [1.0, 0.92, 0.55] {
        if y + h < page.bottom() - m * 0.6 {
            bar(p, pos2(x, y), full * w, h, LINE);
        }
        y += h * 2.1;
    }
}

fn sheet(p: &Painter, r: Rect, accent: Color32) {
    let sheet = Rect::from_center_size(
        r.center(),
        vec2(r.width().min(r.height() * 1.35), r.height()),
    );
    paper(p, sheet);
    let cols = 5usize;
    let rows = 7usize;
    let head = sheet.height() * 0.11;
    let side = sheet.width() * 0.1;
    let cw = (sheet.width() - side) / cols as f32;
    let rh = (sheet.height() - head) / rows as f32;
    let top = sheet.top() + head;
    let left = sheet.left() + side;
    p.rect_filled(
        Rect::from_min_max(sheet.min, pos2(sheet.right(), top)),
        radius(3.0),
        accent.gamma_multiply(0.55),
    );
    p.rect_filled(
        Rect::from_min_max(pos2(sheet.left(), top), pos2(left, sheet.bottom())),
        radius(2.0),
        accent.gamma_multiply(0.3),
    );
    for c in 0..=cols {
        let x = left + cw * c as f32;
        p.line_segment(
            [pos2(x, sheet.top()), pos2(x, sheet.bottom())],
            Stroke::new(1.0, LINE),
        );
    }
    for rr in 0..=rows {
        let y = top + rh * rr as f32;
        p.line_segment(
            [pos2(sheet.left(), y), pos2(sheet.right(), y)],
            Stroke::new(1.0, LINE),
        );
    }
    // A few filled cells, as if numbers were typed in them.
    let numbers: [(usize, usize, f32); 9] = [
        (0, 0, 0.6),
        (0, 1, 0.5),
        (0, 2, 0.7),
        (1, 0, 0.45),
        (1, 1, 0.65),
        (1, 2, 0.5),
        (2, 0, 0.55),
        (2, 1, 0.4),
        (2, 2, 0.6),
    ];
    for (rr, c, fill) in numbers {
        let cell = Rect::from_min_size(
            pos2(left + cw * c as f32, top + rh * rr as f32),
            vec2(cw, rh),
        );
        bar(
            p,
            pos2(cell.left() + cw * 0.12, cell.center().y - rh * 0.1),
            cw * fill,
            rh * 0.2,
            Color32::from_rgb(0x8A, 0x8F, 0x96),
        );
    }
    // A column chart over the lower right corner.
    let chart = Rect::from_min_max(
        pos2(left + cw * 2.1, top + rh * 3.2),
        pos2(sheet.right() - cw * 0.12, sheet.bottom() - rh * 0.3),
    );
    p.rect_filled(chart, radius(4.0), PAPER);
    p.rect_stroke(
        chart,
        radius(4.0),
        Stroke::new(1.0, accent),
        StrokeKind::Inside,
    );
    let heights = [0.35, 0.6, 0.45, 0.85, 0.7];
    let n = heights.len() as f32;
    let bw = chart.width() / (n * 1.6 + 0.6);
    for (i, hh) in heights.iter().enumerate() {
        let x = chart.left() + bw * 0.6 + i as f32 * bw * 1.6;
        let ch = (chart.height() - bw) * hh;
        p.rect_filled(
            Rect::from_min_max(
                pos2(x, chart.bottom() - bw * 0.5 - ch),
                pos2(x + bw, chart.bottom() - bw * 0.5),
            ),
            radius(2.0),
            accent,
        );
    }
}

fn slides(p: &Painter, r: Rect, accent: Color32) {
    let strip_w = r.width() * 0.17;
    let gap = r.width() * 0.03;
    // The strip of thumbnails on the left.
    let th = (r.height() - 2.0 * gap) / 3.0;
    for i in 0..3 {
        let t = Rect::from_min_size(
            pos2(r.left(), r.top() + i as f32 * (th + gap)),
            vec2(strip_w, th),
        );
        paper(p, t);
        let hh = (t.height() * 0.12).max(2.0);
        bar(
            p,
            pos2(t.left() + t.width() * 0.14, t.top() + t.height() * 0.2),
            t.width() * 0.5,
            hh,
            if i == 0 { accent } else { LINE },
        );
        bar(
            p,
            pos2(t.left() + t.width() * 0.14, t.top() + t.height() * 0.5),
            t.width() * 0.7,
            hh * 0.7,
            LINE,
        );
    }
    p.rect_stroke(
        Rect::from_min_size(pos2(r.left(), r.top()), vec2(strip_w, th)).expand(1.5),
        radius(4.0),
        Stroke::new(1.5, accent),
        StrokeKind::Outside,
    );
    // The slide itself, 16:9.
    let avail = Rect::from_min_max(pos2(r.left() + strip_w + gap * 1.6, r.top()), r.max);
    let w = avail.width().min(avail.height() * 16.0 / 9.0);
    let slide = Rect::from_center_size(avail.center(), vec2(w, w * 9.0 / 16.0));
    paper(p, slide);
    p.rect_filled(
        Rect::from_min_size(slide.min, vec2(slide.width() * 0.035, slide.height())),
        radius(2.0),
        accent,
    );
    let x = slide.left() + slide.width() * 0.1;
    let h = (slide.height() * 0.06).max(2.0);
    bar(
        p,
        pos2(x, slide.top() + slide.height() * 0.16),
        slide.width() * 0.5,
        h * 2.4,
        accent,
    );
    let mut y = slide.top() + slide.height() * 0.42;
    for w2 in [0.4, 0.34, 0.38] {
        p.circle_filled(pos2(x + h * 0.6, y + h / 2.0), h * 0.5, accent);
        bar(p, pos2(x + h * 2.0, y), slide.width() * w2, h, LINE);
        y += h * 2.3;
    }
    let c = pos2(
        slide.right() - slide.width() * 0.2,
        slide.center().y + slide.height() * 0.08,
    );
    p.circle_filled(c, slide.height() * 0.2, accent.gamma_multiply(0.55));
    p.rect_filled(
        Rect::from_center_size(
            c + vec2(slide.width() * 0.04, slide.height() * 0.1),
            vec2(slide.width() * 0.16, slide.height() * 0.22),
        ),
        radius(5.0),
        accent.gamma_multiply(0.85),
    );
}
