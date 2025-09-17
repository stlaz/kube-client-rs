use client_rs::corev1::CoreV1Client;
use client_rs::rest;

use clap::Parser;

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
    let resp = corev1_client
        .pods(&opts.namespace)
        .get(&opts.pod)
        .expect("Failed to get pod");
    println!("Response: {:#?}", resp);

    let resp2 = corev1_client
        .pods(&opts.namespace)
        .list()
        .expect("Failed to list pods");

    println!("Response: {:#?}", resp2);
}
