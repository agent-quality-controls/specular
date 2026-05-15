# Goal

Compare what the same `spec3` source contract looks like in CUE, Pkl, and JSON.

This is scratch material, not a decision record.

# Shared Contract

The three examples express the same contract:

- schema version is `1`
- `tree` requires `README.md`, `Cargo.toml`, and `src/main.rs`
- `tree` forbids `tests` and `**/*_tests.rs`
- `text` forbids Rust test markers and `cargo test`
- future categories are present but empty
- one external verifier is declared

# CUE

```cue
package spec3

version: 1

requirements: {
	tree: [{
		id:     "TREE_ROOT"
		reason: "Rust CLI must expose the expected root files."
		required: {
			files: ["README.md", "Cargo.toml"]
			dirs: src: files: ["main.rs"]
		}
		forbidden: ["tests", "**/*_tests.rs"]
	}]
	text: [{
		id:     "NO_RUST_TESTS"
		reason: "This repo uses fixture behavior checks, not Rust tests."
		scope: ["**/*.rs", "Cargo.toml"]
		forbidden: ["#[test]", "#[cfg(test)]", "cargo test"]
	}]
	dependencies: []
	exports: []
	enumerations: []
	schemas: []
	fixtures: []
}

verifiers: [{
	id:    "g3rs"
	type:  "external"
	files: ["guardrail3-rs.toml"]
}]
```

# Pkl

```pkl
version = 1

requirements {
  tree = new Listing {
    new {
      id = "TREE_ROOT"
      reason = "Rust CLI must expose the expected root files."
      required {
        files = List("README.md", "Cargo.toml")
        dirs {
          ["src"] { files = List("main.rs") }
        }
      }
      forbidden = List("tests", "**/*_tests.rs")
    }
  }
  text = new Listing {
    new {
      id = "NO_RUST_TESTS"
      reason = "This repo uses fixture behavior checks, not Rust tests."
      scope = List("**/*.rs", "Cargo.toml")
      forbidden = List("#[test]", "#[cfg(test)]", "cargo test")
    }
  }
  dependencies = new Listing {}
  exports = new Listing {}
  enumerations = new Listing {}
  schemas = new Listing {}
  fixtures = new Listing {}
}

verifiers = new Listing {
  new { id = "g3rs"; type = "external"; files = List("guardrail3-rs.toml") }
}
```

# JSON

```json
{
  "version": 1,
  "requirements": {
    "tree": [
      {
        "id": "TREE_ROOT",
        "reason": "Rust CLI must expose the expected root files.",
        "required": {
          "files": ["README.md", "Cargo.toml"],
          "dirs": {
            "src": { "files": ["main.rs"] }
          }
        },
        "forbidden": ["tests", "**/*_tests.rs"]
      }
    ],
    "text": [
      {
        "id": "NO_RUST_TESTS",
        "reason": "This repo uses fixture behavior checks, not Rust tests.",
        "scope": ["**/*.rs", "Cargo.toml"],
        "forbidden": ["#[test]", "#[cfg(test)]", "cargo test"]
      }
    ],
    "dependencies": [],
    "exports": [],
    "enumerations": [],
    "schemas": [],
    "fixtures": []
  },
  "verifiers": [
    {
      "id": "g3rs",
      "type": "external",
      "files": ["guardrail3-rs.toml"]
    }
  ]
}
```

# Immediate Observations Before Tooling

## CUE

- The data shape is compact.
- Empty categories are natural.
- It can express schema and constraints in the same file if we want that later.
- The `dirs: src: files: ["main.rs"]` shorthand is concise but easy to misread.
- CUE uses unquoted field names by default, which makes examples shorter but adds syntax rules.

## Pkl

- The example is much harder to write from memory.
- Object/list syntax is less familiar.
- The `Listing` syntax is not obvious unless the Pkl schema is already defined.
- Pkl likely needs a module/schema file before source specs look clean.
- It may be strong after scaffolding, but it is not self-evident for a first raw spec.

## JSON

- It is verbose.
- It is easy for agents and tools.
- It has no comments.
- It cannot express constraints by itself.
- It needs external schema validation or Rust validation.
- It is the easiest interchange and lock-adjacent format.

# Tooling Validation

Tools installed during this comparison:

- `cue 0.16.1`
- `pkl 0.31.1`
- system `jq`

Validation commands:

```bash
cue export .scratch/source-format/spec.cue
pkl eval -f json .scratch/source-format/spec.pkl
jq . .scratch/source-format/spec.json
```

Results:

- CUE example exported valid JSON.
- Pkl example exported valid JSON.
- JSON example parsed with `jq`.

Mistakes found during this pass:

- No syntax mistakes in the CUE example.
- No syntax mistakes in the Pkl example, but the syntax was the least obvious to write.
- No syntax mistakes in the JSON example.

Important qualification:

- This validates only syntax and basic evaluation.
- It does not validate the `spec3` domain shape.
- Domain validation still needs either a schema/constraint layer or Rust typed validation.
