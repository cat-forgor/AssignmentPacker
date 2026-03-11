# ap

CLI tool that builds the exact folder and zip structure Canvas wants for C assignment submissions.

## Install

### Cargo (any platform)

```sh
cargo install assignment_packer
```

### Homebrew (macOS/Linux)

```sh
brew tap cat-forgor/ap https://github.com/cat-forgor/AssignmentPacker
brew install ap
```

### Scoop (Windows)

```powershell
scoop bucket add ap https://github.com/cat-forgor/AssignmentPacker
scoop install ap
```

### Chocolatey (Windows)

```powershell
choco install ap
```

### WinGet (Windows)

```powershell
winget install cat-forgor.ap
```

### Nix (NixOS / any platform)

```sh
nix run github:cat-forgor/AssignmentPacker              # try without installing
nix profile install github:cat-forgor/AssignmentPacker  # install to profile
```

### AUR (Arch Linux)

```sh
yay -S ap-bin
```

### Debian/Ubuntu (.deb)

Grab the `.deb` from [releases](../../releases/latest):

```sh
sudo dpkg -i ap_*.deb
```

### Windows Installer (.msi)

Download `ap-windows-x64.msi` from [releases](../../releases/latest) and run it. It installs to Program Files and adds `ap` to your PATH automatically.

### Pre-built binaries

Head to [releases](../../releases/latest) and grab the latest binary.

**Windows:** rename it to `ap.exe` and add it to your PATH.

**Linux/macOS:**

```sh
chmod +x ap-linux-x64
mv ap-linux-x64 ~/.local/bin/ap
```

### From source

Requires Rust 1.85+ (edition 2024):

```sh
cargo install --path .
```

---

## Quick start

Two commands and you're good to go:

```sh
ap init  # saves your name, ID, and auto-doc preference
ap -a 7  # auto-detects your .c file and packs everything up
```

Once you've run `ap init`, you'll rarely need to type anything more than `ap -a 7`.

---

## Usage

```sh
# full command with auto-doc
ap -a 7 -n JoeBloggs -i 123456789 -c main.c --auto-doc

# bring your own doc instead
ap -a 7 -n JoeBloggs -i 123456789 -c main.c -d Assignment7_JoeBloggs_123456789.doc

# minimal (once you've run ap init)
ap -a 7

# pipe stdin input non-interactively
ap -a 7 --input "5\nhello"

# custom timeout in seconds (clamped to 5 to 300)
ap -a 7 --timeout 5
```

### Flags

| Flag                     | Short | Description                                                              |
| ------------------------ | ----- | ------------------------------------------------------------------------ |
| `--assignment`           | `-a`  | Assignment number or label (e.g. `7` or `Assignment7`)                   |
| `--name`                 | `-n`  | Student name                                                             |
| `--id`                   | `-i`  | Student ID                                                               |
| `--c-file`               | `-c`  | Path to `.c` file (auto-detected if only one exists in cwd)              |
| `--doc-file`             | `-d`  | Path to an existing `.doc` file                                          |
| `--auto-doc`             |       | Generate a `.doc` automatically                                          |
| `--run-command`          |       | Custom shell command to compile and run your program                     |
| `--input`                |       | Pipe stdin input (supports `\n`, `\r`, `\0`, `\xNN` escapes)             |
| `--timeout`              |       | Run timeout in seconds (default 30, clamped to 5 to 300)                 |
| `--run-display-template` |       | Customize what the terminal prompt shows in the screenshot               |
| `--theme`                | `-t`  | Screenshot theme (`default`, `light`, `dracula`, `monokai`, `solarized`) |
| `--output-dir`           | `-o`  | Output directory (defaults to `.`)                                       |
| `--no-watermark`         |       | Turns off the watermark at the bottom of the doc                         |
| `--force`                | `-f`  | Overwrite existing output                                                |

---

## Config

```sh
ap init              # interactive first-time setup
ap config show       # view your saved defaults
ap config path       # print the config file location
ap config editor     # open the config in your editor
ap config reset      # wipe everything
```

### Setting defaults

Save any CLI flag as a default using `ap config set`:

```sh
ap config set --name JoeBloggs --id 123456789 --auto-doc true
ap config set --theme dracula
ap config set --output-dir ~/submissions
ap config set --watermark false
ap config set --run-command "make && ./a.out"
ap config set --run-display-template "./{c_stem}"
ap config set --editor "code --wait"
ap config set --input "5\nhello"
ap config set --timeout 45
```

Need to clear a saved value? Use the `--clear-*` variants:

```sh
ap config set --clear-run-command
ap config set --clear-input
ap config set --clear-run-display-template
ap config set --clear-theme
ap config set --clear-editor
```

CLI flags always override config values. The config itself is plain TOML and lives at `~/.config/assignment_packer/config.toml` on Linux/macOS or `%APPDATA%\assignment_packer\config.toml` on Windows.

You can set a preferred editor with `--editor`. If you don't, `ap` checks `$VISUAL` and `$EDITOR` first then looks for common editors in your PATH.

---

## Auto-doc

Turn on `--auto-doc` and `ap` takes care of everything for you:

1. Finds `gcc` or `clang` and compiles your `.c` file
2. Runs the binary and captures stdout/stderr
3. Renders a terminal screenshot as a PNG
4. Packages your code, the screenshot, and the captured output into a `.doc`

### Custom run command

By default `ap` compiles with `gcc`/`clang` and runs the result. Need something different? Just override it:

```sh
# custom compile + run
ap -a 7 --run-command "make && ./myprogram"

# run a pre-built binary
ap -a 7 --run-command "./a.out"
```

Save it to config so you don't have to type it every time:

```sh
ap config set --run-command "make && ./myprogram"
ap config set --clear-run-command   # remove it later
```

### Programs that need input

If your program reads from `stdin`, you've got two options:

```sh
# 1) input directly
ap -a 7 --input "42"
ap -a 7 --input "5\nhello\n3.14"

# 2) interactive terminal input (default when --input isn't set)
ap -a 7
```

Interactive mode prints a hint while your program runs:

`Program is running. If it doesn't exit on its own, press Ctrl+Z/Ctrl+D.`

### Display template

The screenshot shows a `$ command` prompt line. By default it uses the assignment name like `$ Assignment7`. Use `--run-display-template` to change it:

```sh
ap -a 7 --run-display-template "./home/{name}/assignments/{assignment}/{c_stem}"
# expands to something like: ./home/cat/assignments/19/main
```

Available placeholders:

| Placeholder           | Example value |
| --------------------- | ------------- |
| `{assignment}`        | `Assignment7` |
| `{assignment_number}` | `7`           |
| `{name}`              | `JoeBloggs`   |
| `{id}`                | `123456789`   |
| `{c_file}`            | `main.c`      |
| `{c_stem}`            | `main`        |

Save it to config like anything else:

```sh
ap config set --run-display-template "./{c_stem}"
```

---

## Themes

Five built-in themes control how the terminal screenshot looks in your doc:

`default` `light` `dracula` `monokai` `solarized`

```sh
ap themes                                        # list them
ap -a 7 -c main.c --auto-doc --theme dracula    # use one
```

Want a custom theme? Drop a TOML file into `~/.config/assignment_packer/themes/`:

```toml
# ~/.config/assignment_packer/themes/nord.toml
bg = "#2E3440"
fg = "#D8DEE9"
scale = 2     # 1 to 4
padding = 16  # max 64
font = "JetBrainsMono-Regular.ttf"  # relative to themes dir, or absolute path
font_size = 16                      # pixel height (8 to 72)
```

Then use it with `--theme nord`. Subdirectories work fine too:

```
~/.config/assignment_packer/themes/
  nord.toml
  dark/
    dracula.toml
    monokai.toml
```

```sh
ap -a 7 --theme dark/dracula
```

Screenshots are capped at 8192×8192 pixels.

---

## Updates

Check if a newer release is out:

```sh
ap update
```

---

## Output structure

Running `ap -a 7 -n JoeBloggs -i 123456789` produces:

```
Assignment7_JoeBloggs_123456789_Submission/
  Assignment7_JoeBloggs_123456789.doc   # if --auto-doc or --doc-file was used
  main.c                                 # your source file
  ... (all non-binary files in cwd)
Assignment7_JoeBloggs_123456789_Submission.zip
```

Upload the zip to Canvas and you're done.
