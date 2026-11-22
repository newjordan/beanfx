//! The beanfx binary.
//!
//!     beanfx version              prints one JSON line: {"version": "<kit version>", "protocol": 2}
//!     beanfx gate ...             the brief gate (see gate.rs)
//!     beanfx list [--json]        list built-in parametric VFX presets
//!     beanfx schema               print JSON schema for `beanfx render --spec`
//!     beanfx render ...           render a preset or JSON spec headlessly to sprite sheets + JSON
//!     beanfx sfx <sheet.json>     synthesise WAV track and cues for an exported sheet JSON
//!     beanfx check <sheet.json>   run deterministic craft and budget checks on an exported sheet
//!
//! protocol 2 means the kit is the Rust library: drafts link it directly and no JSON job travels on stdin.

mod cli;
mod gate;

pub const PROTOCOL: u32 = 2;

const USAGE: &str = "usage: beanfx version
       beanfx gate <prompt|-> [--context <text>]
       beanfx list [--json]
       beanfx schema
       beanfx render [<preset>] [--spec <file|->] [--out <dir>] [--json] ...
       beanfx sfx <sheet.json> [--out <dir>] [--voice retro|soft|real] [--element <el>] [--seed N] [--json]
       beanfx check <sheet.json> [--max-colours N] [--max-mb M] [--no-trim] [--json]";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("version") => println!("{{\"version\": \"{}\", \"protocol\": {PROTOCOL}}}", beanfx::VERSION),
        Some("gate") => std::process::exit(gate::run(&args[1..])),
        Some("list") => std::process::exit(cli::list(&args[1..])),
        Some("schema") => std::process::exit(cli::schema(&args[1..])),
        Some("render") => std::process::exit(cli::render(&args[1..])),
        Some("sfx") => std::process::exit(cli::sfx(&args[1..])),
        Some("check") => std::process::exit(cli::check(&args[1..])),
        Some("-h") | Some("--help") | Some("help") => println!("{USAGE}"),
        _ => {
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    }
}
