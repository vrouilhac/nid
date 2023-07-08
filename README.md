# Note ID Generator (NID)

## DISCLAMER

This is a learning project

## Introduction

This is an id generator which generate custom id for note taking and save already taken id so it generate unique id every time.
It also has the ability to generate on the fly without saving the id.

## Commands

```
nid --help                  // show help

nid                         // basic generation

nid --save                  // save generation to avoid duplicate

nid --save --verbose        // shows human friendly text

nid --list                  // list all saved id

nid --list --verbose

nid --clip                  // Generate and save to clipboard
```

The id generated are `AAABB` with A = `[a-zA-Z]` and B = `[0-9]`
