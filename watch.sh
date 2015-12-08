#!/usr/bin/env bash
nodemon -e rs -w src -w tests -x 'cargo test ; cargo test'
