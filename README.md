# ap

CLI tool that builds the exact folder + zip structure Canvas wants for C assignment submissions.

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

Download the `.deb` from [releases](../../releases/latest):

```sh
sudo dpkg -i ap_*.deb
```

### Windows Installer (.msi)

Download `ap-windows-x64.msi` from [releases](../../releases/latest) and run it. Installs to Program Files and adds `ap` to your PATH automatically.

### Pre-built binaries

Grab the latest binary from [releases](../../releases/latest).

**Windows:** rename to `ap.exe` and add to your PATH.

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

## Quick start

```sh
ap init                       # setup
ap -a 7 -c main.c --auto-doc  # pack assignment 7 with auto-generated doc (no setup example)
```

After `ap init`, you don't need to pass your id or name or auto-doc each time

## Usage

```sh
# full command with auto-doc
ap -a 7 -n JoeBloggs -i 123456789 -c main.c --auto-doc

# bring your own doc instead
ap -a 7 -n JoeBloggs -i 123456789 -c main.c -d Assignment7_JoeBloggs_123456789.doc

# minimal (if you used ap init)
ap -a 7
```

### Flags

| Flag                     | Short | Description                                                              |
| ------------------------ | ----- | ------------------------------------------------------------------------ |
| `--assignment`           | `-a`  | Assignment number or label (e.g. `7` or `Assignment7`)                   |
| `--name`                 | `-n`  | Student name                                                             |
| `--id`                   | `-i`  | Student ID                                                               |
| `--c-file`               | `-c`  | Path to `.c` file (auto-detected if only one in cwd)                     |
| `--doc-file`             | `-d`  | Path to an existing `.doc` file                                          |
| `--auto-doc`             |       | Generate `.doc` automatically                                            |
| `--run-command`          |       | Custom shell command to compile/run the program                          |
| `--run-display-template` |       | Customize what the terminal prompt shows in the screenshot               |
| `--theme`                | `-t`  | Screenshot theme (`default`, `light`, `dracula`, `monokai`, `solarized`) |
| `--output-dir`           | `-o`  | Output directory (defaults to `.`)                                       |
| `--no-watermark`         |       | Turns off watermark at the bottom of the doc                             |
| `--force`                | `-f`  | Overwrite existing output                                                |

## Config

```sh
ap init              # interactive first-time setup (name, id, auto-doc)
ap config show       # view saved defaults
ap config path       # print config file location
ap config editor     # open config in your editor
ap config reset      # wipe all saved config
```

### Setting defaults

Any CLI flag can be saved as a default with `ap config set`:

```sh
ap config set --name JoeBloggs --id 123456789 --auto-doc true
ap config set --theme dracula
ap config set --output-dir ~/submissions
ap config set --watermark false
ap config set --run-command "make && ./a.out"
ap config set --run-display-template "./{c_stem}"
ap config set --editor "code --wait"
```

To clear a saved value, use the `--clear-*` variants:

```sh
ap config set --clear-run-command
ap config set --clear-run-display-template
ap config set --clear-theme
ap config set --clear-editor
```

Config values are used as defaults and can always be overridden by CLI flags. The config file is TOML and lives at `~/.config/assignment_packer/config.toml` (Linux/macOS) or `%APPDATA%\assignment_packer\config.toml` (Windows).

You can set a preferred editor with `--editor`. Otherwise it checks `$VISUAL`, `$EDITOR`, then probes for common editors in PATH.

## Auto-doc

When `--auto-doc` is enabled, `ap` will:

1. Find `gcc` or `clang` and compile your `.c` file
2. Run the binary and capture stdout/stderr
3. Render a terminal screenshot as a PNG
4. Generate an RTF `.doc` containing your source code, the screenshot, and captured output

### Custom run command

By default `ap` finds `gcc`/`clang`, compiles your `.c` file, and runs it. Use `--run-command` to override this:

```sh
# custom compile + run
ap -a 7 --run-command "make && ./myprogram"

# run a pre-built binary
ap -a 7 --run-command "./a.out"
```

You can also save it in config so you don't have to pass it every time:

```sh
ap config set --run-command "make && ./myprogram"
ap config set --clear-run-command   # remove it
```

### Programs that need input

If your program reads from `stdin` (e.g. `scanf`), it will hang and eventually time out. Use `--run-command` to pipe the input:

```sh
# single input
ap -a 7 --force --run-command "echo '42' | ./a.out"

# multiple inputs (one per line)
ap -a 7 --force --run-command "printf '5\nhello\n3.14\n' | ./a.out"

# or use a heredoc
ap -a 7 --force --run-command "./a.out <<EOF
5
hello
3.14
EOF"
```

Each value corresponds to one input in your program.

### Display template

The screenshot shows a `$ command` prompt line. By default it shows the assignment name (e.g. `$ Assignment7`). Use `--run-display-template` to customize what appears:

```sh
ap -a 7 --run-display-template "./home/{name}/assignments/{assignment}/{c_stem}"
```

This could expand to ./home/cat/assignments/19/main

Available placeholders:

| Placeholder           | Example value |
| --------------------- | ------------- |
| `{assignment}`        | `Assignment7` |
| `{assignment_number}` | `7`           |
| `{name}`              | `JoeBloggs`   |
| `{id}`                | `123456789`   |
| `{c_file}`            | `main.c`      |
| `{c_stem}`            | `main`        |

You can also save it in config:

```sh
ap config set --run-display-template "./{c_stem}"
```

## Themes

The screenshot in the generated doc uses a terminal-style theme. Built-in options:

`default` `light` `dracula` `monokai` `solarized`

```sh
ap -a 7 -c main.c --auto-doc --theme dracula
```

You can also create custom themes as TOML files in `~/.config/assignment_packer/themes/`:

```toml
# ~/.config/assignment_packer/themes/nord.toml
bg = "#2E3440"
fg = "#D8DEE9"
scale = 2    # 1-4
padding = 16 # max 64
font = "JetBrainsMono-Regular.ttf" # in themes dir, or absolute path
font_size = 16                     # pixel height (8-72)
```

Then use it with `--theme nord`.

Themes can also be organized in subfolders:

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

## Output structure

Running `ap -a 7 -n JoeBloggs -i 123456789` produces:

```
Assignment7_JoeBloggs_123456789_Submission/
  Assignment7_JoeBloggs_123456789.doc   # if --auto-doc or --doc-file
  main.c                                 # your source files
  ... (all non-binary files in cwd)
Assignment7_JoeBloggs_123456789_Submission.zip
```

The zip is ready to upload to Canvas.

## Notes

- `--force` overwrites existing output folders and zips
- All non-binary files in the current directory are copied into the submission folder
- If no `.doc` is provided and `--auto-doc` is off, the submission is created without one (with a warning)
- The program times out after 30 seconds — if your program needs input, see [Programs that need input](#programs-that-need-input)
