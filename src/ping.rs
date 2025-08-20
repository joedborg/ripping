use socket2::{Domain, Protocol, Socket, Type};
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use std::net::{IpAddr, Ipv4Addr};
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
    !sum as u16
}

fn ping(host: &str, timeout: u64, size: u64) -> PingResult {
    // Convert timeout from milliseconds to Duration
    let timeout_duration = Duration::from_millis(timeout);

    // Try to parse the host IP
    let ip: Ipv4Addr = match host.parse() {
        Ok(ip) => ip,
        Err(e) => {
            println!("Failed to parse host IP: {}", e);
            std::process::exit(1);
        }
    };

    // Create raw ICMP socket
    let socket = match Socket::new_raw(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)) {
        Ok(socket) => socket,
        Err(_) => {
            println!("Failed to create socket.  You may need to run with `sudo`.");
            std::process::exit(1);
        }
    };

    socket
        .set_read_timeout(Some(timeout_duration))
        .expect("Failed to set read timeout");

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

    // Send packet
    let start_time = Instant::now();
    let addr = std::net::SocketAddr::new(IpAddr::V4(ip), 0);
    if socket.send_to(&packet, &addr.into()).is_err() {
        return PingResult {
            dropped: true,
            latency_ms: timeout,
        };
    }

    // Receive reply
    let mut buffer = [std::mem::MaybeUninit::<u8>::uninit(); 1024];
    match socket.recv_from(&mut buffer) {
        Ok((bytes_received, _)) => {
            let end_time = Instant::now();
            let latency = end_time.duration_since(start_time).as_millis() as u64;

            // Basic validation - check if it's an ICMP Echo Reply
            if bytes_received >= 28 {
                // IP header (20) + ICMP header (8)
                let icmp_type = unsafe { buffer[20].assume_init() }; // ICMP type is at offset 20 (after IP header)
                if icmp_type == 0 {
                    // Echo Reply
                    return PingResult {
                        dropped: false,
                        latency_ms: latency,
                    };
                }
            }
            // Invalid reply
            return PingResult {
                dropped: true,
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
        "Total: {}, Succeeded: {}, Failed: {}, %: {:.3}",
        result.total, result.succeeded, result.failed, percent_succeeded
    );
    println!(
        "Max: {:.3} ms, Min: {:.3} ms, Avg: {:.3} ms",
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
        io::stdout().flush().expect("Failed to flush stdout");

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
}
