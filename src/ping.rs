use socket2::{Domain, Protocol, Socket, Type};
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, ToSocketAddrs};
use std::time::{Duration, Instant};
use term_size::dimensions;
use textplots::{Chart, Plot, Shape};

struct PingResult {
    dropped: bool,
    latency_ms: u64,
}

struct PingRunResult {
    total: u64,
    succeeded: u64,
    failed: u64,
    max_latency: u64,
    min_latency: u64,
    average_latency: u64,
}

fn calculate_icmp_checksum(packet: &[u8]) -> u16 {
    let mut sum: u32 = 0;

    // Sum all 16-bit words
    for chunk in packet.chunks(2) {
        let word = if chunk.len() == 2 {
            u16::from_be_bytes([chunk[0], chunk[1]])
        } else {
            u16::from_be_bytes([chunk[0], 0])
        };
        sum += word as u32;
    }

    // Add carry bits
    while (sum >> 16) != 0 {
        sum = (sum & 0xFFFF) + (sum >> 16);
    }

    // One's complement
    return !sum as u16;
}

fn resolve_host(host: &str) -> Result<IpAddr, String> {
    // First try to parse as IP address directly
    if let Ok(ip) = host.parse::<IpAddr>() {
        return Ok(ip);
    }

    // Try DNS resolution
    let address = format!("{}:80", host); // Add dummy port for resolution
    match address.to_socket_addrs() {
        Ok(mut addrs) => {
            if let Some(addr) = addrs.next() {
                return Ok(addr.ip());
            } else {
                return Err(format!("No addresses found for host: {}", host));
            }
        }
        Err(e) => return Err(format!("Failed to resolve host {}: {}", host, e)),
    }
}

fn ping(host: &str, timeout: u64, size: u64) -> PingResult {
    // Convert timeout from milliseconds to Duration
    let timeout_duration = Duration::from_millis(timeout);

    // Resolve the host to an IP address
    let ip = match resolve_host(host) {
        Ok(ip) => ip,
        Err(e) => {
            println!("{}", e);
            std::process::exit(1);
        }
    };

    // Create raw ICMP socket based on IP version
    let (socket, _domain, _protocol) = match ip {
        IpAddr::V4(_) => {
            let socket = match Socket::new_raw(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)) {
                Ok(socket) => socket,
                Err(_) => {
                    println!("Failed to create IPv4 socket. You may need to run with `sudo`.");
                    std::process::exit(1);
                }
            };
            (socket, Domain::IPV4, Protocol::ICMPV4)
        }
        IpAddr::V6(_) => {
            let socket = match Socket::new_raw(Domain::IPV6, Type::RAW, Some(Protocol::ICMPV6)) {
                Ok(socket) => socket,
                Err(_) => {
                    println!("Failed to create IPv6 socket. You may need to run with `sudo`.");
                    std::process::exit(1);
                }
            };
            (socket, Domain::IPV6, Protocol::ICMPV6)
        }
    };

    socket.set_read_timeout(Some(timeout_duration)).unwrap();

    // Build ICMP packet based on IP version
    let packet = match ip {
        IpAddr::V4(_) => build_icmpv4_packet(size),
        IpAddr::V6(_) => build_icmpv6_packet(size),
    };

    // Send packet
    let start_time = Instant::now();
    let addr = std::net::SocketAddr::new(ip, 0);
    socket.send_to(&packet, &addr.into()).unwrap();

    // Receive reply
    let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];
    match socket.recv_from(&mut buffer) {
        Ok((bytes_received, _)) => {
            let end_time = Instant::now();
            let latency = end_time.duration_since(start_time).as_millis() as u64;

            // Validate reply based on IP version
            let is_valid = match ip {
                IpAddr::V4(_) => validate_icmpv4_reply(&buffer, bytes_received),
                IpAddr::V6(_) => validate_icmpv6_reply(&buffer, bytes_received),
            };

            return PingResult {
                dropped: !is_valid,
                latency_ms: latency,
            };
        }
        Err(_) => {
            // Timeout or other error
            let end_time = Instant::now();
            let latency = end_time.duration_since(start_time).as_millis() as u64;
            return PingResult {
                dropped: true,
                latency_ms: latency,
            };
        }
    }
}

fn build_icmpv4_packet(size: u64) -> Vec<u8> {
    // Build ICMP Echo Request packet
    // ICMP header (8 bytes) + size padding
    let mut packet = vec![0u8; 8 + size as usize];

    // ICMP Header
    packet[0] = 8; // Type: Echo Request
    packet[1] = 0; // Code: 0
    packet[2] = 0; // Checksum (will be calculated)
    packet[3] = 0; // Checksum
    packet[4] = 0; // Identifier (can be process ID)
    packet[5] = 1; // Identifier
    packet[6] = 0; // Sequence number
    packet[7] = 1; // Sequence number

    // Fill data portion with pattern
    for i in 8..packet.len() {
        packet[i] = (i % 256) as u8;
    }

    // Calculate checksum
    let checksum = calculate_icmp_checksum(&packet);
    packet[2] = (checksum >> 8) as u8;
    packet[3] = (checksum & 0xff) as u8;

    return packet;
}

fn build_icmpv6_packet(size: u64) -> Vec<u8> {
    // Build ICMPv6 Echo Request packet
    // ICMPv6 header (8 bytes) + size padding
    let mut packet = vec![0u8; 8 + size as usize];

    // ICMPv6 Header
    packet[0] = 128; // Type: Echo Request (ICMPv6)
    packet[1] = 0; // Code: 0
    packet[2] = 0; // Checksum (calculated by kernel for IPv6)
    packet[3] = 0; // Checksum
    packet[4] = 0; // Identifier
    packet[5] = 1; // Identifier
    packet[6] = 0; // Sequence number
    packet[7] = 1; // Sequence number

    // Fill data portion with pattern
    for i in 8..packet.len() {
        packet[i] = (i % 256) as u8;
    }

    return packet;
}

fn validate_icmpv4_reply(buffer: &[std::mem::MaybeUninit<u8>], bytes_received: usize) -> bool {
    // Basic validation - check if it's an ICMP Echo Reply
    if bytes_received >= 28 {
        // IP header (20) + ICMP header (8)
        let icmp_type = unsafe { buffer[20].assume_init() }; // ICMP type is at offset 20 (after IP header)
        return icmp_type == 0; // Echo Reply
    } else {
        return false;
    }
}

fn validate_icmpv6_reply(buffer: &[std::mem::MaybeUninit<u8>], bytes_received: usize) -> bool {
    // For ICMPv6, no IP header to skip, ICMP header starts immediately
    if bytes_received >= 8 {
        let icmp_type = unsafe { buffer[0].assume_init() };
        return icmp_type == 129; // ICMPv6 Echo Reply
    } else {
        return false;
    }
}

fn average(responses: &Vec<PingResult>) -> PingRunResult {
    let mut result = PingRunResult {
        total: 0,
        succeeded: 0,
        failed: 0,
        max_latency: 0,
        min_latency: 0,
        average_latency: 0,
    };

    for response in responses {
        result.total += 1;

        if response.dropped == true {
            result.failed += 1;
        } else {
            result.succeeded += 1;
        }

        if response.latency_ms > result.max_latency {
            result.max_latency = response.latency_ms;
        }

        if response.latency_ms < result.min_latency || result.min_latency == 0 {
            if response.latency_ms > 0 {
                result.min_latency = response.latency_ms;
            }
        }

        result.average_latency += response.latency_ms;
    }

    result.average_latency /= result.total;

    return result;
}

fn plot(responses: &Vec<PingResult>) {
    let mut seq: u32 = 0;
    let mut points: Vec<(f32, f32)> = Vec::new();
    let (w, _h) = dimensions().unwrap();
    let mut width: u32 = w.try_into().unwrap();
    width *= 2;
    width -= 14;

    for response in responses {
        points.push((seq as f32, response.latency_ms as f32));
        seq += 1;
    }

    println!("");
    Chart::new(width, 120, 0.0, seq as f32)
        .lineplot(&Shape::Lines(&points[..]))
        .display();
    println!("");
}

fn report(result: &PingRunResult) {
    let percent_succeeded: f64 = cast::f64(result.succeeded) / cast::f64(result.total) * 100.0;

    println!(
        "Total: {}, Succeeded: {}, Failed: {}, %: {:.2}",
        result.total, result.succeeded, result.failed, percent_succeeded
    );
    println!(
        "Max: {} ms, Min: {} ms, Avg: {} ms",
        result.max_latency, result.min_latency, result.average_latency
    );
}

pub fn run(host: &str, number: u64, timeout: u64, size: u64, draw_plot: bool) {
    let mut responses: Vec<PingResult> = Vec::new();
    for _ in 0..number {
        let response = ping(host, timeout, size);

        if response.dropped == true {
            print!(".");
        } else {
            print!("!");
        }
        io::stdout().flush().unwrap();

        responses.push(response);
    }
    println!("");

    let result = average(&responses);
    if draw_plot {
        plot(&responses);
    }
    report(&result);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn test_average() {
        let mut responses: Vec<PingResult> = Vec::new();
        responses.push(PingResult {
            dropped: false,
            latency_ms: 100,
        });
        responses.push(PingResult {
            dropped: true,
            latency_ms: 300,
        });
        let result = average(&responses);
        assert_eq!(result.average_latency, 200);
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.max_latency, 300);
        assert_eq!(result.min_latency, 100);
        assert_eq!(result.total, 2);
    }

    #[test]
    fn test_resolve_host_ipv4() {
        let result = resolve_host("127.0.0.1");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)));
    }

    #[test]
    fn test_resolve_host_ipv6() {
        let result = resolve_host("::1");
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_resolve_host_localhost() {
        let result = resolve_host("localhost");
        assert!(result.is_ok());
        // localhost should resolve to either 127.0.0.1 or ::1
        let ip = result.unwrap();
        assert!(
            ip == IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
                || ip == IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1))
        );
    }

    #[test]
    fn test_resolve_host_invalid() {
        let result = resolve_host("invalid.domain.that.does.not.exist.12345");
        assert!(result.is_err());
    }

    #[test]
    fn test_build_icmpv4_packet() {
        let packet = build_icmpv4_packet(8);

        // Should be 8 (header) + 8 (size) = 16 bytes
        assert_eq!(packet.len(), 16);

        // Check ICMP header fields
        assert_eq!(packet[0], 8); // Type: Echo Request
        assert_eq!(packet[1], 0); // Code: 0
        assert_eq!(packet[4], 0); // Identifier high byte
        assert_eq!(packet[5], 1); // Identifier low byte
        assert_eq!(packet[6], 0); // Sequence high byte
        assert_eq!(packet[7], 1); // Sequence low byte

        // Check data pattern
        for i in 8..packet.len() {
            assert_eq!(packet[i], (i % 256) as u8);
        }

        // Checksum should be calculated (non-zero)
        assert!(packet[2] != 0 || packet[3] != 0);
    }

    #[test]
    fn test_build_icmpv6_packet() {
        let packet = build_icmpv6_packet(8);

        // Should be 8 (header) + 8 (size) = 16 bytes
        assert_eq!(packet.len(), 16);

        // Check ICMPv6 header fields
        assert_eq!(packet[0], 128); // Type: Echo Request (ICMPv6)
        assert_eq!(packet[1], 0); // Code: 0
        assert_eq!(packet[2], 0); // Checksum (calculated by kernel)
        assert_eq!(packet[3], 0); // Checksum
        assert_eq!(packet[4], 0); // Identifier high byte
        assert_eq!(packet[5], 1); // Identifier low byte
        assert_eq!(packet[6], 0); // Sequence high byte
        assert_eq!(packet[7], 1); // Sequence low byte

        // Check data pattern
        for i in 8..packet.len() {
            assert_eq!(packet[i], (i % 256) as u8);
        }
    }

    #[test]
    fn test_validate_icmpv4_reply_valid() {
        // Create a mock IPv4 ICMP Echo Reply packet
        let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Fill IP header (20 bytes) + ICMP header (8 bytes)
        for i in 0..28 {
            buffer[i] = std::mem::MaybeUninit::new(0);
        }
        // Set ICMP type to Echo Reply (0) at offset 20
        buffer[20] = std::mem::MaybeUninit::new(0);

        assert!(validate_icmpv4_reply(&buffer, 28));
    }

    #[test]
    fn test_validate_icmpv4_reply_invalid_type() {
        let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Fill IP header (20 bytes) + ICMP header (8 bytes)
        for i in 0..28 {
            buffer[i] = std::mem::MaybeUninit::new(0);
        }
        // Set ICMP type to something other than Echo Reply
        buffer[20] = std::mem::MaybeUninit::new(8); // Echo Request instead of Reply

        assert!(!validate_icmpv4_reply(&buffer, 28));
    }

    #[test]
    fn test_validate_icmpv4_reply_too_short() {
        let buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Packet too short (less than 28 bytes)
        assert!(!validate_icmpv4_reply(&buffer, 20));
    }

    #[test]
    fn test_validate_icmpv6_reply_valid() {
        let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Set ICMPv6 type to Echo Reply (129) at offset 0
        buffer[0] = std::mem::MaybeUninit::new(129);
        for i in 1..8 {
            buffer[i] = std::mem::MaybeUninit::new(0);
        }

        assert!(validate_icmpv6_reply(&buffer, 8));
    }

    #[test]
    fn test_validate_icmpv6_reply_invalid_type() {
        let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Set ICMPv6 type to something other than Echo Reply
        buffer[0] = std::mem::MaybeUninit::new(128); // Echo Request instead of Reply
        for i in 1..8 {
            buffer[i] = std::mem::MaybeUninit::new(0);
        }

        assert!(!validate_icmpv6_reply(&buffer, 8));
    }

    #[test]
    fn test_validate_icmpv6_reply_too_short() {
        let buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];

        // Packet too short (less than 8 bytes)
        assert!(!validate_icmpv6_reply(&buffer, 4));
    }

    #[test]
    fn test_calculate_icmp_checksum() {
        // Test with a simple packet
        let mut packet = vec![8, 0, 0, 0, 0, 1, 0, 1]; // Basic ICMP header

        // Clear checksum field
        packet[2] = 0;
        packet[3] = 0;

        let checksum = calculate_icmp_checksum(&packet);

        // Checksum should be non-zero for this packet
        assert_ne!(checksum, 0);

        // Verify checksum by including it in the packet and recalculating
        packet[2] = (checksum >> 8) as u8;
        packet[3] = (checksum & 0xff) as u8;

        let verify_checksum = calculate_icmp_checksum(&packet);
        assert_eq!(verify_checksum, 0); // Should be 0 when checksum is correct
    }
}
