//! I want to be able to do some simple performance testing.
//! The initial design for this module is to provide a stringly typed set of timers. The time will
//! be averaged by frame

use egui::{Align2, Color32, FontId, Pos2, Sense, Stroke, Vec2};
use glam::Vec3;
use once_cell::sync::Lazy;
use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::mem;
use std::sync::Mutex;
use std::time::Instant;

#[derive(Clone)]
pub struct Section {
    time: u128,
    name: String,
}

impl Section {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn as_millis(&self) -> f32 {
        self.time as f32 / 1e6
    }
}

pub struct Span {
    start: Instant,
    name: String,
}

impl Drop for Span {
    fn drop(&mut self) {
        let mut timer = TIMER.lock().unwrap();
        let name = timer.entry(self.name.clone()).or_insert_with(|| Section {
            time: 0,
            name: self.name.clone(),
        });
        name.time += self.start.elapsed().as_nanos();
    }
}

static TIMER: Lazy<Mutex<HashMap<String, Section>>> = Lazy::new(Default::default);

pub fn start(id: impl Into<String>) -> Span {
    let id = id.into();
    Span {
        start: Instant::now(),
        name: id,
    }
}

pub(crate) fn raw_section(id: impl Into<String>, time: u128) {
    let id = id.into();
    let mut timer = TIMER.lock().unwrap();
    timer.insert(id.clone(), Section { name: id, time });
}

/// Get all benchmark data since last report() call
pub fn report() -> Vec<Section> {
    let mut sections = HashMap::new();
    mem::swap(&mut sections, &mut TIMER.lock().unwrap());
    sections.into_values().collect()
}

/// Draws the contents of report() into a ui
pub fn egui_report(ui: &mut egui::Ui) {
    const GRAPH_CAP: usize = 256;
    static PREV_LEGEND_LEN: Mutex<usize> = Mutex::new(0);
    static GRAPH: Lazy<Mutex<VecDeque<Vec<Section>>>> =
        Lazy::new(|| Mutex::new(VecDeque::with_capacity(GRAPH_CAP)));

    let mut graph = GRAPH.lock().unwrap();

    if graph.len() == GRAPH_CAP {
        let _ = graph.pop_front();
    }
    graph.push_back(report());

    let mut peak = 6.0;
    for frame in graph.iter() {
        for section in frame {
            if section.as_millis() > peak {
                peak = section.as_millis();
            }
        }
    }

    let (response, painter) = ui.allocate_painter(Vec2::splat(300.0), Sense::hover());
    let rect = response.rect;
    let ll = rect.left_bottom();

    let reserve_legend = (*PREV_LEGEND_LEN.lock().unwrap() as f32 / 3.0 + 1.0) * 14.0;
    let effective_height = rect.height() - reserve_legend;
    let y_scale = effective_height / peak;
    let x_scale = rect.width() / GRAPH_CAP as f32;
    let mut prev = HashMap::new();
    let mut legend = HashMap::new();

    let peak = peak.floor();
    let label_scale = (peak.max(10.0) / 10.0).round() as usize;
    for i in 0..=peak as usize {
        let height = i as f32;
        if i % label_scale == 0 {
            let color = if i < 1_000 / 144 {
                Color32::DARK_GRAY
            } else if i < 1_000 / 60 {
                Color32::GRAY
            } else if i < 1_000 / 30 {
                Color32::YELLOW
            } else {
                Color32::RED
            };
            ui.painter().text(
                Pos2::new(ll.x, ll.y - height * y_scale),
                Align2::LEFT_CENTER,
                format!("{i}"),
                FontId::default(),
                color,
            );
            let grid_stroke = Stroke::new(1.0, color);
            ui.painter().line_segment(
                [
                    Pos2::new(ll.x + 16.0, ll.y - height * y_scale),
                    Pos2::new(ll.x + (256.0 * x_scale), ll.y - height * y_scale),
                ],
                grid_stroke,
            );
        }
    }
    let ll = Pos2::new(ll.x + 20.0, ll.y);

    for (x, frame) in graph.iter().enumerate() {
        // we want to draw a line for each section
        // the x axis is time in frames
        // y is the time of the section
        for section in frame {
            let y = section.as_millis();
            let x = x as f32;
            let (px, py) = *prev.get(section.name()).unwrap_or(&(x, y));
            prev.insert(section.name(), (x, y));

            let color = legend.entry(section.name()).or_insert_with(|| {
                let mut hasher = DefaultHasher::new();
                section.name().hash(&mut hasher);
                let hash = hasher.finish();
                let r = (hash & 0xFF) as f32;
                let g = ((hash & 0xFF00) >> 8) as f32;
                let b = ((hash & 0xFF0000) >> 16) as f32;

                let col = Vec3::new(r, g, b);
                let col = col.normalize() * col.length().max(130.0);

                Color32::from_rgb(col.x as u8, col.y as u8, col.z as u8)
            });

            let stroke = Stroke::new(1.0, *color);

            // scale our points
            let y = y * y_scale;
            let py = py * y_scale;
            let x = x * x_scale;
            let px = px * x_scale;
            painter.line_segment(
                [
                    Pos2::new(ll.x + px, ll.y - py),
                    Pos2::new(ll.x + x, ll.y - y),
                ],
                stroke,
            );
        }
    }
    // draw legend
    let tl = rect.left_top();
    let mut legend: Vec<_> = legend.iter().collect();
    legend.sort_by_key(|e| e.0);
    *PREV_LEGEND_LEN.lock().unwrap() = legend.len();
    for (i, (entry, color)) in legend.into_iter().enumerate() {
        let col = i % 3;
        let base = Pos2::new(
            tl.x + 7.0 + (col as f32 * rect.width() / 3.0),
            tl.y + (i / 3) as f32 * 14.0,
        );

        painter.circle(
            Pos2::new(base.x, base.y + 7.0),
            5.0,
            *color,
            Stroke::new(1.0, Color32::GRAY),
        );
        painter.text(
            Pos2::new(base.x + 10.0, base.y),
            Align2::LEFT_TOP,
            entry,
            FontId::default(),
            *color,
        );
    }
}
