"""Collector for SMBMC metrics."""
import time
from prometheus_client import Summary
from prometheus_client.core import GaugeMetricFamily
from smbmc import Client, SensorTypeEnum, SensorStateEnum, PowerSupplyFlag

COLLECTION_TIME = Summary(
    "smbmc_collector_collect_seconds",
    "Time spent to collect metrics from SMBMC",
)


class SMBMCCollector:
    """Provides an interface to SMBMC collector."""

    def __init__(self, hostname, username, password):
        """Initialises SMBMC Collector."""
        self._client = Client(hostname, username, password)

    def collect(self):
        start = time.time()

        # acquire data
        metrics = self._client.get_metrics()

        # initialise sensor gauges
        sensor_prefix = "smbmc_sensor_"
        status_gauge = GaugeMetricFamily(
            f"{sensor_prefix}status",
            "Sensor Status",
            labels=["id", "name"],
        )
        temperature_gauge = GaugeMetricFamily(
            f"{sensor_prefix}temperature",
            "Sensor Temperature",
            labels=["id", "name"],
            unit="celsius",
        )
        temperature_threshold_gauge = GaugeMetricFamily(
            f"{sensor_prefix}threshold_temperature",
            "Sensor Threshold - Temperature",
            labels=["id", "type"],
            unit="celsius",
        )
        voltage_gauge = GaugeMetricFamily(
            f"{sensor_prefix}voltage",
            "Sensor Voltage",
            labels=["id", "name"],
            unit="volts",
        )
        voltage_threshold_gauge = GaugeMetricFamily(
            f"{sensor_prefix}threshold_voltage",
            "Sensor Threshold - Voltage",
            labels=["id", "type"],
            unit="volts",
        )
        fan_gauge = GaugeMetricFamily(
            f"{sensor_prefix}fan_speed",
            "Fan Speed",
            labels=["id", "name"],
            unit="rpm",
        )
        fan_threshold_gauge = GaugeMetricFamily(
            f"{sensor_prefix}threshold_fan_speed",
            "Fan Speed - Threshold",
            labels=["id", "type"],
            unit="rpm",
        )

        # process sensor metrics
        for sensor in metrics["sensor"]:
            sensor_id = str(sensor.id)
            state = 0

            # analogue sensors
            if sensor.type is SensorTypeEnum.TEMPERATURE:
                temperature_gauge.add_metric(
                    [sensor_id, sensor.name],
                    sensor.reading,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "LNR"],
                    sensor.lnr,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "LC"],
                    sensor.lc,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "LNC"],
                    sensor.lnc,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "UNC"],
                    sensor.unc,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "UC"],
                    sensor.uc,
                )
                temperature_threshold_gauge.add_metric(
                    [sensor_id, "UNR"],
                    sensor.unr,
                )
            elif sensor.type is SensorTypeEnum.VOLTAGE:
                voltage_gauge.add_metric(
                    [sensor_id, sensor.name],
                    sensor.reading,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "LNR"],
                    sensor.lnr,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "LC"],
                    sensor.lc,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "LNC"],
                    sensor.lnc,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "UNC"],
                    sensor.unc,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "UC"],
                    sensor.uc,
                )
                voltage_threshold_gauge.add_metric(
                    [sensor_id, "UNR"],
                    sensor.unr,
                )
            elif sensor.type is SensorTypeEnum.FAN:
                fan_gauge.add_metric(
                    [sensor_id, sensor.name],
                    sensor.reading,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "LNR"],
                    sensor.lnr,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "LC"],
                    sensor.lc,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "LNC"],
                    sensor.lnc,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "UNC"],
                    sensor.unc,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "UC"],
                    sensor.uc,
                )
                fan_threshold_gauge.add_metric(
                    [sensor_id, "UNR"],
                    sensor.unr,
                )

            # sensor states (plus discrete sensors)
            if sensor.type is SensorTypeEnum.POWER_SUPPLY:
                if sensor.flags == PowerSupplyFlag.PRESENCE_DETECTED:
                    state = 1
            else:
                # check sensor state
                if sensor.state is SensorStateEnum.PRESENT:
                    state = 1

            status_gauge.add_metric(
                [sensor_id, sensor.name],
                state,
            )

        yield temperature_gauge
        yield temperature_threshold_gauge
        yield voltage_gauge
        yield voltage_threshold_gauge
        yield fan_gauge
        yield fan_threshold_gauge
        yield status_gauge

        # initialise pmbus gauges
        pmbus_prefix = "smbmc_pmbus_"
        pmbus_status_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}status",
            "Power Supply status",
            labels=["id"],
        )
        pmbus_voltage_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}voltage",
            "Power Supply Voltage",
            labels=["id", "type"],
            unit="volts",
        )
        pmbus_current_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}current",
            "Power Supply Current",
            labels=["id", "type"],
            unit="amperes",
        )
        pmbus_power_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}power",
            "Power Supply Consumption",
            labels=["id", "type"],
            unit="watts",
        )
        pmbus_temperature_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}temperature",
            "Power Supply Temperature",
            labels=["id", "type"],
            unit="celsius",
        )
        pmbus_fan_gauge = GaugeMetricFamily(
            f"{pmbus_prefix}fan_speed",
            "Power Supply Fan Speed",
            labels=["id", "type"],
            unit="rpm",
        )

        # process pmbus metrics
        for psu in metrics["pmbus"]:
            psu_id = str(psu.id)
            state = 0

            if psu.status == "1":
                state = 1

            pmbus_status_gauge.add_metric(
                [psu_id],
                state,
            )
            pmbus_voltage_gauge.add_metric(
                [psu_id, "Input (AC)"],
                psu.input_voltage,
            )
            pmbus_voltage_gauge.add_metric(
                [psu_id, "Output (DC)"],
                psu.output_voltage,
            )
            pmbus_current_gauge.add_metric(
                [psu_id, "Input"],
                psu.input_current,
            )
            pmbus_current_gauge.add_metric(
                [psu_id, "Output"],
                psu.output_current,
            )
            pmbus_power_gauge.add_metric(
                [psu_id, "Input"],
                psu.input_power,
            )
            pmbus_power_gauge.add_metric(
                [psu_id, "Output"],
                psu.output_power,
            )
            pmbus_temperature_gauge.add_metric(
                [psu_id, "Input"],
                psu.temp_1,
            )
            pmbus_temperature_gauge.add_metric(
                [psu_id, "Output"],
                psu.temp_2,
            )
            pmbus_fan_gauge.add_metric(
                [psu_id, "Input"],
                psu.fan_1,
            )
            pmbus_fan_gauge.add_metric(
                [psu_id, "Output"],
                psu.fan_2,
            )

        yield pmbus_status_gauge
        yield pmbus_voltage_gauge
        yield pmbus_current_gauge
        yield pmbus_power_gauge
        yield pmbus_temperature_gauge
        yield pmbus_fan_gauge

        duration = time.time() - start

        COLLECTION_TIME.observe(duration)
