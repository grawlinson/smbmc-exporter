smbmc-exporter
==============

Prometheus exporter for `smbmc <https://github.com/grawlinson/smbmc>`_ metrics.

Usage
-----

Variables can be passed to the command-line interface via flags or environment variables.

- ``--hostname``/``SMBMC_HOSTNAME``: Hostname of the SMBMC web-interface.
- ``--username``/``SMBMC_USERNAME``: Username for the SMBMC web-interface.
- ``--password``/``SMBMC_PASSWORD``: Password for the SMBMC web-interface.
- ``--listen-port``/``LISTEN_PORT``: Port for daemon to listen on.
- ``--listen-addr``/``LISTEN_ADDR``: Address for daemon to listen on.

Example Metrics
---------------

::

    # HELP smbmc_collector_collect_seconds Time spent to collect metrics from SMBMC
    # TYPE smbmc_collector_collect_seconds summary
    smbmc_collector_collect_seconds_count 3.0
    smbmc_collector_collect_seconds_sum 1.0082290172576904
    # HELP smbmc_collector_collect_seconds_created Time spent to collect metrics from SMBMC
    # TYPE smbmc_collector_collect_seconds_created gauge
    smbmc_collector_collect_seconds_created 1.6049614358040833e+09
    # HELP smbmc_sensor_temperature_celsius Sensor Temperature
    # TYPE smbmc_sensor_temperature_celsius gauge
    smbmc_sensor_temperature_celsius{id="0",name="System Temp"} 27.0
    smbmc_sensor_temperature_celsius{id="19",name="SAS2 FTemp1"} 29.0
    # HELP smbmc_sensor_threshold_temperature_celsius Sensor Threshold - Temperature
    # TYPE smbmc_sensor_threshold_temperature_celsius gauge
    smbmc_sensor_threshold_temperature_celsius{id="0",type="LNR"} -9.0
    smbmc_sensor_threshold_temperature_celsius{id="0",type="LC"} -7.0
    smbmc_sensor_threshold_temperature_celsius{id="0",type="LNC"} -5.0
    smbmc_sensor_threshold_temperature_celsius{id="0",type="UNC"} 80.0
    smbmc_sensor_threshold_temperature_celsius{id="0",type="UC"} 85.0
    smbmc_sensor_threshold_temperature_celsius{id="0",type="UNR"} 90.0
    # HELP smbmc_sensor_voltage_volts Sensor Voltage
    # TYPE smbmc_sensor_voltage_volts gauge
    smbmc_sensor_voltage_volts{id="1",name="12VCC"} 12.192
    smbmc_sensor_voltage_volts{id="2",name="5VCC"} 5.027
    smbmc_sensor_voltage_volts{id="3",name="3.3VCC"} 3.333
    smbmc_sensor_voltage_volts{id="4",name="5VSBY"} 4.974
    smbmc_sensor_voltage_volts{id="5",name="3.3VSBY"} 3.248
    smbmc_sensor_voltage_volts{id="6",name="1.2VSB"} 1.251
    smbmc_sensor_voltage_volts{id="7",name="1.5VSB"} 1.56
    smbmc_sensor_voltage_volts{id="8",name="VBAT"} 3.112
    smbmc_sensor_voltage_volts{id="23",name="SAS2 F5V"} 5.0
    smbmc_sensor_voltage_volts{id="24",name="SAS2 F12V"} 12.0
    # HELP smbmc_sensor_threshold_voltage_volts Sensor Threshold - Voltage
    # TYPE smbmc_sensor_threshold_voltage_volts gauge
    smbmc_sensor_threshold_voltage_volts{id="1",type="LNR"} 10.144
    smbmc_sensor_threshold_voltage_volts{id="1",type="LC"} 10.272
    smbmc_sensor_threshold_voltage_volts{id="1",type="LNC"} 10.784
    smbmc_sensor_threshold_voltage_volts{id="1",type="UNC"} 12.96
    smbmc_sensor_threshold_voltage_volts{id="1",type="UC"} 13.28
    smbmc_sensor_threshold_voltage_volts{id="1",type="UNR"} 13.408
    # HELP smbmc_sensor_fan_speed_rpm Fan Speed
    # TYPE smbmc_sensor_fan_speed_rpm gauge
    smbmc_sensor_fan_speed_rpm{id="9",name="FAN1"} 3500.0
    # HELP smbmc_sensor_threshold_fan_speed_rpm Fan Speed - Threshold
    # TYPE smbmc_sensor_threshold_fan_speed_rpm gauge
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="LNR"} 400.0
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="LC"} 600.0
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="LNC"} 800.0
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="UNC"} 25300.0
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="UC"} 25400.0
    smbmc_sensor_threshold_fan_speed_rpm{id="9",type="UNR"} 25500.0
    # HELP smbmc_sensor_status Sensor Status
    # TYPE smbmc_sensor_status gauge
    smbmc_sensor_status{id="0",name="System Temp"} 1.0
    smbmc_sensor_status{id="1",name="12VCC"} 1.0
    smbmc_sensor_status{id="9",name="FAN1"} 1.0
    smbmc_sensor_status{id="10",name="FAN2"} 0.0
    smbmc_sensor_status{id="19",name="SAS2 FTemp1"} 1.0
    smbmc_sensor_status{id="23",name="SAS2 F5V"} 1.0
    smbmc_sensor_status{id="24",name="SAS2 F12V"} 1.0
    smbmc_sensor_status{id="27",name="PS2 Status"} 1.0
    # HELP smbmc_pmbus_status Power Supply status
    # TYPE smbmc_pmbus_status gauge
    smbmc_pmbus_status{id="0"} 0.0
    smbmc_pmbus_status{id="1"} 1.0
    # HELP smbmc_pmbus_voltage_volts Power Supply Voltage
    # TYPE smbmc_pmbus_voltage_volts gauge
    smbmc_pmbus_voltage_volts{id="1",type="Input (AC)"} 242.0
    smbmc_pmbus_voltage_volts{id="1",type="Output (DC)"} 12.1
    # HELP smbmc_pmbus_current_amperes Power Supply Current
    # TYPE smbmc_pmbus_current_amperes gauge
    smbmc_pmbus_current_amperes{id="1",type="Input"} 0.359
    smbmc_pmbus_current_amperes{id="1",type="Output"} 5.75
    # HELP smbmc_pmbus_power_watts Power Supply Consumption
    # TYPE smbmc_pmbus_power_watts gauge
    smbmc_pmbus_power_watts{id="1",type="Input"} 86.0
    smbmc_pmbus_power_watts{id="1",type="Output"} 69.0
    # HELP smbmc_pmbus_temperature_celsius Power Supply Temperature
    # TYPE smbmc_pmbus_temperature_celsius gauge
    smbmc_pmbus_temperature_celsius{id="1",type="Input"} 37.0
    smbmc_pmbus_temperature_celsius{id="1",type="Output"} 51.0
    # HELP smbmc_pmbus_fan_speed_rpm Power Supply Fan Speed
    # TYPE smbmc_pmbus_fan_speed_rpm gauge
    smbmc_pmbus_fan_speed_rpm{id="1",type="Input"} 2858.0
    smbmc_pmbus_fan_speed_rpm{id="1",type="Output"} 3847.0