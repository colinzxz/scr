# Coverage

The parser is tested against [sass-spec] for conformance.

Clone the test files beforehand

```bash
git submodule update --init --recursive --remote

```

## Development

```bash
# full run
cargo coverage
cargo coverage sass # for sass-spec

# run in watch
cargo watch -x 'coverage sass'

# filter for a file path
cargo watch -x 'coverage sass --filter <filter-file-path>'
```

[sass-spec]: https://github.com/sass/sass-spec
