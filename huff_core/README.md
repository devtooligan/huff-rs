## Huff Core

Core [Compiler](struct.Compiler.html) for the [Huff Language](https://huff.sh).

#### Usage

Compiling source code with the [Compiler](struct.Compiler.html) is very straightforward.

Once you instantiate a [Compiler](struct.Compiler.html) (WLOG, `compiler`) with the file source, you can generate the compiled artifacts by simply running:

```rust,ignore
let artifacts: Result<Vec<Artifact>, CompilerError<'_>> = compiler.execute();
```

Below we demonstrate taking a source file `../huff-examples/erc20/contracts/ERC20.huff`, and generating the copmiled artifacts.

```rust
use huff_core::Compiler;
use huff_utils::error::CompilerError;
use huff_utils::artifact::Artifact;
use std::sync::Arc;

// Instantiate the Compiler Instance
let mut compiler = Compiler::new(Arc::new(vec!["../huff-examples/erc20/contracts/ERC20.huff".to_string()]), None, None, false);

// Execute the compiler
let res: Result<Vec<Arc<Artifact>>, Arc<CompilerError<'_>>> = compiler.execute();
assert!(res.is_ok());
```

The [Compiler](struct.Compiler.html) is easily configurable upon instantiation.




#### Inner Workings

The [Compiler](struct.Compiler.html) is composed of several compilation phases and bundles them together in one process.

```txt

[Files] -> Lexer -> Parser -> Codegen -> [Bytecode]

```

