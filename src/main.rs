//! WasmFlow - WebAssembly Node-Based Visual Composition System
//!
//! Main entry point for the desktop application.

use eframe::egui;
use std::path::PathBuf;
use wasmflow::ui;

/// T099: Command-line arguments
#[derive(Debug)]
struct Args {
    /// Graph file to open on startup
    graph_file: Option<PathBuf>,
    /// Hide the component palette
    no_palette: bool,
    /// Log level (error, warn, info, debug, trace)
    log_level: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            graph_file: None,
            no_palette: false,
            log_level: "info".to_string(),
        }
    }
}

/// Parse command-line arguments
fn parse_args() -> Args {
    let mut args = Args::default();
    let mut iter = std::env::args().skip(1); // Skip program name

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--graph" => {
                if let Some(path) = iter.next() {
                    args.graph_file = Some(PathBuf::from(path));
                } else {
                    eprintln!("Error: --graph requires a file path");
                    std::process::exit(1);
                }
            }
            "--no-palette" => {
                args.no_palette = true;
            }
            "--log-level" => {
                if let Some(level) = iter.next() {
                    args.log_level = level;
                } else {
                    eprintln!("Error: --log-level requires a value (error, warn, info, debug, trace)");
                    std::process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {}", arg);
                eprintln!("Use --help for usage information");
                std::process::exit(1);
            }
        }
    }

    args
}

/// Print help message
fn print_help() {
    println!("WasmFlow v{} - WebAssembly Node-Based Visual Composition", env!("CARGO_PKG_VERSION"));
    println!();
    println!("USAGE:");
    println!("    wasmflow [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --graph <FILE>         Open the specified graph file on startup");
    println!("    --no-palette           Hide the component palette panel");
    println!("    --log-level <LEVEL>    Set log level (error, warn, info, debug, trace)");
    println!("    -h, --help             Print this help message");
    println!();
    println!("EXAMPLES:");
    println!("    wasmflow --graph my-graph.wasmflow");
    println!("    wasmflow --log-level debug");
    println!("    RUST_LOG=wasmflow=debug wasmflow");
}

fn main() -> Result<(), eframe::Error> {
    // T099: Parse command-line arguments
    let args = parse_args();

    // T086: Initialize logging with configured level
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or(&args.log_level)
    ).init();

    log::info!("Starting WasmFlow v{}", env!("CARGO_PKG_VERSION"));
    log::debug!("Arguments: {:?}", args);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("WasmFlow - Visual Composition"),
        ..Default::default()
    };

    // Store args for app initialization
    let graph_file = args.graph_file.clone();
    let show_palette = !args.no_palette;

    eframe::run_native(
        "WasmFlow",
        options,
        Box::new(move |cc| {
            let mut app = ui::WasmFlowApp::new(cc);

            // T099: Load graph file if specified
            if let Some(path) = graph_file {
                log::info!("Loading graph from: {}", path.display());
                app.load_graph_from_path(path);
            }

            // T099: Hide palette if requested
            if !show_palette {
                log::info!("Palette hidden by --no-palette flag");
                app.set_palette_visible(false);
            }

            Ok(Box::new(app))
        }),
    )
}
