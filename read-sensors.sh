#!/usr/bin/env bash
set -u

SERIAL_DEVICE=${SERIAL_DEVICE:-/dev/ttyS0}
POLL_INTERVAL=${POLL_INTERVAL:-10}
RESPONSE_TIMEOUT=${RESPONSE_TIMEOUT:-2}
once=false
[[ ${1:-} == --once ]] && once=true

if [[ -t 1 ]]; then
    green=$'\033[32m'
    yellow=$'\033[33m'
    reset=$'\033[0m'
else
    green= yellow= reset=
fi

original_stty=$(stty -g -F "$SERIAL_DEVICE")
cleanup() {
    exec 3>&-
    stty -F "$SERIAL_DEVICE" "$original_stty" 2>/dev/null || true
}
trap cleanup EXIT
trap 'exit 130' INT
trap 'exit 143' TERM

stty -F "$SERIAL_DEVICE" 115200 cs8 -cstopb -parenb -crtscts clocal raw -echo
exec 3<>"$SERIAL_DEVICE"

query() {
    local command=$1 response
    while IFS= read -r -t 0.05 -u 3; do :; done
    printf '%s\r\n' "$command" >&3
    if IFS= read -r -t "$RESPONSE_TIMEOUT" -u 3 response; then
        printf '%s' "${response%$'\r'}"
    else
        printf '%s' 'Timeout'
    fi
}

fahrenheit() {
    awk -v c="$1" 'BEGIN { printf "%.1f", (c * 9 / 5) + 32 }'
}

print_soil() {
    local channel=$1 response=$2 moisture temperature
    if [[ $response =~ ^[0-9A-Za-z]([+-][0-9]+([.][0-9]+)?)([+-][0-9]+([.][0-9]+)?)$ ]]; then
        moisture=${BASH_REMATCH[1]#+}
        temperature=${BASH_REMATCH[3]}
        printf '%sSOIL%s moisture_raw=%s temperature_F=%s%s\n' "$green" "$channel" "$moisture" "$(fahrenheit "$temperature")" "$reset"
    else
        printf '%sSOIL%s ERROR: %s%s\n' "$yellow" "$channel" "$response" "$reset"
    fi
}

print_tc() {
    local channel=$1 response=$2
    if [[ $response =~ ^-?[0-9]+([.][0-9]+)?$ ]]; then
        printf '%sTC%s temperature_F=%s%s\n' "$green" "$channel" "$(fahrenheit "$response")" "$reset"
    else
        printf '%sTC%s ERROR: %s%s\n' "$yellow" "$channel" "$response" "$reset"
    fi
}

while true; do
    printf '%s\n' '========== SENSOR SAMPLE =========='
    for channel in 0 1 2; do
        print_soil "$channel" "$(query "RAW ${channel}R0!")"
    done
    for channel in 0 1; do
        print_tc "$channel" "$(query "TC ${channel}")"
    done
    printf '%s\n' '==================================='
    $once && break
    sleep "$POLL_INTERVAL"
done
