#!/bin/bash

# ===== SETUP

setup() {
  load ../load/base.bash
  load ../load/orchestrator.bash
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

wait_till_container_starts() {
  container_to_listen_to="$1"
  timeout 250s bash <<EOT
    while true; do
      sleep 2
      docker logs "$container_to_listen_to" >/dev/null || continue
      break
    done
EOT
}

wait_till_successful_run_or_error() {
  container_to_listen_to="$1"
  # Wait till consumer exits and grab the exit code
  consumer_exit_code=$(docker wait "$container_to_listen_to")

  if [ "$consumer_exit_code" -eq 137 ] ; then
    exit_code=0
    return
  fi

  exit_code=$consumer_exit_code
}

exit_on_successful() {
  container_to_listen_to="$1"
  while true; do
    logs=$(docker logs "$container_to_listen_to")
    if [[ "$logs" == *"The example run was successful 🥳"$'\n'* ]]; then
      docker stop "$container_to_listen_to"
      return
    fi
    sleep 1
  done
}

# ===== TESTS

@test "examples - kafka - apache docker" {
  cd examples/command/portals/kafka/apache/docker
  ./run.sh >/dev/null &
  BGPID=$!
  trap 'kill $BGPID; exit' INT

  run_success wait_till_container_starts "application_team-consumer-1"

  exit_on_successful "application_team-consumer-1" &

  wait_till_successful_run_or_error "application_team-consumer-1"
  assert_equal "$exit_code" "0"
}

@test "examples - kafka - redpanda docker" {
  cd examples/command/portals/kafka/apache/docker
  ./run.sh >/dev/null &
  BGPID=$!
  trap 'kill $BGPID; exit' INT

  run_success wait_till_container_starts "application_team-consumer-1"

  exit_on_successful "application_team-consumer-1" &

  wait_till_successful_run_or_error "application_team-consumer-1"
  assert_equal "$exit_code" "0"
}
