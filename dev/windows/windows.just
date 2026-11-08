# Windows-specific settings/recipes (imported by top-level justfile)

# on Windows `bash` resolves to the WSL launcher, not Git Bash: use PowerShell 7 instead
[windows]
set shell := ["pwsh", "-NoLogo", "-NoProfile", "-Command"]

# envs

[windows]
deps:
    ./dev/windows/support/deps.ps1

[windows]
clean:
    if (Test-Path node_modules) { Remove-Item -Recurse -Force node_modules }
    cargo clean
