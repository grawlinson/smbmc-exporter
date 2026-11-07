//! End-to-end tests: a mock BMC (`wiremock`) behind the exporter's router.

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use http_body_util::BodyExt;
use prometheus_client::encoding::text::encode;
use prometheus_client::registry::Registry;
use smbmc_exporter::{Exporter, Snapshot, router, server::CONTENT_TYPE};
use tower::ServiceExt;
use wiremock::matchers::{body_string_contains, header as header_is, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"))
}

fn fixture_snapshot() -> Snapshot {
    Snapshot {
        sensors: Some(smbmc::parse_sensors(&fixture("ipmi_response_sensors.xml")).unwrap()),
        power_supplies: Some(
            smbmc::parse_power_supplies(&fixture("ipmi_response_pmbus.xml")).unwrap(),
        ),
    }
}

fn encode_snapshot(snapshot: Snapshot) -> String {
    let mut registry = Registry::default();
    registry.register_collector(Box::new(snapshot));
    let mut body = String::new();
    encode(&mut body, &registry).unwrap();
    body
}

/// Lines of `body` for the metric family `name` (samples only).
fn samples<'a>(body: &'a str, name: &str) -> Vec<&'a str> {
    body.lines()
        .filter(|l| {
            l.strip_prefix(name)
                .is_some_and(|rest| rest.starts_with(['{', ' ']))
        })
        .collect()
}

/// Compare against `tests/fixtures/metrics.txt`; set `UPDATE_GOLDEN=1` to
/// rewrite it.
#[test]
fn golden_output() {
    let body = encode_snapshot(fixture_snapshot());
    let path = format!("{}/tests/fixtures/metrics.txt", env!("CARGO_MANIFEST_DIR"));
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&path, &body).unwrap();
    }
    assert_eq!(body, fixture("metrics.txt"));
}

#[test]
fn metric_names_match_python_exporter() {
    let body = encode_snapshot(fixture_snapshot());
    for name in [
        "smbmc_sensor_temperature_celsius",
        "smbmc_sensor_threshold_temperature_celsius",
        "smbmc_sensor_voltage_volts",
        "smbmc_sensor_threshold_voltage_volts",
        "smbmc_sensor_fan_speed_rpm",
        "smbmc_sensor_threshold_fan_speed_rpm",
        "smbmc_sensor_status",
        "smbmc_pmbus_status",
        "smbmc_pmbus_voltage_volts",
        "smbmc_pmbus_current_amperes",
        "smbmc_pmbus_power_watts",
        "smbmc_pmbus_temperature_celsius",
        "smbmc_pmbus_fan_speed_rpm",
    ] {
        assert!(!samples(&body, name).is_empty(), "missing {name}\n{body}");
    }
    assert_eq!(samples(&body, "smbmc_up"), ["smbmc_up 1.0"]);
}

#[test]
fn absent_sensors_have_no_readings() {
    let mut snapshot = fixture_snapshot();
    let sensors = snapshot.sensors.as_mut().unwrap();
    let fan = sensors
        .iter_mut()
        .find(|s| s.sensor_type == smbmc::SensorType::Fan)
        .unwrap();
    fan.state = smbmc::SensorState::NotPresent;
    let id = fan.id;

    let body = encode_snapshot(snapshot);
    let label = format!("{{id=\"{id}\",");
    assert!(
        !samples(&body, "smbmc_sensor_fan_speed_rpm")
            .iter()
            .any(|l| l.contains(&label))
    );
    assert!(
        samples(&body, "smbmc_sensor_status")
            .iter()
            .any(|l| l.contains(&label) && l.ends_with(" 0.0"))
    );
}

/// Status of the power supply sensor (id 27 in the fixture) for `flags`.
fn psu_sensor(flags: smbmc::PowerSupplyFlag) -> String {
    let mut snapshot = fixture_snapshot();
    for s in snapshot.sensors.as_mut().unwrap() {
        if s.sensor_type == smbmc::SensorType::PowerSupply {
            s.flags = Some(flags);
        }
    }
    encode_snapshot(snapshot)
}

fn psu_sensor_status(flags: smbmc::PowerSupplyFlag) -> String {
    let body = psu_sensor(flags);
    let line = samples(&body, "smbmc_sensor_status")
        .into_iter()
        .find(|l| l.contains("name=\"PS2 Status\""))
        .unwrap();
    line.rsplit(' ').next().unwrap().to_owned()
}

#[test]
fn power_supply_sensor_status() {
    use smbmc::PowerSupplyFlag as F;

    assert_eq!(psu_sensor_status(F::PRESENCE_DETECTED), "1.0");
    // Standby isn't a fault (the Python exporter required the flags to be
    // exactly "presence detected").
    assert_eq!(psu_sensor_status(F::PRESENCE_DETECTED | F::STANDBY), "1.0");
    assert_eq!(psu_sensor_status(F::empty()), "0.0");
    assert_eq!(psu_sensor_status(F::PRESENCE_DETECTED | F::FAILURE), "0.0");
    assert_eq!(
        psu_sensor_status(F::PRESENCE_DETECTED | F::SOURCE_INPUT_LOST),
        "0.0"
    );
}

#[test]
fn power_supply_sensor_flags() {
    use smbmc::PowerSupplyFlag as F;

    let body = psu_sensor(F::PRESENCE_DETECTED | F::FAILURE);
    let flags = samples(&body, "smbmc_sensor_power_supply_flag");
    assert_eq!(flags.len(), 8);
    let set: Vec<_> = flags.iter().filter(|l| l.ends_with(" 1.0")).collect();
    assert_eq!(set.len(), 2, "{set:?}");
    assert!(set.iter().any(|l| l.contains("flag=\"failure\"")));
    assert!(set.iter().any(|l| l.contains("flag=\"presence_detected\"")));
}

#[test]
fn empty_psu_slots_only_report_status() {
    let body = encode_snapshot(fixture_snapshot());
    // The fixture has a single populated slot (1) out of four.
    assert_eq!(samples(&body, "smbmc_pmbus_status").len(), 4);
    assert_eq!(
        samples(&body, "smbmc_pmbus_info"),
        ["smbmc_pmbus_info{id=\"1\",name=\"PSU0SERIAL0NO00\"} 1.0"]
    );
    assert!(
        samples(&body, "smbmc_pmbus_power_watts")
            .iter()
            .all(|l| l.contains("id=\"1\""))
    );
}

#[test]
fn failed_fetch_reports_down() {
    let body = encode_snapshot(Snapshot {
        sensors: None,
        ..fixture_snapshot()
    });
    assert_eq!(samples(&body, "smbmc_up"), ["smbmc_up 0.0"]);
    assert!(samples(&body, "smbmc_sensor_status").is_empty());
    assert!(!samples(&body, "smbmc_pmbus_status").is_empty());
}

async fn mock_bmc() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/cgi/login.cgi"))
        .respond_with(
            ResponseTemplate::new(200).append_header("Set-Cookie", "SID=abc123; path=/ ; HttpOnly"),
        )
        .expect(1)
        .mount(&server)
        .await;
    for (field, body) in [
        ("SENSOR_INFO.XML", "ipmi_response_sensors.xml"),
        ("Get_PSInfoReadings.XML", "ipmi_response_pmbus.xml"),
    ] {
        Mock::given(method("POST"))
            .and(path("/cgi/ipmi.cgi"))
            .and(header_is("cookie", "SID=abc123"))
            .and(body_string_contains(field))
            .respond_with(ResponseTemplate::new(200).set_body_string(fixture(body)))
            .mount(&server)
            .await;
    }
    server
}

async fn get(exporter: &Arc<Exporter>, uri: &str) -> (StatusCode, Option<String>, String) {
    let response = router(exporter.clone())
        .oneshot(Request::get(uri).body(Body::empty()).unwrap())
        .await
        .unwrap();
    let status = response.status();
    let content_type = response
        .headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().to_owned());
    let body = response.into_body().collect().await.unwrap().to_bytes();
    (
        status,
        content_type,
        String::from_utf8(body.to_vec()).unwrap(),
    )
}

#[tokio::test]
async fn scrape() {
    let bmc = mock_bmc().await;
    let client = smbmc::Client::new(&bmc.uri(), "ADMIN", "secret").unwrap();
    let exporter = Arc::new(Exporter::new(client));

    for scrape in 1..=2 {
        let (status, content_type, body) = get(&exporter, "/metrics").await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(content_type.as_deref(), Some(CONTENT_TYPE));
        assert_eq!(samples(&body, "smbmc_up"), ["smbmc_up 1.0"]);
        assert_eq!(
            samples(&body, "smbmc_collector_collect_seconds_count"),
            [format!("smbmc_collector_collect_seconds_count {scrape}")]
        );
        assert!(body.ends_with("# EOF\n"));
    }
    // The session is reused: `expect(1)` on the login mock is checked on drop.
}

#[tokio::test]
async fn scrape_with_bmc_down() {
    let bmc = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&bmc)
        .await;
    let client = smbmc::Client::new(&bmc.uri(), "ADMIN", "secret").unwrap();
    let exporter = Arc::new(Exporter::new(client));

    let (status, _, body) = get(&exporter, "/metrics").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(samples(&body, "smbmc_up"), ["smbmc_up 0.0"]);
    assert!(samples(&body, "smbmc_sensor_status").is_empty());
    assert!(samples(&body, "smbmc_pmbus_status").is_empty());
}

#[tokio::test]
async fn index() {
    let bmc = MockServer::start().await;
    let client = smbmc::Client::new(&bmc.uri(), "ADMIN", "secret").unwrap();
    let (status, _, body) = get(&Arc::new(Exporter::new(client)), "/").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body.contains("href=\"metrics\""));
}
