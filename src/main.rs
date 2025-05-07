use {
    anyhow::{anyhow, Result},
    clap::Parser,
    futures::future::join_all,
    jito_protos::shredstream::{
        shredstream_proxy_client::ShredstreamProxyClient, SubscribeEntriesRequest,
    },
    latency_collector::LatencyCollector,
    reqwest::Client,
    serde_json::json,
    solana_entry::entry::Entry,
    std::{
        collections::HashSet,
        env, io,
        str::FromStr,
        sync::Arc,
        time::{Duration, SystemTime, UNIX_EPOCH},
    },
    tokio::{sync::Mutex, time::sleep},
    tonic::transport::Endpoint,
};

#[derive(Debug, Clone, Parser)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value_t = String::from("http://127.0.0.1:9999"))]
    shredstream_proxy_uri: String,

    /// RPC URL to use for fetching block times (required)
    #[clap(long, required = true)]
    rpc_url: String,

    /// Timeout in seconds (default: 60)
    #[clap(long, default_value_t = 60)]
    timeout_dur: u64,
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    env::set_var(
        env_logger::DEFAULT_FILTER_ENV,
        env::var_os(env_logger::DEFAULT_FILTER_ENV).unwrap_or_else(|| "info".into()),
    );
    env_logger::init();

    let args = Args::parse();
    let lc = Arc::new(Mutex::new(LatencyCollector::new()));

    let timeout = tokio::time::sleep(Duration::from_secs(args.timeout_dur));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => {
                println!("Timeout reached. Generating report...");
                let collector = lc.lock().await;
                collector.generate_report();
                break;
            }
            result = connect_and_stream(&args.shredstream_proxy_uri, &args.rpc_url, lc.clone()) => {
                if let Err(e) = result {
                    eprintln!("Stream error: {e}. Retrying...");
                } else {
                    println!("Stream ended gracefully. Reconnecting...");
                }
                sleep(Duration::from_secs(1)).await;
            }
        }
    }

    Ok(())
}

async fn connect_and_stream(
    endpoint: &str,
    rpc_url: &str,
    lc: Arc<Mutex<LatencyCollector>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let endpoint = Endpoint::from_str(endpoint)?
        .keep_alive_while_idle(true)
        .http2_keep_alive_interval(Duration::from_secs(5))
        .keep_alive_timeout(Duration::from_secs(10))
        .tcp_keepalive(Some(Duration::from_secs(15)))
        .connect_timeout(Duration::from_secs(5));

    let channel = endpoint.connect().await?;
    let mut client = ShredstreamProxyClient::new(channel);

    let mut stream = client
        .subscribe_entries(SubscribeEntriesRequest {})
        .await?
        .into_inner();

    let mut unique_slots = HashSet::new();
    let http_client = Client::new();
    let mut rpc_tasks = Vec::new();

    while let Some(result) = stream.message().await.transpose() {
        match result {
            Ok(slot_entry) => {
                let entries = match bincode::deserialize::<Vec<Entry>>(&slot_entry.entries) {
                    Ok(e) => e,
                    Err(e) => {
                        eprintln!("Deserialization failed: {e}");
                        continue;
                    }
                };

                if unique_slots.insert(slot_entry.slot) {
                    let received_timestamp = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .expect("Time went backwards")
                        .as_millis() as u64;

                    let http_client = http_client.clone();
                    let slot = slot_entry.slot;
                    let lc = lc.clone();
                    let rpc_url = rpc_url.to_string();

                    let task = tokio::spawn(async move {
                        match get_block_time(&http_client, &rpc_url, slot).await {
                            Ok(block_time) => {
                                let mut collector = lc.lock().await;
                                collector.collect_data(block_time, received_timestamp);
                                let latency = received_timestamp - block_time;
                                println!("Latency for slot {}: {} ms", slot, latency);
                            }
                            Err(e) => {
                                if e.to_string() == "Block not available for slot" {
                                    let mut collector = lc.lock().await;
                                    collector.collect_data(received_timestamp, received_timestamp);
                                    println!(
                                        "Assumed latency for slot {}: 0 ms (Block not available for slot)",
                                        slot
                                    );
                                } else {
                                    eprintln!("Failed to get block time for slot {}: {}", slot, e);
                                }
                            }
                        }
                    });

                    rpc_tasks.push(task);
                }

                println!(
                    "slot {}, entries: {}, transactions: {}",
                    slot_entry.slot,
                    entries.len(),
                    entries.iter().map(|e| e.transactions.len()).sum::<usize>()
                );
            }
            Err(e) => {
                eprintln!("stream error: {e}");
                return Err(Box::new(e));
            }
        }
    }

    join_all(rpc_tasks).await;
    Ok(())
}

async fn get_block_time(client: &Client, rpc_url: &str, slot: u64) -> Result<u64, anyhow::Error> {
    let response = client
        .post(rpc_url)
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getBlockTime",
            "params": [slot]
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                let json_response = resp.json::<serde_json::Value>().await?;
                if let Some(result) = json_response["result"].as_u64() {
                    Ok(result * 1000)
                } else if json_response["error"]["message"]
                    == format!("Block not available for slot {}", slot)
                {
                    Err(anyhow::Error::msg("Block not available for slot"))
                } else {
                    Err(anyhow!(
                        "Invalid response from getBlockTime: {}",
                        json_response
                    ))
                }
            } else {
                let status = resp.status();
                let text = resp.text().await?;
                Err(anyhow!("HTTP error {}: {}", status, text))
            }
        }
        Err(e) => Err(anyhow!("Request error: {}", e)),
    }
}
