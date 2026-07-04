sudo apt update -y && sudo apt upgrade -y

# node.js + npm installation
curl -o- https://fnm.vercel.app/install | bash
\. "$HOME/.nvm/nvm.sh"
nvm install 24

# install openapi
npm install @openapitools/openapi-generator-cli