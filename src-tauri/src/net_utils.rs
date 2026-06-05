use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

pub const TCP_CONNECT_TIMEOUT: Duration = Duration::from_millis(250);

pub fn parse_port(value: &str) -> Option<u16> {
    value.trim().parse::<u16>().ok().filter(|port| *port > 0)
}

pub fn tcp_local_port_accepts(port: u16) -> bool {
    tcp_endpoint_accepts("127.0.0.1", port)
}

pub fn tcp_endpoint_accepts(host: &str, port: u16) -> bool {
    let Some(connect_host) = normalize_connect_host(host) else {
        return false;
    };
    let Ok(addrs) = (connect_host.as_str(), port).to_socket_addrs() else {
        return false;
    };
    tcp_addrs_accept(addrs.take(4))
}

fn normalize_connect_host(host: &str) -> Option<String> {
    let host = host.trim();
    if host.is_empty() {
        return None;
    }
    let connect_host = match host {
        "0.0.0.0" | "::" | "[::]" => "127.0.0.1",
        other => other,
    };
    let connect_host = connect_host
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(connect_host);
    Some(connect_host.to_string())
}

fn tcp_addrs_accept<I>(addrs: I) -> bool
where
    I: IntoIterator<Item = SocketAddr>,
{
    addrs
        .into_iter()
        .any(|addr| TcpStream::connect_timeout(&addr, TCP_CONNECT_TIMEOUT).is_ok())
}

#[cfg(test)]
mod tests {
    use super::{normalize_connect_host, parse_port};

    #[test]
    fn parse_port_rejects_zero_and_invalid_values() {
        assert_eq!(parse_port("15722"), Some(15722));
        assert_eq!(parse_port(" 15723 "), Some(15723));
        assert_eq!(parse_port("0"), None);
        assert_eq!(parse_port("abc"), None);
    }

    #[test]
    fn normalize_connect_host_maps_wildcards_to_loopback() {
        assert_eq!(
            normalize_connect_host("0.0.0.0").as_deref(),
            Some("127.0.0.1")
        );
        assert_eq!(normalize_connect_host("::").as_deref(), Some("127.0.0.1"));
        assert_eq!(normalize_connect_host("[::]").as_deref(), Some("127.0.0.1"));
    }

    #[test]
    fn normalize_connect_host_trims_bracketed_ipv6() {
        assert_eq!(normalize_connect_host("[::1]").as_deref(), Some("::1"));
        assert_eq!(
            normalize_connect_host(" localhost ").as_deref(),
            Some("localhost")
        );
        assert_eq!(normalize_connect_host(" ").as_deref(), None);
    }
}
