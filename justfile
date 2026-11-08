import 'dev/linux/linux.just'
import 'dev/windows/windows.just'

# generate crates/polito-api from spec/openapi.yaml (uses local node_modules)
[group("gen")]
codegen:
    cargo xtask sync-version
    npm exec --no -- openapi-generator-cli batch --fail-fast "target/codegen/*.yml"

check:
    cargo clippy --all-targets -- -D warnings
    cargo test

publish-check:
    cargo package --list
    cargo publish --dry-run

build: check
    cargo build -r

# test ci pipelines locally
[group("ci")]
spec-sync:
    act -j sync -s GITHUB_TOKEN="$(gh auth token)" -W .github/workflows/spec-sync.yml