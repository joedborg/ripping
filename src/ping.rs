use oping::{Ping, PingItem};
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use term_size::dimensions;
use textplots::{Chart, Plot, Shape};

struct PingRunResult {
    total: u64,
    succeeded: u64,
    failed: u64,
    max_latency: f64,
    min_latency: f64,
    average_latency: f64,
}

fn ping(host: &str, timeout: f64) -> PingItem {
    let mut ping = Ping::new();
    ping.set_timeout(timeout).unwrap();
    ping.add_host(host).unwrap();
    let responses = ping.send().unwrap_or_else(|_| {
        println!("Cannot send ping.  Try running with `sudo`.");
        std::process::exit(1);
    });
    return responses.last().unwrap();
}

fn average(responses: &Vec<PingItem>) -> PingRunResult {
    let mut result = PingRunResult {
        total: 0,
        succeeded: 0,
        failed: 0,
        max_latency: 0.0,
        min_latency: 0.0,
        average_latency: 0.0,
    };

    for response in responses {
        result.total += 1;

        if response.dropped == 1 {
            result.failed += 1;
        } else {
            result.succeeded += 1;
        }

        if response.latency_ms > result.max_latency {
            result.max_latency = response.latency_ms;
        }

        if response.latency_ms < result.min_latency || result.min_latency == 0.0 {
            if response.latency_ms > 0.0 {
                result.min_latency = response.latency_ms;
            }
        }

        result.average_latency += response.latency_ms;
    }

    result.average_latency /= cast::f64(result.total);

    return result;
}

fn plot(responses: &Vec<PingItem>) {
    let mut seq: f32 = 0.0;
    let mut points: Vec<(f32, f32)> = Vec::new();
    let (w, _h) = dimensions().unwrap();
    let mut width: u32 = w.try_into().unwrap();
    width *= 2;
    width -= 14;

    for response in responses {
        points.push((seq, cast::f32(response.latency_ms).unwrap()));
        seq += 1.0;
    }

    println!("");
    Chart::new(width, 120, 0.0, seq)
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
        "Max: {:.3}, Min: {:.3}, Avg: {:.3}",
        result.max_latency, result.min_latency, result.average_latency
    );
}

pub fn run(host: &str, number: u32, timeout: f64, draw_plot: bool) {
    let mut responses: Vec<PingItem> = Vec::new();
    for _ in 0..number {
        let response = ping(host, timeout);

        if response.dropped == 1 {
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
    use oping::AddrFamily;

    #[test]
    fn test_ping() {
        let host = "127.0.0.1";
        let response = ping(host, 0.1);
        assert_eq!(response.address, host);
    }

    #[test]
    fn test_average() {
        let mut responses: Vec<PingItem> = Vec::new();
        responses.push(PingItem {
            address: "127.0.0.1".to_string(),
            hostname: "127.0.0.1".to_string(),
            dropped: 0,
            latency_ms: 0.1,
            family: AddrFamily::IPV4,
            recv_qos: 0,
            recv_ttl: 0,
            seq: 0,
        });
        responses.push(PingItem {
            address: "127.0.0.1".to_string(),
            hostname: "127.0.0.1".to_string(),
            dropped: 1,
            latency_ms: 0.3,
            family: AddrFamily::IPV4,
            recv_qos: 0,
            recv_ttl: 0,
            seq: 0,
        });

        let result = average(&responses);
        assert_eq!(result.average_latency, 0.2);
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.max_latency, 0.3);
        assert_eq!(result.min_latency, 0.1);
        assert_eq!(result.total, 2);
    }
}
