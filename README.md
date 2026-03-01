# MORRIS

**Memory Organization & Reactive Recursive Intent‑driven System**

`morris` is an intent‑driven shell and data environment written in Rust. Instead of imperative commands, you express **intents** such as:

- `set x = 5`
- `ensure total = price * quantity`
- `writeout(report)`
- `craft my-change { ... }` / `forge` / `smelt`

Morris parses these into a structured internal intent model, maintains a reactive variable graph, and uses a propagation and transaction engine to keep derived values consistent while giving you strong control over how changes are previewed, applied, and rolled back.

---

## Overview

Morris combines three main ideas:

1. **Intent language** – a domain‑specific language that describes *what* you want to achieve (set, ensure, derive, write, navigate, craft, forge, etc.).
2. **Reactive environment** – variables and expressions are tracked with explicit dependencies, allowing automatic propagation when inputs change.
3. **Transactional change engine** – a blacksmithing‑themed transaction system (`craft` / `forge` / `smelt` / `temper`) that lets you preview and safely apply multi‑variable changes.

The implementation is organized as a single binary crate (`morris`) with core logic under `src/core` and an interactive REPL plus `.msh` script runner in `src/main.rs`.

---

## Features

### 1. Intent‑driven command model

Morris defines an intent vocabulary that is parsed from natural, shell‑like commands into structured `Intent` objects. Intents are grouped by responsibility; the full DSL is documented in `intents.txt`.

#### Data and variable intents

- `set` – create or update variables (with optional type declaration and propagation controls).
- `ensure` – enforce values or invariants.
- `derive` – perform type‑aware conversions.
- `analyze` – inspect variable metadata and dependencies.
- `find` – search for variables.
- `freeze` – mark variables as constant.
- `collection`, `dictionary` – helpers for structured values.

#### File and I/O intents

- `read`, `write`, `append` – move data between files and variables.
- `mkdir`, `list`, `info`, `exists` – filesystem inspection and mutation.
- `save`, `load` – persist and restore environment snapshots.
- `writeout` – structured terminal output.

#### Navigation (book metaphor) intents

- `page` – show the current location.
- `turn`, `chapter` – move between directories/"pages".
- `bookmark`, `bookmarks`, `remove` – manage named locations.
- `volume`, `volumes` – define and list logical volumes.
- `jump`, `goto`, `return`, `back`, `peek`, `mark`, `shelve`, `unshelve`, `index`, `annotate`, `read_annotation`, `skim`, `library` – higher‑level navigation, history peeking, annotations, and quick previews.

#### History intents

- `history` – show recent intents.
- `history search`, `history tag`, `history replay`, `history clear`, `history save` – filter, tag, replay, and manage history.

#### Engine and change‑engine intents

- `engine status`, `engine save`, `engine load`, `engine validate` – manage the change engine state.
- `engine define`, `engine rule`, `engine hook` – define intents, rules, and hooks stored in the change engine.

#### Transaction intents

- `craft`, `forge`, `smelt`, `temper`, `inspect`, `anneal`, `quench`, `transaction` – transactional change management.
- Planned extensions (not all wired into the REPL yet): `polish`, `alloy`, `engrave`, `gild`, `patina`.

#### Analysis and what‑if intents

- `what-if` – specify hypothetical changes and an optional check expression.

#### JSON intents (experimental)

These verbs are **temporary, experimental, and unstable**. They exist to simplify JSON workflows while the expression engine and `set` support for structured data evolve. They may be removed or replaced by `set`/expression‑based JSON helpers in future versions.

- `parse-json`, `to-json`, `from-json` – convert between strings, values, and JSON documents.
- `json-get`, `json-set` – query and update JSON via JSON‑path‑like expressions.

#### Inspection and meta‑programming intents

- `examine` – inspect intents, variables, engine state, rules, or safety configuration.
- `define intent`, `construct`, `evolve`, `grow` – define new intents, evolve existing ones, and build compositions.
- `reflect`, `test`, `adopt` – meta‑level evaluation, testing, and promotion of intents.

Each `Intent` contains:

- A UUID and timestamp.
- A `Verb` and optional `Target` (`Variable`, `File`, `Expression`, `Service`, `Process`, `Port`).
- Parameters (`HashMap<String, String>`) and context (`HashMap<String, String>`).
- A lifecycle `IntentState` and execution metadata.
- Optional **composition** information for defined multi‑step intents:
  - `is_composition`, `composition_name`, `sub_intents`, `parameter_defs`, `execution_guard`, and `intent_source`.
- Additional **integrity and safety** metadata:
  - `IntentIntegrity` (content hash, creation/modification timestamps, origin, modification count) to detect tampering or drift.
  - `SafetyLevel` (`SystemCritical`, `CoreFunction`, `UserDefined`, `Experimental`) and `allowed_operations` (`Read`, `Execute`, `Modify`, `Extend`, `Introspect`) to express how each intent may be used and evolved.

This model allows Morris to go beyond a traditional shell: commands can be inspected, serialized, analyzed, composed, safety‑qualified, and replayed programmatically.

### 2. Reactive types and environment

The core data model is defined in `src/core/types.rs` and `src/core/env.rs`.

#### Value and Variable types

- `Value` supports structured data:
  - `Str(String)`, `Int(i64)`, `Float(f64)`, `Bool(bool)`, `List<Vec<Value>>`, `Dict<HashMap<String, Value>>`.
  - Utility methods: `type_name()`, `to_string()`, and `display()` for user‑friendly printing.
- `Variable` wraps a `Value` with metadata:
  - `is_constant` (for `freeze`‑style semantics).
  - Optional `expression` string (source of computed variables).
  - `source: VariableSource` (`Direct`, `Computed`, `Propagated`).
  - `last_updated: DateTime<Utc>` and `update_count` for observability.

This gives the environment enough structure to audit and visualize state, while staying flexible for higher‑level features.

#### Type declaration

Variables may optionally be declared with an explicit type. When a variable has a declared type, subsequent modifications are validated against that type; incompatible assignments are rejected, preserving the existing value and returning a clear error. If no type is declared, variables remain dynamically typed and follow the normal conversion rules of the expression engine. This allows you to tighten guarantees only where you need them, for example declaring configuration or boundary values as `int`, `float`, or `bool` while leaving exploratory variables flexible.

#### Reaction delay and limits (`~+n` / `~-n`)

Morris supports fine‑grained control over how often a dependent variable reacts to upstream changes:

- **Reaction limit (`~+n`)** – limits propagation from a dependency to at most `n` successful reactions. After `n` changes have propagated, the dependent variable becomes immune to further propagation from that source and stops auto‑updating. Manual writes are still allowed.
- **Reaction delay (`~-n`)** – delays propagation for the first `n` upstream changes. While the delay window is in effect, the dependent variable is immune to those changes. After `n` changes have been observed, subsequent changes start propagating normally.

These modifiers can be attached to reactive expressions to model effects such as "update this only a limited number of times" or "ignore the first few fluctuations, then start reacting" without giving up the benefits of the propagation engine.

#### Environment and dependency tracking

`src/core/env.rs` defines the `Env` struct, which is the in‑memory execution context:

- Maintains:
  - `variables: HashMap<String, Variable>`.
  - `expressions: HashMap<String, Expr>`.
  - `dependents` and `dependencies` maps for the variable graph.
  - A `PropagationEngine` for declarative change propagation.
  - A `TransactionEngine` for transactional changes.
- Provides operations to:
  - Create, update, and remove variables.
  - Register and evaluate expressions.
  - Track dependencies whenever expressions reference other variables.
  - Restore from snapshots and roll back on failure.

The environment is responsible for keeping the variable graph internally consistent, delegating evaluation to `expr` and applying structural rules from the propagation engine.

### 3. Propagation engine controls

Morris supports a legacy and a new propagation engine, both managed via the REPL in `src/main.rs` and the `Env` API:

- `engine on` – enable the new propagation engine with a chosen `PropagationStrategy`.
- `engine off` – revert to the legacy propagation behavior.
- `engine migrate` – migrate existing variables into the new engine representation.
- `engine visualize` – render a textual view of the dependency graph for debugging.
- `engine history` – inspect recent propagation events.
- `engine status` – report whether the new engine is enabled and how many variables are being tracked.

These operations are backed by the `PropagationEngine` type in `src/core/propagation` and additional helpers in `env.rs`.

### 4. Transaction and change management

The **transaction system** uses a blacksmithing metaphor and is implemented primarily in `src/core/env.rs` and `src/core/transaction`:

- `craft(name: Option<&str>)` – start a transaction, capturing a snapshot of all current variables.
- `forge()` – build a dependency‑ordered evaluation plan and apply changes atomically:
  - Direct value changes are applied first.
  - Expressions are evaluated next using the current environment.
  - On any failure, the environment is rolled back to the snapshot, and any newly‑created variables are removed.
- `smelt()` – discard the current transaction and restore variables from the snapshot without applying changes.
- `temper()` – produce a `TransactionPreview` to show which variables will change and how, without mutating the environment.
- `inspect_transaction()` – generate a detailed textual summary of the active transaction (ID, state, timestamps, number of changes, and per‑variable details).

This system is designed so that complex multi‑variable updates can be shaped, inspected, and either committed or abandoned with predictable behavior and clear reporting.

### 5. Persistent change engine

The **change engine** extends the transactional model with persistent metadata. It is implemented in `src/core/change_engine.rs`.

Key concepts:

- `ChangeEngine` holds:
  - Versioning and timestamps (`created`, `last_modified`).
  - `variables: HashMap<String, EngineVariable>` with `VariableMetadata` (description, units, confidence, optional validation timestamp, tags).
  - `computed_expressions: HashMap<String, ComputedExpression>` capturing expressions, dependencies, triggers, cached results, and validation rules.
  - `intent_definitions: HashMap<String, IntentDefinition>` for parameterized, documented intents.
  - `propagation_rules` and `hooks` for advanced automation.
  - Tags and annotations.
  - Session information (`SessionInfo`), including current and recent sessions.

- `ChangeEngineManager` is responsible for:
  - Locating the engine file under `~/.morris/change_engine.json`.
  - Initializing default content when no file exists.
  - Loading and saving engine state (with optional auto‑save behavior).

This layer enables higher‑level features such as documented reusable intents, rule‑based propagation, and richer auditability of system behavior across sessions.

### 6. History and observability

`src/core/history.rs` implements a structured **history subsystem**:

- `HistoryEntry` records:
  - Intent ID and timestamp.
  - The rendered intent string and verb.
  - Target, execution state, textual result.
  - Execution duration in milliseconds.
  - Arbitrary context and tags.
- `HistoryManager`:
  - Stores history in `~/.morris/history.json`.
  - Enforces a configurable maximum entry count.
  - Supports `record`, `load`, and `save` operations with safe write‑then‑rename semantics.
  - Exposes queries such as `search`, `get_last_n`, `get_by_id`, stats, tagging, clearing, and exporting.

The REPL exposes a `history` command that uses this manager to display recent commands and outcomes, and additional history‑related verbs are parsed through the intent system.

### 7. Interactive REPL

The interactive shell is implemented by `src/repl.rs` and orchestrated in `src/main.rs`.

- Uses `rustyline::DefaultEditor` to provide line editing, history, and basic keybindings.
- Stores REPL history in `~/.morris/repl_history.txt`, creating `~/.morris` on first run.
- Supports **single‑line** and **multi‑line** input:
  - A line ending with `{` enters block mode, collecting lines until a closing `}`.
- Handles control signals:
  - `Ctrl+C` clears the current line and returns to the prompt.
  - `Ctrl+D` exits the REPL.
- Provides built‑in commands handled before intent parsing:
  - `help` – print high‑level command help.
  - `env` – show the current environment state.
  - `history` – show history summary.
  - `clear` – clear the screen (Windows and Unix implementations).
  - Engine controls: `engine on`, `engine off`, `engine migrate`, `engine visualize`, `engine history`, `engine status`.
  - `exit` / `quit` – exit the REPL.

When input is not one of these built‑ins, it is passed to the intent parser (`parse_to_intent` in `src/core/intent.rs`) and executed against the environment, history manager, and change engine.

### 8. Script execution (`.msh` files)

In addition to the REPL, Morris can execute scripts containing intents:

- The binary entry point (`src/main.rs`) accepts an optional argument:
  - When invoked as `morris <file.msh>`, the file is validated to ensure it has a `.msh` extension.
  - Script execution is delegated to `execute_msh_file`, which uses the same intent parsing and environment machinery as the REPL.
- On successful execution, a success message is printed via `Printer`.
- On failure, an error message is printed and the process exits with a non‑zero status code.

This allows repeatable workflows, configuration pipelines, and batch operations to be expressed as intent scripts.

### 9. Output abstraction

`src/output.rs` provides a small but focused `Printer` abstraction used for all user‑facing messages:

- Basic color handling:
  - On non‑Windows, color is enabled when `TERM` is not `dumb`.
  - On Windows, colors are currently disabled to avoid terminal compatibility issues.
- Prefixed message helpers:
  - `success` – `[+]` prefix, green when available.
  - `error` – `[-]` prefix, red when available.
  - `warning` – `[!]` prefix, yellow when available.
  - `info` – `[?]` prefix, cyan when available.
  - `neutral` – `[•]` prefix.
- Formatting utilities:
  - `header`, `subheader`, `separator` for sectioning.
  - `print_key_value`, `print_list_item`, and `print_indented` for structured display.

Centralizing output ensures a consistent, legible UX across the REPL, script runner, history, and engine diagnostics.

---

## Project layout

Key files and directories:

- `Cargo.toml` – crate metadata and dependencies. Defines the `morris` binary with `src/main.rs` as the entry point.
- `Cargo.lock` – dependency lockfile.
- `src/main.rs` – CLI entry point, REPL loop, script execution, engine toggles, and wiring into core modules.
- `src/test_main.rs` – small test harness exercising basic intent parsing and evaluation.
- `src/output.rs` – terminal output abstraction.
- `src/repl.rs` – REPL implementation using `rustyline` and disk‑backed history.
- `src/core/` – core engine modules:
  - `mod.rs` – module declarations.
  - `types.rs` – core `Value` and `Variable` types.
  - `env.rs` – reactive environment, dependency graphs, transaction plumbing, engine toggles.
  - `expr.rs` – expression representation and evaluation.
  - `derive.rs`, `propagate.rs` – derivation and propagation helpers.
  - `intent.rs` – verbs, targets, intent state, and composition.
  - `filesystem.rs` – filesystem operations backing file‑related intents.
  - `builtins.rs` – built‑in commands and helpers.
  - `template.rs`, `library.rs` – templates and higher‑level intent library.
  - `history.rs` – persistent command history.
  - `change_engine.rs` – change engine model and manager.
  - `propagation/`, `transaction/` – specialized propagation and transaction logic.
- `src/output/` – sample output artifacts (e.g., `data.csv`, `pipeline_state.menv`, `processed.txt`).
- `examples/` – example `.msh` scripts and supporting files for configuration, finance, pipelines, and test scenarios.

---

## Installation

### Prerequisites

- Rust toolchain (Rust 1.70 or later is recommended).
- A terminal capable of basic ANSI output. Colors are best‑effort and can be disabled at the `Printer` layer for maximum compatibility.

### Build

From the project root:

```bash
cargo build
```

This produces the `morris` binary under `target/debug`.

### Run the REPL

From the project root:

```bash
cargo run
```

This starts an interactive session similar to:

```text
intent> 
```

You can then issue intents:

```text
intent> help
intent> set revenue = 1200
intent> set costs = 400
intent> ensure profit = revenue - costs
intent> env
intent> history
```

### Run a script

To execute an `.msh` script:

```bash
cargo run -- examples/test.msh
```

Or any other script in the `examples/` directory, for example:

```bash
cargo run -- examples/finance.msh
```

If the argument does not end with `.msh`, Morris prints a usage hint and exits without executing.

---

## Data files and persistence locations

Morris maintains per‑user state under the home directory:

- `~/.morris/repl_history.txt` – REPL command history.
- `~/.morris/history.json` – serialized `HistoryEntry` list.
- `~/.morris/change_engine.json` – serialized `ChangeEngine` state.

Example scripts and sample output live within the repository:

- `examples/` – `.msh` scripts for configuration, conversions, finance, and pipelines.
- `src/output/` – sample output such as `data.csv`, `pipeline_state.menv`, and `processed.txt`.

These locations and formats are designed to be inspectable by other tools and safe to back up or version‑control if desired.

---

## Status and roadmap

The codebase already implements a substantial portion of the intent model, environment, propagation, transaction engine, and persistence layers. Some verbs and features are marked as "coming soon" in the intent definitions (for example, `polish`, `alloy`, `engrave`, `gild`, `patina`, advanced expression constructs, and deeper template integration). Their presence in the model indicates planned capabilities even if they are not yet fully wired into the REPL.

Morris is an experimental environment and is not intended as a general‑purpose, day‑to‑day shell. It is best suited for exploratory modeling, rich stateful sessions, and workflows where explicit intent history, propagation, and transactions are valuable.

The design prioritizes:

- Clear modeling of intents and state.
- Predictable, inspectable propagation and transaction semantics.
- Structured history and change tracking suitable for long‑running, iterative workflows.

Contributions and extensions should preserve these properties and favor explicit, auditable behavior over hidden side effects.
