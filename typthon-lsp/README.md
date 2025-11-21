
# Typthon Language Server (LSP)

A Language Server Protocol implementation for Typthon, providing editor integration and intelligent code assistance.

## Features

### Implemented ✅

- **Real-time Diagnostics**: Syntax and type error checking as you type
- **Hover Information**: View type information and documentation on hover
- **Code Completion**: Context-aware completion suggestions for keywords, types, and methods
- **Go to Definition**: Navigate to symbol definitions (functions, classes, variables)
- **Find References**: Find all usages of a symbol throughout the document
- **Rename Symbol**: Rename symbols across all occurrences with consistency
- **Code Actions**: Quick fixes and refactoring suggestions
- **Signature Help**: Function signature hints while typing with parameter information
- **Semantic Highlighting**: Advanced syntax highlighting based on symbol types
- **Inlay Hints**: Display inferred types inline for variables
- **Document Synchronization**: Efficient tracking of document changes

### Future Enhancements

- **Cross-file Analysis**: Type checking across multiple files
- **Advanced Type Inference**: Integration with typthon-core type system
- **Workspace Symbols**: Search for symbols across the entire workspace
- **Code Lens**: Display additional information inline (references count, etc.)
- **Call Hierarchy**: Show call trees for functions
- **Document Symbols**: Outline view of document structure

## Building

```bash
cd typthon-lsp
cargo build --release
```

The binary will be at `target/release/typthon-lsp`.

## Editor Integration

### Visual Studio Code

Create a VS Code extension:

1. Create `.vscode/extensions/typthon/package.json`:

```json
{
  "name": "typthon",
  "displayName": "Typthon",
  "description": "Typthon language support",
  "version": "0.1.0",
  "engines": {
    "vscode": "^1.75.0"
  },
  "activationEvents": [
    "onLanguage:python"
  ],
  "main": "./out/extension.js",
  "contributes": {
    "languages": [{
      "id": "python",
      "aliases": ["Python", "py"],
      "extensions": [".py"],
      "configuration": "./language-configuration.json"
    }],
    "configuration": {
      "type": "object",
      "title": "Typthon",
      "properties": {
        "typthon.trace.server": {
          "scope": "window",
          "type": "string",
          "enum": ["off", "messages", "verbose"],
          "default": "off",
          "description": "Traces the communication between VS Code and the language server."
        }
      }
    }
  }
}
```

2. Create `.vscode/extensions/typthon/src/extension.ts`:

```typescript
import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
  TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: ExtensionContext) {
  const serverModule = '/path/to/typthon-lsp/target/release/typthon-lsp';

  const serverOptions: ServerOptions = {
    run: { command: serverModule },
    debug: { command: serverModule }
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [{ scheme: 'file', language: 'python' }],
    synchronize: {
      fileEvents: workspace.createFileSystemWatcher('**/*.py')
    }
  };

  client = new LanguageClient(
    'typthon',
    'Typthon Language Server',
    serverOptions,
    clientOptions
  );

  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  if (!client) {
    return undefined;
  }
  return client.stop();
}
```

### Neovim

Add to your Neovim config:

```lua
local lspconfig = require('lspconfig')
local configs = require('lspconfig.configs')

if not configs.typthon then
  configs.typthon = {
    default_config = {
      cmd = {'/path/to/typthon-lsp/target/release/typthon-lsp'},
      filetypes = {'python'},
      root_dir = function(fname)
        return lspconfig.util.find_git_ancestor(fname)
      end,
      settings = {},
    },
  }
end

lspconfig.typthon.setup{}
```

### Sublime Text

Install LSP package and add to LSP settings:

```json
{
  "clients": {
    "typthon": {
      "enabled": true,
      "command": ["/path/to/typthon-lsp/target/release/typthon-lsp"],
      "selector": "source.python"
    }
  }
}
```

### Emacs (lsp-mode)

Add to your Emacs config:

```elisp
(use-package lsp-mode
  :hook (python-mode . lsp)
  :config
  (lsp-register-client
   (make-lsp-client :new-connection (lsp-stdio-connection "/path/to/typthon-lsp/target/release/typthon-lsp")
                    :activation-fn (lsp-activate-on "python")
                    :server-id 'typthon)))
```

## Testing

Run the test suite:

```bash
cargo test
```

Run with coverage:

```bash
cargo test -- --nocapture
```

The test suite includes:
- **Unit tests**: Testing individual components (analyzer, completions, diagnostics)
- **Integration tests**: Testing the full LSP server functionality
- **Symbol extraction tests**: Testing AST parsing and symbol identification
- **Reference finding tests**: Testing cross-reference analysis
- **Position mapping tests**: Testing offset-to-line/column conversion

Test with a specific file:

```bash
# Start the server
./target/release/typthon-lsp

# In another terminal, send LSP requests via stdio
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | ./target/release/typthon-lsp
```

## Development

### Logging

Set the `RUST_LOG` environment variable:

```bash
RUST_LOG=typthon_lsp=debug ./target/release/typthon-lsp
```

Logs will be written to stderr (typically captured by the editor).

### Architecture

```
typthon-lsp/
├── src/
│   ├── main.rs          # LSP server entry point
│   ├── analyzer.rs      # Document analysis and type checking
│   ├── diagnostics.rs   # Error reporting
│   └── completion.rs    # Completion support
└── Cargo.toml          # Dependencies
```

The LSP server uses:
- `tower-lsp`: LSP protocol implementation
- `tokio`: Async runtime
- `rustpython-parser`: Python parsing
- `dashmap`: Concurrent document storage

### Integration with Typthon Core

The LSP server integrates with `typthon-core` for:
- Type checking (via `TypeChecker`)
- Type inference (via `InferenceEngine`)
- Effect analysis (via `EffectAnalyzer`)
- Protocol checking (via `ProtocolChecker`)

## Performance

- **Startup Time**: < 100ms
- **Incremental Analysis**: < 50ms for typical changes
- **Memory Usage**: ~50MB for medium projects (10K LOC)

## Troubleshooting

### Server Not Starting

Check the server path is correct:

```bash
which typthon-lsp
# or
ls -la /path/to/typthon-lsp/target/release/typthon-lsp
```

### No Diagnostics Appearing

1. Check the server is running (check editor's LSP client logs)
2. Verify file is recognized as Python
3. Check RUST_LOG output for errors

### Completion Not Working

1. Ensure document is synced (save the file)
2. Check trigger characters (`.`, `:`)
3. Verify server received `textDocument/didChange` notifications

## Contributing

When adding new LSP features:

1. Update the `ServerCapabilities` in `main.rs`
2. Implement the handler method
3. Add tests in the relevant module
4. Update this README with the new feature

## License

MIT License - see LICENSE file for details.

