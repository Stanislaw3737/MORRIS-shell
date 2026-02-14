# Morris Examples

This document provides concrete examples for the current Morris implementation. All examples can be run either interactively in the REPL (`cargo run`) or from `.msh` scripts in the `examples/` directory.

---

## 1. Basic variables and expressions

### 1.1 Scalars and arithmetic

```text
set x = 10
set y = 20
set sum = x + y
set ratio = x / y
writeout("x={x}, y={y}, sum={sum}, ratio={ratio}")
```

### 1.2 Lists and dictionaries

```text
set items = [1, 2, 3]
set user = {"name": "Alice", "role": "admin"}
writeout("items={items}, user={user}")
```

### 1.3 String interpolation and functions

```text
set name = "Alice"
set greeting = "Hello {name}!"              # template interpolation
set text = "Hello World"
set length = len(text)
set upper = upper(text)
writeout("{greeting} (len={length}, upper={upper})")
```

See `examples/test.msh` for more expression and conditional coverage.

---

## 2. Type declaration and stricter updates

Morris supports optional type declarations on variables. When a type is declared, incompatible updates are rejected.

```text
# Declare typed variables
set temperature:int = 25
set debug:bool = true

# Valid updates
set temperature = 30          # still an int
set debug = false             # still a bool

# Invalid update (rejected at runtime)
set temperature = "hot"      # type mismatch: string vs int
```

Type information is parsed from the left‑hand side (`name:type`) and enforced when values are evaluated.

---

## 3. Reaction delay and limits (`~+n` / `~-n`)

You can control how many times a reactive variable responds to upstream changes or delay its reaction.

### 3.1 Reaction limit (`~+n`)

```text
set source = 0
set limited = source ~+2      # reacts to at most 2 changes

ensure source = 1
ensure source = 2
ensure source = 3

writeout("source={source}, limited={limited}")
# `limited` will reflect the value after the second change, and ignore later updates.
```

### 3.2 Reaction delay (`~-n`)

```text
set counter = 0
set delayed = counter ~-2     # ignore first 2 changes, then start reacting

ensure counter = 1
ensure counter = 2
ensure counter = 3

writeout("counter={counter}, delayed={delayed}")
# `delayed` will remain at its initial value for the first 2 updates, then start tracking.
```

These suffixes are parsed from the value expression and stored as propagation delay/limit on the corresponding `set` intent.

---

## 4. Advanced conditionals and logic

The expression engine supports rich conditionals with `when`/`|`, comparisons, logical operators, and functions. See `examples/test.msh` and `examples/test2.msh`.

### 4.1 Multi‑branch conditional

```text
set x = 10
set y = 20
set z = 30

set result = "A" when x > 5 and y > 15 and z > 25 |
             "B" when x > 5 and y > 15 |
             "C" when x > 5 |
             "D"

writeout("result={result}")
```

### 4.2 Nested arithmetic and logic

```text
set result2 = "high"   when (x + y) > 25 and (y * 2) > z |
              "medium" when (x + y) > 15 or z > 20 |
              "low"

writeout("result2={result2}")
```

### 4.3 `not`, `and`, `or`

```text
set a = 10
set b = 20

set is_valid = true  when not (a < 0 or b < 0) | false
set status   = "ok"  when is_valid == true     | "bad"

writeout("is_valid={is_valid}, status={status}")
```

See `examples/test.msh` and `examples/data_pipeine.msh` for many more edge‑case tests.

---

## 5. Propagation and derived values

### 5.1 Unit conversions (`examples/convert.msh`)

```text
# Base unit
set meters = 1000

# Metric conversions
set kilometers  = meters / 1000
set centimeters = meters * 100
set millimeters = meters * 1000

# Imperial conversions
set feet  = meters * 3.28084
set inches = feet * 12
set yards  = feet / 3
set miles  = kilometers * 0.621371

writeout("=== Unit Conversions ===")
writeout("Base: {meters} meters")

# Change base and watch everything update
ensure meters = 500
writeout("Updated: {meters} m -> {kilometers} km, {feet} ft, {miles} mi")
```

All derived variables recompute automatically when `meters` changes.

### 5.2 Financial model (`examples/finance.msh`)

```text
# Revenue
set product_sales   = 50000
set service_revenue = 30000
set subscription    = 20000

# Expenses
set salaries       = 40000
set marketing      = 15000
set infrastructure = 8000
set miscellaneous  = 5000

# Aggregates
set total_revenue  = product_sales + service_revenue + subscription
set total_expenses = salaries + marketing + infrastructure + miscellaneous
set gross_profit   = total_revenue - total_expenses
set profit_margin  = gross_profit / total_revenue
```

Changing any source variable (`marketing`, `salaries`, etc.) automatically updates all dependent metrics.

---

## 6. Transactions: craft / temper / forge / smelt / anneal / quench

Morris lets you stage changes transactionally.

### 6.1 Basic transaction flow

```text
# Start crafting a change set
craft "data_processing"

# Shape changes inside the transaction
set raw_data  = "1,2,3,4,5"
set count     = len(raw_data.split(","))
set summary   = "Processed {count} items"

# Preview without applying
temper

# Commit atomically
forge
```

If evaluation fails or constraints are violated, `forge` rolls back to the snapshot and discards new variables.

### 6.2 Cancel and inspect

```text
craft "temp-experiment"
set tmp = 42
inspect           # inspect_transaction via Env
smelt             # discard crafted changes
```

### 6.3 Anneal and quench

```text
craft "multi-step"
set a = 1
set b = 2
set c = a + b

anneal 1          # apply first change only
anneal            # apply next change
quench            # apply all remaining changes immediately
```

---

## 7. Filesystem and book‑style navigation

Morris models navigation as a “book” of locations.

```text
# Where am I?
page

# Turn pages (change directory)
turn "projects/rust"
turn ..        # up one
turn -1        # back one page

# Chapters and bookmarks
chapter "documentation"           # alias for turn
bookmark add "work" "./workdir"
bookmarks                         # list all
bookmark remove "work"

# Volumes
volume add "projects" "./projects" "My projects"
volumes

# Shelving / unshelving
shelve
# ... wander around ...
unshelve

# Back and index
back 3
index

# Annotations
annotate "README.md" "Important doc"
read_annotation "README.md"

# Skim files
skim "large_file.txt"
```

See `examples/config_manager.msh` and `examples/convert.msh` for filesystem usage.

---

## 8. File I/O and configuration management

### 8.1 Reading and writing files

```text
# Write a message
write "./message.txt" "Hello World"

# Read JSON into a variable
read "./config.json" into settings

# Append to logs
append "./log.txt" "New entry" 

# Create directories and list contents
mkdir "./backups"
list "./backups"
```

### 8.2 Config manager example (`examples/config_manager.msh`)

```text
# Build a JSON config from pieces
set config_line1 = "{"
set config_line2 = '  "app_name": "Morris",'
set config_line3 = '  "version": "0.5.0",'
set config_line4 = '  "debug": true,'
set config_line5 = '  "max_connections": 100'
set config_line6 = "}"

set config = "{config_line1}\n{config_line2}\n{config_line3}\n{config_line4}\n{config_line5}\n{config_line6}"

write "./config.json" config

set timestamp   = now()
set backup_path = "./backups/config_" + timestamp + ".json"
mkdir "./backups"
write backup_path config
list "./backups"
```

---

## 9. JSON utilities and paths

Morris provides several JSON‑related verbs and methods.

### 9.1 Parsing and serializing JSON

```text
set json_str = '{"name": "Alice", "roles": ["admin", "user"]}'

# Parse JSON from string
set parsed = json_str.parse_json()

# Convert values back to JSON
set json_again = parsed.to_json()
```

### 9.2 JSON path operations

```text
# Using JSON verbs
from-json "{\"name\": \"Alice\", \"age\": 30}" into user
json-get user $.name
json-set user.name = "Bob"

# Using methods
set name   = parsed.get("$.name")
set roles  = parsed.get("$.roles")
set first  = roles.get(0)
set keys   = parsed.keys()
set values = parsed.values()
```

These map to `ParseJson`, `ToJson`, `FromJson`, `JsonGet`, and `JsonSet` intents and to the method logic implemented in `expr.rs` and `builtins.rs`.

---

## 10. History and what‑if analysis

### 10.1 History commands

```text
history                    # last commands
history search "set"       # filter by text
history save
history clear
```

### 10.2 What‑if scenarios

```text
# Predict impact of hypothetical changes
what-if x=100, y=200 check "x + y < 500"
what-if config.debug=true check safety
```

The `WhatIf` intent records the scenario variables and an optional `check_condition` expression.

---

## 11. Engine inspection and meta‑intents

### 11.1 Engine management

```text
engine status
engine save
engine load
engine validate
engine rule add when "profit_margin < 0.1" then "alert_low_margin"
engine hook on "transaction_forged" do "writeout(\"Forged\")"
```

### 11.2 Examine state

```text
examine intents
examine variables
examine engine
examine rules
examine safety
```

### 11.3 Defining and evolving intents

```text
# Define a parameterized intent
define intent "greet" with (name="world") { "Hello {name}!" }

# Construct and evolve intents (meta‑level)
construct intent "greet_user" with (user) { "Hello {user}!" }
evolve greet add_param "title" default="Dr."
grow greet_v2 from greet

# Reflect and test
reflect "set x = 1 + 2"
test greet with name="Alice"
adopt greet_v2
```

These map to `EngineDefine`, `Construct`, `Evolve`, `Grow`, `Reflect`, `Test`, and `Adopt` verbs and the associated parsers in `intent.rs`.

---

## 12. Running the examples

From the project root:

```bash
cargo run -- examples/convert.msh
cargo run -- examples/finance.msh
cargo run -- examples/config_manager.msh
cargo run -- examples/test.msh
cargo run -- examples/data_pipeine.msh
```

Or start the REPL and paste any of the snippets above:

```bash
cargo run
# then at the prompt
intent> set x = 10
intent> set y = 20
intent> set sum = x + y
intent> writeout("sum={sum}")
```