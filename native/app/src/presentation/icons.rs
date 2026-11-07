//! Ikon garis (outline) procedural gaya Doodle (Hand-drawn),
//! diadopsi dari `assets/icons/doodle`.
//!
//! Setiap ikon di-render berbasis kotak 24×24 lalu diskalakan ke `rect` tujuan,
//! dengan dukungan kurva Bezier (Cubic & Quadratic) agar goresan tangan organik
//! tetap tajam, konsisten, dan antialiased pada resolusi layar apa pun.

use eframe::egui::{self, Color32, Painter, Pos2, Rect, Stroke};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Icon {
    Home,
    Music,
    Tasks,
    Timer,
    Activity,
    Sparkles,
    Settings,
    Close,
    Plus,
    Cpu,
    Ram,
    Check,
    Trash,
    Play,
    Pause,
    Prev,
    Next,
    PlayCircle,
    PauseCircle,
    ChevronLeft,
    ChevronRight,
    ChatBubble,
    Lyrics,
    LyricsOn,
    LyricsOff,
    System,
    Speaker,
    VolumeOn,
    VolumeMute,
    Water,
    Bolt,
    Reset,
    Refresh,
    ArrowUpRight,
    Sun,
    Moon,
    Cloud,
    PartlyCloudy,
    Rain,
    Snow,
    Storm,
    Fog,
    // Settings tab icons
    Display,
    Palette,
    MusicNote,
    Todo,
    Focus,
    AI,
    AiQuota,
    About,
    Gear,
}

const BOX: f32 = 24.0;

/// Helper untuk membaca float dari slice byte SVG
fn read_float(bytes: &[u8], i: &mut usize) -> f32 {
    while *i < bytes.len() && (bytes[*i].is_ascii_whitespace() || bytes[*i] == b',') {
        *i += 1;
    }
    let start = *i;
    if *i < bytes.len() && (bytes[*i] == b'+' || bytes[*i] == b'-') {
        *i += 1;
    }
    while *i < bytes.len() && (bytes[*i].is_ascii_digit() || bytes[*i] == b'.') {
        *i += 1;
    }
    if start < *i {
        if let Ok(s) = std::str::from_utf8(&bytes[start..*i]) {
            if let Ok(val) = s.parse::<f32>() {
                return val;
            }
        }
    }
    0.0
}

/// Sampel busur lingkaran, sudut dalam radian (0 = kanan, positif = ke bawah).
fn arc(cx: f32, cy: f32, r: f32, a0: f32, a1: f32, n: usize) -> Vec<(f32, f32)> {
    (0..=n)
        .map(|i| {
            let t = a0 + (a1 - a0) * i as f32 / n as f32;
            (cx + r * t.cos(), cy + r * t.sin())
        })
        .collect()
}

struct Pen<'a> {
    painter: &'a Painter,
    rect: Rect,
    stroke: Stroke,
}

impl<'a> Pen<'a> {
    fn pt(&self, x: f32, y: f32) -> Pos2 {
        Pos2::new(
            self.rect.left() + (x / BOX) * self.rect.width(),
            self.rect.top() + (y / BOX) * self.rect.height(),
        )
    }

    fn line(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        if v.len() >= 2 {
            self.painter.add(egui::Shape::line(v, self.stroke));
        }
    }

    fn closed(&self, pts: &[(f32, f32)]) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        if v.len() >= 3 {
            self.painter.add(egui::Shape::closed_line(v, self.stroke));
        }
    }

    fn dot(&self, cx: f32, cy: f32, r: f32) {
        let c = self.pt(cx, cy);
        let rr = (r / BOX) * self.rect.width();
        self.painter.circle_stroke(c, rr, self.stroke);
    }

    fn fill_circle(&self, cx: f32, cy: f32, r: f32, color: Color32) {
        let c = self.pt(cx, cy);
        let rr = (r / BOX) * self.rect.width();
        self.painter.circle_filled(c, rr, color);
    }

    fn boxed(&self, x0: f32, y0: f32, x1: f32, y1: f32, rounding: f32) {
        let r = Rect::from_min_max(self.pt(x0, y0), self.pt(x1, y1));
        let rad = (rounding / BOX) * self.rect.width();
        self.painter
            .rect_stroke(r, egui::Rounding::same(rad), self.stroke);
    }

    fn fill_poly(&self, pts: &[(f32, f32)], color: Color32) {
        let v: Vec<Pos2> = pts.iter().map(|&(x, y)| self.pt(x, y)).collect();
        if v.len() >= 3 {
            self.painter
                .add(egui::Shape::convex_polygon(v, color, Stroke::NONE));
        }
    }

    /// Menjalankan path SVG mandiri (mendukung M, L, Q, C, Z)
    fn path(&self, d: &str) {
        let bytes = d.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        let mut cur = Pos2::ZERO;
        let mut subpath_start = Pos2::ZERO;
        let mut pts: Vec<Pos2> = Vec::with_capacity(32);
        let mut cmd = b' ';

        while i < len {
            while i < len && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
                i += 1;
            }
            if i >= len {
                break;
            }

            let b = bytes[i];
            if b.is_ascii_alphabetic() {
                cmd = b;
                i += 1;
                while i < len && (bytes[i].is_ascii_whitespace() || bytes[i] == b',') {
                    i += 1;
                }
            }

            match cmd {
                b'M' | b'm' => {
                    if !pts.is_empty() {
                        self.painter.add(egui::Shape::line(std::mem::take(&mut pts), self.stroke));
                    }
                    let x = read_float(bytes, &mut i);
                    let y = read_float(bytes, &mut i);
                    cur = self.pt(x, y);
                    subpath_start = cur;
                    pts.push(cur);
                    if cmd == b'M' { cmd = b'L'; } else { cmd = b'l'; }
                }
                b'L' | b'l' => {
                    let x = read_float(bytes, &mut i);
                    let y = read_float(bytes, &mut i);
                    cur = self.pt(x, y);
                    pts.push(cur);
                }
                b'Q' | b'q' => {
                    let x1 = read_float(bytes, &mut i);
                    let y1 = read_float(bytes, &mut i);
                    let x = read_float(bytes, &mut i);
                    let y = read_float(bytes, &mut i);
                    let p1 = self.pt(x1, y1);
                    let p = self.pt(x, y);
                    for step in 1..=4 {
                        let t = step as f32 / 4.0;
                        let mt = 1.0 - t;
                        let px = mt * mt * cur.x + 2.0 * mt * t * p1.x + t * t * p.x;
                        let py = mt * mt * cur.y + 2.0 * mt * t * p1.y + t * t * p.y;
                        pts.push(Pos2::new(px, py));
                    }
                    cur = p;
                }
                b'C' | b'c' => {
                    let x1 = read_float(bytes, &mut i);
                    let y1 = read_float(bytes, &mut i);
                    let x2 = read_float(bytes, &mut i);
                    let y2 = read_float(bytes, &mut i);
                    let x = read_float(bytes, &mut i);
                    let y = read_float(bytes, &mut i);
                    let p1 = self.pt(x1, y1);
                    let p2 = self.pt(x2, y2);
                    let p = self.pt(x, y);
                    for step in 1..=6 {
                        let t = step as f32 / 6.0;
                        let mt = 1.0 - t;
                        let px = mt * mt * mt * cur.x
                            + 3.0 * mt * mt * t * p1.x
                            + 3.0 * mt * t * t * p2.x
                            + t * t * t * p.x;
                        let py = mt * mt * mt * cur.y
                            + 3.0 * mt * mt * t * p1.y
                            + 3.0 * mt * t * t * p2.y
                            + t * t * t * p.y;
                        pts.push(Pos2::new(px, py));
                    }
                    cur = p;
                }
                b'Z' | b'z' => {
                    pts.push(subpath_start);
                    cur = subpath_start;
                    if pts.len() >= 2 {
                        self.painter.add(egui::Shape::line(std::mem::take(&mut pts), self.stroke));
                    }
                }
                _ => {
                    i += 1;
                }
            }
        }
        if pts.len() >= 2 {
            self.painter.add(egui::Shape::line(pts, self.stroke));
        }
    }
}

/// Gambar satu ikon ke dalam `rect` (persegi).
pub fn draw(painter: &Painter, icon: Icon, rect: Rect, color: Color32, width: f32) {
    let pen = Pen {
        painter,
        rect,
        stroke: Stroke::new(width, color),
    };

    match icon {
        Icon::Home => {
            pen.path("M3.88 10.2Q12.0 3.0 20.17 10.2Q20.35 15.0 20.17 20.5Q14.5 20.3 9.5 20.5Q9.5 14.0 14.5 14.0L14.5 20.5L3.88 20.5Z");
        }

        Icon::Music | Icon::MusicNote => {
            pen.path("M6.37 14.98C6.56 14.99 7.20 14.89 7.54 15.02C7.88 15.16 8.17 15.50 8.42 15.79C8.67 16.08 8.91 16.40 9.04 16.75C9.18 17.10 9.27 17.54 9.22 17.90C9.17 18.27 8.94 18.62 8.74 18.95C8.55 19.29 8.35 19.65 8.05 19.89C7.75 20.13 7.34 20.31 6.96 20.40C6.57 20.48 6.08 20.54 5.72 20.41C5.35 20.29 5.05 19.93 4.76 19.65C4.46 19.38 4.13 19.12 3.96 18.78C3.80 18.43 3.79 18.01 3.75 17.60C3.72 17.20 3.62 16.71 3.75 16.33C3.88 15.95 4.21 15.54 4.54 15.30C4.88 15.07 5.33 15.04 5.74 14.90C6.15 14.76 6.79 14.54 7.00 14.47");
            pen.path("M18.24 13.25C18.39 13.37 18.92 13.68 19.12 13.98C19.33 14.28 19.42 14.69 19.47 15.05C19.52 15.41 19.50 15.77 19.45 16.13C19.40 16.50 19.37 16.95 19.17 17.24C18.96 17.53 18.55 17.70 18.21 17.87C17.88 18.04 17.52 18.20 17.15 18.24C16.79 18.28 16.36 18.24 16.01 18.11C15.67 17.97 15.37 17.69 15.09 17.43C14.81 17.16 14.48 16.87 14.33 16.52C14.17 16.16 14.16 15.72 14.17 15.31C14.18 14.90 14.21 14.41 14.41 14.06C14.60 13.71 14.99 13.42 15.34 13.19C15.69 12.95 16.09 12.72 16.49 12.65C16.90 12.57 17.36 12.63 17.77 12.73C18.18 12.83 18.76 13.17 18.96 13.25");
            pen.path("M9.08 17.10Q9.43 11.47 9.32 5.82Q14.46 4.31 19.75 3.49Q19.29 9.18 19.52 14.88");
        }

        Icon::Tasks | Icon::Todo => {
            pen.path("M2.60 6.59Q3.72 7.27 4.70 8.14Q6.16 6.07 8.00 4.33");
            pen.path("M2.76 13.13Q3.84 13.95 4.76 14.93Q6.21 12.86 7.85 10.95");
            pen.path("M4.59 17.53C4.72 17.49 5.10 17.31 5.36 17.30C5.62 17.30 5.88 17.41 6.15 17.51C6.41 17.60 6.77 17.67 6.97 17.87C7.16 18.06 7.26 18.42 7.32 18.70C7.37 18.99 7.34 19.30 7.28 19.58C7.22 19.86 7.12 20.15 6.96 20.40C6.80 20.64 6.58 20.88 6.33 21.03C6.09 21.18 5.77 21.28 5.47 21.31C5.18 21.34 4.84 21.32 4.56 21.21C4.29 21.11 4.01 20.90 3.82 20.69C3.62 20.47 3.47 20.18 3.38 19.91C3.29 19.63 3.29 19.35 3.26 19.04C3.23 18.72 3.07 18.28 3.20 18.02C3.33 17.76 3.78 17.67 4.05 17.47C4.32 17.27 4.71 16.95 4.84 16.84");
            pen.path("M11.07 5.99Q16.12 5.53 21.17 5.94");
            pen.path("M10.87 12.45Q16.02 12.94 21.18 12.67");
            pen.path("M10.89 19.16Q15.89 18.71 20.89 19.26");
        }

        Icon::Timer | Icon::Focus => {
            pen.path("M10.95 3.07C11.58 3.15 13.57 3.16 14.76 3.56C15.94 3.97 17.13 4.65 18.05 5.49C18.97 6.33 19.76 7.46 20.27 8.61C20.78 9.76 21.14 11.14 21.10 12.40C21.06 13.65 20.61 15.00 20.04 16.15C19.47 17.30 18.67 18.50 17.68 19.30C16.69 20.09 15.33 20.63 14.07 20.91C12.82 21.19 11.41 21.22 10.14 20.99C8.86 20.77 7.46 20.30 6.42 19.55C5.37 18.79 4.49 17.60 3.88 16.45C3.26 15.30 2.84 13.94 2.72 12.63C2.60 11.33 2.71 9.86 3.15 8.62C3.59 7.38 4.39 6.08 5.35 5.19C6.31 4.30 7.66 3.71 8.91 3.29C10.17 2.87 12.21 2.75 12.87 2.65");
            pen.path("M12.77 7.22C13.08 7.37 14.07 7.74 14.61 8.13C15.16 8.51 15.67 8.99 16.06 9.54C16.46 10.09 16.89 10.77 17.00 11.44C17.11 12.11 16.99 12.94 16.74 13.58C16.49 14.22 15.97 14.77 15.48 15.27C15.00 15.77 14.47 16.30 13.85 16.57C13.22 16.84 12.44 16.94 11.76 16.89C11.08 16.85 10.37 16.61 9.75 16.30C9.14 15.99 8.52 15.56 8.07 15.03C7.62 14.50 7.25 13.81 7.05 13.12C6.85 12.44 6.69 11.59 6.85 10.90C7.01 10.22 7.56 9.57 8.02 9.02C8.48 8.47 9.01 7.95 9.61 7.59C10.22 7.23 10.97 6.92 11.68 6.85C12.38 6.79 13.50 7.15 13.86 7.21");
            pen.path("M12.17 11.31C12.24 11.31 12.47 11.25 12.56 11.31C12.64 11.36 12.64 11.54 12.70 11.65C12.77 11.76 12.93 11.86 12.93 11.97C12.94 12.08 12.80 12.21 12.73 12.31C12.67 12.42 12.61 12.51 12.52 12.58C12.44 12.65 12.34 12.73 12.23 12.76C12.13 12.78 12.03 12.71 11.91 12.71C11.78 12.72 11.59 12.85 11.49 12.80C11.40 12.75 11.39 12.54 11.34 12.43C11.29 12.31 11.23 12.23 11.20 12.11C11.16 12.00 11.09 11.85 11.11 11.73C11.14 11.62 11.25 11.50 11.33 11.41C11.42 11.32 11.53 11.26 11.64 11.18C11.76 11.10 11.89 10.97 12.02 10.94C12.16 10.92 12.39 11.04 12.46 11.05");
        }

        Icon::Activity | Icon::Cpu | Icon::Ram | Icon::System => {
            pen.path("M5.95 6.04Q11.94 6.42 17.92 5.84Q18.33 11.99 18.00 18.15Q12.08 17.60 6.15 18.09Q6.40 12.06 5.95 6.04L6.85 6.02");
            pen.path("M9.61 9.68Q12.06 9.50 14.50 9.75Q14.29 12.03 14.45 14.32Q12.10 14.11 9.74 14.30Q9.48 11.99 9.61 9.68L10.51 9.69");
            pen.path("M8.96 2.86Q9.18 4.37 9.12 5.90");
            pen.path("M12.21 2.43Q11.83 4.20 11.83 6.02");
            pen.path("M15.08 2.87Q15.17 4.37 14.99 5.86");
            pen.path("M9.02 17.80Q9.06 19.46 8.88 21.12");
            pen.path("M12.13 18.18Q11.88 19.88 11.82 21.59");
            pen.path("M14.98 18.06Q14.97 19.51 15.18 20.95");
            pen.path("M3.07 8.87Q4.59 8.92 6.10 9.14");
            pen.path("M2.78 12.21Q4.50 12.22 6.21 12.02");
            pen.path("M3.10 14.79Q4.55 14.91 6.00 14.80");
            pen.path("M18.12 8.99Q19.46 8.97 20.79 9.14");
            pen.path("M17.80 12.01Q19.63 12.20 21.48 12.18");
            pen.path("M18.01 15.10Q19.58 15.22 21.15 15.08");
        }

        Icon::Sparkles | Icon::ChatBubble | Icon::AI => {
            pen.path("M3.88 4.61Q12.03 5.06 20.17 4.53Q19.85 10.28 20.35 16.03Q15.35 16.37 10.35 16.01Q8.53 18.30 6.38 20.29Q6.42 18.24 6.28 16.20Q5.11 16.21 3.96 16.05Q4.32 10.33 3.88 4.61L4.78 4.61");
            pen.path("M12.22 6.73Q12.59 7.85 12.88 9.00Q14.22 9.50 15.49 10.18Q14.40 10.85 13.24 11.39Q12.63 12.41 11.96 13.39Q11.41 12.31 10.94 11.19Q9.93 10.74 8.88 10.39Q10.03 9.82 11.13 9.15Q11.71 7.96 12.22 6.73L12.47 7.60");
        }

        Icon::AiQuota => {
            pen.path("M5.80 18.70C5.47 18.17 4.19 16.68 3.83 15.55C3.46 14.42 3.48 13.09 3.60 11.90C3.72 10.71 4.02 9.50 4.54 8.43C5.07 7.36 5.82 6.22 6.76 5.50C7.70 4.78 8.99 4.37 10.17 4.11C11.36 3.85 12.71 3.67 13.86 3.93C15.02 4.18 16.21 4.89 17.13 5.64C18.06 6.40 18.83 7.41 19.41 8.45C20.00 9.49 20.50 10.70 20.62 11.88C20.75 13.06 20.59 14.43 20.15 15.54C19.72 16.65 18.38 18.03 18.02 18.52");
            pen.path("M11.82 8.83Q12.65 10.11 13.36 11.46Q14.41 11.89 15.43 12.39Q14.17 12.84 12.98 13.46Q12.60 14.72 12.13 15.96Q11.59 14.75 10.87 13.63Q9.55 13.16 8.26 12.60Q9.45 11.96 10.69 11.41Q11.29 10.14 11.82 8.83L12.27 9.60");
        }

        Icon::Lyrics => {
            pen.path("M4.05 5.65Q8.49 5.12 12.95 5.31");
            pen.path("M3.89 10.54Q11.92 11.00 19.94 10.48");
            pen.path("M3.84 15.45Q9.95 14.95 16.06 15.42");
            pen.path("M3.89 20.36Q7.26 20.23 10.61 20.69");
        }

        Icon::LyricsOn => {
            pen.path("M2.83 4.09Q11.93 4.51 21.03 4.18Q21.46 12.18 21.19 20.18Q12.08 20.41 2.97 20.13Q3.13 12.11 2.83 4.09L3.73 4.10");
            pen.path("M7.58 9.04Q12.10 9.50 16.63 9.11");
            pen.path("M7.52 14.15Q10.29 14.28 13.03 13.86");
        }

        Icon::LyricsOff => {
            pen.path("M2.90 3.79Q11.96 3.50 21.01 4.02Q20.55 12.11 21.03 20.21Q12.05 20.50 3.07 20.13Q3.44 11.96 2.90 3.79L3.80 3.80");
            pen.path("M7.68 8.82Q12.12 9.10 16.56 8.84");
            pen.path("M7.38 14.12Q10.19 14.31 13.01 14.12");
            pen.path("M2.54 2.53Q12.30 11.64 21.44 21.39");
        }

        Icon::Settings | Icon::Gear => {
            pen.path("M10.96 5.33Q11.04 3.99 11.03 2.64Q11.94 2.66 12.84 2.76Q12.91 4.11 13.12 5.45Q14.40 6.09 15.75 6.61Q16.76 5.65 17.70 4.64Q18.55 5.37 19.29 6.21Q18.40 7.33 17.42 8.37Q17.90 9.72 18.55 11.01Q19.84 11.16 21.13 11.13Q21.29 12.14 21.39 13.15Q19.90 13.17 18.42 13.06Q18.09 14.39 17.59 15.66Q18.46 16.89 19.45 18.02Q18.60 18.61 17.69 19.11Q16.85 18.15 15.89 17.30Q14.40 17.91 12.97 18.65Q13.04 20.01 12.94 21.38Q11.90 21.36 10.86 21.21Q10.95 19.75 10.94 18.29Q9.54 17.79 8.08 17.49Q7.14 18.41 6.33 19.43Q5.43 18.68 4.60 17.86Q5.62 16.79 6.71 15.79Q6.20 14.37 5.58 13.01Q4.07 12.95 2.56 12.80Q2.62 11.92 2.55 11.05Q4.02 10.91 5.51 10.88Q6.03 9.50 6.44 8.08Q5.59 7.14 4.64 6.30Q5.47 5.59 6.36 4.96Q7.16 5.73 8.04 6.40Q9.46 5.78 10.96 5.33L10.98 4.43");
            pen.path("M13.36 9.52C13.55 9.63 14.18 9.87 14.47 10.18C14.76 10.49 15.00 10.95 15.08 11.37C15.15 11.79 15.02 12.26 14.91 12.68C14.80 13.10 14.68 13.56 14.42 13.89C14.16 14.22 13.74 14.45 13.36 14.64C12.98 14.84 12.55 15.00 12.13 15.05C11.70 15.09 11.20 15.07 10.80 14.90C10.41 14.74 10.05 14.41 9.75 14.08C9.45 13.76 9.19 13.39 9.00 12.97C8.81 12.56 8.58 12.05 8.61 11.60C8.64 11.14 8.92 10.66 9.18 10.26C9.44 9.87 9.78 9.52 10.16 9.23C10.54 8.94 11.00 8.61 11.46 8.53C11.93 8.44 12.46 8.61 12.94 8.73C13.42 8.86 14.10 9.20 14.34 9.29");
        }

        Icon::Play => {
            pen.path("M8.18 5.27Q7.81 12.08 7.99 18.91Q13.32 15.11 19.09 12.01Q13.51 8.84 8.18 5.27L8.17 6.17");
        }

        Icon::Pause => {
            pen.path("M6.46 4.83Q8.31 5.04 10.16 4.94Q9.66 11.93 9.85 18.94Q8.10 19.14 6.34 18.96Q6.10 11.90 6.46 4.83L7.36 4.86");
            pen.path("M13.83 5.17Q15.72 4.95 17.63 5.05Q17.82 11.98 17.44 18.90Q15.76 19.09 14.08 19.03Q13.45 12.11 13.83 5.17L14.73 5.14");
        }

        Icon::Prev => {
            pen.path("M18.06 6.64Q18.28 12.10 17.81 17.54Q13.19 15.08 9.11 11.80Q13.41 8.91 18.06 6.64L18.04 7.54");
            pen.path("M6.00 5.71Q5.60 11.91 5.96 18.12");
        }

        Icon::Next => {
            pen.path("M5.90 6.69Q6.41 12.14 6.07 17.61Q10.68 15.26 14.79 12.12Q10.47 9.20 5.90 6.69L5.91 7.59");
            pen.path("M17.92 5.76Q17.73 12.03 18.04 18.30");
        }

        Icon::PlayCircle => {
            pen.path("M7.90 4.17C8.51 3.98 10.31 3.16 11.56 3.07C12.80 2.98 14.20 3.18 15.38 3.63C16.56 4.08 17.79 4.83 18.65 5.75C19.50 6.67 20.10 7.96 20.50 9.17C20.91 10.38 21.22 11.77 21.09 13.02C20.96 14.26 20.38 15.58 19.71 16.66C19.03 17.74 18.09 18.74 17.06 19.49C16.02 20.24 14.75 20.92 13.49 21.16C12.23 21.39 10.77 21.26 9.51 20.92C8.26 20.58 6.95 19.94 5.97 19.10C4.98 18.26 4.15 17.07 3.61 15.88C3.06 14.70 2.69 13.30 2.69 12.00C2.69 10.71 3.06 9.31 3.61 8.13C4.15 6.94 4.99 5.79 5.96 4.91C6.93 4.03 8.86 3.20 9.44 2.86");
            pen.path("M10.15 8.21Q10.27 11.99 9.83 15.74Q12.97 14.14 15.85 12.12Q12.87 10.36 10.15 8.21L10.11 9.11");
        }

        Icon::PauseCircle => {
            pen.path("M13.47 2.94C14.06 3.21 15.97 3.84 17.02 4.57C18.06 5.30 19.08 6.24 19.76 7.30C20.44 8.37 20.94 9.71 21.11 10.98C21.28 12.25 21.20 13.72 20.78 14.93C20.37 16.13 19.53 17.32 18.64 18.24C17.74 19.16 16.59 19.98 15.41 20.45C14.23 20.92 12.85 21.06 11.56 21.03C10.26 21.00 8.79 20.86 7.67 20.28C6.54 19.70 5.56 18.59 4.80 17.54C4.04 16.50 3.43 15.26 3.11 14.02C2.79 12.77 2.65 11.33 2.89 10.06C3.13 8.79 3.77 7.47 4.54 6.42C5.32 5.36 6.38 4.33 7.53 3.72C8.67 3.10 10.10 2.81 11.42 2.72C12.74 2.62 14.77 3.07 15.44 3.14");
            pen.path("M9.81 8.67Q9.70 11.85 10.07 15.02");
            pen.path("M14.12 8.72Q13.93 11.90 14.17 15.08");
        }

        Icon::Speaker | Icon::VolumeOn => {
            pen.path("M3.68 9.46Q5.32 9.40 6.94 9.57Q9.50 7.85 11.66 5.63Q12.11 12.15 11.65 18.67Q9.51 16.41 7.00 14.56Q5.26 14.48 3.51 14.59Q3.80 12.03 3.68 9.46L4.58 9.49");
            pen.path("M14.16 8.60C14.34 8.73 14.98 9.07 15.23 9.42C15.49 9.77 15.57 10.26 15.69 10.69C15.82 11.12 15.99 11.57 15.98 12.00C15.97 12.43 15.79 12.88 15.63 13.29C15.47 13.70 15.24 14.05 15.02 14.43C14.80 14.81 14.42 15.39 14.30 15.59");
            pen.path("M16.27 5.89C16.53 6.18 17.44 6.98 17.83 7.63C18.21 8.27 18.38 9.06 18.59 9.79C18.80 10.52 19.05 11.25 19.09 12.00C19.14 12.75 19.09 13.58 18.86 14.30C18.63 15.01 18.15 15.68 17.70 16.28C17.24 16.89 16.38 17.63 16.11 17.90");
        }

        Icon::VolumeMute => {
            pen.path("M3.44 9.54Q5.14 9.68 6.84 9.49Q9.38 7.72 11.50 5.47Q11.16 11.97 11.67 18.46Q9.19 16.75 6.98 14.69Q5.27 14.81 3.56 14.69Q3.23 12.12 3.44 9.54L4.34 9.53");
            pen.path("M15.73 9.25Q18.11 12.23 21.01 14.71");
            pen.path("M20.81 9.49Q18.09 11.99 15.77 14.86");
        }

        Icon::Close => {
            pen.line(&[(6.0, 6.0), (18.0, 18.0)]);
            pen.line(&[(18.0, 6.0), (6.0, 18.0)]);
        }

        Icon::Plus => {
            pen.line(&[(12.0, 5.0), (12.0, 19.0)]);
            pen.line(&[(5.0, 12.0), (19.0, 12.0)]);
        }

        Icon::Check => pen.line(&[(5.0, 13.0), (9.0, 17.0), (19.0, 7.0)]),

        Icon::Trash => {
            pen.line(&[(3.5, 6.5), (20.5, 6.5)]);
            pen.line(&[(8.5, 6.5), (8.5, 3.5), (15.5, 3.5), (15.5, 6.5)]);
            pen.line(&[(5.5, 6.5), (6.5, 20.5), (17.5, 20.5), (18.5, 6.5)]);
            pen.line(&[(10.0, 10.5), (10.0, 17.0)]);
            pen.line(&[(14.0, 10.5), (14.0, 17.0)]);
        }

        Icon::ChevronLeft => {
            pen.line(&[(14.5, 6.0), (8.5, 12.0), (14.5, 18.0)]);
        }

        Icon::ChevronRight => {
            pen.line(&[(9.5, 6.0), (15.5, 12.0), (9.5, 18.0)]);
        }

        Icon::Water => {
            pen.line(&[
                (12.0, 3.5),
                (17.5, 12.0),
                (17.0, 16.5),
                (14.5, 19.5),
                (9.5, 19.5),
                (7.0, 16.5),
                (6.5, 12.0),
                (12.0, 3.5),
            ]);
        }

        Icon::Bolt => {
            pen.fill_poly(
                &[
                    (13.0, 2.5),
                    (6.5, 13.0),
                    (11.5, 13.0),
                    (10.5, 21.5),
                    (18.0, 10.5),
                    (13.0, 10.5),
                ],
                color,
            );
        }

        Icon::Reset => {
            let mut pts = arc(12.0, 12.0, 7.5, -1.1, 4.4, 22);
            pts.push((12.0, 4.5));
            pen.line(&pts);
            pen.line(&[(12.0, 1.5), (12.0, 6.0)]);
        }

        Icon::Refresh => {
            // Busur hampir penuh (320°) dengan celah 40° di kanan-atas, lalu
            // kepala panah V tepat di ujung busurnya.
            //
            // Angkanya dihitung, bukan ditaksir. Celah dipusatkan di -45°,
            // jadi busur membentang dari 5,8469 rad sampai 11,4319 rad (mod 2π
            // = 5,1487 rad) dan berakhir di titik (14,96; 5,66). Arah lintasan
            // di ujung = garis singgung (−sin θ, cos θ) = (0,9063; 0,4226),
            // yaitu terus searah jarum jam. Dua barb ditarik **mundur** dari
            // ujung itu sejauh 2,8 satuan, melebar ±35°, jadi keduanya
            // menghasilkan (12,20; 6,14) dan (13,56; 3,23).
            //
            // Versi pertama menaruh kepala panah di (18,6; 11,4) — melayang
            // terpisah dari busurnya karena titik ujungnya tidak dihitung.
            pen.line(&arc(12.0, 12.0, 7.0, 5.8469, 11.4319, 24));
            pen.line(&[(12.20, 6.14), (14.96, 5.66), (13.56, 3.23)]);
        }

        Icon::ArrowUpRight => {
            pen.line(&[(7.0, 17.0), (17.0, 7.0)]);
            pen.line(&[(9.0, 7.0), (17.0, 7.0), (17.0, 15.0)]);
        }

        Icon::Sun => {
            pen.dot(12.0, 12.0, 4.2);
            for i in 0..8 {
                let a = std::f32::consts::TAU * i as f32 / 8.0;
                let (s, c) = a.sin_cos();
                pen.line(&[
                    (12.0 + c * 7.0, 12.0 + s * 7.0),
                    (12.0 + c * 9.5, 12.0 + s * 9.5),
                ]);
            }
        }

        Icon::Moon => {
            let mut pts = arc(12.0, 12.0, 8.5, 1.05, 5.24, 20);
            pts.extend(arc(20.0, 12.0, 8.0, 3.40, 5.94, 14));
            pen.line(&pts);
        }

        Icon::Cloud => {
            let mut pts = arc(8.0, 14.0, 3.5, 1.57, 4.71, 10);
            pts.extend(arc(12.5, 10.5, 4.5, 3.14, 6.28, 12));
            pts.extend(arc(17.0, 14.0, 3.5, 4.71, 7.85, 10));
            pts.push((8.0, 17.5));
            pen.line(&pts);
        }

        Icon::PartlyCloudy => {
            pen.dot(9.0, 9.0, 3.2);
            for i in 0..5 {
                let a = std::f32::consts::TAU * i as f32 / 5.0 - 0.6;
                let (s, c) = a.sin_cos();
                pen.line(&[
                    (9.0 + c * 5.2, 9.0 + s * 5.2),
                    (9.0 + c * 6.8, 9.0 + s * 6.8),
                ]);
            }
            let mut pts = arc(9.0, 16.0, 3.0, 1.57, 4.71, 8);
            pts.extend(arc(13.0, 13.5, 3.8, 3.14, 6.28, 10));
            pts.extend(arc(17.0, 16.0, 3.0, 4.71, 7.85, 8));
            pts.push((9.0, 19.0));
            pen.line(&pts);
        }

        Icon::Rain => {
            let mut pts = arc(7.0, 11.0, 3.2, 1.57, 4.71, 8);
            pts.extend(arc(11.5, 8.0, 4.2, 3.14, 6.28, 10));
            pts.extend(arc(16.0, 11.0, 3.2, 4.71, 7.85, 8));
            pts.push((7.0, 14.2));
            pen.line(&pts);
            for x in [7.5, 12.0, 16.5] {
                pen.line(&[(x, 16.5), (x - 1.2, 20.5)]);
            }
        }

        Icon::Snow => {
            let mut pts = arc(7.0, 11.0, 3.2, 1.57, 4.71, 8);
            pts.extend(arc(11.5, 8.0, 4.2, 3.14, 6.28, 10));
            pts.extend(arc(16.0, 11.0, 3.2, 4.71, 7.85, 8));
            pts.push((7.0, 14.2));
            pen.line(&pts);
            for x in [7.5, 12.0, 16.5] {
                pen.line(&[(x - 1.4, 18.4), (x + 1.4, 18.4)]);
                pen.line(&[(x, 17.0), (x, 19.8)]);
                pen.line(&[(x - 1.0, 17.3), (x + 1.0, 19.5)]);
                pen.line(&[(x + 1.0, 17.3), (x - 1.0, 19.5)]);
            }
        }

        Icon::Storm => {
            let mut pts = arc(7.0, 10.0, 3.2, 1.57, 4.71, 8);
            pts.extend(arc(11.5, 7.0, 4.2, 3.14, 6.28, 10));
            pts.extend(arc(16.0, 10.0, 3.2, 4.71, 7.85, 8));
            pts.push((7.0, 13.2));
            pen.line(&pts);
            pen.line(&[(12.5, 14.5), (9.5, 19.0), (13.0, 19.0), (10.5, 22.5)]);
        }

        Icon::Fog => {
            let mut pts = arc(7.0, 10.0, 3.2, 1.57, 4.71, 8);
            pts.extend(arc(11.5, 7.0, 4.2, 3.14, 6.28, 10));
            pts.extend(arc(16.0, 10.0, 3.2, 4.71, 7.85, 8));
            pts.push((7.0, 13.2));
            pen.line(&pts);
            pen.line(&[(5.0, 17.0), (19.0, 17.0)]);
            pen.line(&[(7.5, 20.0), (16.5, 20.0)]);
        }

        Icon::Display => {
            pen.closed(&[(3.0, 3.0), (21.0, 3.0), (21.0, 16.0), (3.0, 16.0)]);
            pen.line(&[(8.0, 16.0), (8.0, 19.0), (16.0, 19.0), (16.0, 16.0)]);
        }

        Icon::Palette => {
            pen.closed(&[
                (7.0, 3.5),
                (17.0, 3.5),
                (21.0, 9.0),
                (18.5, 17.5),
                (9.0, 20.0),
                (3.5, 13.0),
            ]);
            pen.dot(8.5, 8.5, 1.4);
            pen.dot(15.0, 8.5, 1.4);
            pen.dot(11.5, 14.5, 1.4);
        }

        Icon::About => {
            pen.dot(12.0, 12.0, 9.0);
            pen.dot(12.0, 7.5, 1.1);
            pen.line(&[(12.0, 10.5), (12.0, 17.0)]);
        }
    }
}

/// Kotak persegi di tengah sebuah rect, supaya ikon tidak gepeng.
pub fn square_rect(center: Pos2, size: f32) -> Rect {
    Rect::from_center_size(center, egui::vec2(size, size))
}
