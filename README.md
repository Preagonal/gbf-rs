# gbf-rs
[![codecov](https://codecov.io/gh/Preagonal/gbf-rs/graph/badge.svg?token=V66BCXQ5IX)](https://codecov.io/gh/Preagonal/gbf-rs)
[![Rust CI](https://github.com/cernec1999/gbf-rs/actions/workflows/rust-ci.yml/badge.svg)](https://github.com/cernec1999/gbf-rs/actions/workflows/rust-ci.yml)

`gbf-rs` is a Rust library designed to analyze, disassemble, process, and decompile Graal Script 2 (GS2) bytecode. It provides tools for GS2 bytecode analysis, control flow graph (CFG) generation, and abstract syntax tree (AST) construction, with a focus on modern Rust best practices.

## Features

- **GS2 Bytecode Analysis**:
  - Decode and analyze GraalScript bytecode for program analysis.
- **Disassembly and Decompilation**:
  - Disassemble bytecode into human-readable instructions.
- **Control Flow Graph Visualization**:
  - Generate and render CFGs using Graphviz-compatible DOT files.
- **Abstract Syntax Tree (AST) Construction**:
  - Build ASTs from bytecode to enable higher-level program understanding.
- **Customizable Graph Rendering**:
  - Flexible, trait-based rendering for both pre-processed and post-processed CFGs.
- **Modular Design**:
  - Highly extensible and idiomatic Rust library design.

## Getting Started

### Installation

Add `gbf-rs` as a dependency in your `Cargo.toml`:

```toml
[dependencies]
gbf-rs = "0.0.3"
```

### Minimum Supported Rust Version
This project supports Rust 1.70.0 and later.

### Usage

### Decompile GS2 Bytecode

TODO: Add example code.

#### Generate Control Flow Graph (CFG)

This library can generate directed control flow graphs using Graphviz. We can load arbitrary GS2 bytecode using the following Rust code:

```rs
use std::{fs::File, io::Read, path::Path};

use gbf_rs::{
    cfg_dot::{CfgDotConfig, DotRenderableGraph},
};

fn load_bytecode(name: &str) -> Result<impl Read, std::io::Error> {
    let path = Path::new("tests").join("gs2bc").join(name);
    let file = File::open(path)?;
    Ok(file)
}

fn main() {
    // Load `switch.gs2bc` bytecode file
    let reader = load_bytecode("switch.gs2bc").unwrap();
    let module = gbf_rs::module::ModuleBuilder::new()
        .name("switch.gs2".to_string())
        .reader(Box::new(reader))
        .build()
        .unwrap();

    // Get the first function in the module. The "entry"
    // function is 0, so this really is the "first"
    // function.
    let function = module.get(1).unwrap();

    println!("{}", function.render_dot(CfgDotConfig::default()));
}
```

<details> 
  <summary>As an example, consider this simple GS2 switch statement.</summary>

  ```js
  function switchWithMultipleCasesPerNode() {
      temp.server = "classicplus";
      switch (temp.server) {
          case "classic":
          case "classicplus":
              this.loginserver = "loginclassic1.graalonline.com:14900";
              break;
          case "delteria":
          case "delteriaplus":
              this.loginserver = "logindelteria1.graalonline.com:14900";
              break;
          case "foo":
              this.loginserver = "loginfoo1.graalonline.com:14900";
              break;
          default:
              this.loginserver = "loginserver.graalonline.com:14900";
              break;
      }
      temp.i = this.loginserver.pos(":");
      this.loginhost = this.loginserver.substring(0, temp.i);
      this.loginport = this.loginserver.substring(temp.i + 1, 255);
  }
  ```

  The resulting Graphviz code that `gbf-rs` generates will look like this when exported:
  ![Switch CFG](./docs/switch.svg)
</details>

To export the resulting Graphviz code, you can use the popular `dot` utility like so:

```sh
$ dot -Tpng cfg.dot -o cfg.png
```

#### Build Abstract Syntax Trees (ASTs)

TODO: Add example code.

## Development

### Prerequisites

- Rust 1.70.0 or later
- Graphviz (optional, for rendering CFGs)

### Build and Test

To build the library:

```bash
$ cargo build
```

To run tests:

```bash
$ cargo test
```

To check formatting:

```bash
$ cargo fmt --all -- --check
```

To lint the code:

```bash
$ cargo clippy --workspace --all-targets -- -D warnings
```

### Documentation

Generate and view the documentation locally

```bash
$ cargo doc --no-deps --workspace
```

## Contributing

Contributions are welcome! If you’d like to contribute:

1. Fork the repository.
2. Create a new branch (`git checkout -b feature-branch`).
3. Commit your changes (`git commit -m "Add a new feature"`).
4. Push the branch (`git push origin feature-branch`).
5. Open a pull request.

Please ensure your code passes all tests and adheres to the project’s style guidelines.

## License

This project is licensed under the Mozilla Public License v2.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- **Graphviz** is used to make human-readable directed graphs for debugging purposes.
- **petgraph** is used to implement the directed graph functionality.

## Contact

If you have any questions or feedback, feel free to reach out via the repository's [issues](https://github.com/cernec1999/gbf-rs/issues) section.

---

Happy hacking!
