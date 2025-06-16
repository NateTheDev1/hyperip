# hyperip

`hyperip` is a simple CLI tool for Windows (Hyper‑V) that retrieves the IPv4 address of a specified Hyper‑V virtual machine using PowerShell and updates (or initializes) a JSON file with that IP address under a given key.

## Features

- Query a Hyper‑V VM’s primary IP address via PowerShell
- Dynamically update (or create) a JSON file field with the retrieved IP
- Store default settings (`out`, `field`, `vm`) in `~/.hyperip/settings.json`
- Cross‑platform path expansion for JSON files (supports `~` on Unix/Mac/Windows)

## Prerequisites

- **Windows 10/Server** with Hyper‑V enabled
- **PowerShell** available in `PATH`
- **Rust toolchain** (for building from source)

## Usage

```text
USAGE:
    hyperip [OPTIONS]

OPTIONS:
    -o, --out <OUT>           JSON file path to write the IP (default or explicit)
    -f, --field <FIELD>       JSON key under which to store the IP
    -v, --vm <VM>             Name of the Hyper‑V VM to query
    -s, --set-default         Save these options as the default in ~/.hyperip/settings.json
    -h, --help                Print help information
    -V, --version             Show version information
```

### Running without defaults

Fetch the IP of a VM and update (or initialize) your JSON file in one shot:

```bash
cargo run -- --out config.json --field mysqlHost --vm "Machine Name"
# or, after installing:
hyperip --out config.json --field mysqlHost --vm "Machine Name"
```

- Creates `config.json` with `{}` if it does not exist
- Ensures parent directories are created
- Updates the `mysqlHost` key with the VM’s IP string

### Saving defaults

If you always use the same file, field, and VM name, you can save them for future runs:

```bash
hyperip --out config.json --field mysqlHost --vm "Machine Name" --set-default
```

This writes your settings to `~/.hyperip/settings.json`. After that, you can run simply:

```bash
hyperip
```

and `hyperip` will read `out`, `field`, and `vm` from your defaults.

## Examples

1. **Initialize defaults**

   ```bash
   hyperip -o ~/.hyperip/data.json -f webHost -v MyWebVM --set-default
   ```

2. **One‑off update**

   ```bash
   hyperip -o /etc/myapp/config.json -f apiServer -v BackendVM
   ```

3. **Using tilde expansion**

   ```bash
   hyperip --out "~/projects/app/config.json" --field localIP --vm DevVM
   ```

## Contributing

Contributions, issues, and feature requests are welcome! Please check the [issues page](https://github.com/youruser/hyperip/issues) before opening a new one.

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
