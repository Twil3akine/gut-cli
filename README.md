# gut-cli

`gut-cli` is a tiny joke CLI for people who type `gut` when they meant `git`.

It prints a small goose:

```text
 _
__(.)<
/___)
 " "
```

Then it makes fun of your typo in English.

Animation is off by default, but you can enable a tiny entrance animation:

```bash
gut --config animation true
```

Disable it again with:

```bash
gut --config animation false
```

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
