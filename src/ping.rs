extern crate cast;

use oping::{Ping, PingItem};

struct PingRunResult {
    total: u64,
    succeeded: u64,
    failed: u64,
    max_latency: f64,
    min_latency: f64,
    average_latency: f64
}

fn ping(host: &str, timeout: f64) -> PingItem {
    let mut ping = Ping::new();
    ping.set_timeout(timeout).unwrap();
    ping.add_host(host).unwrap();
    let responses = ping.send().unwrap();
    return responses.last().unwrap();
}

fn average(responses: Vec<PingItem>) -> PingRunResult {
    let mut result = PingRunResult{
        total: 0,
        succeeded: 0,
        failed: 0,
        max_latency: 0.0,
        min_latency: 0.0,
        average_latency: 0.0
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
            result.min_latency = response.latency_ms;
        }

        result.average_latency += response.latency_ms;
    }

    result.average_latency /= cast::f64(result.total);

    return result;
}

fn report(result: PingRunResult) {
    let percent_succeeded: f64 = 
        cast::f64(result.succeeded) / cast::f64(result.total) * 100.0;

    println!(
        "Total: {}, Succeeded: {}, Failed: {}, %: {:.3}",
        result.total, result.succeeded, result.failed, percent_succeeded
    );
    println!(
        "Max: {:.3}, Min: {:.3}, Avg: {:.3}",
        result.max_latency, result.min_latency, result.average_latency
    );
}

pub fn run(host: &str, number: u32, timeout: f64) {
    let mut responses: Vec<PingItem> = Vec::new();
    for _ in 0..number {
        let response = ping(host, timeout);

        if response.dropped == 1 {
            print!(".")
        } else {
            print!("!")
        }

        responses.push(response);
    }
    println!("");

    let result = average(responses);
    report(result);
}
