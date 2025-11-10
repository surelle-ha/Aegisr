# Termenu - Rust CLI Template

<p align="center">
  <img src="https://user-images.githubusercontent.com/8974888/231858967-7c37bf1e-335b-4f5a-9760-da97be9f54bb.png" alt="Termenu" width="480" />
</p>

> A personal, minimal, Rust CLI template for building terminal menu-driven applications — batteries included.

---

## Table of contents

* [About](#about)
* [Features](#features)
* [Quickstart](#quickstart)
* [Usage](#usage)
* [Commands & Workflows](#commands--workflows)
* [Testing](#testing)
* [Resizing the README screenshot (how-to)](#resizing-the-readme-screenshot-how-to)
* [Contributing](#contributing)
* [License](#license)

---

## About

**Termenu** is a small, ergonomic Rust project template aimed at creating interactive CLI tools that use terminal menus. It contains:

* Project scaffolding (`Cargo.toml`)
* CI workflow examples (build & test)
* Opinionated linting & formatter configs (`clippy`, `rustfmt`)

This template is perfect if you want a starting point for a terminal-based utility with good developer ergonomics.

---

## Features

* Single-file example CLI menu
* Build and test GitHub Actions examples
* Minimal dependencies and clear separation of concerns
* Examples for packaging and distributing a binary

---

## Quickstart

1. Clone the template:

```bash
git clone https://github.com/surelle-ha/Termenu-Rust-Template.git
cd Termenu-Rust-Template
```

2. Build the project:

```bash
cargo build --release
```

3. Run the example app:

```bash
cargo run
# or the built binary
./target/release/termenu
```

---

## Usage

The example menu binary exposes a simple interactive menu with a few demo actions (e.g., `Hello`, `About`, `Exit`). Replace the example actions with your own logic and commands.

### Example: add a command

Open `src/main.rs` and add a new menu item in the `menu_items` array. Each action should be a small function so it remains easy to test.

---

## Commands & Workflows

Add (or use) these GitHub Actions workflows under `.github/workflows`:

* `build-and-test.yml` — runs `cargo build --release` and `cargo test` on pushes and PRs
* `deps.yml` — optional: scheduled dependency updates

Example `build-and-test.yml` steps:

```yaml
name: Build & Test
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Build
        run: cargo build --verbose
      - name: Test
        run: cargo test --verbose
```

---

## Testing

Use `cargo test` for unit tests. If you add integration tests, place them in `tests/` and run `cargo test --tests`.

---

## Resizing the README screenshot (how-to)

Below are common ways to resize the screenshot used in the README depending on where you want to resize it (in the Markdown, or the image file itself).

### 1) Resize in the README (render-level)

GitHub supports *raw HTML* inside Markdown. To control visual size of the image in the README, use an `<img>` tag with `width`/`height` attributes or inline CSS:

```markdown
<!-- set width in pixels -->
<img src="https://user-images.githubusercontent.com/8974888/231858967-7c37bf1e-335b-4f5a-9760-da97be9f54bb.png" alt="Termenu" width="360" />

<!-- or use CSS style -->
<img src="..." alt="Termenu" style="max-width:100%;height:auto;" />
```

**Note:** GitHub's Markdown renderer allows HTML, so this approach works in README files on GitHub.

### 2) Resize the image file (preferred for performance)

If you want the file itself to be smaller (faster load, smaller repo size), resize it locally before committing:

#### Using ImageMagick (recommended):

```bash
# Install ImageMagick (macOS: brew install imagemagick)
magick convert screenshot.png -resize 480x screenshot-480.png
# or maintain aspect ratio by specifying only width
magick convert screenshot.png -resize 480x screenshot-480.png
```

#### Using pngquant (lossy but smaller):

```bash
pngquant --quality=60-80 --output screenshot-min.png -- screenshot.png
```

#### Using Python Pillow:

```python
from PIL import Image
img = Image.open('screenshot.png')
img.thumbnail((480, 480))
img.save('screenshot-480.png', optimize=True)
```

After resizing the file, replace the README image URL with the new file path or upload it to your hosting (e.g., GitHub issues/images or the repository itself).

### 3) Resize directly on GitHub (not recommended)

GitHub does not provide a UI to resize an image in-place in the repository. The recommended approach is to resize locally and push the optimized image.

---

## Contributing

Contributions are welcome! Please open an issue or a pull request. Suggested workflow:

1. Fork the repo
2. Create a feature branch
3. Run tests: `cargo test`
4. Open a PR with a clear description of changes

Add a `CONTRIBUTING.md` if you want to standardize the process (code style, commit message format, PR template).

---

## License

MIT — see `LICENSE` file.

---

If you'd like, I can also:

* Generate a smaller, optimized screenshot and add it to the repo.
* Create example `main.rs` and menu code that demonstrates a real menu pattern.
* Add CI workflow files to `.github/workflows`.

Tell me which of those you want and I will add them.
