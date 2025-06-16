use std::{fs, io::Write, path::Path, process::exit};

use clap::Parser;
use powershell_script::PsScriptBuilder;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the JSON file to input the VM IP address
    #[arg(short, long)]
    out: String,

    #[arg(short, long)]
    field: String,

    #[arg(short, long)]
    vm: String,

    #[arg(short, long)]
    set_default: bool,
}

#[derive(Deserialize, Debug)]
struct Config {
    pub vm: String,
    pub out: String,
    pub field: String,
}

fn main() {
    let mut args = Args::parse();

    let mut default = false;

    if args.set_default {
        default = true;
    }

    let dir = Path::new("~/.hyperip");
    let file = Path::new("~/.hyperip/settings.json");

    if default {
        if args.out.is_empty() || args.field.is_empty() || args.vm.is_empty() {
            eprintln!("Missing fields.");
            exit(1);
        }

        if !dir.exists() {
            fs::create_dir_all(dir).expect("Failed to create settings directory.");
        }

        if !file.exists() {
            let mut f = fs::File::create(file).unwrap();
            let default = json!({"out": args.out, "field": args.field, "vm": args.vm});
            f.write_all(default.to_string().as_bytes())
                .expect("Failed to write settings file.");
        }

        let ps = PsScriptBuilder::new()
            .no_profile(true)
            .non_interactive(true)
            .hidden(true)
            .print_commands(false)
            .build();

        let script = "(Get-VM -Name 'MACHINE_NAME').NetworkAdapters.IPAddresses[0]";

        let output = ps.run(script).expect("Failed to run script");

        let file = fs::read_to_string(&args.out).expect("File was unable to be read.");

        let mut data: Value = serde_json::from_str(&file).expect("Unable to read JSON");

        let new_val = json!(output.stdout().unwrap().trim());

        match data {
            Value::Object(ref mut map) => {
                map.insert(args.field, new_val);
            }
            _ => {
                eprintln!("Failed to match data to type. Unable to update field");
                exit(1)
            }
        }

        fs::write(
            &args.out,
            serde_json::to_string_pretty(&data).expect("Failed to serialize JSON"),
        )
        .expect("Failed to write to file");
    } else {
        if args.out.is_empty() || args.field.is_empty() || args.vm.is_empty() {
            if dir.exists() && file.exists() {
                let config = fs::read_to_string(&file).unwrap();

                let config: Config =
                    serde_json::from_str(&config).expect("Unable to read config for default.");

                args.field = config.field;
                args.out = config.out;
                args.vm = config.vm;
            } else {
                eprintln!("Missing fields.");

                exit(1);
            }
        }

        let ps = PsScriptBuilder::new()
            .no_profile(true)
            .non_interactive(true)
            .hidden(true)
            .print_commands(false)
            .build();

        let script = format!(
            "(Get-VM -Name '{}').NetworkAdapters.IPAddresses[0]",
            args.vm
        );

        let output = ps.run(&script).expect("Failed to run script");

        let file = fs::read_to_string(&args.out).expect("File was unable to be read.");

        let mut data: Value = serde_json::from_str(&file).expect("Unable to read JSON");

        let new_val = json!(output.stdout().unwrap().trim());

        match data {
            Value::Object(ref mut map) => {
                map.insert(args.field, new_val);
            }
            _ => {
                eprintln!("Failed to match data to type. Unable to update field");
                exit(1)
            }
        }

        fs::write(
            &args.out,
            serde_json::to_string_pretty(&data).expect("Failed to serialize JSON"),
        )
        .expect("Failed to write to file");
    }
}
