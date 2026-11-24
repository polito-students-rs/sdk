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

[group("integration")]
[doc("run the integration tests against the mock (mock started, or reused if `just mock` runs); args: `--list` or submodule(s), e.g. `announcements`")]
integration *submodules:
    cargo xtask apply-overlays
    cargo test -p polito-client --test integration -- {{submodules}}

[group("e2e")]
[doc("run the end-to-end tests against the real API, read-only (prompts for credentials); args: `--list` or submodule(s), e.g. `service_link`")]
e2e *submodules:
    cargo test -p polito-client --test e2e -- {{submodules}}

publish-check:
    cargo package --list
    cargo publish --dry-run

build: check
    cargo build -r

# test ci pipelines locally
[group("ci")]
spec-sync:
    act -j sync -s GITHUB_TOKEN="$(gh auth token)" -W .github/workflows/spec-sync.yml