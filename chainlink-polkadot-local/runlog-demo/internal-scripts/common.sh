#!/bin/bash

LOG_PATH=./tmp/logs
mkdir -p "$LOG_PATH"

waitFor() {
  [ -z "$2" ] && timeout=60 || timeout=$2
  sleepCount=0
  while [ "$sleepCount" -le "$timeout" ] && ! eval "$1" >/dev/null; do
    sleep 1
    sleepCount=$((sleepCount + 1))
  done

  if [ "$sleepCount" -gt "$timeout" ]; then
    printf -- "\033[31mTimed out waiting for '%s' (waited %ss).\033[0m\n" "$1" "${timeout}"
    exit 1
  fi
}

waitForResponse() {
  title "Waiting for $1."
  waitFor "curl -s \"$1\""
  title "Service on $1 is ready."
}

launch_chainlink() {
  waitForResponse "$1"
  title "Chainlink node $1 is running."
}

login_cl() {
  CL_URL=$1

  username=""
  password=""

  while IFS= read -r line; do
    if [[ "$username" == "" ]]; then
      username=${line}
    else
      password=${line}
    fi
  done <"./secrets/apicredentials"

  echo "" >./tmp/cookiefile

  curl -s -c ./tmp/cookiefile -d "{\"email\":\"${username}\", \"password\":\"${password}\"}" -X POST -H 'Content-Type: application/json' "$CL_URL/sessions" &>/dev/null
}

run_ei() {
  title "Running External Initiator #$1..."

  EI_CI_ACCESSKEY=$2
  EI_CI_SECRET=$3
  EI_IC_ACCESSKEY=$4
  EI_IC_SECRET=$5

  if [ "$EI_CI_ACCESSKEY" != "" ]; then
    {
      echo "EI_CI_ACCESSKEY=$EI_CI_ACCESSKEY"
      echo "EI_CI_SECRET=$EI_CI_SECRET"
      echo "EI_IC_ACCESSKEY=$EI_IC_ACCESSKEY"
      echo "EI_IC_SECRET=$EI_IC_SECRET"
    } >"external_initiator$1.env"
  fi

  docker-compose up -d "external-initiator-node$1"
}

start_docker() {
  title "Starting Chainlink Docker containers"

  docker-compose up -d chain-runlog chainlink-node1 chainlink-node2 chainlink-node3 substrate-adapter1 substrate-adapter2 substrate-adapter3

  launch_chainlink "http://localhost:6691/"
  launch_chainlink "http://localhost:6692/"
  launch_chainlink "http://localhost:6693/"

  title "Done starting Chainlink Docker containers"
}

stop_docker() {
  title "Stopping Docker containers"

  docker-compose down

  title "Done stopping Docker containers"
}

build_docker() {
  title "Building Docker images"

  docker-compose build

  title "Done building Docker images"
}

reset_volumes() {
  title "Removing Docker volumes"

  docker volume rm runlog-demo_cl1
  docker volume rm runlog-demo_cl2
  docker volume rm runlog-demo_cl3
  docker volume rm runlog-demo_pg1
  docker volume rm runlog-demo_pg2
  docker volume rm runlog-demo_pg3

  title "Done removing Docker volumes"
}

print_logs() {
  for log in $(find "$LOG_PATH" -maxdepth 1 -type f -iname '*.log'); do
    heading "$log"
    cat "$log"
  done
}

exit_handler() {
  errno=$?
  # Print all the logs if the test fails
  if [ $errno -ne 0 ]; then
    title "ABORTING TEST"
    printf -- "Exited with code %s\n" "$errno"
    print_logs
  fi
  exit $errno
}

title() {
  printf -- "\033[34m%s\033[0m\n" "$1"
}

heading() {
  printf -- "\n--------------------------------------------------------------------------------\n"
  title "$1"
  printf -- "--------------------------------------------------------------------------------\n\n"
}

