//! Command-line interface.

use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

/// Prometheus exporter for Supermicro BMC metrics.
#[derive(Debug, Clone, Parser)]
#[command(version, about)]
pub struct Args {
    /// Address of the BMC web interface, e.g. `https://192.168.1.1`.
    /// `https://` is assumed when no scheme is given.
    #[arg(long, visible_alias = "server", env = "SMBMC_HOSTNAME")]
    pub hostname: String,

    /// Username for the BMC web interface.
    #[arg(long, env = "SMBMC_USERNAME")]
    pub username: String,

    /// Password for the BMC web interface.
    #[arg(
        long,
        env = "SMBMC_PASSWORD",
        hide_env_values = true,
        required_unless_present = "password_file",
        conflicts_with = "password_file"
    )]
    pub password: Option<String>,

    /// Read the password from this file instead (trailing newlines are
    /// ignored).
    #[arg(long, env = "SMBMC_PASSWORD_FILE")]
    pub password_file: Option<PathBuf>,

    /// Accept invalid (e.g. self-signed) TLS certificates from the BMC.
    #[arg(long, env = "SMBMC_INSECURE")]
    pub insecure: bool,

    /// Timeout for each request to the BMC, in seconds.
    #[arg(long, env = "SMBMC_TIMEOUT", default_value_t = 30, value_parser = clap::value_parser!(u64).range(1..))]
    pub timeout: u64,

    /// Port for the daemon to listen on.
    #[arg(long, env = "LISTEN_PORT", default_value_t = 8000)]
    pub listen_port: u16,

    /// Address for the daemon to listen on.
    #[arg(long, env = "LISTEN_ADDR", default_value = "0.0.0.0")]
    pub listen_addr: String,
}

impl Args {
    /// The BMC address as a URL.
    pub fn server_url(&self) -> String {
        let host = self.hostname.trim();
        if host.contains("://") {
            host.to_owned()
        } else {
            format!("https://{host}")
        }
    }

    /// The password, read from `--password-file` if needed.
    ///
    /// # Errors
    ///
    /// If the password file cannot be read.
    pub fn password(&self) -> std::io::Result<String> {
        match (&self.password, &self.password_file) {
            (Some(password), _) => Ok(password.clone()),
            (None, Some(path)) => Ok(std::fs::read_to_string(path)?
                .trim_end_matches(['\r', '\n'])
                .to_owned()),
            (None, None) => unreachable!("clap requires one of the two"),
        }
    }

    /// Build the BMC client.
    ///
    /// # Errors
    ///
    /// If the password file cannot be read or the client cannot be built
    /// (e.g. an invalid address).
    pub fn client(&self) -> Result<smbmc::Client, Box<dyn std::error::Error>> {
        Ok(
            smbmc::Client::builder(&self.server_url(), &self.username, &self.password()?)
                .accept_invalid_certs(self.insecure)
                .request_timeout(Duration::from_secs(self.timeout))
                .build()?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Args, clap::Error> {
        Args::try_parse_from(["smbmc-exporter"].iter().chain(args))
    }

    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        Args::command().debug_assert();
    }

    #[test]
    fn defaults() {
        let args = parse(&["--hostname", "bmc", "--username", "u", "--password", "p"]).unwrap();
        assert_eq!(args.server_url(), "https://bmc");
        assert_eq!(args.listen_port, 8000);
        assert_eq!(args.listen_addr, "0.0.0.0");
        assert_eq!(args.timeout, 30);
        assert!(!args.insecure);
        assert_eq!(args.password().unwrap(), "p");
    }

    #[test]
    fn explicit_scheme_is_kept() {
        let args = parse(&[
            "--server",
            "http://10.0.0.1:8080",
            "--username",
            "u",
            "--password",
            "p",
        ])
        .unwrap();
        assert_eq!(args.server_url(), "http://10.0.0.1:8080");
    }

    #[test]
    fn password_is_required() {
        assert!(parse(&["--hostname", "bmc", "--username", "u"]).is_err());
    }

    #[test]
    fn password_and_file_conflict() {
        let err = parse(&[
            "--hostname",
            "bmc",
            "--username",
            "u",
            "--password",
            "p",
            "--password-file",
            "/dev/null",
        ])
        .unwrap_err();
        assert_eq!(err.kind(), clap::error::ErrorKind::ArgumentConflict);
    }

    #[test]
    fn password_file_strips_newline() {
        let path = std::env::temp_dir().join(format!("smbmc-exporter-{}", std::process::id()));
        std::fs::write(&path, "s3cret\n").unwrap();
        let args = parse(&[
            "--hostname",
            "bmc",
            "--username",
            "u",
            "--password-file",
            path.to_str().unwrap(),
        ])
        .unwrap();
        assert_eq!(args.password().unwrap(), "s3cret");
        std::fs::remove_file(path).unwrap();
    }
}
