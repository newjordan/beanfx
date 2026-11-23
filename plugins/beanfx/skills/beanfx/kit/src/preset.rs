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

pub const PRESETS: [PresetInfo; 15] = [
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
    // ---- Elemental Mining & Magic series ----
    PresetInfo {
        id: "pick-strike",
        description: "Mining pickaxe contact flash launching gravity-arced ore chips and hot sparks",
        default_w: 48,
        default_h: 48,
        default_frames: 10,
        default_fps: 24.0,
        looped: false,
        element: "metal",
        glow: "medium",
        palette: &["#2a2428", "#685c62", "#c87830", "#ffd060", "#fffbe8"],
        light_ramp: &["#5a2a08", "#ff9e2c", "#ffe698", "#ffffff"],
    },
    PresetInfo {
        id: "ore-shatter",
        description: "Faceted ore boulder cracking with glowing fault lines and bursting into tumbling chunks",
        default_w: 48,
        default_h: 48,
        default_frames: 14,
        default_fps: 20.0,
        looped: false,
        element: "metal",
        glow: "medium",
        palette: &["#201c2c", "#504664", "#9874b8", "#e8b4ff", "#fff8ff"],
        light_ramp: &["#361c58", "#a868e8", "#ecc8ff", "#ffffff"],
    },
    PresetInfo {
        id: "vein-gleam",
        description: "Seamless looping metallic crystal glint sweeping across a faceted ore vein",
        default_w: 32,
        default_h: 32,
        default_frames: 12,
        default_fps: 14.0,
        looped: true,
        element: "arcane",
        glow: "medium",
        palette: &["#0e2824", "#1c6854", "#38c088", "#98ffd0", "#ffffff"],
        light_ramp: &["#0e4838", "#38d898", "#b8ffe0", "#ffffff"],
    },
    PresetInfo {
        id: "frost-shatter",
        description: "Jagged crystalline ice spikes erupting from a frost ring and shattering into shards",
        default_w: 64,
        default_h: 48,
        default_frames: 14,
        default_fps: 20.0,
        looped: false,
        element: "ice",
        glow: "medium",
        palette: &["#102044", "#245898", "#58b8f0", "#b8f0ff", "#ffffff"],
        light_ramp: &["#103060", "#48b0f8", "#c0f4ff", "#ffffff"],
    },
    PresetInfo {
        id: "chain-bolt",
        description: "Forked jagged lightning bolt with violent core flash, branches, and static breakup",
        default_w: 64,
        default_h: 48,
        default_frames: 10,
        default_fps: 24.0,
        looped: false,
        element: "lightning",
        glow: "strong",
        palette: &["#181848", "#3858c8", "#68c4ff", "#c8f4ff", "#ffffff"],
        light_ramp: &["#182870", "#58b8ff", "#d0f4ff", "#ffffff"],
    },
    PresetInfo {
        id: "crystal-bloom",
        description: "Faceted crystal prism cluster erupting from an arcane rune ring with prismatic twinkles",
        default_w: 64,
        default_h: 64,
        default_frames: 16,
        default_fps: 18.0,
        looped: false,
        element: "arcane",
        glow: "strong",
        palette: &["#200e38", "#642498", "#c848d8", "#ffa4f4", "#fff8ff"],
        light_ramp: &["#381060", "#c848e8", "#ffb8f8", "#ffffff"],
    },
    PresetInfo {
        id: "seismic-crystal",
        description: "Full-weight 4-act centerpiece: rune telegraph, arcane strike with impact rays, erupting monoliths, bouncing rubble, and dust plumes",
        default_w: 128,
        default_h: 96,
        default_frames: 40,
        default_fps: 24.0,
        looped: false,
        element: "arcane",
        glow: "strong",
        palette: &["#160e24", "#46266e", "#8e40cc", "#df88ff", "#fff8ff"],
        light_ramp: &["#2c1052", "#9e48f0", "#f0b8ff", "#ffffff"],
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
            "dust" | "torch" | "frost-shatter" | "crystal-bloom" | "seismic-crystal" => [self.w as f64 / 2.0, (self.h as f64 * 0.78).round()],
            "muzzle-flash" | "chain-bolt" => [(self.w as f64 * 0.16).round(), self.h as f64 / 2.0],
            _ => [self.w as f64 / 2.0, self.h as f64 / 2.0],
        };
        if !self.looped {
            if self.info.id == "seismic-crystal" {
                let f_strike = (self.n as f64 * 0.30).round() as i64;
                let f_erupt = (self.n as f64 * 0.42).round() as i64;
                let f_end = (self.n - 1) as i64;
                sp.hits.push(ex::hit(f_strike, self.fps, ex::Hit {
                    weight: Some("finisher".into()),
                    freeze: Some(0.14),
                    shake: Some(5.0),
                    ..Default::default()
                }));
                sp.hits.push(ex::hit(f_erupt, self.fps, ex::Hit {
                    weight: Some("heavy".into()),
                    freeze: Some(0.06),
                    shake: Some(3.0),
                    ..Default::default()
                }));
                sp.states = vec![
                    ("charge".into(), [0, (f_strike - 1).max(0)]),
                    ("strike".into(), [f_strike, f_erupt]),
                    ("erupt".into(), [f_erupt + 1, ((self.n as f64 * 0.75).round() as i64).min(f_end - 1)]),
                    ("dissolve".into(), [((self.n as f64 * 0.75).round() as i64).min(f_end), f_end]),
                ];
            } else {
                let peak = (self.n / 4).max(1) as i64;
                let weight = match self.info.id {
                    "explosion" | "ore-shatter" => "blast",
                    "slash" | "impact" | "pick-strike" | "frost-shatter" | "chain-bolt" | "crystal-bloom" => "heavy",
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
            "pick-strike" => draw_pick_strike(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "ore-shatter" => draw_ore_shatter(cv, u, f, self.n, seed, cx, cy, s, pal, ls, st),
            "vein-gleam" => draw_vein_gleam(cv, f, self.n, seed, cx, cy, s, pal, ls, st),
            "frost-shatter" => draw_frost_shatter(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "chain-bolt" => draw_chain_bolt(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "crystal-bloom" => draw_crystal_bloom(cv, u, f, self.n, seed, w, h, pal, ls, st),
            "seismic-crystal" => draw_seismic_crystal(cv, u, f, self.n, seed, w, h, pal, ls, st),
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
    let cx = w * 0.36;
    let cy = h * 0.50;
    let rx = w * 0.48;
    let ry = h * 0.40;
    let head_t = ease_out(span(u, 0.0, 0.42));
    let tail_t = ease_in(span(u, 0.14, 0.96));
    let dissolve = span(u, 0.42, 1.0);
    let fade = 1.0 - ease_in(span(u, 0.55, 1.0));

    if !last {
        let a_tail = -1.95 + tail_t * 3.15;
        let a_head = -1.65 + head_t * 3.05;
        let arc_span = (a_head - a_tail).max(0.08);
        let max_thick = (0.34 * (1.0 - 0.55 * dissolve) * (0.45 + 0.55 * ease_out(span(u, 0.0, 0.25)))).max(0.06);

        // Per-pixel polar crescent SDF: razor-sharp pointed tips at both head and tail,
        // concentric hue-shifted speed ribbons, and noise-carved ribbon breakup on dissolve.
        for yi in 2..(cv.h.saturating_sub(2)) {
            for xi in 2..(cv.w.saturating_sub(2)) {
                let [x, y] = cv.world(yi, xi);
                let nx = (x - cx) / rx;
                let ny = (y - cy) / ry;
                let r = np::hypot(nx, ny);
                if !(0.45..=1.12).contains(&r) {
                    continue;
                }
                let ang = ny.atan2(nx);
                if ang < a_tail || ang > a_head {
                    continue;
                }
                let frac = (ang - a_tail) / arc_span; // 0 at tail -> 1 at leading blade tip
                // Taper to 0 at both ends: sharp leading blade tip and sharp trailing tail
                let profile = (frac.powf(0.55) * (1.0 - frac).powf(0.32) * 2.15).min(1.0);
                let r_outer = 1.0 + 0.03 * (frac * std::f64::consts::PI).sin();
                let r_inner = r_outer - max_thick * profile;

                let in_main = r >= r_inner && r <= r_outer;
                // Secondary thin speed-smear echo slightly inside the main crescent during peak
                let echo_r = r_inner - 0.08;
                let in_echo = u > 0.12 && u < 0.72 && frac > 0.18 && frac < 0.86
                    && (r - echo_r).abs() < 0.032 * profile;

                if !in_main && !in_echo {
                    continue;
                }

                // Anisotropic speed streaks & dissolve shredding along the arc
                let streak = vnoise(ang * 4.5 - u * 5.0, r * 16.0, seed ^ 41);
                if dissolve > 0.0 {
                    let shred = dissolve * (1.25 - 0.55 * frac);
                    if streak < shred * 0.85 {
                        continue;
                    }
                }

                if in_echo && !in_main {
                    if streak > 0.35 {
                        cv.px(yi, xi, pal[1], alpha_for(0.75 * fade, st));
                    }
                    continue;
                }

                let depth = ((r - r_inner) / (r_outer - r_inner).max(1e-5)).clamp(0.0, 1.0);
                // Crisp outer razor edge -> bright mid energy -> deep trailing shadow
                let energy = depth * 0.72 + frac * 0.28 + (streak - 0.5) * 0.16 - dissolve * 0.25;
                let col = if energy > 0.76 && frac > 0.22 {
                    pal[4]
                } else if energy > 0.54 {
                    pal[3]
                } else if energy > 0.32 {
                    pal[2]
                } else if energy > 0.14 {
                    pal[1]
                } else {
                    pal[0]
                };
                cv.px(yi, xi, col, alpha_for(fade.max(0.25), st));
            }
        }

        // Tight additive light along the hot outer blade edge only (keeps inner crescent crisp)
        for k in 0..10 {
            let frac = 0.25 + 0.72 * (k as f64 / 9.0);
            let ang = a_tail + arc_span * frac;
            let gx = cx + ang.cos() * rx * 0.96;
            let gy = cy + ang.sin() * ry * 0.96;
            cv.glow(gx, gy, (h * 0.14).max(4.0), (0.20 + 0.22 * frac) * fade * ls);
        }

        // Contact star-glint flash at the leading strike apex (frames 1..4)
        if u > 0.08 && u < 0.52 {
            let gk = 1.0 - span(u, 0.20, 0.52);
            let apex = a_tail + arc_span * 0.82;
            let ax = cx + apex.cos() * rx * 0.96;
            let ay = cy + apex.sin() * ry * 0.96;
            let arm = h * 0.18 * gk;
            let diag = arm * 0.65;
            safe_line(cv, [ax - arm, ay], [ax + arm, ay], pal[4], alpha_for(gk, st));
            safe_line(cv, [ax, ay - arm], [ax, ay + arm], pal[4], alpha_for(gk, st));
            safe_line(cv, [ax - diag, ay - diag], [ax + diag, ay + diag], pal[3], alpha_for(0.8 * gk, st));
            safe_line(cv, [ax - diag, ay + diag], [ax + diag, ay - diag], pal[3], alpha_for(0.8 * gk, st));
            cv.glow(ax, ay, h * 0.26, 0.65 * gk * ls);
        }

        // Tangential spark streaks peeling off the outer cutting edge
        for i in 0..10u64 {
            let base_ang = -1.55 + hash01(seed ^ 43, i) * 2.75 * head_t;
            let fly = span(u, 0.10 + 0.04 * (i as f64), 1.0);
            if fly <= 0.0 {
                continue;
            }
            let r_push = 0.98 + 0.16 * fly * (0.6 + 0.6 * hash01(seed ^ 47, i));
            let px = cx + base_ang.cos() * rx * r_push;
            let py = cy + base_ang.sin() * ry * r_push;
            let tx = -base_ang.sin() * (2.5 + 2.0 * (1.0 - fly));
            let ty = base_ang.cos() * (2.5 + 2.0 * (1.0 - fly));
            let c = pal[if i % 3 == 0 { 4 } else { 3 }];
            safe_line(cv, [px - tx * 0.5, py - ty * 0.5], [px + tx * 0.5, py + ty * 0.5], c, alpha_for(fade, st));
        }
    } else {
        // Tiny dissolving sparks on the final frame (<= 5% of peak pixels)
        let ang: f64 = 1.25;
        safe_put(cv, cx + ang.cos() * rx * 0.95, cy + ang.sin() * ry * 0.95, pal[3], alpha_for(0.75, st));
        safe_put(cv, cx + (ang - 0.18).cos() * rx * 0.92, cy + (ang - 0.18).sin() * ry * 0.92, pal[2], alpha_for(0.65, st));
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
            if !(-0.2..=1.0).contains(&t) {
                continue;
            }
            let t_pos = t.clamp(0.0, 0.999);
            let sway = (phase + t * 3.8).sin() * flame_w * 0.32 * t_pos;
            let half_w = flame_w * (1.0 - t_pos).powf(0.65) * (0.9 + 0.14 * (phase * 2.0 + t * 5.0).cos());
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

/// Helper: crisp 4-point diamond twinkle centered at (cx, cy).
fn safe_twinkle(cv: &mut Canvas, cx: f64, cy: f64, arm: f64, core: Rgb, rim: Rgb, a: f64) {
    if arm < 0.8 || a <= 0.0 {
        return;
    }
    safe_line(cv, [cx - arm, cy], [cx + arm, cy], rim, a);
    safe_line(cv, [cx, cy - arm], [cx, cy + arm], rim, a);
    safe_line(cv, [cx - arm * 0.5, cy], [cx + arm * 0.5, cy], core, a);
    safe_line(cv, [cx, cy - arm * 0.5], [cx, cy + arm * 0.5], core, a);
    safe_put(cv, cx, cy, core, a);
}

/// Helper: faceted shaded rock/ore polygon chunk with top-left highlight and bottom-right shadow rim.
fn safe_chunk(cv: &mut Canvas, cx: f64, cy: f64, r: f64, rot: f64, sides: usize, pal: &[Rgb; 5], a: f64) {
    if r < 0.8 || a <= 0.0 {
        return;
    }
    let (ys, xs) = cv.win(cx - r - 1.0, cx + r + 1.0, cy - r - 1.0, cy + r + 1.0);
    let n = sides.max(4) as f64;
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
            let ang = dy.atan2(dx) - rot;
            // Regular polygon radial distance (faceted edges!)
            let sector = TAU / n;
            let local = ((ang % sector) + sector) % sector - sector * 0.5;
            let poly_r = r * (sector * 0.5).cos() / local.cos().max(0.2);
            if dist > poly_r {
                continue;
            }
            let norm = dist / poly_r.max(1e-5);
            // Directional lighting from top-left (-0.7, -0.7)
            let lit = (-dx - dy) / (r * 1.414).max(1e-5);
            let col = if norm > 0.78 && lit < -0.15 {
                pal[0]
            } else if lit > 0.35 {
                pal[3]
            } else if lit > -0.1 {
                pal[2]
            } else {
                pal[1]
            };
            cv.px(yi, xi, col, a);
        }
    }
}

/// Helper: faceted pointed crystal/ice prism growing from `base` along unit `dir` with length `len` and half-width `hw`.
#[allow(clippy::too_many_arguments)]
fn safe_crystal(
    cv: &mut Canvas,
    base: [f64; 2],
    dir: [f64; 2],
    len: f64,
    hw: f64,
    pal: &[Rgb; 5],
    dissolve: f64,
    seed: u64,
    a: f64,
) {
    if len < 1.5 || hw < 0.7 || a <= 0.0 {
        return;
    }
    let perp = [-dir[1], dir[0]];
    let tip = [base[0] + dir[0] * len, base[1] + dir[1] * len];
    let x0 = base[0].min(tip[0]) - hw - 1.0;
    let x1 = base[0].max(tip[0]) + hw + 1.0;
    let y0 = base[1].min(tip[1]) - hw - 1.0;
    let y1 = base[1].max(tip[1]) + hw + 1.0;
    let (ys, xs) = cv.win(x0, x1, y0, y1);
    for yi in ys {
        if yi < 2 || yi + 2 >= cv.h {
            continue;
        }
        for xi in xs.clone() {
            if xi < 2 || xi + 2 >= cv.w {
                continue;
            }
            let [x, y] = cv.world(yi, xi);
            let rx = x - base[0];
            let ry = y - base[1];
            let u = (rx * dir[0] + ry * dir[1]) / len; // 0 at base -> 1 at tip
            if !(0.0..=1.0).contains(&u) {
                continue;
            }
            let v = (rx * perp[0] + ry * perp[1]) / hw; // -1..+1 across width
            // Hexagonal spire profile: straight bevels then sharp pyramid tip above u = 0.62
            let w_lim = if u < 0.62 {
                0.78 + 0.22 * (u / 0.62)
            } else {
                (1.0 - (u - 0.62) / 0.38).max(0.0)
            };
            if v.abs() > w_lim {
                continue;
            }
            if dissolve > 0.0 {
                let n = vnoise(x / 3.5, y / 3.5, seed);
                if n < dissolve * (1.15 - 0.35 * u) {
                    continue;
                }
            }
            // Crisp 3-facet crystal shading: bright left bevel, mid center ridge, dark right bevel, white tip highlight
            let edge = v.abs() / w_lim.max(1e-5);
            let col = if u > 0.84 || (v < -0.15 && edge > 0.72) {
                pal[4]
            } else if v < -0.08 {
                pal[3]
            } else if v < 0.35 {
                pal[2]
            } else if edge > 0.78 {
                pal[0]
            } else {
                pal[1]
            };
            cv.px(yi, xi, col, a);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_pick_strike(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let ox = w * 0.44;
    let oy = h * 0.52;
    let s = w.min(h) * 0.44;
    let fade = 1.0 - ease_in(span(u, 0.45, 1.0));

    if !last {
        // Sharp metallic strike star & contact core (frames 0..4)
        if u < 0.48 {
            let k = 1.0 - span(u, 0.08, 0.48);
            let arm = s * 0.82 * k;
            // Skewed pickaxe strike flare (emphasizes diagonal impact)
            safe_line(cv, [ox - arm * 0.9, oy - arm * 0.45], [ox + arm * 0.95, oy + arm * 0.45], pal[4], alpha_for(k, st));
            safe_line(cv, [ox - arm * 0.4, oy + arm * 0.85], [ox + arm * 0.45, oy - arm * 0.85], pal[3], alpha_for(k, st));
            safe_disc(cv, ox, oy, (s * 0.24 * k).max(1.2), pal[3], alpha_for(k, st));
            safe_disc(cv, ox, oy, (s * 0.13 * k).max(0.8), pal[4], alpha_for(k, st));
            cv.glow(ox, oy, s * 0.72, 0.95 * k * ls);
        }

        // Small rock-dust puff at strike point
        if u < 0.72 {
            let dk = 1.0 - span(u, 0.2, 0.72);
            let spread = ease_out(span(u, 0.0, 0.55));
            for i in 0..4u64 {
                let ang = -2.4 + (i as f64) * 0.7 + (hash01(seed ^ 19, i) - 0.5) * 0.3;
                let px = ox + ang.cos() * s * 0.35 * spread;
                let py = oy + ang.sin() * s * 0.28 * spread;
                safe_disc(cv, px, py, (s * 0.14 * dk).max(0.9), pal[1], alpha_for(0.75 * dk, st));
            }
        }

        // Gravity-arced faceted rock/ore chips bursting upward & outward
        for i in 0..7u64 {
            let ang = -2.65 + (i as f64 / 6.0) * 2.25 + (hash01(seed ^ 21, i) - 0.5) * 0.25;
            let spd = s * (0.85 + 0.55 * hash01(seed ^ 23, i));
            let t = ease_out(u);
            let px = ox + ang.cos() * spd * t;
            let py = oy + ang.sin() * spd * t + s * 0.65 * u * u; // gravity drop!
            let r = (1.3 + 1.4 * hash01(seed ^ 25, i)) * (1.0 - u * 0.45);
            let rot = u * 5.0 * if i % 2 == 0 { 1.0 } else { -1.0 };
            safe_chunk(cv, px, py, r, rot, 5, pal, alpha_for(fade, st));
        }

        // Fast hot metal sparks shooting out ahead of the chips
        for i in 0..10u64 {
            let ang = -2.8 + hash01(seed ^ 27, i) * 3.2;
            let spd = s * (0.95 + 0.65 * hash01(seed ^ 29, i));
            let t1 = ease_out(span(u, 0.0, 0.85));
            let t0 = (t1 - 0.18 * (1.0 - u * 0.6)).max(0.0);
            let p1 = [ox + ang.cos() * spd * t1, oy + ang.sin() * spd * t1 + s * 0.35 * u * u];
            let p0 = [ox + ang.cos() * spd * t0, oy + ang.sin() * spd * t0 + s * 0.35 * (u * 0.85) * (u * 0.85)];
            let c = pal[if i % 2 == 0 { 4 } else { 3 }];
            safe_line(cv, p0, p1, c, alpha_for(fade, st));
            if u < 0.6 {
                cv.glow(p1[0], p1[1], 3.0, 0.24 * fade * ls);
            }
        }
    } else {
        for i in 0..3u64 {
            let px = ox + (hash01(seed ^ 21, i) - 0.5) * s * 1.1;
            let py = oy + s * (0.35 + 0.25 * hash01(seed ^ 23, i));
            safe_put(cv, px, py, pal[1], alpha_for(0.65, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_ore_shatter(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let fade = 1.0 - ease_in(span(u, 0.55, 1.0));

    if !last {
        if u < 0.24 {
            // Anticipation: intact faceted ore boulder with glowing fault-line cracks snapping across it
            let shake = if f % 2 == 0 { 0.8 } else { -0.8 };
            let r = s * 0.54;
            safe_chunk(cv, cx + shake, cy, r, 0.25, 6, pal, alpha_for(1.0, st));
            let ck = span(u, 0.02, 0.22);
            // Glowing fault-line fissures
            for i in 0..3u64 {
                let a = (i as f64) * TAU / 3.0 + 0.4;
                let mid = [cx + shake + a.cos() * r * 0.42 * ck, cy + a.sin() * r * 0.42 * ck];
                let end = [cx + shake + (a + 0.28).cos() * r * 0.88 * ck, cy + (a + 0.28).sin() * r * 0.88 * ck];
                safe_line(cv, [cx + shake, cy], mid, pal[4], 1.0);
                safe_line(cv, mid, end, pal[3], 1.0);
            }
            cv.glow(cx + shake, cy, r * 1.15, (0.35 + 0.55 * ck) * ls);
        } else {
            // Burst phase: 7 tumbling faceted ore chunks + energy flash + dust ring + crystal glints
            let bu = span(u, 0.22, 1.0);
            let expand = ease_out(bu);

            // Brief shatter flash right at break frame
            if bu < 0.35 {
                let fk = 1.0 - span(bu, 0.0, 0.35);
                safe_disc(cv, cx, cy, s * 0.46 * fk, pal[3], alpha_for(fk, st));
                safe_disc(cv, cx, cy, s * 0.24 * fk, pal[4], alpha_for(fk, st));
                cv.glow(cx, cy, s * 0.95, 0.95 * fk * ls);
            }

            // Tumbling faceted ore chunks with gravity
            for i in 0..7u64 {
                let ang = (i as f64 / 7.0) * TAU + (hash01(seed ^ 31, i) - 0.5) * 0.35;
                let spd = s * (0.58 + 0.42 * hash01(seed ^ 33, i));
                let px = cx + ang.cos() * spd * expand;
                let py = cy + ang.sin() * spd * expand + s * 0.42 * bu * bu;
                let sz = (s * (0.16 + 0.12 * hash01(seed ^ 35, i))) * (1.0 - bu * 0.55);
                let rot = hash01(seed ^ 37, i) * TAU + bu * 4.5 * if i % 2 == 0 { 1.0 } else { -1.0 };
                safe_chunk(cv, px, py, sz, rot, 5 + (i as usize % 2), pal, alpha_for(fade, st));
                if bu < 0.55 && i % 2 == 0 {
                    cv.glow(px, py, sz * 2.2, 0.25 * (1.0 - bu) * ls);
                }
            }

            // Sparkling ore dust motes
            for i in 0..8u64 {
                let ang = hash01(seed ^ 39, i) * TAU;
                let dist = s * (0.25 + 0.68 * expand) * (0.7 + 0.3 * hash01(seed ^ 41, i));
                safe_put(cv, cx + ang.cos() * dist, cy + ang.sin() * dist, pal[if i % 2 == 0 { 4 } else { 3 }], alpha_for(fade, st));
            }
        }
    } else {
        for i in 0..3u64 {
            let ang = hash01(seed ^ 31, i) * TAU;
            safe_put(cv, cx + ang.cos() * s * 0.65, cy + ang.sin() * s * 0.65, pal[1], alpha_for(0.6, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_vein_gleam(cv: &mut Canvas, f: usize, n: usize, _seed: u64, cx: f64, cy: f64, s: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let phase = (f as f64 / n.max(1) as f64) * TAU;
    let u = f as f64 / n.max(1) as f64;
    // Periodic diagonal sheen sweeping cleanly across the faceted gem vein
    let sweep = (u * 2.0 - 0.5).rem_euclid(2.0) - 0.5; // -0.5..1.5 periodic

    // Central faceted crystal cluster (3 angled gems) that stays crisp every frame
    let gems: [([f64; 2], [f64; 2], f64, f64); 3] = [
        ([cx, cy + s * 0.58], [0.0, -1.0], s * 1.18, s * 0.34),
        ([cx - s * 0.34, cy + s * 0.50], [-0.42, -0.91], s * 0.85, s * 0.25),
        ([cx + s * 0.32, cy + s * 0.52], [0.46, -0.89], s * 0.80, s * 0.24),
    ];
    for (base, dir, len, hw) in gems {
        safe_crystal(cv, base, dir, len, hw, pal, 0.0, 17, alpha_for(1.0, st));
    }

    // Diagonal specular sheen bar traveling across opaque gem pixels
    let sheen_pos = (sweep - 0.5) * s * 2.4;
    for yi in 2..(cv.h.saturating_sub(2)) {
        for xi in 2..(cv.w.saturating_sub(2)) {
            if cv.col[[yi, xi, 3]] <= 0.5 {
                continue;
            }
            let [x, y] = cv.world(yi, xi);
            let diag = (x - cx) * 0.72 + (y - cy) * 0.70;
            let d = (diag - sheen_pos).abs();
            if d < 1.6 {
                cv.px(yi, xi, pal[4], alpha_for(1.0 - d / 1.8, st));
            } else if d < 3.0 {
                cv.px(yi, xi, pal[3], alpha_for(0.55, st));
            }
        }
    }

    // Two alternating periodic 4-point diamond twinkles on the gem tips
    let tw1 = (0.5 + 0.5 * phase.sin()).powf(3.0);
    let tw2 = (0.5 + 0.5 * (phase + std::f64::consts::PI).sin()).powf(3.0);
    if tw1 > 0.15 {
        let tx = cx - s * 0.12;
        let ty = cy - s * 0.52;
        safe_twinkle(cv, tx, ty, s * 0.45 * tw1, pal[4], pal[3], alpha_for(tw1, st));
        cv.glow(tx, ty, s * 0.55, 0.65 * tw1 * ls);
    }
    if tw2 > 0.15 {
        let tx = cx + s * 0.46;
        let ty = cy - s * 0.08;
        safe_twinkle(cv, tx, ty, s * 0.36 * tw2, pal[4], pal[3], alpha_for(tw2, st));
        cv.glow(tx, ty, s * 0.48, 0.55 * tw2 * ls);
    }
    cv.glow(cx, cy + s * 0.05, s * 0.85, (0.28 + 0.12 * (phase * 2.0).cos()) * ls);
}

#[allow(clippy::too_many_arguments)]
fn draw_frost_shatter(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let cx = w * 0.5;
    let ground = h * 0.82;
    let rise = ease_out(span(u, 0.0, 0.28));
    let shatter = span(u, 0.44, 1.0);
    let fade = 1.0 - ease_in(span(u, 0.58, 1.0));

    if !last {
        // Ground frost ring
        let rr = w * (0.14 + 0.24 * ease_out(span(u, 0.0, 0.35)));
        if fade > 0.15 {
            for k in 0..28 {
                let ang = (k as f64 / 28.0) * TAU;
                let gx = cx + ang.cos() * rr;
                let gy = ground + ang.sin() * (rr * 0.24);
                safe_put(cv, gx, gy, pal[if k % 3 == 0 { 4 } else { 2 }], alpha_for(0.85 * fade, st));
            }
        }

        // 5 jagged ice spikes in 3-size grouping (tall center, 2 mid, 2 small outer)
        let spikes: [(f64, [f64; 2], f64, f64); 5] = [
            (0.0, [0.0, -1.0], h * 0.64, w * 0.085),
            (-w * 0.12, [-0.28, -0.96], h * 0.48, w * 0.068),
            (w * 0.12, [0.30, -0.95], h * 0.50, w * 0.068),
            (-w * 0.23, [-0.52, -0.85], h * 0.34, w * 0.052),
            (w * 0.23, [0.54, -0.84], h * 0.32, w * 0.052),
        ];
        if shatter < 0.78 {
            for (idx, (ox, dir, max_len, hw)) in spikes.iter().enumerate() {
                let delay = (idx as f64) * 0.03;
                let k = ease_out(span(u, delay, 0.30 + delay));
                if k > 0.02 {
                    safe_crystal(
                        cv,
                        [cx + ox, ground],
                        *dir,
                        max_len * k,
                        hw * (0.65 + 0.35 * k),
                        pal,
                        shatter * 1.15,
                        seed ^ (61 + idx as u64 * 7),
                        alpha_for(fade, st),
                    );
                }
            }
        }

        // Flying diamond ice shards & cold mist on shatter phase
        if shatter > 0.05 {
            let su = span(u, 0.42, 1.0);
            for i in 0..10u64 {
                let ang = -2.85 + (i as f64 / 9.0) * 2.55 + (hash01(seed ^ 71, i) - 0.5) * 0.2;
                let spd = w * (0.20 + 0.26 * hash01(seed ^ 73, i));
                let px = cx + ang.cos() * spd * ease_out(su);
                let py = (ground - h * 0.28) + ang.sin() * spd * ease_out(su) + h * 0.32 * su * su;
                let sz = (2.6 - 1.4 * su).max(0.9);
                safe_chunk(cv, px, py, sz, su * 6.0 + i as f64, 4, pal, alpha_for(fade, st));
            }
        }

        cv.glow(cx, ground - h * 0.24, w * 0.34, (0.72 * rise * fade) * ls);
        cv.glow_ring(cx, ground, rr, 3.5, 0.35 * fade * ls, 0.3);
    } else {
        for i in 0..3u64 {
            let px = cx + (hash01(seed ^ 71, i) - 0.5) * w * 0.55;
            let py = ground - h * (0.15 + 0.3 * hash01(seed ^ 73, i));
            safe_put(cv, px, py, pal[3], alpha_for(0.65, st));
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_chain_bolt(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let x0 = w * 0.10;
    let x1 = w * 0.90;
    let cy = h * 0.50;
    let fade = 1.0 - ease_in(span(u, 0.45, 1.0));

    if !last {
        // Generate jagged 9-node main bolt path, re-jittered cleanly per 2 frames for authentic electric crackle
        let snap = (f / 2) as u64;
        let segs = 8usize;
        let mut pts = Vec::with_capacity(segs + 1);
        for k in 0..=segs {
            let t = k as f64 / segs as f64;
            let env = (t * std::f64::consts::PI).sin();
            let j = (hash01(seed ^ 81 ^ (snap * 13), k as u64) - 0.5) * 2.0;
            let x = x0 + (x1 - x0) * t;
            let y = cy + j * h * 0.26 * env;
            pts.push([x, y]);
        }

        let is_peak = (1..=4).contains(&f);
        let dissolve = span(u, 0.48, 1.0);

        let keep_seg = ((seed as usize).wrapping_add(f)) % segs;
        for k in 0..segs {
            if dissolve > 0.0 && k != keep_seg && hash01(seed ^ 89, k as u64 + f as u64 * 5) < dissolve * 1.15 {
                continue;
            }
            let a = pts[k];
            let b = pts[k + 1];
            // Layered stroke: outer electric halo -> bright cyan -> hot white core
            if is_peak {
                cv.thick(a, b, 3.6, pal[1], alpha_for(0.85 * fade, st));
                cv.thick(a, b, 2.1, pal[3], alpha_for(fade, st));
                safe_line(cv, a, b, pal[4], alpha_for(fade, st));
                cv.glow_line(a, b, 5.5, 0.42 * fade * ls);
            } else {
                cv.thick(a, b, 2.0, pal[2], alpha_for(0.9 * fade, st));
                safe_line(cv, a, b, pal[4], alpha_for(fade, st));
                cv.glow_line(a, b, 3.8, 0.25 * fade * ls);
            }
        }

        // 2 branching secondary forks off nodes 3 and 5
        if u > 0.08 && u < 0.72 {
            for (bi, &node) in [3usize, 5usize].iter().enumerate() {
                let start = pts[node];
                let dir_y = if bi == 0 { -1.0 } else { 1.0 };
                let mid = [
                    start[0] + w * 0.10,
                    (start[1] + dir_y * h * 0.16).clamp(4.0, h - 4.0),
                ];
                let tip = [
                    (mid[0] + w * 0.11).min(w - 4.0),
                    (mid[1] + dir_y * h * 0.12).clamp(4.0, h - 4.0),
                ];
                safe_line(cv, start, mid, pal[3], alpha_for(0.9 * fade, st));
                safe_line(cv, mid, tip, pal[2], alpha_for(0.75 * fade, st));
                cv.glow_line(start, mid, 3.0, 0.22 * fade * ls);
            }
        }

        // Terminal impact flash on right target end
        if (1..=4).contains(&f) {
            let tk = 1.0 - span(f as f64, 1.0, 5.0);
            let end = pts[segs];
            safe_twinkle(cv, end[0], end[1], h * 0.22 * tk, pal[4], pal[3], alpha_for(tk, st));
            cv.glow(end[0], end[1], h * 0.28, 0.75 * tk * ls);
        }
    } else {
        safe_put(cv, x0 + (x1 - x0) * 0.35, cy - 2.0, pal[3], alpha_for(0.7, st));
        safe_put(cv, x0 + (x1 - x0) * 0.72, cy + 2.0, pal[2], alpha_for(0.65, st));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_crystal_bloom(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let cx = w * 0.5;
    let ground = h * 0.80;
    let dissolve = span(u, 0.62, 1.0);
    let fade = 1.0 - ease_in(span(u, 0.68, 1.0));

    if !last {
        // Glowing ground arcane rune ellipse
        let rr = w * (0.16 + 0.22 * ease_out(span(u, 0.0, 0.25)));
        let rot = u * TAU * 0.35;
        if fade > 0.12 {
            for k in 0..24 {
                let ang = rot + (k as f64 / 24.0) * TAU;
                let gx = cx + ang.cos() * rr;
                let gy = ground + ang.sin() * (rr * 0.32);
                let c = if k % 4 == 0 { pal[4] } else { pal[2] };
                safe_put(cv, gx, gy, c, alpha_for(fade, st));
            }
            cv.glow_ring(cx, ground, rr, 4.0, 0.42 * fade * ls, 0.34);
        }

        // 5 faceted prism crystals erupting from the rune circle
        let prisms: [(f64, [f64; 2], f64, f64, f64); 5] = [
            (0.0, [0.0, -1.0], h * 0.62, w * 0.09, 0.05),
            (-w * 0.12, [-0.26, -0.96], h * 0.46, w * 0.07, 0.10),
            (w * 0.12, [0.28, -0.96], h * 0.48, w * 0.07, 0.12),
            (-w * 0.22, [-0.48, -0.88], h * 0.32, w * 0.055, 0.16),
            (w * 0.22, [0.48, -0.88], h * 0.30, w * 0.055, 0.18),
        ];
        for (idx, (ox, dir, max_len, hw, delay)) in prisms.iter().enumerate() {
            let k = ease_out(span(u, *delay, delay + 0.26));
            if k > 0.02 {
                safe_crystal(
                    cv,
                    [cx + ox, ground],
                    *dir,
                    max_len * k,
                    hw * (0.6 + 0.4 * k),
                    pal,
                    dissolve * 1.15,
                    seed ^ (131 + idx as u64 * 11),
                    alpha_for(fade, st),
                );
            }
        }

        // Prismatic twinkles at peak bloom + rising mana motes
        if u > 0.25 && u < 0.85 {
            let tk = (1.0 - ((u - 0.52).abs() / 0.33)).clamp(0.0, 1.0);
            safe_twinkle(cv, cx, ground - h * 0.60, w * 0.12 * tk, pal[4], pal[3], alpha_for(tk, st));
            safe_twinkle(cv, cx - w * 0.22, ground - h * 0.40, w * 0.08 * tk, pal[4], pal[3], alpha_for(tk, st));
            safe_twinkle(cv, cx + w * 0.24, ground - h * 0.42, w * 0.08 * tk, pal[4], pal[3], alpha_for(tk, st));
        }
        for i in 0..8u64 {
            let ang = hash01(seed ^ 141, i) * TAU + u * 1.5;
            let rad = w * (0.12 + 0.24 * hash01(seed ^ 143, i));
            let mx = cx + ang.cos() * rad;
            let my = ground - h * (0.10 + 0.62 * ((u + hash01(seed ^ 145, i) * 0.35).min(1.0)));
            safe_put(cv, mx, my, pal[if i % 2 == 0 { 4 } else { 3 }], alpha_for(fade, st));
        }

        cv.glow(cx, ground - h * 0.28, w * 0.36, 0.78 * ease_out(span(u, 0.0, 0.35)) * fade * ls);
    } else {
        for i in 0..3u64 {
            let px = cx + (hash01(seed ^ 141, i) - 0.5) * w * 0.48;
            let py = ground - h * (0.25 + 0.35 * hash01(seed ^ 143, i));
            safe_put(cv, px, py, pal[3], alpha_for(0.65, st));
        }
    }
}

/// Helper: faceted crystal monolith with crisp 1px dark rim, 3-facet bevels, and glowing internal mana vein.
#[allow(clippy::too_many_arguments)]
fn safe_monolith(
    cv: &mut Canvas,
    base: [f64; 2],
    dir: [f64; 2],
    len: f64,
    hw: f64,
    pal: &[Rgb; 5],
    dissolve: f64,
    pulse: f64,
    seed: u64,
    a: f64,
) {
    if len < 2.0 || hw < 1.0 || a <= 0.0 {
        return;
    }
    let perp = [-dir[1], dir[0]];
    let tip = [base[0] + dir[0] * len, base[1] + dir[1] * len];
    let x0 = base[0].min(tip[0]) - hw - 2.0;
    let x1 = base[0].max(tip[0]) + hw + 2.0;
    let y0 = base[1].min(tip[1]) - hw - 2.0;
    let y1 = base[1].max(tip[1]) + hw + 2.0;
    let (ys, xs) = cv.win(x0, x1, y0, y1);
    for yi in ys {
        if yi < 2 || yi + 2 >= cv.h {
            continue;
        }
        for xi in xs.clone() {
            if xi < 2 || xi + 2 >= cv.w {
                continue;
            }
            let [x, y] = cv.world(yi, xi);
            let rx = x - base[0];
            let ry = y - base[1];
            let u = (rx * dir[0] + ry * dir[1]) / len;
            if !(-0.04..=1.04).contains(&u) {
                continue;
            }
            let v = (rx * perp[0] + ry * perp[1]) / hw;
            let uc = u.clamp(0.0, 1.0);
            let w_lim = if uc < 0.66 {
                0.76 + 0.24 * (uc / 0.66)
            } else {
                (1.0 - (uc - 0.66) / 0.34).max(0.0)
            };
            let rim_pad = 1.05 / hw.max(1.0);
            if v.abs() > w_lim + rim_pad {
                continue;
            }
            if dissolve > 0.0 {
                let n = vnoise(x / 3.8, y / 3.8, seed);
                if n < dissolve * (1.25 - 0.45 * uc) {
                    continue;
                }
            }
            // 1px dark obsidian outline rim for razor readability at 1x
            if u < 0.0 || u > 1.0 || v.abs() > w_lim {
                cv.px(yi, xi, pal[0], a);
                continue;
            }
            // Internal glowing mana vein cracking up the spine
            let vein_x = (vnoise(uc * 5.0, 1.0, seed ^ 19) - 0.5) * 0.45;
            let on_vein = (v - vein_x).abs() < 0.14 && uc > 0.12 && uc < 0.88 && pulse > 0.1;
            let edge = v.abs() / w_lim.max(1e-5);
            let col = if on_vein && pulse > 0.55 {
                pal[4]
            } else if uc > 0.86 || (v < -0.15 && edge > 0.78) {
                pal[4]
            } else if v < -0.12 {
                pal[3]
            } else if v < 0.32 {
                pal[2]
            } else if edge > 0.76 {
                pal[0]
            } else {
                pal[1]
            };
            cv.px(yi, xi, col, a);
        }
    }
}

/// Helper: organic billowing dust/smoke cloud puff with lumpy noise silhouette and directional top-left lighting.
#[allow(clippy::too_many_arguments)]
fn safe_smoke_puff(cv: &mut Canvas, cx: f64, cy: f64, r: f64, pal: &[Rgb; 5], seed: u64, fade: f64, st: bool) {
    if r < 1.2 || fade <= 0.06 {
        return;
    }
    let (ys, xs) = cv.win(cx - r * 1.3, cx + r * 1.3, cy - r * 1.3, cy + r * 1.3);
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
            let n = vnoise(x / 5.5, y / 5.5, seed);
            let lim = r * (0.72 + 0.42 * n);
            if dist > lim {
                continue;
            }
            if fade < 0.75 && vnoise(x / 3.5, y / 3.5, seed ^ 7) > fade * 1.25 {
                continue;
            }
            let lit = (-dx * 0.6 - dy * 0.8) / lim.max(1e-5);
            let col = if lit > 0.32 {
                pal[2]
            } else if lit > -0.18 {
                pal[1]
            } else {
                pal[0]
            };
            cv.px(yi, xi, col, alpha_for(0.88 * fade, st));
        }
    }
}

/// Full-weight 4-act centerpiece: Seismic Crystal Rupture (128x96, 40 frames @ 24fps).
#[allow(clippy::too_many_arguments)]
fn draw_seismic_crystal(cv: &mut Canvas, u: f64, f: usize, n: usize, seed: u64, w: f64, h: f64, pal: &[Rgb; 5], ls: f64, st: bool) {
    let last = f + 1 == n;
    let cx = w * 0.50;
    let ground = h * 0.78;
    let flat = 0.28;
    let fade = 1.0 - ease_in(span(u, 0.78, 1.0));

    if last {
        // 3 tiny drifting cooling embers on final frame (<= 5% peak pixels)
        for i in 0..3u64 {
            let px = cx + (hash01(seed ^ 201, i) - 0.5) * w * 0.44;
            let py = ground - h * (0.18 + 0.36 * hash01(seed ^ 203, i));
            safe_put(cv, px, py, pal[3], alpha_for(0.65, st));
        }
        return;
    }

    // ---- ACT 1: TELEGRAPH & DUAL RUNE CHARGE (u: 0.00 -> 0.32) ----
    let charge = span(u, 0.0, 0.30);
    let ring_fade = 1.0 - ease_in(span(u, 0.72, 1.0));
    let rr_outer = w * (0.14 + 0.28 * ease_out(span(u, 0.0, 0.24)));
    let rr_inner = rr_outer * 0.62;
    let rot = u * TAU * 0.65;

    if ring_fade > 0.08 {
        // Outer + inner perspective arcane rune rings on the ground
        for (rad, dots, dir) in [(rr_outer, 44usize, 1.0), (rr_inner, 28usize, -1.4)] {
            for k in 0..dots {
                let ang = rot * dir + (k as f64 / dots as f64) * TAU;
                if u > 0.75 && hash01(seed ^ 211, k as u64 + f as u64 * 7) < (u - 0.72) * 3.0 {
                    continue;
                }
                let gx = cx + ang.cos() * rad;
                let gy = ground + ang.sin() * (rad * flat);
                let is_node = k % 7 == 0;
                let col = if is_node { pal[4] } else if u < 0.35 { pal[3] } else { pal[2] };
                if is_node {
                    safe_disc(cv, gx, gy, 1.3, col, alpha_for(ring_fade, st));
                } else {
                    safe_put(cv, gx, gy, col, alpha_for(0.88 * ring_fade, st));
                }
            }
        }
        // Inscribed hexagram chords during charge
        if u < 0.38 {
            let ck = (1.0 - span(u, 0.28, 0.38)) * ease_out(span(u, 0.04, 0.22));
            for k in 0..6 {
                let a0 = rot + (k as f64) * TAU / 6.0;
                let a1 = rot + ((k + 2) as f64) * TAU / 6.0;
                let p0 = [cx + a0.cos() * rr_outer * 0.94, ground + a0.sin() * rr_outer * 0.94 * flat];
                let p1 = [cx + a1.cos() * rr_outer * 0.94, ground + a1.sin() * rr_outer * 0.94 * flat];
                safe_line(cv, p0, p1, pal[2], alpha_for(0.75 * ck, st));
            }
        }
        cv.glow_ring(cx, ground, rr_outer, 4.5, (0.25 + 0.45 * charge) * ring_fade * ls, flat);
    }

    // Inward-spiraling pressure motes during charge
    if u < 0.32 {
        for i in 0..14u64 {
            let phase = (u * 3.2 + hash01(seed ^ 215, i)).fract();
            let dist = (1.0 - ease_in(phase)) * rr_outer * 1.15;
            let ang = hash01(seed ^ 217, i) * TAU + phase * 2.2;
            let mx = cx + ang.cos() * dist;
            let my = ground + ang.sin() * dist * flat - (1.0 - phase) * h * 0.24;
            safe_put(cv, mx, my, pal[if i % 2 == 0 { 4 } else { 3 }], alpha_for(1.0 - phase * 0.2, st));
        }
    }

    // Glowing ground fissure crack opening before & persisting after strike
    if u > 0.14 && fade > 0.1 {
        let f_open = ease_out(span(u, 0.14, 0.36));
        let f_len = w * 0.34 * f_open;
        let f_col = if u < 0.55 { pal[4] } else if u < 0.78 { pal[3] } else { pal[1] };
        let pts = [
            [cx - f_len, ground + 1.0],
            [cx - f_len * 0.45, ground - 1.5],
            [cx, ground + 0.5],
            [cx + f_len * 0.48, ground - 1.2],
            [cx + f_len, ground + 1.5],
        ];
        for k in 0..4 {
            cv.thick(pts[k], pts[k + 1], 2.2, pal[0], alpha_for(0.9 * fade, st));
            safe_line(cv, pts[k], pts[k + 1], f_col, alpha_for(fade, st));
        }
        cv.glow(cx, ground, w * 0.22 * f_open, (0.35 + 0.55 * (1.0 - span(u, 0.35, 0.9))) * fade * ls);
    }

    // ---- ACT 2: SKY-TO-GROUND ARCANE LANCE & IMPACT SHOCK-RAYS (u: 0.27 -> 0.42) ----
    if u >= 0.26 && u <= 0.44 {
        let su = span(u, 0.26, 0.44);
        let bk = (1.0 - su).powf(0.75);
        let bw = (w * 0.085 * bk).max(1.0);
        // Vertical lance column descending cleanly inside the frame margin
        let top_y = (bw * 1.2 + 4.0).max(12.0);
        cv.thick([cx, top_y], [cx, ground], bw * 2.2, pal[1], alpha_for(0.8 * bk, st));
        cv.thick([cx, top_y], [cx, ground], bw * 1.35, pal[3], alpha_for(bk, st));
        cv.thick([cx, top_y], [cx, ground], (bw * 0.65).max(1.0), pal[4], alpha_for(bk, st));
        cv.glow_line([cx, top_y], [cx, ground], bw * 2.6 + 3.0, 0.85 * bk * ls);

        // Snap impact frames (frames 11..14): kinetic radial flash rays + blinding contact core!
        if su < 0.55 {
            let ik = 1.0 - span(su, 0.12, 0.55);
            // 12 asymmetric radial impact rays
            for i in 0..12u64 {
                let ang = -std::f64::consts::PI * 0.95 + (i as f64 / 11.0) * std::f64::consts::PI * 0.90;
                let r_in = 6.0 + 8.0 * su;
                let r_out = w * (0.28 + 0.18 * hash01(seed ^ 221, i)) * (0.65 + 0.35 * ik);
                let p0 = [cx + ang.cos() * r_in, ground + ang.sin() * r_in];
                let p1 = [cx + ang.cos() * r_out, ground + ang.sin() * r_out];
                safe_line(cv, p0, p1, pal[if i % 2 == 0 { 4 } else { 3 }], alpha_for(ik, st));
            }
            safe_disc(cv, cx, ground - 2.0, w * 0.11 * ik, pal[3], alpha_for(ik, st));
            safe_disc(cv, cx, ground - 2.0, w * 0.06 * ik, pal[4], alpha_for(ik, st));
            cv.glow(cx, ground - 2.0, w * 0.34, 1.35 * ik * ls);
        }
    }

    // ---- ACT 3 & 4: DEPTH-SORTED MONOLITH ERUPTION, BOUNCING RUBBLE & DUST PLUMES (u: 0.32 -> 0.98) ----
    if u >= 0.31 {
        let eu = span(u, 0.31, 1.0);
        let dissolve = span(u, 0.72, 1.0);
        let vein_pulse = (1.0 - span(u, 0.45, 0.85)).max(0.2);

        // Layer 1 (Back): billowing ground dust/smoke plumes rolling outward left & right
        let smoke_u = span(u, 0.32, 0.96);
        let smoke_fade = 1.0 - ease_in(span(smoke_u, 0.35, 1.0));
        for i in 0..8u64 {
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let tier = (i / 2) as f64 / 3.0;
            let sx = cx + side * w * (0.10 + 0.32 * ease_out(smoke_u) * (0.55 + 0.45 * tier));
            let sy = ground - h * (0.04 + 0.14 * smoke_u * (0.6 + 0.4 * hash01(seed ^ 231, i)));
            let sr = (w * 0.085 * (0.55 + 0.45 * (1.0 - smoke_u * 0.4))).max(2.0);
            safe_smoke_puff(cv, sx, sy, sr, pal, seed ^ (233 + i * 7), smoke_fade, st);
        }

        // Layer 2 (Back spires): 4 darker flanking crystal monoliths tilted outward
        let back_spires: [(f64, [f64; 2], f64, f64, f64); 4] = [
            (-w * 0.17, [-0.56, -0.83], h * 0.40, w * 0.052, 0.06),
            (w * 0.17, [0.56, -0.83], h * 0.38, w * 0.052, 0.08),
            (-w * 0.25, [-0.72, -0.69], h * 0.26, w * 0.040, 0.12),
            (w * 0.25, [0.72, -0.69], h * 0.25, w * 0.040, 0.14),
        ];
        for (idx, (ox, dir, max_len, hw, delay)) in back_spires.iter().enumerate() {
            let k = ease_out(span(eu, *delay, delay + 0.16));
            if k > 0.02 {
                safe_monolith(
                    cv,
                    [cx + ox, ground + 1.0],
                    *dir,
                    max_len * k,
                    hw * (0.7 + 0.3 * k),
                    pal,
                    dissolve * 1.15,
                    vein_pulse * 0.6,
                    seed ^ (241 + idx as u64 * 11),
                    alpha_for(fade, st),
                );
            }
        }

        // Layer 3 (Hero monoliths): massive central spire + 2 heavy flanking pillars
        let hero_spires: [(f64, [f64; 2], f64, f64, f64); 3] = [
            (-w * 0.09, [-0.26, -0.96], h * 0.49, w * 0.068, 0.04),
            (w * 0.09, [0.26, -0.96], h * 0.47, w * 0.068, 0.05),
            (0.0, [0.0, -1.0], h * 0.64, w * 0.088, 0.0),
        ];
        for (idx, (ox, dir, max_len, hw, delay)) in hero_spires.iter().enumerate() {
            let k = ease_out(span(eu, *delay, delay + 0.14));
            if k > 0.02 {
                safe_monolith(
                    cv,
                    [cx + ox, ground + 2.0],
                    *dir,
                    max_len * k,
                    hw * (0.75 + 0.25 * k),
                    pal,
                    dissolve * 1.15,
                    vein_pulse,
                    seed ^ (261 + idx as u64 * 13),
                    alpha_for(fade, st),
                );
            }
        }

        // Layer 4 (Front base): uplifted dark basalt slabs anchoring the monoliths to the ground
        if dissolve < 0.7 {
            let bf = 1.0 - span(dissolve, 0.1, 0.7);
            for (ox, sz) in [(-w * 0.13, 4.2), (-w * 0.04, 4.8), (w * 0.05, 4.6), (w * 0.14, 4.0)] {
                safe_chunk(cv, cx + ox, ground + 1.0, sz, ox * 0.1, 5, pal, alpha_for(bf, st));
            }
        }

        // Real 2D bouncing rubble physics (12 faceted chunks launching & bouncing on y = ground!)
        let t_sec = eu * 1.15;
        let grav = h * 2.1;
        for i in 0..12u64 {
            let side = if i % 2 == 0 { 1.0 } else { -1.0 };
            let vx = side * w * (0.14 + 0.34 * hash01(seed ^ 271, i));
            let vy0 = h * (0.55 + 0.55 * hash01(seed ^ 273, i));
            let px = cx + vx * t_sec;
            // First arc + bounce off ground with restitution 0.45
            let t_land = (2.0 * vy0) / grav;
            let py = if t_sec <= t_land {
                ground - (vy0 * t_sec - 0.5 * grav * t_sec * t_sec)
            } else {
                let tb = t_sec - t_land;
                let vy1 = vy0 * 0.45;
                let y1 = (vy1 * tb - 0.5 * grav * tb * tb).max(0.0);
                ground - y1
            };
            let sz = (2.2 + 2.0 * hash01(seed ^ 275, i)) * (1.0 - eu * 0.45);
            let rot = t_sec * 7.0 * side + hash01(seed ^ 277, i) * TAU;
            safe_chunk(cv, px, py, sz, rot, 5, pal, alpha_for(fade, st));
        }

        // Crown diamond twinkles & rising mana embers
        if u > 0.40 && u < 0.88 {
            let tk = (1.0 - ((u - 0.60).abs() / 0.28)).clamp(0.0, 1.0);
            safe_twinkle(cv, cx, ground - h * 0.62, w * 0.08 * tk, pal[4], pal[3], alpha_for(tk, st));
            safe_twinkle(cv, cx - w * 0.15, ground - h * 0.46, w * 0.055 * tk, pal[4], pal[3], alpha_for(tk, st));
            safe_twinkle(cv, cx + w * 0.15, ground - h * 0.44, w * 0.055 * tk, pal[4], pal[3], alpha_for(tk, st));
        }
        for i in 0..16u64 {
            let mx = cx + (hash01(seed ^ 281, i) - 0.5) * w * 0.58 + (u * 6.0 + i as f64).sin() * 2.5;
            let my = ground - h * (0.08 + 0.72 * ((eu + hash01(seed ^ 283, i) * 0.4).min(1.0)));
            safe_put(cv, mx, my, pal[if i % 3 == 0 { 4 } else { 3 }], alpha_for(fade, st));
        }

        // Hero monolith core light
        cv.glow(cx, ground - h * 0.28, w * 0.32, 0.85 * (1.0 - dissolve * 0.75) * fade * ls);
    }
}
