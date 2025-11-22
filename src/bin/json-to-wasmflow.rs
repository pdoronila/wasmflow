//! Convert JSON graph files to .wasmflow binary format
//!
//! Usage: cargo run --bin json-to-wasmflow -- <input.json> [output.wasmflow]
//!
//! This tool converts plain JSON graph representations into the binary
//! .wasmflow format that includes magic bytes, version, and CRC64 checksum.

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;
use wasmflow::graph::graph::NodeGraph;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("JSON to WasmFlow Binary Converter");
        eprintln!();
        eprintln!("Usage: {} <input.json> [output.wasmflow]", args[0]);
        eprintln!();
        eprintln!("Converts JSON graph files to binary .wasmflow format with:");
        eprintln!("  - Magic bytes (WASMFLOW)");
        eprintln!("  - Version number");
        eprintln!("  - CRC64 checksum");
        eprintln!("  - Bincode serialization");
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  cargo run --bin json-to-wasmflow -- examples/scheduler_demo.json");
        eprintln!("  cargo run --bin json-to-wasmflow -- examples/scheduler_demo.json output.wasmflow");
        std::process::exit(1);
    }

    let input_path = PathBuf::from(&args[1]);
    let output_path = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        input_path.with_extension("wasmflow")
    };

    println!("🔄 Converting {} -> {}", input_path.display(), output_path.display());

    convert_json_to_wasmflow(&input_path, &output_path)?;

    println!("✅ Conversion successful!");
    println!("   Input:  {} bytes (JSON)", fs::metadata(&input_path)?.len());
    println!("   Output: {} bytes (Binary)", fs::metadata(&output_path)?.len());

    Ok(())
}

fn convert_json_to_wasmflow(input: &PathBuf, output: &PathBuf) -> Result<()> {
    // Read JSON file
    let json_content = fs::read_to_string(input)
        .with_context(|| format!("Failed to read input file: {}", input.display()))?;

    // Parse JSON into NodeGraph
    let graph: NodeGraph = serde_json::from_str(&json_content)
        .with_context(|| format!("Failed to parse JSON from: {}", input.display()))?;

    println!("📊 Parsed graph: {} nodes, {} connections",
        graph.nodes.len(),
        graph.connections.len()
    );
    println!("   Name: {}", graph.name);
    println!("   Author: {}", graph.metadata.author);

    // Validate graph structure
    graph.validate_structure()
        .context("Graph structure validation failed")?;
    println!("✓ Graph structure validated");

    // Convert to binary .wasmflow format
    graph.save_to_file(output)
        .with_context(|| format!("Failed to save graph to: {}", output.display()))?;

    println!("✓ Saved to {}", output.display());

    Ok(())
}
