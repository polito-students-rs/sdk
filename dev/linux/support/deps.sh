#!/usr/bin/env bash

set -euo pipefail

nvm_version="v0.40.5"
export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"

if [ ! -s "$NVM_DIR/nvm.sh" ]; then
  curl -fsSL "https://raw.githubusercontent.com/nvm-sh/nvm/$nvm_version/install.sh" | bash
fi

\. "$NVM_DIR/nvm.sh"

nvm install 24
nvm use 24

npm install @openapitools/openapi-generator-cli -D


# act command
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash