//! Pipeline CLI subcommands for the `beanfx` binary:
//! `list`, `schema`, `render`, `sfx`, and `check`.

use beanfx::dr::{self, RunOpts, SoundOpts};
use beanfx::preset::{self, PresetConfig, PresetEffect};
use beanfx::{ex, img, sfx};
use ndarray::{s, Array3};
use serde_json::{json, Value};
use std::io::Read;
use std::path::{Path, PathBuf};

fn read_spec(src: &str) -> Result<Value, String> {
    let text = if src == "-" {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).map_err(|e| e.to_string())?;
        buf
    } else {
        std::fs::read_to_string(src).map_err(|e| format!("{src}: {e}"))?
    };
    serde_json::from_str(&text).map_err(|e| format!("invalid spec JSON: {e}"))
}

fn parse_csv_hex(s: &str) -> Vec<String> {
    s.split(',').map(|x| x.trim().to_string()).filter(|x| !x.is_empty()).collect()
}

pub fn list(args: &[String]) -> i32 {
    let as_json = args.iter().any(|a| a == "--json");
    if as_json {
        let out = json!({ "version": beanfx::VERSION, "presets": preset::presets_json() });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        for p in &preset::PRESETS {
            println!(
                "{:<14} {:>3}x{:<3} {:>2}f {:>4.0}fps {:<7} {:<8} {}",
                p.id,
                p.default_w,
                p.default_h,
                p.default_frames,
                p.default_fps,
                if p.looped { "loop" } else { "oneshot" },
                p.element,
                p.description
            );
        }
    }
    0
}

pub fn schema(_args: &[String]) -> i32 {
    println!("{}", serde_json::to_string_pretty(&preset::spec_schema()).unwrap());
    0
}

pub fn render(args: &[String]) -> i32 {
    match render_inner(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    }
}

fn render_inner(args: &[String]) -> Result<i32, String> {
    let mut preset_arg: Option<String> = None;
    let mut spec_path: Option<String> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut name: Option<String> = None;
    let mut w: Option<usize> = None;
    let mut h: Option<usize> = None;
    let mut frames: Option<usize> = None;
    let mut fps: Option<f64> = None;
    let mut seed: Option<u64> = None;
    let mut looped: Option<bool> = None;
    let mut palette: Option<Vec<String>> = None;
    let mut light_ramp: Option<Vec<String>> = None;
    let mut glow: Option<String> = None;
    let mut clarity: Option<String> = None;
    let mut steps = false;
    let mut view: Option<String> = None;
    let mut facing: Option<String> = None;
    let mut columns: Option<i64> = None;
    let mut sound = false;
    let mut voice: Option<String> = None;
    let mut element: Option<String> = None;
    let mut sheet_only = false;
    let mut gif = false;
    let mut as_json = false;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let next = |i: &mut usize, flag: &str| -> Result<String, String> {
            *i += 1;
            args.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
        };
        match a {
            "--spec" => spec_path = Some(next(&mut i, a)?),
            "--out" => out_dir = Some(PathBuf::from(next(&mut i, a)?)),
            "--name" => name = Some(next(&mut i, a)?),
            "--width" | "-w" => w = Some(next(&mut i, a)?.parse().map_err(|_| "bad --width")?),
            "--height" | "-h" => h = Some(next(&mut i, a)?.parse().map_err(|_| "bad --height")?),
            "--frames" | "-n" => frames = Some(next(&mut i, a)?.parse().map_err(|_| "bad --frames")?),
            "--fps" => fps = Some(next(&mut i, a)?.parse().map_err(|_| "bad --fps")?),
            "--seed" => seed = Some(next(&mut i, a)?.parse().map_err(|_| "bad --seed")?),
            "--loop" => looped = Some(true),
            "--oneshot" => looped = Some(false),
            "--palette" => palette = Some(parse_csv_hex(&next(&mut i, a)?)),
            "--light-ramp" => light_ramp = Some(parse_csv_hex(&next(&mut i, a)?)),
            "--glow" => glow = Some(next(&mut i, a)?),
            "--clarity" => clarity = Some(next(&mut i, a)?),
            "--steps" => steps = true,
            "--view" => view = Some(next(&mut i, a)?),
            "--facing" => facing = Some(next(&mut i, a)?),
            "--columns" => columns = Some(next(&mut i, a)?.parse().map_err(|_| "bad --columns")?),
            "--sound" => sound = true,
            "--voice" => voice = Some(next(&mut i, a)?),
            "--element" => element = Some(next(&mut i, a)?),
            "--sheet-only" => sheet_only = true,
            "--gif" => gif = true,
            "--json" => as_json = true,
            s if !s.starts_with('-') && preset_arg.is_none() => preset_arg = Some(s.to_string()),
            other => return Err(format!("unknown render argument: {other}")),
        }
        i += 1;
    }

    let mut cfg = if let Some(sp) = &spec_path {
        PresetConfig::from_json(&read_spec(sp)?)?
    } else if let Some(p) = preset_arg {
        PresetConfig::new(&p)
    } else {
        return Err("render needs a <preset> name or --spec <file|-> (try `beanfx list`)".into());
    };

    if let Some(v) = name { cfg.name = Some(v); }
    if let Some(v) = w { cfg.w = Some(v); }
    if let Some(v) = h { cfg.h = Some(v); }
    if let Some(v) = frames { cfg.frames = Some(v); }
    if let Some(v) = fps { cfg.fps = Some(v); }
    if let Some(v) = seed { cfg.seed = v; }
    if let Some(v) = looped { cfg.looped = Some(v); }
    if let Some(v) = palette { cfg.palette = Some(v); }
    if let Some(v) = light_ramp { cfg.light_ramp = Some(v); }
    if let Some(v) = glow { cfg.glow = Some(v); }
    if let Some(v) = clarity { cfg.clarity = Some(v); }
    if steps { cfg.steps = true; }
    if let Some(v) = view { cfg.view = v; }
    if let Some(v) = facing { cfg.facing = v; }
    if let Some(v) = columns { cfg.columns = Some(v); }
    if sound { cfg.sound = true; }
    if let Some(v) = voice { cfg.voice = v; }
    if let Some(v) = element { cfg.element = Some(v); }

    let fx = PresetEffect::build(cfg)?;
    let want_sound = fx.cfg.sound;
    let sound_opts = want_sound.then(|| SoundOpts {
        seed: fx.cfg.seed.max(1),
        voice: fx.cfg.voice.clone(),
        element: Some(fx.element.clone()),
        sounds: None,
    });
    let out = out_dir.unwrap_or_else(|| PathBuf::from("out"));
    let opts = RunOpts {
        out: Some(out.clone()),
        json: as_json,
        sheet_only,
        gif,
        src: None,
        sound: sound_opts,
    };
    let rep = dr::run_with(&fx, &out, &opts)?;
    if as_json {
        let mut j = rep.to_json();
        if let Some(o) = j.as_object_mut() {
            o.insert("preset".into(), json!(fx.info.id));
        }
        println!("{}", serde_json::to_string(&j).map_err(|e| e.to_string())?);
    }
    Ok(if rep.ok() { 0 } else { 1 })
}

pub fn sfx(args: &[String]) -> i32 {
    match sfx_inner(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    }
}

fn sfx_inner(args: &[String]) -> Result<i32, String> {
    let mut sheet_json: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut voice = "retro".to_string();
    let mut element: Option<String> = None;
    let mut seed: u64 = 1;
    let mut sounds: Option<String> = None;
    let mut as_json = false;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let next = |i: &mut usize, flag: &str| -> Result<String, String> {
            *i += 1;
            args.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
        };
        match a {
            "--out" => out_dir = Some(PathBuf::from(next(&mut i, a)?)),
            "--voice" => voice = next(&mut i, a)?,
            "--element" => element = Some(next(&mut i, a)?),
            "--seed" => seed = next(&mut i, a)?.parse().map_err(|_| "bad --seed")?,
            "--sounds" => {
                let s = next(&mut i, a)?;
                sounds = if s == "none" { None } else { Some(s) };
            }
            "--json" => as_json = true,
            s if !s.starts_with('-') && sheet_json.is_none() => sheet_json = Some(PathBuf::from(s)),
            other => return Err(format!("unknown sfx argument: {other}")),
        }
        i += 1;
    }

    let jp = sheet_json.ok_or("sfx needs <sheet.json>")?;
    let text = std::fs::read_to_string(&jp).map_err(|e| format!("{}: {e}", jp.display()))?;
    let mut js: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", jp.display()))?;
    let name = jp
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or("sheet.json has no stem")?
        .to_string();
    let out = out_dir.unwrap_or_else(|| jp.parent().unwrap_or_else(|| Path::new(".")).to_path_buf());
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let sound_block = sfx::write(
        &out,
        &name,
        Some(&mut js),
        sfx::Write { seed, voice, element, layers: None, sounds },
    )?;
    let problems = sfx::check(&out, &js)?;
    let ok = problems.is_empty();
    if as_json {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "ok": ok,
                "name": name,
                "out_dir": out.display().to_string(),
                "sound": sound_block,
                "problems": problems,
            }))
            .map_err(|e| e.to_string())?
        );
    } else {
        println!("sound: {} ({})", out.join(format!("{name}_sound.wav")).display(), if ok { "clean" } else { "problems" });
        for p in &problems {
            println!("{p}");
        }
    }
    Ok(if ok { 0 } else { 1 })
}

pub fn check(args: &[String]) -> i32 {
    match check_inner(args) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("error: {e}");
            2
        }
    }
}

fn slice_frames(sheet: &Array3<u8>, rects: &[(usize, usize, usize, usize)]) -> Result<Vec<Array3<u8>>, String> {
    let (sh, sw, _) = sheet.dim();
    let mut frames = Vec::with_capacity(rects.len());
    for &(x, y, w, h) in rects {
        if x + w > sw || y + h > sh {
            return Err(format!("frame rect ({x},{y},{w},{h}) exceeds sheet {sw}x{sh}"));
        }
        frames.push(sheet.slice(s![y..y + h, x..x + w, ..]).to_owned());
    }
    Ok(frames)
}

fn check_inner(args: &[String]) -> Result<i32, String> {
    let mut sheet_json: Option<PathBuf> = None;
    let mut max_colours: Option<f64> = None;
    let mut max_mb: Option<f64> = Some(ex::BUDGET_MB);
    let mut trim: Option<f64> = Some(ex::TRIM);
    let mut as_json = false;

    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        let next = |i: &mut usize, flag: &str| -> Result<String, String> {
            *i += 1;
            args.get(*i).cloned().ok_or_else(|| format!("{flag} needs a value"))
        };
        match a {
            "--max-colours" => max_colours = Some(next(&mut i, a)?.parse().map_err(|_| "bad --max-colours")?),
            "--max-mb" => max_mb = Some(next(&mut i, a)?.parse().map_err(|_| "bad --max-mb")?),
            "--no-trim" => trim = None,
            "--json" => as_json = true,
            s if !s.starts_with('-') && sheet_json.is_none() => sheet_json = Some(PathBuf::from(s)),
            other => return Err(format!("unknown check argument: {other}")),
        }
        i += 1;
    }

    let jp = sheet_json.ok_or("check needs <sheet.json>")?;
    let dir = jp.parent().unwrap_or_else(|| Path::new("."));
    let text = std::fs::read_to_string(&jp).map_err(|e| format!("{}: {e}", jp.display()))?;
    let js: Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", jp.display()))?;
    let bfx = js.pointer("/meta/beanfx").ok_or("missing meta.beanfx in JSON")?;
    let frames_arr = js.get("frames").and_then(Value::as_array).ok_or("missing frames array in JSON")?;
    if frames_arr.is_empty() {
        return Err("frames array is empty".into());
    }
    let mut rects = Vec::with_capacity(frames_arr.len());
    for entry in frames_arr {
        let r = entry.get("frame").ok_or("frame entry missing rect")?;
        rects.push((
            r["x"].as_u64().ok_or("bad frame.x")? as usize,
            r["y"].as_u64().ok_or("bad frame.y")? as usize,
            r["w"].as_u64().ok_or("bad frame.w")? as usize,
            r["h"].as_u64().ok_or("bad frame.h")? as usize,
        ));
    }

    let color_file = bfx
        .get("color")
        .and_then(Value::as_str)
        .or_else(|| bfx.get("sheet").and_then(Value::as_str))
        .or_else(|| js.pointer("/meta/image").and_then(Value::as_str))
        .ok_or("cannot find sheet image path in JSON")?;
    let color_img = img::read(dir.join(color_file))?;
    let frames = slice_frames(&color_img, &rects)?;

    let light_frames = match bfx.get("light").and_then(Value::as_str) {
        Some(lf) if dir.join(lf).is_file() => {
            let limg = img::read_rgb(dir.join(lf))?;
            Some(slice_frames(&limg, &rects)?)
        }
        _ => None,
    };

    let looped = bfx.get("loop").and_then(Value::as_bool).unwrap_or(false);
    let hold = bfx.get("end").and_then(Value::as_str) == Some("hold");
    let fps = bfx.get("fps").and_then(Value::as_f64).unwrap_or(24.0);

    let mut problems = ex::check(
        &frames,
        ex::Check {
            light: light_frames.as_ref().map(|v| v.into()),
            looped,
            hold,
            src: None,
            max_colours,
            trim,
            max_mb,
        },
    )?;
    if let Some(act) = bfx.get("act") {
        problems.extend(ex::check_act(act, frames.len() as i64, fps, looped)?);
    }
    if bfx.get("sound").is_some() {
        problems.extend(sfx::check(dir, &js)?);
    }

    let mem_mb = ex::memory_mb(&frames, light_frames.is_some());
    let framing = ex::framing(&frames)?;
    let ok = problems.is_empty();
    if as_json {
        let fr_val = framing.map(|f| {
            json!({
                "box": [f.x0, f.y0, f.x1, f.y1],
                "size": [f.w, f.h],
                "cut": f.cut,
                "trim": f.trim,
            })
        });
        println!(
            "{}",
            serde_json::to_string(&json!({
                "ok": ok,
                "json": jp.display().to_string(),
                "frames": frames.len(),
                "memory_mb": mem_mb,
                "framing": fr_val,
                "problems": problems,
            }))
            .map_err(|e| e.to_string())?
        );
    } else {
        println!("check: {} ({} frames, {:.2} MB) — {}", jp.display(), frames.len(), mem_mb, if ok { "clean" } else { "problems" });
        for p in &problems {
            println!("{p}");
        }
    }
    Ok(if ok { 0 } else { 1 })
}
