# gbf-rs

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
gbf-rs = "0.1.0"
```

### Minimum Supported Rust Version
This project supports Rust 1.61.0 and later.

### Usage

### Decompile GS2 Bytecode

TODO: Add example code.

#### Generate Control Flow Graph (CFG)

TODO: Add example code.

#### Render CFGs with Graphviz

Save the DOT output to a file and render it using Graphviz:

```bash
dot -Tpng cfg.dot -o cfg.png
```

#### Build Abstract Syntax Trees (ASTs)

TODO: Add example code.

## Development

### Prerequisites

- Rust 1.61.0 or later
- Graphviz (optional, for rendering CFGs)

### Build and Test

To build the library:

```bash
cargo build
```

To run tests:

```bash
cargo test
```

To check formatting:

```bash
cargo fmt --all -- --check
```

To lint the code:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

### Documentation

Generate and view the documentation locally

```bash
cargo doc --no-deps --workspace
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

- **Graphviz** for CFG visualization.
- **Rust Analyzer** for excellent developer tooling.

## Contact

If you have any questions or feedback, feel free to reach out via the repository's [issues](https://github.com/cernec1999/gbf-rs/issues) section.

---

Happy hacking!
