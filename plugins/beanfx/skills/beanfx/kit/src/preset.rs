//! Built-in parametric pixel-art VFX presets for headless CLI and pipeline rendering.
//!
//! Every preset implements `dr::Effect`, scales cleanly to any cell size, respects custom game palettes,
//! uses seeded determinism (`dr::hash01`, `dr::vnoise`), and passes `ex::check`, `ex::check_seeds`, and `ex::check_act`.

use crate::dr::{ease_in, ease_io, ease_out, hash01, hex, span, vnoise, Canvas, Effect, Layer, Rgb, Spec};
use crate::{ex, fc, lt, np};
use serde_json::{json, Value};
use std::f64::consts::TAU;

/// Metadata describing a built-in preset.
#[derive(Clone, Copy, Debug)]
pub struct PresetInfo {
    pub id: &'static str,
    pub description: &'static str,
    pub default_w: usize,
    pub default_h: usize,
    pub default_frames: usize,
    pub default_fps: f64,
    pub looped: bool,
    pub element: &'static str,
    pub glow: &'static str,
    pub palette: &'static [&'static str],
    pub light_ramp: &'static [&'static str],
}

pub const PRESETS: [PresetInfo; 8] = [
    PresetInfo {
        id: "impact",
        description: "Crisp radial impact burst with shock ring, flying shards, and hot core flash",
        default_w: 48,
        default_h: 48,
        default_frames: 12,
        default_fps: 20.0,
        looped: false,
        element: "metal",
        glow: "medium",
        palette: &["#3a1420", "#b83622", "#ff7a28", "#ffd468", "#fffbe0"],
        light_ramp: &["#5a1c08", "#ff8a30", "#ffe09a", "#ffffff"],
    },
    PresetInfo {
        id: "hit-sparks",
        description: "High-velocity directional hit sparks with central cross flash and embers",
        default_w: 48,
        default_h: 48,
        default_frames: 10,
        default_fps: 24.0,
        looped: false,
        element: "metal",
        glow: "medium",
        palette: &["#402010", "#c85a18", "#ffaa2c", "#ffe680", "#ffffff"],
        light_ramp: &["#5a2a08", "#ffa030", "#ffea98", "#ffffff"],
    },
    PresetInfo {
        id: "slash",
        description: "Sweeping crescent blade arc with tapered bright edge and trailing sparks",
        default_w: 64,
        default_h: 48,
        default_frames: 10,
        default_fps: 24.0,
        looped: false,
        element: "metal",
        glow: "medium",
        palette: &["#162a48", "#2d68a8", "#68c0f0", "#c8f4ff", "#ffffff"],
        light_ramp: &["#103058", "#48a8f0", "#b8ecff", "#ffffff"],
    },
    PresetInfo {
        id: "muzzle-flash",
        description: "Directional muzzle blast cone with bright spike core and dissipating smoke",
        default_w: 48,
        default_h: 32,
        default_frames: 8,
        default_fps: 24.0,
        looped: false,
        element: "fire",
        glow: "strong",
        palette: &["#3c241c", "#b84418", "#ff8822", "#ffe066", "#fffce8"],
        light_ramp: &["#602408", "#ff9028", "#ffe490", "#ffffff"],
    },
    PresetInfo {
        id: "explosion",
        description: "Billowing pixel fireball with hot core, flying debris, and smoke dissolve",
        default_w: 64,
        default_h: 64,
        default_frames: 16,
        default_fps: 20.0,
        looped: false,
        element: "fire",
        glow: "strong",
        palette: &["#281820", "#8a221c", "#e85c1c", "#ffb838", "#fff4c0"],
        light_ramp: &["#5a1808", "#ff7820", "#ffda80", "#ffffff"],
    },
    PresetInfo {
        id: "dust",
        description: "Billowing ground landing dust puffs expanding outward with grit specks",
        default_w: 48,
        default_h: 32,
        default_frames: 12,
        default_fps: 16.0,
        looped: false,
        element: "wind",
        glow: "subtle",
        palette: &["#3a3430", "#685d54", "#9a8d80", "#cbbeb0", "#ebe2d8"],
        light_ramp: &["#302a24", "#8a7e70", "#d8cfc4", "#ffffff"],
    },
    PresetInfo {
        id: "magic",
        description: "Arcane sigil ring with pulsing vortex core and rising crystalline motes",
        default_w: 64,
        default_h: 64,
        default_frames: 16,
        default_fps: 18.0,
        looped: false,
        element: "arcane",
        glow: "strong",
        palette: &["#1c1238", "#4e2690", "#9048e0", "#d498ff", "#f8e8ff"],
        light_ramp: &["#2e1458", "#8e48e8", "#dca8ff", "#ffffff"],
    },
    PresetInfo {
        id: "torch",
        description: "Seamless looping pixel-art flame with flickering core and rising embers",
        default_w: 32,
        default_h: 48,
        default_frames: 12,
        default_fps: 14.0,
        looped: true,
        element: "fire",
        glow: "medium",
        palette: &["#401418", "#b02e18", "#f06e1c", "#ffc438", "#fff6c8"],
        light_ramp: &["#5a1c08", "#ff7e24", "#ffdc84", "#ffffff"],
    },
];

pub fn find_preset(id: &str) -> Option<&'static PresetInfo> {
    PRESETS.iter().find(|p| p.id == id)
}

pub fn presets_json() -> Value {
    Value::Array(
        PRESETS
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "description": p.description,
                    "width": p.default_w,
                    "height": p.default_h,
                    "frames": p.default_frames,
                    "fps": p.default_fps,
                    "loop": p.looped,
                    "element": p.element,
                    "glow": p.glow,
                    "palette": p.palette,
                    "light_ramp": p.light_ramp,
                })
            })
            .collect(),
    )
}

/// JSON Schema for `beanfx render --spec <file|->`.
pub fn spec_schema() -> Value {
    let ids: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "beanfx render spec",
        "type": "object",
        "required": ["preset"],
        "properties": {
            "preset": { "type": "string", "enum": ids },
            "name": { "type": "string", "minLength": 1 },
            "width": { "type": "integer", "minimum": 16, "maximum": 512 },
            "height": { "type": "integer", "minimum": 16, "maximum": 512 },
            "frames": { "type": "integer", "minimum": 4, "maximum": 120 },
            "fps": { "type": "number", "minimum": 1.0, "maximum": 60.0 },
            "seed": { "type": "integer", "minimum": 0 },
            "loop": { "type": "boolean" },
            "palette": { "type": "array", "items": { "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" }, "minItems": 2, "maxItems": 16 },
            "light_ramp": { "type": "array", "items": { "type": "string", "pattern": "^#[0-9a-fA-F]{6}$" }, "minItems": 2, "maxItems": 8 },
            "glow": { "type": "string", "enum": ["subtle", "medium", "strong"] },
            "clarity": { "type": "string", "enum": ["clear", "balanced", "hazy"] },
            "steps": { "type": "boolean" },
            "view": { "type": "string", "enum": ["side", "topdown"] },
            "facing": { "type": "string", "enum": ["east", "west"] },
            "columns": { "type": "integer", "minimum": 1 },
            "sound": { "type": "boolean" },
            "voice": { "type": "string", "enum": ["retro", "soft", "real"] },
            "element": { "type": "string", "enum": ["metal", "lightning", "fire", "ice", "wind", "arcane"] }
        }
    })
}

/// Configuration for a headless preset render.
#[derive(Clone, Debug)]
pub struct PresetConfig {
    pub preset: String,
    pub name: Option<String>,
    pub w: Option<usize>,
    pub h: Option<usize>,
    pub frames: Option<usize>,
    pub fps: Option<f64>,
    pub seed: u64,
    pub looped: Option<bool>,
    pub palette: Option<Vec<String>>,
    pub light_ramp: Option<Vec<String>>,
    pub glow: Option<String>,
    pub clarity: Option<String>,
    pub steps: bool,
    pub view: String,
    pub facing: String,
    pub columns: Option<i64>,
    pub sound: bool,
    pub voice: String,
    pub element: Option<String>,
}

impl PresetConfig {
    pub fn new(preset: &str) -> Self {
        PresetConfig {
            preset: preset.into(),
            name: None,
            w: None,
            h: None,
            frames: None,
            fps: None,
            seed: 7,
            looped: None,
            palette: None,
            light_ramp: None,
            glow: None,
            clarity: None,
            steps: false,
            view: "side".into(),
            facing: "east".into(),
            columns: None,
            sound: false,
            voice: "retro".into(),
            element: None,
        }
    }

    pub fn from_json(v: &Value) -> Result<Self, String> {
        let obj = v.as_object().ok_or("spec must be a JSON object")?;
        let preset = obj
            .get("preset")
            .and_then(Value::as_str)
            .ok_or("spec needs a string 'preset'")?;
        let mut c = PresetConfig::new(preset);
        if let Some(s) = obj.get("name").and_then(Value::as_str) {
            c.name = Some(s.into());
        }
        if let Some(n) = obj.get("width").and_then(Value::as_u64) {
            c.w = Some(n as usize);
        }
        if let Some(n) = obj.get("height").and_then(Value::as_u64) {
            c.h = Some(n as usize);
        }
        if let Some(n) = obj.get("frames").and_then(Value::as_u64) {
            c.frames = Some(n as usize);
        }
        if let Some(f) = obj.get("fps").and_then(Value::as_f64) {
            c.fps = Some(f);
        }
        if let Some(s) = obj.get("seed").and_then(Value::as_u64) {
            c.seed = s;
        }
        if let Some(b) = obj.get("loop").and_then(Value::as_bool) {
            c.looped = Some(b);
        }
        if let Some(arr) = obj.get("palette").and_then(Value::as_array) {
            c.palette = Some(parse_hex_array(arr, "palette")?);
        }
        if let Some(arr) = obj.get("light_ramp").and_then(Value::as_array) {
            c.light_ramp = Some(parse_hex_array(arr, "light_ramp")?);
        }
        if let Some(s) = obj.get("glow").and_then(Value::as_str) {
            c.glow = Some(s.into());
        }
        if let Some(s) = obj.get("clarity").and_then(Value::as_str) {
            c.clarity = Some(s.into());
        }
        if let Some(b) = obj.get("steps").and_then(Value::as_bool) {
            c.steps = b;
        }
        if let Some(s) = obj.get("view").and_then(Value::as_str) {
            c.view = s.into();
        }
        if let Some(s) = obj.get("facing").and_then(Value::as_str) {
            c.facing = s.into();
        }
        if let Some(n) = obj.get("columns").and_then(Value::as_i64) {
            c.columns = Some(n);
        }
        if let Some(b) = obj.get("sound").and_then(Value::as_bool) {
            c.sound = b;
        }
        if let Some(s) = obj.get("voice").and_then(Value::as_str) {
            c.voice = s.into();
        }
        if let Some(s) = obj.get("element").and_then(Value::as_str) {
            c.element = Some(s.into());
        }
        Ok(c)
    }
}

fn valid_hex(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].bytes().all(|b| b.is_ascii_hexdigit())
}

fn parse_hex_array(arr: &[Value], field: &str) -> Result<Vec<String>, String> {
    if arr.len() < 2 {
        return Err(format!("{field} needs at least 2 hex colours"));
    }
    let mut out = Vec::with_capacity(arr.len());
    for v in arr {
        let s = v.as_str().ok_or_else(|| format!("{field} entries must be hex strings"))?;
        if !valid_hex(s) {
            return Err(format!("{field}: invalid hex colour '{s}' (expected #rrggbb)"));
        }
        out.push(s.to_string());
    }
    Ok(out)
}

fn lerp_rgb(a: Rgb, b: Rgb, t: f64) -> Rgb {
    let t = np::clip(t, 0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn rgb_to_hex(c: Rgb) -> String {
    let u = |v: f64| np::round(np::clip(v, 0.0, 1.0) * 255.0) as u8;
    format!("#{:02x}{:02x}{:02x}", u(c[0]), u(c[1]), u(c[2]))
}

/// Resample a user palette of 2..N hex strings into a 5-tone dark-to-bright ramp sorted by luminance.
fn build_5_ramp(hexes: &[String]) -> [Rgb; 5] {
    let mut cols: Vec<Rgb> = hexes.iter().map(|s| hex(s)).collect();
    cols.sort_by(|a, b| {
        let la = 0.2126 * a[0] + 0.7152 * a[1] + 0.0722 * a[2];
        let lb = 0.2126 * b[0] + 0.7152 * b[1] + 0.0722 * b[2];
        la.partial_cmp(&lb).unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut out = [[0.0; 3]; 5];
    let last = (cols.len() - 1) as f64;
    for (i, slot) in out.iter_mut().enumerate() {
        let pos = i as f64 / 4.0 * last;
        let lo = (pos.floor() as usize).min(cols.len() - 1);
        let hi = (lo + 1).min(cols.len() - 1);
        *slot = lerp_rgb(cols[lo], cols[hi], pos - lo as f64);
    }
    out
}

fn derive_light_ramp(pal: &[Rgb; 5]) -> Vec<String> {
    vec![
        rgb_to_hex(lerp_rgb(pal[0], pal[1], 0.5)),
        rgb_to_hex(pal[2]),
        rgb_to_hex(lerp_rgb(pal[3], [1.0, 1.0, 1.0], 0.35)),
        "#ffffff".into(),
    ]
}

/// Compiled runnable preset effect.
#[derive(Clone, Debug)]
pub struct PresetEffect {
    pub info: &'static PresetInfo,
    pub cfg: PresetConfig,
    pub pal: [Rgb; 5],
    pub light_ramp: Vec<String>,
    pub w: usize,
    pub h: usize,
    pub n: usize,
    pub fps: f64,
    pub looped: bool,
    pub glow: String,
    pub clarity: String,
    pub name: String,
    pub element: String,
}

impl PresetEffect {
    pub fn build(cfg: PresetConfig) -> Result<Self, String> {
        let info = find_preset(&cfg.preset).ok_or_else(|| {
            let names: Vec<&str> = PRESETS.iter().map(|p| p.id).collect();
            format!("unknown preset '{}'; available: {}", cfg.preset, names.join(", "))
        })?;
        let w = cfg.w.unwrap_or(info.default_w);
        let h = cfg.h.unwrap_or(info.default_h);
        if !(16..=512).contains(&w) || !(16..=512).contains(&h) {
            return Err(format!("size {w}x{h} out of range (16..512)"));
        }
        let n = cfg.frames.unwrap_or(info.default_frames);
        if !(4..=120).contains(&n) {
            return Err(format!("frames {n} out of range (4..120)"));
        }
        let fps = cfg.fps.unwrap_or(info.default_fps);
        if !(1.0..=60.0).contains(&fps) {
            return Err(format!("fps {fps} out of range (1..60)"));
        }
        let glow = cfg.glow.clone().unwrap_or_else(|| info.glow.into());
        if lt::glow_level(&glow).is_none() {
            return Err(format!("glow '{glow}' must be subtle, medium or strong"));
        }
        let clarity = cfg.clarity.clone().unwrap_or_else(|| "clear".into());
        if !["clear", "balanced", "hazy"].contains(&clarity.as_str()) {
            return Err(format!("clarity '{clarity}' must be clear, balanced or hazy"));
        }
        if !["side", "topdown"].contains(&cfg.view.as_str()) {
            return Err(format!("view '{}' must be side or topdown", cfg.view));
        }
        if !["east", "west"].contains(&cfg.facing.as_str()) {
            return Err(format!("facing '{}' must be east or west", cfg.facing));
        }
        let pal_hexes: Vec<String> = match &cfg.palette {
            Some(p) => {
                if p.len() < 2 {
                    return Err("palette needs at least 2 hex colours".into());
                }
                for h in p {
                    if !valid_hex(h) {
                        return Err(format!("invalid palette hex '{h}'"));
                    }
                }
                p.clone()
            }
            None => info.palette.iter().map(|s| (*s).to_string()).collect(),
        };
        let pal = build_5_ramp(&pal_hexes);
        let light_ramp = match &cfg.light_ramp {
            Some(r) => {
                if r.len() < 2 {
                    return Err("light_ramp needs at least 2 hex colours".into());
                }
                for h in r {
                    if !valid_hex(h) {
                        return Err(format!("invalid light_ramp hex '{h}'"));
                    }
                }
                r.clone()
            }
            None => {
                if cfg.palette.is_some() {
                    derive_light_ramp(&pal)
                } else {
                    info.light_ramp.iter().map(|s| (*s).to_string()).collect()
                }
            }
        };
        let name = cfg.name.clone().unwrap_or_else(|| info.id.replace('-', "_"));
        let looped = cfg.looped.unwrap_or(info.looped);
        let element = cfg.element.clone().unwrap_or_else(|| info.element.into());
        Ok(PresetEffect {
            info,
            cfg,
            pal,
            light_ramp,
            w,
            h,
            n,
            fps,
            looped,
            glow,
            clarity,
            name,
            element,
        })
    }

    fn light_scale(&self) -> f64 {
        let base = match self.glow.as_str() {
            "subtle" => 0.55,
            "strong" => 1.25,
            _ => 0.9,
        };
        if self.cfg.steps { base * 0.55 } else { base }
    }
}

/// Safe pixel put that keeps a 2 px inner margin so drawings never touch the cell edge.
fn safe_put(cv: &mut Canvas, x: f64, y: f64, c: Rgb, a: f64) {
    let xi = (x - cv.ox).floor();
    let yi = (y - cv.oy).floor();
    if xi >= 2.0 && yi >= 2.0 && xi <= (cv.w as f64 - 3.0) && yi <= (cv.h as f64 - 3.0) {
        cv.px(yi as usize, xi as usize, c, a);
    }
}

fn safe_disc(cv: &mut Canvas, cx: f64, cy: f64, r: f64, c: Rgb, a: f64) {
    if r <= 0.0 || a <= 0.0 {
        return;
    }
    let (ys, xs) = cv.win(cx - r, cx + r, cy - r, cy + r);
    for yi in ys {
        if yi < 2 || yi + 2 >= cv.h {
            continue;
        }
        for xi in xs.clone() {
            if xi < 2 || xi + 2 >= cv.w {
                continue;
            }
            let [x, y] = cv.world(yi, xi);
            let dx = x - cx;
            let dy = y - cy;
            if dx * dx + dy * dy <= r * r {
                cv.px(yi, xi, c, a);
            }
        }
    }
}

fn safe_line(cv: &mut Canvas, p0: [f64; 2], p1: [f64; 2], c: Rgb, a: f64) {
    let steps = ((p1[0] - p0[0]).abs().max((p1[1] - p0[1]).abs()) as i64 + 1).max(1);
    for k in 0..=steps {
        let t = k as f64 / steps as f64;
        safe_put(cv, p0[0] + (p1[0] - p0[0]) * t, p0[1] + (p1[1] - p0[1]) * t, c, a);
    }
}

/// Quantize alpha to crisp 1.0 when stepped retro mode is active to keep colour count <= 48.
fn alpha_for(a: f64, stepped: bool) -> f64 {
    if stepped {
        if a >= 0.35 { 1.0 } else { 0.0 }
    } else {
        np::clip(a, 0.0, 1.0)
    }
}

impl Effect for PresetEffect {
    fn spec(&self) -> Spec {
        let mut sp = Spec::new(&self.name, self.w, self.h, self.n, self.fps);
        sp.seed = self.cfg.seed;
        sp.looped = self.looped;
        sp.view = self.cfg.view.clone();
        sp.dirs = fc::Meta { facing: self.cfg.facing.clone(), ..Default::default() };
        sp.glow = self.glow.clone();
        sp.clarity = self.clarity.clone();
        sp.light_ramp = self.light_ramp.clone();
        sp.steps = self.cfg.steps.then(lt::Steps::default);
        sp.edge_fade = 3.0;
        sp.columns = self.cfg.columns;
        sp.pivot = match self.info.id {
            "dust" | "torch" => [self.w as f64 / 2.0, (self.h as f64 * 0.82).round()],
            "muzzle-flash" => [(self.w as f64 * 0.22).round(), self.h as f64 / 2.0],
            _ => [self.w as f64 / 2.0, self.h as f64 / 2.0],
        };
        if !self.looped {
            let peak = (self.n / 4).max(1) as i64;
            let weight = match self.info.id {
                "explosion" => "blast",
                "slash" | "impact" => "heavy",
                _ => "light",
            };
            if self.info.id != "dust" {
                sp.hits.push(ex::hit(peak, self.fps, ex::Hit { weight: Some(weight.into()), ..Default::default() }));
            }
            let mid = (self.n / 3).max(1) as i64;
            let end = (self.n - 1) as i64;
            if mid < end {
                sp.states = vec![("strike".into(), [0, mid]), ("dissolve".into(), [mid + 1, end])];
            }
        }
        sp.extra.insert("preset".into(), json!(self.info.id));
        sp
    }

    fn draw(&self, cv: &mut Canvas, f: usize, seed: u64, layer: Layer) -> Result<(), String> {
        if layer != Layer::Effect {
            return Ok(());
        }
        let u = if self.n <= 1 { 0.0 } else { f as f64 / (self.n - 1) as f64 };
        let ls = self.light_scale();
        let st = self.cfg.steps;
        let pal = &self.pal;
        let (w, h) = (self.w as f64, self.h as f64);
        let (cx, cy) = (w * 0.5, h * 0.5);
        let s = w.min(h) * 0.42;

        match self.info.id {
            "impact" => draw_impact(cv, u, f, self.n, seed, cx, cy, s, pal, ls, st),
            "hit-sparks" => draw_hit_sparks(cv, u, f, self.n, seed, cx, cy, s, pal, ls, st),
            "slash" => draw_slash(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "muzzle-flash" => draw_muzzle(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "explosion" => draw_explosion(cv, u, f, self.n, seed, cx, cy, s, pal, ls, st),
            "dust" => draw_dust(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "magic" => draw_magic(cv, u, f, self.n, seed, cx, cy, s, pal, ls, st),
            "torch" => draw_torch(cv, f, self.n, seed, w, h, pal, ls, st),
            _ => {}
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_impact(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let expand = ease_out(span(u, 0.0, 0.45));
    let fade = 1.0 - ease_in(span(u, 0.35, 1.0));
    if !last {
        // Shockwave ring
        let rr = s * (0.25 + 0.72 * expand);
        let thick = (s * 0.14 * fade).max(1.0);
        let steps = 48;
        for k in 0..steps {
            let ang = k as f64 / steps as f64 * TAU;
            let jitter = 0.92 + 0.16 * hash01(seed ^ 11, k as u64);
            if u > 0.65 && hash01(seed ^ 29, k as u64 + (f as u64) * 7) < (u - 0.55) * 1.8 {
                continue;
            }
            let r = rr * jitter;
            safe_disc(cv, cx + ang.cos() * r, cy + ang.sin() * r, thick * 0.55, pal[2], alpha_for(0.95 * fade, st));
        }
        // Central hot burst core
        let core_r = s * (0.52 * (1.0 - span(u, 0.12, 0.72)));
        if core_r > 0.8 {
            safe_disc(cv, cx, cy, core_r, pal[1], alpha_for(0.95, st));
            safe_disc(cv, cx, cy, core_r * 0.72, pal[3], alpha_for(1.0, st));
            safe_disc(cv, cx, cy, core_r * 0.42, pal[4], alpha_for(1.0, st));
        }
        // Radial shards
        let shard_n = 10u64;
        for i in 0..shard_n {
            let ang = (i as f64 / shard_n as f64) * TAU + (hash01(seed, i) - 0.5) * 0.35;
            let spd = 0.65 + 0.45 * hash01(seed ^ 3, i);
            let r0 = s * (0.15 + 0.55 * expand * spd);
            let len = s * (0.28 * (1.0 - u * 0.75)) * (0.7 + 0.5 * hash01(seed ^ 5, i));
            let r1 = (r0 + len).min(s * 0.96);
            if r1 > r0 + 0.5 && fade > 0.12 {
                let p0 = [cx + ang.cos() * r0, cy + ang.sin() * r0];
                let p1 = [cx + ang.cos() * r1, cy + ang.sin() * r1];
                safe_line(cv, p0, p1, pal[if u < 0.35 { 4 } else { 2 }], alpha_for(fade, st));
            }
        }
        cv.glow(cx, cy, s * (0.95 * (0.5 + 0.5 * expand)), 0.85 * fade * ls);
        cv.glow_ring(cx, cy, rr, (s * 0.25).max(3.0), 0.35 * fade * ls, 1.0);
    } else {
        // Tiny dissolving wisps on the final frame (<= 5% of peak pixels)
        for i in 0..3u64 {
            let ang = hash01(seed ^ 91, i) * TAU;
            let r = s * (0.55 + 0.25 * hash01(seed ^ 97, i));
            safe_put(cv, cx + ang.cos() * r, cy + ang.sin() * r, pal[1], alpha_for(0.7, st));
        }
        cv.glow(cx, cy, s * 0.3, 0.06 * ls);
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_hit_sparks(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let expand = ease_out(span(u, 0.0, 0.7));
    let fade = 1.0 - ease_in(span(u, 0.4, 1.0));
    if !last {
        // Sharp 4-point cross flash near impact
        if u < 0.45 {
            let k = 1.0 - span(u, 0.05, 0.45);
            let arm = s * 0.62 * k;
            safe_line(cv, [cx - arm, cy], [cx + arm, cy], pal[4], 1.0);
            safe_line(cv, [cx, cy - arm], [cx, cy + arm], pal[4], 1.0);
            safe_disc(cv, cx, cy, (s * 0.22 * k).max(1.2), pal[3], 1.0);
            safe_disc(cv, cx, cy, (s * 0.12 * k).max(0.8), pal[4], 1.0);
        }
        // Flying sparks with velocity streaks
        for i in 0..14u64 {
            let ang = hash01(seed ^ 17, i) * TAU;
            let spd = 0.55 + 0.55 * hash01(seed ^ 23, i);
            let dist = s * (0.12 + 0.82 * expand * spd);
            let tail = s * 0.24 * (1.0 - u * 0.7);
            let p1 = [cx + ang.cos() * dist.min(s * 0.94), cy + ang.sin() * dist.min(s * 0.94)];
            let p0 = [cx + ang.cos() * (dist - tail).max(0.0), cy + ang.sin() * (dist - tail).max(0.0)];
            let col = pal[if i % 3 == 0 { 4 } else if i % 3 == 1 { 3 } else { 2 }];
            safe_line(cv, p0, p1, col, alpha_for(fade, st));
            if u < 0.65 {
                cv.glow(p1[0], p1[1], (s * 0.22).max(2.5), 0.32 * fade * ls);
            }
        }
        cv.glow(cx, cy, s * 0.65, 0.75 * (1.0 - span(u, 0.1, 0.85)) * ls);
    } else {
        for i in 0..2u64 {
            let ang = hash01(seed ^ 17, i) * TAU;
            let dist = s * 0.72;
            safe_put(cv, cx + ang.cos() * dist, cy + ang.sin() * dist, pal[2], alpha_for(0.7, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_slash(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let cx = w * 0.42;
    let cy = h * 0.52;
    let rx = w * 0.36;
    let ry = h * 0.36;
    let head_t = ease_out(span(u, 0.0, 0.55));
    let tail_t = ease_in(span(u, 0.18, 0.95));
    let fade = 1.0 - ease_in(span(u, 0.5, 1.0));

    if !last {
        let a_start = -2.15 + tail_t * 3.5;
        let a_end = -1.9 + head_t * 3.5;
        let samples = 44;
        for k in 0..=samples {
            let frac = k as f64 / samples as f64;
            let ang = a_start + (a_end - a_start) * frac;
            let taper = (frac * (1.0 - frac * 0.15)).sqrt() * (1.0 - tail_t * 0.5);
            let width = (h * 0.12 * taper * fade).max(0.8);
            let x = cx + ang.cos() * rx;
            let y = cy + ang.sin() * ry;
            safe_disc(cv, x, y, width, pal[1], alpha_for(0.9 * fade, st));
            safe_disc(cv, x, y, width * 0.68, pal[2], alpha_for(0.95 * fade, st));
            if frac > 0.35 {
                safe_disc(cv, x, y, (width * 0.42).max(0.7), pal[4], alpha_for(fade, st));
            }
            if k % 6 == 0 {
                cv.glow(x, y, width * 2.4 + 2.0, 0.28 * fade * ls);
            }
        }
        // Trailing spark motes along the outer rim
        for i in 0..8u64 {
            let ang = -1.8 + hash01(seed ^ 41, i) * 3.0 * head_t;
            let push = 1.0 + 0.15 * u + 0.1 * hash01(seed ^ 43, i);
            let x = cx + ang.cos() * rx * push;
            let y = cy + ang.sin() * ry * push;
            safe_put(cv, x, y, pal[3], alpha_for(fade, st));
        }
    } else {
        let ang: f64 = 1.35;
        safe_put(cv, cx + ang.cos() * rx * 0.95, cy + ang.sin() * ry * 0.95, pal[2], alpha_for(0.7, st));
        safe_put(cv, cx + ang.cos() * rx * 0.9, cy + ang.sin() * ry * 0.85, pal[1], alpha_for(0.6, st));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_muzzle(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let ox = w * 0.2;
    let oy = h * 0.5;
    let max_len = w * 0.68;
    let max_r = h * 0.34;
    let fade = 1.0 - ease_in(span(u, 0.3, 1.0));

    if !last {
        if u < 0.65 {
            let k = 1.0 - span(u, 0.15, 0.65);
            let len = max_len * (0.45 + 0.55 * ease_out(span(u, 0.0, 0.3))) * k;
            // Forward cone
            for step in 0..24 {
                let t = step as f64 / 23.0;
                let x = ox + len * t;
                let rad = max_r * (1.0 - t * 0.85) * (0.65 + 0.35 * k);
                safe_disc(cv, x, oy, rad, pal[1], alpha_for(0.9 * k, st));
                safe_disc(cv, x, oy, rad * 0.68, pal[3], alpha_for(k, st));
                safe_disc(cv, x, oy, rad * 0.38, pal[4], alpha_for(k, st));
            }
            // Side flash spikes
            for side in [-1.0_f64, 1.0] {
                let tip = [ox + max_len * 0.25 * k, oy + side * max_r * 1.15 * k];
                safe_line(cv, [ox, oy], tip, pal[3], alpha_for(k, st));
            }
            cv.glow(ox + len * 0.35, oy, max_r * 1.8, 0.9 * k * ls);
        }
        // Dissipating smoke puffs forward
        for i in 0..5u64 {
            let t = span(u, 0.15, 1.0);
            let dx = max_len * (0.2 + 0.65 * t) * (0.7 + 0.3 * hash01(seed ^ 51, i));
            let dy = (hash01(seed ^ 53, i) - 0.5) * max_r * 1.2 * (0.4 + 0.6 * t);
            let pr = (max_r * 0.34 * (1.0 - t * 0.65)).max(1.0);
            safe_disc(cv, ox + dx, oy + dy, pr, pal[0], alpha_for(0.75 * fade, st));
        }
    } else {
        safe_put(cv, ox + max_len * 0.65, oy - 1.0, pal[0], alpha_for(0.6, st));
        safe_put(cv, ox + max_len * 0.72, oy + 1.0, pal[0], alpha_for(0.5, st));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_explosion(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let grow = ease_out(span(u, 0.0, 0.4));
    let dissolve = span(u, 0.45, 1.0);
    if !last {
        let base_r = s * (0.25 + 0.72 * grow);
        let (ys, xs) = cv.win(cx - base_r - 2.0, cx + base_r + 2.0, cy - base_r - 2.0, cy + base_r + 2.0);
        for yi in ys {
            if yi < 2 || yi + 2 >= cv.h {
                continue;
            }
            for xi in xs.clone() {
                if xi < 2 || xi + 2 >= cv.w {
                    continue;
                }
                let [x, y] = cv.world(yi, xi);
                let dx = x - cx;
                let dy = y - cy;
                let dist = np::hypot(dx, dy);
                let nval = vnoise(x / 6.0 + u * 1.5, y / 6.0 - u * 1.2, seed ^ 71);
                let edge_r = base_r * (0.72 + 0.42 * nval);
                if dist > edge_r {
                    continue;
                }
                // Carve holes during smoke dissolve
                let hole = vnoise(x / 4.5, y / 4.5, seed ^ 83);
                if dissolve > 0.0 && hole < dissolve * 1.05 {
                    continue;
                }
                let norm = dist / edge_r.max(1e-6);
                let heat = (1.0 - norm) * (1.0 - dissolve * 0.85) + (nval - 0.5) * 0.25;
                let idx = if heat > 0.72 { 4 } else if heat > 0.50 { 3 } else if heat > 0.30 { 2 } else if heat > 0.14 { 1 } else { 0 };
                cv.px(yi, xi, pal[idx], alpha_for(1.0 - dissolve * 0.25, st));
            }
        }
        // Debris specks
        for i in 0..8u64 {
            let ang = hash01(seed ^ 61, i) * TAU;
            let dist = s * (0.3 + 0.65 * ease_out(u)) * (0.75 + 0.25 * hash01(seed ^ 67, i));
            safe_put(cv, cx + ang.cos() * dist, cy + ang.sin() * dist, pal[if u < 0.5 { 3 } else { 1 }], alpha_for(1.0 - dissolve * 0.8, st));
        }
        cv.glow(cx, cy, base_r * 1.05, 0.95 * (1.0 - ease_io(span(u, 0.25, 0.95))) * ls);
    } else {
        for i in 0..4u64 {
            let ang = hash01(seed ^ 61, i) * TAU;
            let dist = s * (0.45 + 0.3 * hash01(seed ^ 67, i));
            safe_put(cv, cx + ang.cos() * dist, cy + ang.sin() * dist, pal[0], alpha_for(0.65, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_dust(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let cx = w * 0.5;
    let ground = h * 0.78;
    let spread = ease_out(span(u, 0.0, 0.65));
    let fade = 1.0 - ease_in(span(u, 0.45, 1.0));
    if !last {
        for i in 0..9u64 {
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let lane = (i / 2) as f64 / 4.0;
            let x = cx + side * w * (0.06 + 0.34 * spread * (0.5 + 0.5 * lane));
            let lift = h * (0.14 + 0.32 * spread) * (0.6 + 0.5 * hash01(seed ^ 31, i));
            let y = ground - lift;
            let r = (h * 0.24 * (0.45 + 0.55 * (1.0 - u * 0.65)) * (0.7 + 0.4 * hash01(seed ^ 37, i))).max(1.2);
            if fade > 0.15 {
                safe_disc(cv, x, y, r, pal[1], alpha_for(0.85 * fade, st));
                safe_disc(cv, x - r * 0.2, y - r * 0.25, r * 0.68, pal[3], alpha_for(0.9 * fade, st));
                if u < 0.5 {
                    safe_disc(cv, x - r * 0.25, y - r * 0.3, r * 0.35, pal[4], alpha_for(0.85 * fade, st));
                }
            }
        }
        cv.glow(cx, ground - h * 0.15, w * 0.3, 0.18 * fade * ls);
    } else {
        safe_put(cv, cx - w * 0.28, ground - h * 0.35, pal[1], alpha_for(0.55, st));
        safe_put(cv, cx + w * 0.26, ground - h * 0.32, pal[1], alpha_for(0.55, st));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_magic(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let build = ease_out(span(u, 0.0, 0.4));
    let fade = 1.0 - ease_in(span(u, 0.55, 1.0));
    if !last {
        // Rotating arcane sigil ring
        let rr = s * (0.35 + 0.55 * build);
        let rot = u * TAU * 0.5;
        let dots = 36;
        for k in 0..dots {
            let ang = rot + k as f64 / dots as f64 * TAU;
            if u > 0.7 && hash01(seed ^ 101, k as u64 + f as u64 * 5) < (u - 0.65) * 2.4 {
                continue;
            }
            let x = cx + ang.cos() * rr;
            let y = cy + ang.sin() * rr;
            let c = if k % 6 == 0 { pal[4] } else { pal[2] };
            safe_disc(cv, x, y, if k % 6 == 0 { 1.4 } else { 0.9 }, c, alpha_for(fade, st));
        }
        // Inscribed triangle / star spokes
        let verts: Vec<[f64; 2]> = (0..3)
            .map(|k| {
                let a = rot + k as f64 * TAU / 3.0;
                [cx + a.cos() * rr * 0.88, cy + a.sin() * rr * 0.88]
            })
            .collect();
        if fade > 0.22 {
            for k in 0..3 {
                safe_line(cv, verts[k], verts[(k + 1) % 3], pal[1], alpha_for(0.8 * fade, st));
            }
        }
        // Central pulsing core
        let cr = s * 0.28 * (0.6 + 0.4 * (u * TAU * 2.0).sin().abs()) * fade;
        if cr > 0.8 {
            safe_disc(cv, cx, cy, cr, pal[2], alpha_for(fade, st));
            safe_disc(cv, cx, cy, cr * 0.55, pal[4], alpha_for(fade, st));
        }
        // Rising arcane motes
        for i in 0..8u64 {
            let ang = hash01(seed ^ 107, i) * TAU + u * 1.4;
            let mr = s * (0.2 + 0.65 * hash01(seed ^ 109, i));
            let mx = cx + ang.cos() * mr;
            let my = cy + ang.sin() * mr * 0.7 - u * s * 0.35;
            safe_put(cv, mx, my, pal[4], alpha_for(fade, st));
        }
        cv.glow(cx, cy, s * 0.95, 0.85 * fade * ls);
        cv.glow_ring(cx, cy, rr, (s * 0.22).max(2.5), 0.42 * fade * ls, 1.0);
    } else {
        for i in 0..3u64 {
            let ang = hash01(seed ^ 107, i) * TAU;
            safe_put(cv, cx + ang.cos() * s * 0.5, cy + ang.sin() * s * 0.5, pal[3], alpha_for(0.65, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_torch(cv: &mut Canvas, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let phase = (f as f64 / n.max(1) as f64) * TAU;
    let cx = w * 0.5;
    let base_y = h * 0.78;
    let flame_h = h * 0.56;
    let flame_w = w * 0.28;

    let (ys, xs) = cv.win(cx - flame_w * 1.6, cx + flame_w * 1.6, base_y - flame_h * 1.2, base_y + flame_w);
    for yi in ys {
        if yi < 2 || yi + 2 >= cv.h {
            continue;
        }
        for xi in xs.clone() {
            if xi < 2 || xi + 2 >= cv.w {
                continue;
            }
            let [x, y] = cv.world(yi, xi);
            let t = (base_y - y) / flame_h; // 0 at base, 1 at tip
            if !(-0.2..=1.08).contains(&t) {
                continue;
            }
            let sway = (phase + t * 3.8).sin() * flame_w * 0.32 * t.max(0.0);
            let half_w = flame_w * (1.0 - t.max(0.0)).powf(0.65) * (0.9 + 0.14 * (phase * 2.0 + t * 5.0).cos());
            let dx = (x - (cx + sway)).abs();
            if t < 0.0 {
                let d = np::hypot(dx / flame_w, -t / 0.22);
                if d > 1.0 {
                    continue;
                }
            } else if dx > half_w {
                continue;
            }
            let edge = if half_w > 1e-6 { dx / half_w } else { 1.0 };
            let heat = (1.0 - edge * 0.85) * (1.0 - t.max(0.0) * 0.75);
            let idx = if heat > 0.75 { 4 } else if heat > 0.52 { 3 } else if heat > 0.30 { 2 } else if heat > 0.14 { 1 } else { 0 };
            cv.px(yi, xi, pal[idx], alpha_for(1.0, st));
        }
    }
    // Smoothly wrapping rising embers
    for i in 0..4u64 {
        let off = hash01(seed ^ 127, i);
        let et = (f as f64 / n.max(1) as f64 + off).fract();
        let ex = cx + ((phase + i as f64 * 1.7).sin() * flame_w * 0.75);
        let ey = base_y - flame_h * (0.25 + 0.9 * et);
        if ey >= 3.0 {
            safe_put(cv, ex, ey, pal[if et < 0.5 { 3 } else { 2 }], alpha_for(1.0 - et * 0.7, st));
        }
    }
    let flick = 0.92 + 0.12 * (phase * 2.0).sin();
    cv.glow(cx, base_y - flame_h * 0.32, w * 0.44 * flick, 0.8 * flick * ls);
}
