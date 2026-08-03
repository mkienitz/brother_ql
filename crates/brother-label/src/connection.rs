use std::{
    fmt,
    io::Write,
    net::{IpAddr, Ipv6Addr, TcpStream, ToSocketAddrs},
    num::NonZeroU16,
    path::PathBuf,
    str::FromStr,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail};
use brother_ql::{
    connection::{KernelConnection, PrinterConnection, UsbConnection, UsbConnectionInfo},
    printer::PrinterModel,
    printjob::PrintJob,
    status::StatusInformation,
};

const DEFAULT_NETWORK_PORT: NonZeroU16 = NonZeroU16::new(9100).unwrap();
const NETWORK_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub(crate) enum BidirectionalConnectionSelection {
    Usb(PrinterModel),
    UsbAutoDiscover,
    Device(PathBuf),
}

#[derive(Debug)]
pub(crate) enum PrintTargetSelection {
    Bidirectional(BidirectionalConnectionSelection),
    Network(NetworkEndpoint),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NetworkEndpoint {
    host: String,
    port: NonZeroU16,
}

impl FromStr for NetworkEndpoint {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.is_empty() {
            return Err("network host must not be empty".into());
        }

        if let Some(bracketed) = value.strip_prefix('[') {
            let (host, suffix) = bracketed
                .split_once(']')
                .ok_or_else(|| "missing closing ']' in IPv6 address".to_owned())?;
            host.parse::<Ipv6Addr>()
                .map_err(|_| "invalid bracketed IPv6 address".to_owned())?;
            let port = match suffix.strip_prefix(':') {
                Some(port) => parse_port(port)?,
                None if suffix.is_empty() => DEFAULT_NETWORK_PORT,
                None => return Err("expected a port after the bracketed address".into()),
            };
            return Ok(Self {
                host: host.to_owned(),
                port,
            });
        }

        // A bare IP address, including IPv6, uses the default port.
        if value.parse::<IpAddr>().is_ok() {
            return Ok(Self {
                host: value.to_owned(),
                port: DEFAULT_NETWORK_PORT,
            });
        }

        let (host, port) = match value.rsplit_once(':') {
            Some((host, port)) => (host, parse_port(port)?),
            None => (value, DEFAULT_NETWORK_PORT),
        };
        if host.is_empty() {
            return Err("network host must not be empty".into());
        }
        if host.contains(':') {
            return Err("IPv6 addresses with an explicit port must use brackets".into());
        }

        Ok(Self {
            host: host.to_owned(),
            port,
        })
    }
}

fn parse_port(value: &str) -> Result<NonZeroU16, String> {
    value
        .parse::<NonZeroU16>()
        .map_err(|_| format!("invalid network port '{value}'; expected a value from 1 to 65535"))
}

impl fmt::Display for NetworkEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.host.contains(':') {
            write!(f, "[{}]:{}", self.host, self.port)
        } else {
            write!(f, "{}:{}", self.host, self.port)
        }
    }
}

pub(crate) enum BidirectionalConnection {
    Usb(UsbConnection),
    Kernel(KernelConnection),
}

impl BidirectionalConnection {
    pub(crate) fn open(selection: BidirectionalConnectionSelection) -> Result<Self> {
        match selection {
            BidirectionalConnectionSelection::Usb(model) => Ok(Self::Usb(UsbConnection::open(
                UsbConnectionInfo::from_model(model),
            )?)),
            BidirectionalConnectionSelection::UsbAutoDiscover => {
                let info = UsbConnectionInfo::discover()?
                    .ok_or_else(|| anyhow!("could not auto-discover a supported USB printer"))?;
                Ok(Self::Usb(UsbConnection::open(info)?))
            }
            BidirectionalConnectionSelection::Device(path) => {
                Ok(Self::Kernel(KernelConnection::open(path)?))
            }
        }
    }

    pub(crate) fn print(&mut self, job: PrintJob) -> Result<()> {
        match self {
            Self::Usb(connection) => connection.print(job)?,
            Self::Kernel(connection) => connection.print(job)?,
        }
        Ok(())
    }

    pub(crate) fn get_status(&mut self) -> Result<StatusInformation> {
        Ok(match self {
            Self::Usb(connection) => connection.get_status()?,
            Self::Kernel(connection) => connection.get_status()?,
        })
    }
}

pub(crate) enum PrintTarget {
    Bidirectional(BidirectionalConnection),
    Network(NetworkEndpoint),
}

impl PrintTarget {
    pub(crate) fn open(selection: PrintTargetSelection) -> Result<Self> {
        Ok(match selection {
            PrintTargetSelection::Bidirectional(selection) => {
                Self::Bidirectional(BidirectionalConnection::open(selection)?)
            }
            PrintTargetSelection::Network(endpoint) => Self::Network(endpoint),
        })
    }

    pub(crate) fn print(&mut self, job: PrintJob) -> Result<()> {
        match self {
            Self::Bidirectional(connection) => connection.print(job),
            Self::Network(endpoint) => send_network_data(endpoint, &job.compile()),
        }
    }
}

fn send_network_data(endpoint: &NetworkEndpoint, data: &[u8]) -> Result<()> {
    let addresses = (endpoint.host.as_str(), endpoint.port.get())
        .to_socket_addrs()
        .with_context(|| format!("could not resolve network printer {endpoint}"))?;

    let deadline = Instant::now() + NETWORK_TIMEOUT;
    let mut resolved_address = false;
    let mut last_error = None;

    for address in addresses {
        resolved_address = true;
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }

        match TcpStream::connect_timeout(&address, remaining) {
            Ok(mut stream) => {
                stream
                    .set_write_timeout(Some(NETWORK_TIMEOUT))
                    .with_context(|| format!("could not configure connection to {endpoint}"))?;
                // Never retry after writing begins: a failed write may still have delivered a
                // partial job, and retrying could print duplicate labels.
                stream
                    .write_all(data)
                    .with_context(|| format!("could not send print job to {endpoint}"))?;
                return Ok(());
            }
            Err(error) => last_error = Some(error),
        }
    }

    if !resolved_address {
        bail!("network printer {endpoint} did not resolve to any addresses");
    }

    let seconds = NETWORK_TIMEOUT.as_secs();
    match last_error {
        Some(error) => Err(error).with_context(|| {
            format!("could not connect to network printer {endpoint} within {seconds} seconds")
        }),
        None => bail!("could not connect to network printer {endpoint} within {seconds} seconds"),
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Read, net::TcpListener, thread};

    use brother_ql::{media::Media, printjob::PrintJobBuilder};
    use image::{DynamicImage, RgbImage};

    use super::{NetworkEndpoint, PrintTarget};

    #[test]
    fn endpoint_uses_default_port_for_hosts_and_ip_addresses() {
        assert_eq!(
            "printer.local"
                .parse::<NetworkEndpoint>()
                .unwrap()
                .to_string(),
            "printer.local:9100"
        );
        assert_eq!(
            "192.168.178.39"
                .parse::<NetworkEndpoint>()
                .unwrap()
                .to_string(),
            "192.168.178.39:9100"
        );
    }

    #[test]
    fn endpoint_accepts_explicit_ports_and_ipv6() {
        assert_eq!(
            "printer.local:9101"
                .parse::<NetworkEndpoint>()
                .unwrap()
                .to_string(),
            "printer.local:9101"
        );
        assert_eq!(
            "[2001:db8::1]:9102"
                .parse::<NetworkEndpoint>()
                .unwrap()
                .to_string(),
            "[2001:db8::1]:9102"
        );
        assert_eq!(
            "2001:db8::1"
                .parse::<NetworkEndpoint>()
                .unwrap()
                .to_string(),
            "[2001:db8::1]:9100"
        );
    }

    #[test]
    fn endpoint_rejects_malformed_hosts_and_ports() {
        for value in [
            "",
            "printer.local:0",
            "printer.local:65536",
            "printer.local:not-a-port",
            "printer.local::9100",
            "[not-ipv6]:9100",
            "[2001:db8::1",
        ] {
            assert!(
                value.parse::<NetworkEndpoint>().is_err(),
                "unexpectedly accepted {value:?}"
            );
        }
    }

    #[test]
    fn network_target_writes_compiled_job_and_closes_connection() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let receiver = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut received = Vec::new();
            stream.read_to_end(&mut received).unwrap();
            received
        });

        let endpoint = address.to_string().parse().unwrap();
        let make_job = || {
            PrintJobBuilder::new(Media::C62)
                .add_label(DynamicImage::ImageRgb8(RgbImage::new(
                    Media::C62.width_dots(),
                    10,
                )))
                .build()
                .unwrap()
        };
        let expected = make_job().compile();

        let mut target = PrintTarget::Network(endpoint);
        target.print(make_job()).unwrap();

        assert_eq!(receiver.join().unwrap(), expected);
    }
}
