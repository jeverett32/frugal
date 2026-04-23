#!/usr/bin/env bash

export APP_ENV=dev
source .env

build() {
  cargo build
}

echo start
