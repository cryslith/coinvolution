#!/bin/bash

set -euo pipefail

tsc
cp -t target src/*.html
