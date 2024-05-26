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
  ./run.sh cleanup || true
  cd -
  teardown_home_dir
}


# ===== TESTS

@test "examples - database - mongodb docker" {
  cd examples/command/portals/databases/mongodb/docker
  ./run.sh >/dev/null &
  BGPID=$!
  trap 'kill $BGPID; exit' INT

  container_to_watch="analysis_corp-app-1"
  run_success wait_till_container_starts "$container_to_watch"

  exit_on_successful "$container_to_watch" &

  wait_till_successful_run_or_error "$container_to_watch"
  assert_equal "$exit_code" "0"
}