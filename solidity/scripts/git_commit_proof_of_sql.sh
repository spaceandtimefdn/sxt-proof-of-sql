#!/bin/bash
set -euo pipefail
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
cd $SCRIPT_DIR/..
find . -type f -name "*.post.sol" -delete
scripts/install_deps.sh
scripts/pre_forge.sh clean
cd ./src

# Variables
REPO_URL="https://github.com/spaceandtimefdn/sxt-proof-of-sql"
CLONE_DIR="temp-proof-of-sql"
CLONED_PROOF_OF_SQL="$CLONE_DIR/sxt-proof-of-sql"
BRANCH_NAME="#2"
ERRORS_SOURCE_PATH="base/Errors.sol"
CONSTANTS_SOURCE_PATH="base/Constants.sol"
VERIFIER_SOURCE_PATH="verifier/Verifier.post.sol"
ERRORS_TARGET_PATH="solidity/src/base/Errors.sol"
CONSTANTS_TARGET_PATH="solidity/src/base/Constants.sol"
VERIFIER_TARGET_PATH="solidity/src/verifier/Verifier.post.sol"
COMMIT_MESSAGE="$1"

# Checkout to the desired branch

rm -rf $CLONE_DIR
mkdir $CLONE_DIR
cd $CLONE_DIR
git clone -b $BRANCH_NAME $REPO_URL
cd ../


# Copy the contents of the local files to the target files
cp $ERRORS_SOURCE_PATH $CLONED_PROOF_OF_SQL/$ERRORS_TARGET_PATH
cp $CONSTANTS_SOURCE_PATH $CLONED_PROOF_OF_SQL/$CONSTANTS_TARGET_PATH
cp $VERIFIER_SOURCE_PATH $CLONED_PROOF_OF_SQL/$VERIFIER_TARGET_PATH

cd $CLONED_PROOF_OF_SQL

# Add the modified files to the staging area
git add $ERRORS_TARGET_PATH $CONSTANTS_TARGET_PATH $VERIFIER_TARGET_PATH

# Commit the changes
git commit -m "$COMMIT_MESSAGE"

# Push the changes to the remote repository
git push origin $BRANCH_NAME

cd ../../

# Delete the cloned repository
rm -rf $CLONE_DIR
