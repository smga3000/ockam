#!/bin/bash

# ===== SETUP

setup() {
  load ../load/base.bash
  load ../load/orchestrator.bash
  load ./setup.bash
  load_bats_ext
  setup_home_dir
  skip_if_orchestrator_tests_not_enabled
  copy_enrolled_home_dir
}

teardown() {
  echo "#==== $EXTRA_ARG" >&3
  ./run.sh cleanup $EXTRA_ARG || true
  unset EXTRA_ARG
  cd -
  teardown_home_dir
}

# pass
@test "examples - coderepos amazon ec2" {
  skip
  cd examples/command/portals/coderepos/gitlab/amazon_ec2/aws_cli
  run ./run.sh
  assert_output --partial "The example run was successful 🥳"$'\n'
}