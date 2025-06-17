# zed-harper

Zed extension for the
[Harper Grammar Checker](https://github.com/elijah-potter/harper) language server.

## Supported platforms

| Platform | X86_64 | ARM64 |
| -------- | ------ | ----- |
| Linux    | ✅     | ✅    |
| MacOS    | ✅     | ✅    |
| Windows  | ✅     | ❌    |

## Installation

1. [Open the Extension Gallery](https://zed.dev/docs/extensions/installing-extensions)
2. Search for `harper-ls` in the Gallery
3. Click `Install`!

## Configuration

You can [configure](https://zed.dev/docs/configuring-zed#lsp) `harper-ls` language server by adding the following to your Zed's `lsp` section in `settings.json`:

```json
{
  "lsp": {
    "harper-ls": {
      "binary": {
        "path": "harper-ls",
        "arguments": ["--stdio"]
      },
      "settings": {
        "harper-ls": {
          "diagnosticSeverity": "warning",
          "dialect": "Canadian",
          "codeActions": {
            "ForceStable": true
          },
          "markdown": {
            "IgnoreLinkTitle": true
          },
          "linters": {
            "SpellCheck": true
          }
        }
      }
    }
  }
}
```

Other possible configuration options can be found in the [Harper LS documentation](https://writewithharper.com/docs/integrations/language-server#Configuration).
