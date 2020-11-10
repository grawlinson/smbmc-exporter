"""Command line interface."""
import time

import click
from prometheus_client import start_http_server
from prometheus_client.core import REGISTRY

from . import __version__
from .collector import SMBMCCollector


@click.command()
@click.version_option(version=__version__)
@click.option(
    "--hostname",
    help="Hostname of the SMBMC web-interface.",
    envvar="SMBMC_HOSTNAME",
    show_envvar=True,
    required=True,
)
@click.option(
    "--username",
    help="Username for the SMBMC web-interface.",
    envvar="SMBMC_USERNAME",
    show_envvar=True,
    required=True,
)
@click.option(
    "--password",
    help="Password for the SMBMC web-interface.",
    envvar="SMBMC_PASSWORD",
    show_envvar=True,
    required=True,
)
@click.option(
    "--listen-port",
    help="Port for daemon to listen on. (default: 8000)",
    envvar="LISTEN_PORT",
    show_envvar=True,
    default=8000,
)
@click.option(
    "--listen-addr",
    help="Address for daemon to listen on. (default: 0.0.0.0)",
    envvar="LISTEN_ADDR",
    show_envvar=True,
    default="0.0.0.0",
)
def main(hostname, username, password, listen_port, listen_addr):
    """Prometheus exporter for smbmc metrics."""
    click.echo(f"Connecting to {hostname}")
    click.echo(f"Listening on {listen_addr}:{listen_port}")

    REGISTRY.register(SMBMCCollector(hostname, username, password))

    start_http_server(listen_port, listen_addr)

    while True:
        time.sleep(1)
