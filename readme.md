# Peer Assessment Report Generator

[![Release](https://img.shields.io/github/v/release/Plarpoon/assessment-report)](https://github.com/Plarpoon/assessment-report/releases/latest)
[![Build](https://img.shields.io/github/actions/workflow/status/Plarpoon/assessment-report/release.yml?branch=master)](https://github.com/Plarpoon/assessment-report/actions)
[![License](https://img.shields.io/github/license/Plarpoon/assessment-report)](LICENSE)

A tool for distributing 50 virtual Euros (vEuros) across group members as part of the peer assessment process for a university course.

## Usage

### Linux & macOS

Extract the archive and run:

```sh
tar -xzf peer-assessment-report-generator-<platform>.tar.gz
cd "Peer Assessment Report Generator"
```

Run the GUI (default):

```sh
./peer-assessment-report-generator
```

Run in console mode:

```sh
./peer-assessment-report-generator --console
```

### Windows

Run the GUI:

```sh
peer-assessment-report-generator.exe
```

Run in console mode (powershell or CMD):

```sh
.\peer-assessment-report-generator.exe --console
```

---

On first launch a setup wizard guides you through creating a `config.toml` file. This file is placed next to the binary and can be edited at any time via the **Edit config** button in the app, or manually.

Once all 50 vEuros are distributed and confirmed, the output file is written to the same directory as the binary with the following format:

```
1DV508WEEK<week><GroupName>By<YourName>.txt
```

## Building from source

```sh
cargo build --release
```

Be sure to check the Linux dependencies for RayLib in case you need to install them.