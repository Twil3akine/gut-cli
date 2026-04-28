# gut-cli

`gut-cli` is a tiny joke CLI for people who type `gut` when they meant `git`.

It prints a small goose:

```text
   _
__(.)<
/___)
 " "
```

Then it makes fun of your typo in English or Japanese.

The mascot and message style are configurable:

- `animation`: enable or disable the entrance animation
- `language`: switch between English and Japanese roast messages
- `character`: choose `goose`, `duck`, `owl`, or `random`

Available characters:

### goose

```text
   _
__(.)<
/___)
 " "
```

### duck

```text
    __
___( o)>
\ <_. )
 `---'
```

### owl

```text
 ,___,
 [O,O]
 /)__)
/--"-"
```

You can inspect the current settings:

```bash
gut config show
gut --config show
```

Animation is off by default, but you can enable a tiny entrance animation:

```bash
gut --config animation true
```

Disable it again with:

```bash
gut --config animation false
```

Switch the roast language:

```bash
gut --config language en
gut --config language ja
```

Switch the character:

```bash
gut --config character goose
gut --config character duck
gut --config character owl
gut --config character random
```

When `character=random`, `gut` picks `goose`, `duck`, or `owl` on each run.
Message selection is randomized independently from character selection.


## Install

### From source

```bash
cargo install --path .
```

### From crates.io

```bash
cargo install gut-cli
```

## Run

```bash
gut
```

## Why this exists

Because mistyping `git` deserves immediate consequences.
