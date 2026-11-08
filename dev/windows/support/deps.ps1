#Requires -Version 7.4

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

$node_version = '24'
$winget_already_installed = 0x8A150061
$script:needs_restart = $false

# winget install, skipped when the command is already on PATH
function Install-IfMissing([string]$Command, [string]$WingetId) {
    if (Get-Command $Command -ErrorAction SilentlyContinue) { return }

    $PSNativeCommandUseErrorActionPreference = $false
    winget install --id $WingetId --exact --silent --no-upgrade --accept-package-agreements --accept-source-agreements
    if ($LASTEXITCODE -notin 0, $winget_already_installed) {
        throw "winget install $WingetId failed with exit code $LASTEXITCODE"
    }
}

Install-IfMissing nvm CoreyButler.NVMforWindows

nvm install $node_version
nvm use $node_version

npm install @openapitools/openapi-generator-cli -D

# openapi-generator-cli runs the generator jar with java
Install-IfMissing java Microsoft.OpenJDK.21

# act command
Install-IfMissing act nektos.act

if ($needs_restart) {
    Write-Warning 'PATH changed since this terminal was started: restart the terminal app (e.g. VS Code), not just the tab, before running other recipes'
}
