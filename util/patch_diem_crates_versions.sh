#!/bin/bash

# Script to aid with overriding the source for dependencies pulled from the diem repo.
# Supply an alternative github org in $1 that hosts a forked diem repo (or supply "0LNetworkCommunity" to target the main repo
# Supply a branch or tag name in $2
# e.g. to pull diem crates from github/com/acme-chains-r-us/diem, branch test-me use:
# <script-name> acme-chains-r-us test-me

# TODO: make this work for local path override too

cargo_file="Cargo.toml"
# Check we have a Cargo file to work with
if [[ ! -f ${cargo_file} ]]; then
    echo "Error: no ${cargo_file} file found in the current directory"
    exit 1
fi

# Check $1
if [[ -z ${1} ]]; then
    echo "Error: no github org supplied"
    exit 1
else
    github_org=$1
fi

# Check $2
if [[ -z ${2} ]]; then
    echo "Error: no branch supplied"
    exit 1
else
    branch=$2
fi

echo "Add the following lines to the end of Cargo.toml:"
echo "-------------------------------------------------"

echo "[patch.'https://github.com/0LNetworkCommunity/diem.git']"
grep "https://github.com/0LNetworkCommunity/diem.git" Cargo.toml | \
    grep -v '^\[patch' | sed s/0LNetworkCommunity/${github_org}/ | \
    sed 's/branch = "release"/branch = "'${branch}'"/'
