# Unix-specific (Linux, macOS) settings/recipes (imported by top-level justfile)

[unix]
set shell := ["bash", "-uc"]

# envs

[unix]
deps:
    ./dev/linux/support/deps.sh

[unix]
clean:
    rm -rf node_modules
    cargo clean
