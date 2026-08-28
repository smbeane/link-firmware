# Changelog

## Unreleased — STM32G031F6, UART, and sensor diagnostics

### `.cargo/config.toml`

- Kept the `STM32G031F6` probe-rs runner and added Cargo aliases for thermocouple, read-only SDI-12, UART-heartbeat, and guarded onsite SDI-12 address-change diagnostics.

### `Cargo.toml`

- Corrected the MCU feature from `stm32g031k8` to the installed `stm32g031f6` with its real 32 KiB flash limit.
- Made RTT/defmt optional and limited it to diagnostic binaries.
- Declared production and diagnostic binaries explicitly.
- Enabled size-oriented release settings (`opt-level = "z"`, LTO, and one codegen unit).

### `Cargo.lock`

- Updated dependency resolution after making diagnostic logging optional and changing Cargo target configuration.

### `build.rs`

- Links `defmt.x` only for the `diagnostics` feature, keeping RTT metadata out of production firmware.

### `src/main.rs`

- Preserved command-driven UART operation, sensor handling, and the independent watchdog task.
- Removed production RTT logging to fit the STM32G031F6 flash.
- Selected `LPUART1` for PA2/PA3, matching the STM32 pin configuration metadata and the hardware `.ioc` configuration.

### `src/sdi12.rs`

- Removed unconditional defmt logging and formatting from the production SDI-12 driver to reduce flash use without changing its wire protocol.

### `src/bin/tc-diagnostic.rs`

- Added a reusable ST-Link/RTT thermocouple diagnostic. Both installed MAX31856 channels returned valid values with fault byte `0x00` during testing.

### `src/bin/sdi12-diagnostic.rs`

- Added a read-only ST-Link/RTT SDI-12 scan and measurement diagnostic.
- Testing confirmed an address-0 response and soil data, but responses interleaved or failed parity when both sensors were connected. This is consistent with two sensors sharing address 0 or delayed responses overlapping.
- This diagnostic never sends an address-change command.

### `src/bin/uart-diagnostic.rs`

- Added a once-per-second `STM32_UART_DIAGNOSTIC` heartbeat on LPUART1 PA2 at 115200 8N1.
- The heartbeat was not received on Linux `/dev/ttyS0`, `/dev/ttyS4`, `/dev/ttyS5`, or `/dev/ttyS6`. This isolates the remaining fault to LattePanda UART routing/mapping or the physical connection, rather than firmware command parsing or sensor code.

### `src/bin/sdi12-set-address-0-to-1.rs`

- Added an onsite-only ST-Link utility that sends `0A1!` and verifies address `1`.
- **Only run this binary while exactly one address-0 soil sensor is physically connected.** If both are connected, both can change to address 1 and remain in collision.

### `read-sensors.sh`

- Added automatic 10-second UART polling for three soil addresses and two thermocouples.
- Added `--once`, response timeouts, Celsius-to-Fahrenheit conversion, colored status, and cleanup traps that close the serial descriptor and restore terminal settings on normal exit, errors, SIGINT, or SIGTERM.

## Known hardware and deployment issues

- Normal production operation still requires a working Linux UART path. The STM32 heartbeat test produced no data on any exposed LattePanda tty device.
- A BIOS UART-mapping update may resolve the internal LattePanda route, but it was intentionally not attempted remotely because a failed update or reboot could make the site unreachable.
- A direct 3.3 V USB-to-UART adapter connected to STM32 PA2, PA3, and common ground is the lowest-risk UART workaround.
- Two SDI-12 sensors with the same address cannot be read independently or safely re-addressed while connected to the same bus. Physically isolate one sensor before changing its address.
