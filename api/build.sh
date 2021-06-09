#!/bin/bash 

# exit when any command fails
set -e

# keep track of the last executed command
trap 'last_command=$current_command; current_command=$BASH_COMMAND' DEBUG
# echo an error message before exiting
trap 'echo "\"${last_command}\" command filed with exit code $?."' EXIT

echo "Run cleanup step"
for executable in $(cargo metadata --no-deps --format-version=1 | jq -r '.packages[] | .targets[] | select(.kind[] | contains("bin")) | .name'); do
  executable_path="./target/lambda/release/${executable}"
  if [[ -f "${executable_path}" ]]; then
    echo "Remove ${executable_path}"
    rm "${executable_path}"
    rm "${executable_path}.debug"
  fi
done

echo "Run docker build project step"
docker run --rm \
  -e PROFILE=release \
  -e PACKAGE=false \
  -v ${PWD}:/code \
  -v ${HOME}/.cargo/registry:/root/.cargo/registry \
  -v ${HOME}/.cargo/git:/root/.cargo/git \
  aslamplr/lambda-rust

echo "Completed docker build project step"

echo "Run sam build project step"
sam build
echo "Completed sam build project step"

echo "Ready for sam deploy"
