use clap::Parser;
use pingora::prelude::*;
use pingora::upstreams::peer::HttpPeer;
use url::Url;
use bytes::Bytes;
use std::time::{Instant, Duration};
use http::{HeaderName, HeaderValue};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// URL to connect to
    #[arg(long)]
    url: String,

    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    #[arg(long, default_value = "GET")]
    method: String,

    /// Request body (for POST, PUT, etc.)
    #[arg(long)]
    body: Option<String>,

    /// Allow insecure connections (skip certificate verification)
    #[arg(long, default_value_t = false)]
    insecure: bool,

    /// Duration in seconds to run the benchmark (0 = single request)
    #[arg(long, default_value_t = 0)]
    duration: u64,

    /// Number of requests to make (0 = unlimited, use with --duration)
    #[arg(short = 'n', long, default_value_t = 0)]
    requests: u64,

    /// Custom headers to include in requests (format: "Key: Value")
    #[arg(short = 'H', long = "header", value_name = "HEADER")]
    headers: Vec<String>,
}

struct BenchStats {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    total_duration: Duration,
    min_latency: Duration,
    max_latency: Duration,
    latencies: Vec<Duration>,
}

impl BenchStats {
    fn new() -> Self {
        Self {
            total_requests: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration: Duration::ZERO,
            min_latency: Duration::MAX,
            max_latency: Duration::ZERO,
            latencies: Vec::new(),
        }
    }

    fn add_success(&mut self, latency: Duration) {
        self.total_requests += 1;
        self.successful_requests += 1;
        self.latencies.push(latency);
        
        if latency < self.min_latency {
            self.min_latency = latency;
        }
        if latency > self.max_latency {
            self.max_latency = latency;
        }
    }

    fn add_failure(&mut self) {
        self.total_requests += 1;
        self.failed_requests += 1;
    }

    fn avg_latency(&self) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let total: Duration = self.latencies.iter().sum();
        total / self.latencies.len() as u32
    }

    fn percentile(&self, p: f64) -> Duration {
        if self.latencies.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted = self.latencies.clone();
        sorted.sort();
        let idx = ((sorted.len() as f64 * p / 100.0).ceil() as usize).saturating_sub(1);
        sorted[idx]
    }

    fn print_summary(&self) {
        println!("\n=== Benchmark Results ===");
        println!("Total Duration: {:.3}s", self.total_duration.as_secs_f64());
        println!("Total Requests: {}", self.total_requests);
        println!("Successful: {}", self.successful_requests);
        println!("Failed: {}", self.failed_requests);
        println!("Requests/sec: {:.2}", 
                 self.total_requests as f64 / self.total_duration.as_secs_f64());
        
        if !self.latencies.is_empty() {
            println!("\n=== Latency Statistics ===");
            println!("Min: {:.3}ms", self.min_latency.as_secs_f64() * 1000.0);
            println!("Max: {:.3}ms", self.max_latency.as_secs_f64() * 1000.0);
            println!("Avg: {:.3}ms", self.avg_latency().as_secs_f64() * 1000.0);
            println!("P50: {:.3}ms", self.percentile(50.0).as_secs_f64() * 1000.0);
            println!("P90: {:.3}ms", self.percentile(90.0).as_secs_f64() * 1000.0);
            println!("P95: {:.3}ms", self.percentile(95.0).as_secs_f64() * 1000.0);
            println!("P99: {:.3}ms", self.percentile(99.0).as_secs_f64() * 1000.0);
        }
    }
}

async fn make_request(
    connector: &pingora::connectors::http::Connector,
    peer: &HttpPeer,
    method: &str,
    full_path: &str,
    host: &str,
    body: &Option<String>,
    headers: &[String],
    show_output: bool,
) -> Result<Duration> {
    let req_start = Instant::now();
    
    // Connect to the peer
    let (mut http, _reused) = connector.get_http_session(peer).await?;

    // Build request header
    let mut req = RequestHeader::build(method, full_path.as_bytes(), None)?;
    req.insert_header("Host", host)?;
    req.insert_header("User-Agent", "pingora-bench/0.1.0")?;

    // Add custom headers
    for header in headers {
        if let Some((key, value)) = header.split_once(':') {
            let header_name = HeaderName::from_bytes(key.trim().as_bytes())
                .map_err(|e| Error::because(ErrorType::InvalidHTTPHeader, "Invalid header name", e))?;
            let header_value = HeaderValue::from_str(value.trim())
                .map_err(|e| Error::because(ErrorType::InvalidHTTPHeader, "Invalid header value", e))?;
            req.headers.insert(header_name, header_value);
        }
    }

    // Add Content-Length header if body is provided
    if let Some(body) = body {
        req.insert_header("Content-Length", body.len().to_string())?;
        req.insert_header("Content-Type", "application/json")?;
    }

    // Send the request header
    http.write_request_header(Box::new(req)).await?;

    // Send the request body if provided
    if let Some(body) = body {
        http.write_request_body(Bytes::from(body.clone()), true).await?;
    } else {
        http.finish_request_body().await?;
    }

    // Read the response header
    http.read_response_header().await?;
    
    // Get the response header
    if show_output {
        if let Some(resp) = http.response_header() {
            println!("Response Status: {}", resp.status);
            println!("Response Headers:");
            for (name, value) in resp.headers.iter() {
                println!("  {}: {}", name, value.to_str().unwrap_or("<binary>"));
            }
            println!();
        }
    }

    // Read the response body
    if show_output {
        println!("Response Body:");
    }
    loop {
        match http.read_response_body().await? {
            Some(chunk) => {
                if show_output {
                    print!("{}", String::from_utf8_lossy(&chunk));
                }
            }
            None => break,
        }
    }
    if show_output {
        println!();
    }

    Ok(req_start.elapsed())
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Parse the URL
    let parsed_url = Url::parse(&args.url)
        .map_err(|e| Error::because(ErrorType::InvalidHTTPHeader, format!("Invalid URL: {}", e), e))?;

    let host = parsed_url.host_str()
        .ok_or_else(|| Error::explain(ErrorType::InvalidHTTPHeader, "URL must have a host"))?;
    
    let port = parsed_url.port_or_known_default()
        .ok_or_else(|| Error::explain(ErrorType::InvalidHTTPHeader, "Could not determine port"))?;

    let is_tls = parsed_url.scheme() == "https";
    let path = if parsed_url.path().is_empty() {
        "/"
    } else {
        parsed_url.path()
    };

    // Build the path with query string if present
    let full_path = if let Some(query) = parsed_url.query() {
        format!("{}?{}", path, query)
    } else {
        path.to_string()
    };

    // Define the Peer
    let mut peer = HttpPeer::new((host, port), is_tls, host.to_string());
    
    // Configure peer to skip certificate verification if insecure flag is set
    if is_tls && args.insecure {
        peer.options.verify_cert = false;
        peer.options.verify_hostname = false;
    }

    // Create a connector
    let connector = pingora::connectors::http::Connector::new(None);
    
    if args.duration == 0 {
        // Single request mode
        match make_request(&connector, &peer, &args.method, &full_path, host, &args.body, &args.headers, true).await {
            Ok(latency) => {
                println!("Request completed in: {:.3}s ({:.0}ms)", 
                         latency.as_secs_f64(), 
                         latency.as_millis());
            }
            Err(e) => {
                eprintln!("Request failed: {}", e);
                return Err(e);
            }
        }
    } else {
        // Benchmark mode
        let mut stats = BenchStats::new();
        let bench_start = Instant::now();
        let bench_duration = Duration::from_secs(args.duration);
        
        println!("Starting benchmark for {} seconds...", args.duration);
        if args.requests > 0 {
            println!("Request limit: {}", args.requests);
        }
        println!("URL: {}", args.url);
        println!("Method: {}", args.method);
        if args.body.is_some() {
            println!("Body: <provided>");
        }
        println!();

        while bench_start.elapsed() < bench_duration && (args.requests == 0 || stats.total_requests < args.requests) {
            match make_request(&connector, &peer, &args.method, &full_path, host, &args.body, &args.headers, false).await {
                Ok(latency) => {
                    stats.add_success(latency);
                    if stats.total_requests % 100 == 0 {
                        print!("\rRequests: {} | Elapsed: {:.1}s", 
                               stats.total_requests, 
                               bench_start.elapsed().as_secs_f64());
                        std::io::Write::flush(&mut std::io::stdout()).ok();
                    }
                }
                Err(e) => {
                    stats.add_failure();
                    eprintln!("\nRequest failed: {}", e);
                }
            }
        }
        
        stats.total_duration = bench_start.elapsed();
        println!("\n");
        stats.print_summary();
    }

    Ok(())
}