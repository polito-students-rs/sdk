import 'dev/linux/linux.just'
import 'dev/windows/windows.just'

# generate crates/polito-api from spec/openapi.yaml + spec/overlays (uses local node_modules)
[group("gen")]
codegen:
    cargo xtask sync-version
    cargo xtask apply-overlays
    npm install
    npm exec --no -- openapi-generator-cli batch --fail-fast "target/codegen/*.yml"

check:
    cargo clippy --all-targets -- -D warnings
    cargo test

[group("mock")]
[doc("run the mock integration tests (mock started, or reused if `just mock` runs); args: `--list` or submodule(s), e.g. `announcements`")]
test-mock *submodules:
    cargo test -p polito-client --test mock -- {{submodules}}

publish-check:
    cargo package --list
    cargo publish --dry-run

build: check
    cargo build -r

# test ci pipelines locally
[group("ci")]
spec-sync:
    act -j sync -s GITHUB_TOKEN="$(gh auth token)" -W .github/workflows/spec-sync.yml