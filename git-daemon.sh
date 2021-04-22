#!/bin/bash

# This scripts launches a git daemon which exposes this project at
# `git://localhost:9418/mpc_kpi`
# 
# It is useful to locally test modifications using the UI on the CipherCompute EAP version
# 1. Make a change and test it using `./simulate.sh`
# 2. commit the change to the local git and note the git commit
# 3. Lauhch this script
# 4. Create/update a computation using the git URL above and the commit you want to test
# 5. Run the computation fron the UI

echo "git daemon starting on port 9418...ctrl+c to stop"
git daemon --reuseaddr --base-path=$(pwd)/.. --export-all --verbose $(pwd)/..
echo "git daemon stopped"