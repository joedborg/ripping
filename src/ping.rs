use single_ping::{ping, PingResult};
use std::convert::TryInto;
use std::io;
use std::io::prelude::*;
use term_size::dimensions;
use textplots::{Chart, Plot, Shape};

struct PingRunResult {
    total: u64,
    succeeded: u64,
    failed: u64,
    max_latency: u64,
    min_latency: u64,
    average_latency: u64,
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

        if response.dropped {
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
        let response = ping(host, timeout, size).unwrap_or_else(|e| {
            println!("{}", e);
            std::process::exit(1);
        });

        if response.dropped {
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

    #[test]
    fn test_average() {
        let responses = vec![
            PingResult {
                dropped: false,
                latency_ms: 100,
            },
            PingResult {
                dropped: true,
                latency_ms: 300,
            },
        ];
        let result = average(&responses);
        assert_eq!(result.average_latency, 200);
        assert_eq!(result.succeeded, 1);
        assert_eq!(result.failed, 1);
        assert_eq!(result.max_latency, 300);
        assert_eq!(result.min_latency, 100);
        assert_eq!(result.total, 2);
    }
}
