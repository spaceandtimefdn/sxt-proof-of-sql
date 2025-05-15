set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
find . -type f -name "*.post.sol" -delete
scripts/install_deps.sh
scripts/pre_forge.sh clean
cd ./src
rm -rf solidity_proof_of_sql
forge init solidity_proof_of_sql
rm -rf ./solidity_proof_of_sql/test/*
rm -rf ./solidity_proof_of_sql/script
rm -rf ./solidity_proof_of_sql/.github
rm -rf ./solidity_proof_of_sql/lib/*
rm -rf ./solidity_proof_of_sql/src/*
rm ./solidity_proof_of_sql/.gitignore
mkdir -p ./solidity_proof_of_sql/src/verifier
mkdir -p ./solidity_proof_of_sql/src/base
cp ./base/Constants.sol ./solidity_proof_of_sql/src/base/Constants.sol
cp ./base/Errors.sol ./solidity_proof_of_sql/src/base/Errors.sol
cp ./verifier/Verifier.post.sol ./solidity_proof_of_sql/src/verifier/Verifier.post.sol
