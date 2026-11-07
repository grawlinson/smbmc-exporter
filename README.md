# smbmc-exporter

[![crates.io](https://img.shields.io/crates/v/smbmc-exporter.svg)](https://crates.io/crates/smbmc-exporter)
[![PyPI](https://img.shields.io/pypi/v/smbmc-exporter.svg)](https://pypi.org/project/smbmc-exporter/)

Prometheus exporter for Supermicro BMC metrics, built on
[smbmc](https://github.com/grawlinson/smbmc).

## Installation

Any of:

```sh
cargo install smbmc-exporter
uv tool install smbmc-exporter      # or: pipx install smbmc-exporter
```

Static Linux binaries (x86_64 and aarch64) are attached to each
[GitHub release](https://github.com/grawlinson/smbmc-exporter/releases).

## Usage

Options can be passed as flags or environment variables.

| Flag              | Environment variable  | Default   | Description                                                     |
|-------------------|-----------------------|-----------|-----------------------------------------------------------------|
| `--hostname`      | `SMBMC_HOSTNAME`      |           | Address of the BMC web interface; `https://` if no scheme given |
| `--username`      | `SMBMC_USERNAME`      |           | Username for the BMC web interface                              |
| `--password`      | `SMBMC_PASSWORD`      |           | Password for the BMC web interface                              |
| `--password-file` | `SMBMC_PASSWORD_FILE` |           | Read the password from a file instead                           |
| `--insecure`      | `SMBMC_INSECURE`      | `false`   | Accept the BMC's self-signed TLS certificate                    |
| `--timeout`       | `SMBMC_TIMEOUT`       | `30`      | Timeout for each request to the BMC, in seconds                 |
| `--listen-port`   | `LISTEN_PORT`         | `8000`    | Port for the daemon to listen on                                |
| `--listen-addr`   | `LISTEN_ADDR`         | `0.0.0.0` | Address for the daemon to listen on                             |

```sh
SMBMC_PASSWORD=secret smbmc-exporter --hostname https://192.168.1.1 --username ADMIN --insecure
curl http://localhost:8000/metrics
```

The BMC is queried on every scrape, and overlapping scrapes are serialised so
a slow BMC isn't overwhelmed. Log verbosity is controlled with `RUST_LOG`
(default `info`).

## Metrics

Metrics are served in the OpenMetrics text format.

| Metric                                         | Labels             | Description                                                  |
|------------------------------------------------|--------------------|--------------------------------------------------------------|
| `smbmc_up`                                     |                    | `1` if the last scrape of the BMC succeeded                  |
| `smbmc_collector_collect_seconds`              |                    | Histogram of time spent collecting from the BMC              |
| `smbmc_sensor_status`                          | `id`, `name`       | `1` if the sensor is present (and, for PSUs, not faulted)    |
| `smbmc_sensor_temperature_celsius`             | `id`, `name`       | Temperature reading                                          |
| `smbmc_sensor_threshold_temperature_celsius`   | `id`, `type`       | Temperature thresholds (`LNR`, `LC`, `LNC`, `UNC`, `UC`, `UNR`) |
| `smbmc_sensor_voltage_volts`                   | `id`, `name`       | Voltage reading                                              |
| `smbmc_sensor_threshold_voltage_volts`         | `id`, `type`       | Voltage thresholds                                           |
| `smbmc_sensor_fan_speed_rpm`                   | `id`, `name`       | Fan speed                                                    |
| `smbmc_sensor_threshold_fan_speed_rpm`         | `id`, `type`       | Fan speed thresholds                                         |
| `smbmc_sensor_power_supply_flag`               | `id`, `name`, `flag` | Power supply sensor event flags                            |
| `smbmc_pmbus_status`                           | `id`               | `1` if the PSU slot is populated and enabled                 |
| `smbmc_pmbus_info`                             | `id`, `name`       | Always `1`; `name` is the PSU serial number                  |
| `smbmc_pmbus_voltage_volts`                    | `id`, `type`       | PSU voltage (`Input (AC)`, `Output (DC)`)                    |
| `smbmc_pmbus_current_amperes`                  | `id`, `type`       | PSU current (`Input`, `Output`)                              |
| `smbmc_pmbus_power_watts`                      | `id`, `type`       | PSU power (`Input`, `Output`)                                |
| `smbmc_pmbus_temperature_celsius`              | `id`, `type`       | PSU temperatures (`Input`, `Output`)                         |
| `smbmc_pmbus_fan_speed_rpm`                    | `id`, `type`       | PSU fan speeds (`Input`, `Output`)                           |

See [`tests/fixtures/metrics.txt`](tests/fixtures/metrics.txt) for example
output.

### Changes from 0.2 (Python)

Metric names and labels are unchanged, so existing dashboards keep working.
The behavioural differences are:

- Sensors the BMC reports as not present, and empty PSU slots, no longer
  export meaningless zero readings and thresholds; they still appear in
  `smbmc_sensor_status` / `smbmc_pmbus_status` with a value of `0`.
- A power supply sensor's status is `1` when presence is detected and no fault
  flag is set; previously any other flag (e.g. standby) also reported `0`.
- A failed scrape of the BMC returns `smbmc_up 0` instead of an HTTP error.
- `smbmc_collector_collect_seconds` is a histogram rather than a summary
  (`_sum` and `_count` are unchanged).
- New: `smbmc_up`, `smbmc_pmbus_info`, `smbmc_sensor_power_supply_flag`,
  `--password-file`, `--insecure`, `--timeout`.
- `--hostname` accepts a bare host (`https://` is assumed) and is also
  available as `--server`.

## License

GPL-3.0-only
