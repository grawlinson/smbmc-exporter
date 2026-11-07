//! Conversion of BMC readings into Prometheus metric families.

use prometheus_client::collector::Collector;
use prometheus_client::encoding::{DescriptorEncoder, EncodeMetric};
use prometheus_client::metrics::MetricType;
use prometheus_client::metrics::gauge::ConstGauge;
use prometheus_client::registry::Unit;
use smbmc::{PowerSupply, PowerSupplyFlag, Sensor, SensorState, SensorType};

/// PMBus `a_b_PS_Status_I2C` value of a populated, enabled slot.
const PSU_STATUS_OK: u8 = 0x01;
/// PMBus `a_b_PS_Status_I2C` value of an empty slot.
const PSU_STATUS_ABSENT: u8 = 0xff;

/// Power supply sensor flags that indicate a fault.
const PSU_FAULTS: PowerSupplyFlag = PowerSupplyFlag::FAILURE
    .union(PowerSupplyFlag::PREDICTIVE_FAILURE)
    .union(PowerSupplyFlag::SOURCE_INPUT_LOST)
    .union(PowerSupplyFlag::SOURCE_INPUT_OUT_OF_RANGE)
    .union(PowerSupplyFlag::SOURCE_INPUT_DETECTED_OUT_OF_RANGE)
    .union(PowerSupplyFlag::CONFIGURATION_ERROR);

/// Label values for each named [`PowerSupplyFlag`].
const PSU_FLAG_LABELS: [(PowerSupplyFlag, &str); 8] = [
    (PowerSupplyFlag::PRESENCE_DETECTED, "presence_detected"),
    (PowerSupplyFlag::FAILURE, "failure"),
    (PowerSupplyFlag::PREDICTIVE_FAILURE, "predictive_failure"),
    (PowerSupplyFlag::SOURCE_INPUT_LOST, "source_input_lost"),
    (
        PowerSupplyFlag::SOURCE_INPUT_OUT_OF_RANGE,
        "source_input_out_of_range",
    ),
    (
        PowerSupplyFlag::SOURCE_INPUT_DETECTED_OUT_OF_RANGE,
        "source_input_detected_out_of_range",
    ),
    (PowerSupplyFlag::CONFIGURATION_ERROR, "configuration_error"),
    (PowerSupplyFlag::STANDBY, "standby"),
];

/// One scrape's worth of BMC data, encoded on demand.
///
/// A kind that could not be fetched is `None`: its metric families are left
/// out and `smbmc_up` is `0`.
#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    /// Sensors, if they were fetched successfully.
    pub sensors: Option<Vec<Sensor>>,
    /// Power supplies, if they were fetched successfully.
    pub power_supplies: Option<Vec<PowerSupply>>,
}

impl Snapshot {
    /// Whether every kind of metric was fetched successfully.
    pub fn is_complete(&self) -> bool {
        self.sensors.is_some() && self.power_supplies.is_some()
    }
}

impl Collector for Snapshot {
    fn encode(&self, mut encoder: DescriptorEncoder) -> Result<(), std::fmt::Error> {
        ConstGauge::new(f64::from(u8::from(self.is_complete()))).encode(
            encoder.encode_descriptor(
                "smbmc_up",
                "Whether the last scrape of the BMC succeeded",
                None,
                MetricType::Gauge,
            )?,
        )?;
        if let Some(sensors) = &self.sensors {
            encode_sensors(&mut encoder, sensors)?;
        }
        if let Some(psus) = &self.power_supplies {
            encode_power_supplies(&mut encoder, psus)?;
        }
        Ok(())
    }
}

type Labels<'a> = &'a [(&'a str, &'a str)];

/// Encode one gauge family. Families without samples are skipped.
fn gauge<'a>(
    encoder: &mut DescriptorEncoder,
    name: &str,
    help: &str,
    unit: Option<&Unit>,
    samples: impl IntoIterator<Item = (Labels<'a>, f64)>,
) -> Result<(), std::fmt::Error> {
    let mut samples = samples.into_iter().peekable();
    if samples.peek().is_none() {
        return Ok(());
    }
    let mut family = encoder.encode_descriptor(name, help, unit, MetricType::Gauge)?;
    for (labels, value) in samples {
        ConstGauge::new(value).encode(family.encode_family(&labels)?)?;
    }
    Ok(())
}

/// Readings are meaningless for sensors the BMC reports as absent.
fn has_reading(sensor: &Sensor) -> bool {
    sensor.state != SensorState::NotPresent
}

/// A sensor is healthy if it is present; a power supply sensor additionally
/// must not report a fault.
fn sensor_ok(sensor: &Sensor) -> bool {
    match (sensor.sensor_type, sensor.flags) {
        (SensorType::PowerSupply, Some(flags)) => {
            flags.contains(PowerSupplyFlag::PRESENCE_DETECTED) && !flags.intersects(PSU_FAULTS)
        }
        _ => sensor.state == SensorState::Present,
    }
}

fn encode_sensors(
    encoder: &mut DescriptorEncoder,
    sensors: &[Sensor],
) -> Result<(), std::fmt::Error> {
    let ids: Vec<String> = sensors.iter().map(|s| s.id.to_string()).collect();
    let with_ids = || sensors.iter().zip(&ids);

    let analogue = [
        (
            SensorType::Temperature,
            "temperature",
            "Sensor Temperature",
            "Sensor Threshold - Temperature",
            Unit::Celsius,
        ),
        (
            SensorType::Voltage,
            "voltage",
            "Sensor Voltage",
            "Sensor Threshold - Voltage",
            Unit::Volts,
        ),
        (
            SensorType::Fan,
            "fan_speed",
            "Fan Speed",
            "Fan Speed - Threshold",
            Unit::Other("rpm".into()),
        ),
    ];

    for (kind, name, help, threshold_help, unit) in analogue {
        let of_kind = || with_ids().filter(move |(s, _)| s.sensor_type == kind && has_reading(s));

        let readings: Vec<_> = of_kind()
            .map(|(s, id)| ([("id", id.as_str()), ("name", s.name.as_str())], s.reading))
            .collect();
        gauge(
            encoder,
            &format!("smbmc_sensor_{name}"),
            help,
            Some(&unit),
            readings.iter().map(|(l, v)| (&l[..], *v)),
        )?;

        let thresholds: Vec<_> = of_kind()
            .flat_map(|(s, id)| {
                [
                    ("LNR", s.lnr),
                    ("LC", s.lc),
                    ("LNC", s.lnc),
                    ("UNC", s.unc),
                    ("UC", s.uc),
                    ("UNR", s.unr),
                ]
                .map(|(t, v)| ([("id", id.as_str()), ("type", t)], v))
            })
            .collect();
        gauge(
            encoder,
            &format!("smbmc_sensor_threshold_{name}"),
            threshold_help,
            Some(&unit),
            thresholds.iter().map(|(l, v)| (&l[..], *v)),
        )?;
    }

    let status: Vec<_> = with_ids()
        .map(|(s, id)| {
            (
                [("id", id.as_str()), ("name", s.name.as_str())],
                f64::from(u8::from(sensor_ok(s))),
            )
        })
        .collect();
    gauge(
        encoder,
        "smbmc_sensor_status",
        "Sensor Status",
        None,
        status.iter().map(|(l, v)| (&l[..], *v)),
    )?;

    let flags: Vec<_> = with_ids()
        .filter_map(|(s, id)| Some((s, id, s.flags?)))
        .flat_map(|(s, id, flags)| {
            PSU_FLAG_LABELS.map(|(flag, label)| {
                (
                    [
                        ("id", id.as_str()),
                        ("name", s.name.as_str()),
                        ("flag", label),
                    ],
                    f64::from(u8::from(flags.contains(flag))),
                )
            })
        })
        .collect();
    gauge(
        encoder,
        "smbmc_sensor_power_supply_flag",
        "Power supply sensor event flags",
        None,
        flags.iter().map(|(l, v)| (&l[..], *v)),
    )?;

    Ok(())
}

fn encode_power_supplies(
    encoder: &mut DescriptorEncoder,
    psus: &[PowerSupply],
) -> Result<(), std::fmt::Error> {
    let ids: Vec<String> = psus.iter().map(|p| p.id.to_string()).collect();
    let with_ids = || psus.iter().zip(&ids);
    let present = || with_ids().filter(|(p, _)| p.status != PSU_STATUS_ABSENT);

    let status: Vec<_> = with_ids()
        .map(|(p, id)| {
            (
                [("id", id.as_str())],
                f64::from(u8::from(p.status == PSU_STATUS_OK)),
            )
        })
        .collect();
    gauge(
        encoder,
        "smbmc_pmbus_status",
        "Power Supply status",
        None,
        status.iter().map(|(l, v)| (&l[..], *v)),
    )?;

    let info: Vec<_> = present()
        .map(|(p, id)| ([("id", id.as_str()), ("name", p.name.as_str())], 1.0))
        .collect();
    gauge(
        encoder,
        "smbmc_pmbus_info",
        "Power Supply information; the name label is the serial number",
        None,
        info.iter().map(|(l, v)| (&l[..], *v)),
    )?;

    type Reading = fn(&PowerSupply) -> [(&'static str, f64); 2];
    let families: [(&str, &str, Unit, Reading); 5] = [
        ("voltage", "Power Supply Voltage", Unit::Volts, |p| {
            [
                ("Input (AC)", f64::from(p.input_voltage)),
                ("Output (DC)", p.output_voltage),
            ]
        }),
        ("current", "Power Supply Current", Unit::Amperes, |p| {
            [("Input", p.input_current), ("Output", p.output_current)]
        }),
        (
            "power",
            "Power Supply Consumption",
            Unit::Other("watts".into()),
            |p| {
                [
                    ("Input", f64::from(p.input_power)),
                    ("Output", f64::from(p.output_power)),
                ]
            },
        ),
        (
            "temperature",
            "Power Supply Temperature",
            Unit::Celsius,
            |p| {
                [
                    ("Input", f64::from(p.temp_1)),
                    ("Output", f64::from(p.temp_2)),
                ]
            },
        ),
        (
            "fan_speed",
            "Power Supply Fan Speed",
            Unit::Other("rpm".into()),
            |p| {
                [
                    ("Input", f64::from(p.fan_1)),
                    ("Output", f64::from(p.fan_2)),
                ]
            },
        ),
    ];

    for (name, help, unit, reading) in families {
        let samples: Vec<_> = present()
            .flat_map(|(p, id)| reading(p).map(|(t, v)| ([("id", id.as_str()), ("type", t)], v)))
            .collect();
        gauge(
            encoder,
            &format!("smbmc_pmbus_{name}"),
            help,
            Some(&unit),
            samples.iter().map(|(l, v)| (&l[..], *v)),
        )?;
    }

    Ok(())
}
