use std::thread::sleep;

use client_rs::corev1::CoreV1Client;
use client_rs::rest;

use clap::Parser;

use apimachinery::watch::ResourceWatcher;

#[derive(Parser, Debug)]
struct Options {
    // Namespace of the pod
    #[arg(short, long)]
    namespace: String,

    // Name of the pod
    #[arg(short, long)]
    pod: String,

    #[arg(short, long)]
    cluster_url: String,

    #[arg(short, long, env = "KUBE_TOKEN")]
    token: String,
}

fn main() {
    let opts = Options::parse();
    let rest_client = rest::rest_client_for(&rest::Config {
        base_url: opts.cluster_url.to_string(),
        user_agent: None,
        bearer_token: opts.token.to_string().into(),
    });

    let corev1_client = CoreV1Client::new(&rest_client);
    let pod_client = corev1_client.pods(&opts.namespace);
    let mut watcher = pod_client.watch(&opts.pod).expect("Failed to get pod");

    loop {
        match watcher.next() {
            Ok(event) => println!("{event:?}"),
            Err(e) => {
                if e == "Request timed out" {
                    eprint!("{e}");
                    break;
                }
                eprintln!("Watch error: {e}");
                sleep(std::time::Duration::from_secs(1));
                continue;
            }
        }
    }
}
