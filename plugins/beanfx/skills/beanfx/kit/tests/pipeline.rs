//! Integration tests for the headless pipeline CLI (`list`, `schema`, `render`, `sfx`, `check`)
//! and the built-in parametric VFX presets (`beanfx::preset`).

mod common;

use beanfx::dr::{self, RunOpts, SoundOpts};
use beanfx::preset::{self, PresetConfig, PresetEffect, PRESETS};
use beanfx::serde_json::{json, Value};
use std::io::Write;
use std::process::{Command, Stdio};

fn beanfx_bin() -> &'static str {
    env!("CARGO_BIN_EXE_beanfx")
}

#[test]
fn all_builtin_presets_pass_every_craft_check() {
    let dir = common::tmpdir("presets_all");
    for p in &PRESETS {
        let out = dir.join(p.id);
        let cfg = PresetConfig::new(p.id);
        let fx = PresetEffect::build(cfg).unwrap_or_else(|e| panic!("build {}: {e}", p.id));
        let rep = dr::run_with(
            &fx,
            &out,
            &RunOpts { out: Some(out.clone()), json: true, sheet_only: true, gif: false, src: None, sound: None },
        )
        .unwrap_or_else(|e| panic!("run {}: {e}", p.id));
        assert!(
            rep.ok(),
            "preset '{}' failed checks: {:?}",
            p.id,
            rep.problems
        );
        assert!(rep.sheet.is_file(), "preset '{}' missing sheet", p.id);
        assert!(rep.color.is_file(), "preset '{}' missing color sheet", p.id);
        assert!(rep.light.is_file(), "preset '{}' missing light sheet", p.id);
        assert!(rep.json_path.is_file(), "preset '{}' missing JSON", p.id);
    }
}

#[test]
fn palette_override_steps_columns_and_sound_work_together() {
    let dir = common::tmpdir("preset_custom");
    let mut cfg = PresetConfig::new("impact");
    cfg.name = Some("custom_boom".into());
    cfg.w = Some(48);
    cfg.h = Some(48);
    cfg.frames = Some(10);
    cfg.fps = Some(20.0);
    cfg.palette = Some(vec!["#102030".into(), "#2060a0".into(), "#60d0ff".into(), "#f0ffff".into()]);
    cfg.steps = true;
    cfg.columns = Some(10);
    let fx = PresetEffect::build(cfg).unwrap();
    let rep = dr::run_with(
        &fx,
        &dir,
        &RunOpts {
            out: Some(dir.clone()),
            json: true,
            sheet_only: true,
            gif: true,
            src: None,
            sound: Some(SoundOpts { seed: 3, voice: "retro".into(), element: Some("ice".into()), sounds: None }),
        },
    )
    .unwrap();
    assert!(rep.ok(), "custom render had problems: {:?}", rep.problems);
    assert!(rep.gif.as_ref().unwrap().is_file(), "expected gif output");
    assert!(rep.sound.is_some(), "expected sound metadata");
    assert_eq!(rep.meta["meta"]["beanfx"]["columns"].as_i64(), Some(10));
    assert!(dir.join("sheets").join("custom_boom_sound.wav").is_file());
}

#[test]
fn preset_config_from_json_and_validation() {
    let schema = preset::spec_schema();
    assert_eq!(schema["type"], "object");
    let v = json!({
        "preset": "slash",
        "name": "ice_slash",
        "width": 64,
        "height": 48,
        "frames": 10,
        "fps": 24,
        "seed": 11,
        "palette": ["#102040", "#4090d0", "#c0f0ff"],
        "glow": "strong",
        "clarity": "clear",
        "steps": false,
        "view": "side",
        "facing": "east",
        "columns": 10,
        "sound": true,
        "voice": "soft",
        "element": "ice"
    });
    let cfg = PresetConfig::from_json(&v).unwrap();
    let fx = PresetEffect::build(cfg).unwrap();
    assert_eq!(fx.name, "ice_slash");
    assert_eq!(fx.element, "ice");

    assert!(PresetEffect::build(PresetConfig::new("nope")).is_err());
    let mut bad = PresetConfig::new("impact");
    bad.w = Some(4);
    assert!(PresetEffect::build(bad).is_err());
}

#[test]
fn cli_list_schema_render_check_and_sfx_end_to_end() {
    let dir = common::tmpdir("cli_e2e");

    // 1. list --json
    let out = Command::new(beanfx_bin()).args(["list", "--json"]).output().unwrap();
    assert!(out.status.success(), "list failed: {}", String::from_utf8_lossy(&out.stderr));
    let listed: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(listed["presets"].as_array().unwrap().len(), PRESETS.len());

    // 2. schema
    let out = Command::new(beanfx_bin()).args(["schema"]).output().unwrap();
    assert!(out.status.success());
    let sch: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(sch["properties"]["preset"].is_object());

    // 3. render via --spec - (stdin) with --sheet-only --gif --json
    let out_dir = dir.join("render_out");
    let mut child = Command::new(beanfx_bin())
        .args(["render", "--spec", "-", "--out", out_dir.to_str().unwrap(), "--sheet-only", "--gif", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let spec = json!({
        "preset": "hit-sparks",
        "name": "spark_test",
        "width": 48,
        "height": 48,
        "frames": 10,
        "fps": 24,
        "columns": 10
    });
    child.stdin.as_mut().unwrap().write_all(spec.to_string().as_bytes()).unwrap();
    let res = child.wait_with_output().unwrap();
    assert!(
        res.status.success(),
        "render failed (code {:?}): stdout={} stderr={}",
        res.status.code(),
        String::from_utf8_lossy(&res.stdout),
        String::from_utf8_lossy(&res.stderr)
    );
    let rep: Value = serde_json::from_slice(&res.stdout).unwrap();
    assert_eq!(rep["ok"], true, "render rep not ok: {rep}");
    assert_eq!(rep["preset"], "hit-sparks");
    let json_path = rep["json"].as_str().unwrap();

    // 4. sfx standalone on exported sheet.json
    let out = Command::new(beanfx_bin())
        .args(["sfx", json_path, "--voice", "retro", "--element", "metal", "--seed", "5", "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "sfx failed: {}", String::from_utf8_lossy(&out.stderr));
    let sfx_rep: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(sfx_rep["ok"], true, "sfx rep: {sfx_rep}");
    assert!(sfx_rep["sound"]["track"].as_str().unwrap().ends_with(".wav"));

    // 5. check standalone on exported sheet.json (including sound check!)
    let out = Command::new(beanfx_bin())
        .args(["check", json_path, "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "check failed: {}", String::from_utf8_lossy(&out.stderr));
    let chk: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(chk["ok"], true, "check rep: {chk}");
    assert_eq!(chk["frames"], 10);
}
