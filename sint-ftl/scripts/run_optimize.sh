#!/bin/bash

exec cargo run --bin optimize --release -- "$@"
