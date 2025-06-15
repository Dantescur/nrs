# nrs - Node.js Registry Switcher 🧭

A CLI written in Rust to manage multiple Node.js registries easily and efficiently.

![version](https://img.shields.io/badge/status-beta-blue)
![license](https://img.shields.io/badge/license-MIT-green)

---

## ✨ Features

- Switch between registries (`npm`, `yarn`, `taobao`, `github`, etc.).
- Add, remove, or edit custom registries.
- Stores configuration in `~/.nrsrc` and modifies `~/.npmrc`.
- Run latency tests to check registry availability.

---

## 📦 Instalation

```bash
cargo install nrs
```

Or if you prefer to build yourself

```bash
git clone https://github.com/Dantescur/nrs
cd nrs
cargo build --release
```

## 🧪 Usage

```bash
nrs ls # List all registries
nrs use npm # Use the "npm" registry
nrs add myreg https://custom.registry.com/ # Add a new registry
nrs remove myreg # Remove a registry
nrs current # Show the current registry
nrs test # Test ping for all listed registries
nrs show # Show the current .npmrc file
```

## 🧠 Autocomplete

Install autocompletition for bash/zsh/fish/elvish and powershell:

```bash
nrs complete
```

## 📂 Archivos

~/.nrsrc: Persistent file config for the cli.

~/.npmrc: The npm config file

## License

This project is published under MIT [License](./LICENSE)
