use clap::Parser;
use dirs::home_dir;
use powershell_script::PsScriptBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use shellexpand;
use std::{
    fs,
    io::{self},
    path::PathBuf,
    process::exit,
};

/// CLI arguments: --out, --field, --vm are optional when not setting defaults
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// JSON file to update with the VM IP
    #[arg(short, long)]
    out: Option<String>,

    /// JSON field name for the IP address
    #[arg(short, long)]
    field: Option<String>,

    /// Name of the Hyper-V VM to query
    #[arg(short, long)]
    vm: Option<String>,

    /// Store these args as defaults in ~/.hyperip/settings.json
    #[arg(short, long)]
    set_default: bool,
}

/// Stored default configuration
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    out: String,
    field: String,
    vm: String,
}

/// Expand a potentially-tilde path into a full PathBuf
fn expand_path(input: &str) -> PathBuf {
    PathBuf::from(shellexpand::tilde(input).into_owned())
}

/// Load defaults from the config file
fn load_config(path: &PathBuf) -> Config {
    let raw = fs::read_to_string(path).expect("Failed to read default config");
    serde_json::from_str(&raw).expect("Default config is invalid JSON")
}

/// Save defaults to the config file (pretty-printed)
fn save_config(path: &PathBuf, cfg: &Config) {
    let data = serde_json::to_string_pretty(cfg).expect("Failed to serialize default config");
    fs::write(path, data).expect("Failed to write default config file");
}

/// Query the VM's IP via PowerShell
fn query_vm_ip(vm_name: &str) -> String {
    let ps = PsScriptBuilder::new()
        .no_profile(true)
        .non_interactive(true)
        .hidden(true)
        .print_commands(false)
        .build();

    let script = format!(
        "(Get-VM -Name '{vm}').NetworkAdapters.IPAddresses[0]",
        vm = vm_name
    );
    let output = ps.run(&script).expect("PowerShell query failed");
    output.stdout().unwrap_or_default().trim().to_string()
}

/// Read, update a field in, and write back a JSON object
fn update_json(path: &PathBuf, key: &str, value: Value) {
    let raw = fs::read_to_string(path).expect("Failed to read target JSON file");
    let mut data: Value = serde_json::from_str(&raw).expect("Target JSON is invalid");

    if let Value::Object(ref mut map) = data {
        map.insert(key.to_string(), value);
        let out = serde_json::to_string_pretty(&data).expect("Failed to serialize updated JSON");
        fs::write(path, out).expect("Failed to write updated JSON");
    } else {
        eprintln!("Target JSON must be an object at the root");
        exit(1);
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    // Determine config directory and file: ~/.hyperip/settings.json
    let config_dir = home_dir()
        .expect("Could not locate home directory")
        .join(".hyperip");
    let config_file = config_dir.join("settings.json");

    // Ensure the config directory exists
    fs::create_dir_all(&config_dir).expect("Failed to create configuration directory");

    // If setting defaults, require all three args, then save and exit
    if args.set_default {
        let out = args.out.as_deref().unwrap_or_else(|| {
            eprintln!("--out is required when using --set-default");
            exit(1)
        });
        let field = args.field.as_deref().unwrap_or_else(|| {
            eprintln!("--field is required when using --set-default");
            exit(1)
        });
        let vm = args.vm.as_deref().unwrap_or_else(|| {
            eprintln!("--vm is required when using --set-default");
            exit(1)
        });

        let cfg = Config {
            out: out.to_string(),
            field: field.to_string(),
            vm: vm.to_string(),
        };
        save_config(&config_file, &cfg);
        println!("Defaults saved to {}", config_file.display());
        return Ok(());
    }

    let defaults = if config_file.exists() {
        Some(load_config(&config_file))
    } else {
        None
    };

    let out = args
        .out
        .or_else(|| defaults.as_ref().map(|c| c.out.clone()))
        .unwrap_or_else(|| {
            eprintln!("Output file must be specified or default set");
            exit(1)
        });

    let field = args
        .field
        .or_else(|| defaults.as_ref().map(|c| c.field.clone()))
        .unwrap_or_else(|| {
            eprintln!("Field must be specified or default set");
            exit(1)
        });

    let vm = args
        .vm
        .or_else(|| defaults.as_ref().map(|c| c.vm.clone()))
        .unwrap_or_else(|| {
            eprintln!("VM name must be specified or default set");
            exit(1)
        });

    let target = expand_path(&out);
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    if !target.exists() {
        fs::write(&target, "{}")?;
    }

    let ip = query_vm_ip(&vm);
    update_json(&target, &field, json!(ip));

    Ok(())
}
