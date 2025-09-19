# Typed Kubernetes client for Rust

This repository contains an experimental, learning project that tries to implement
a basic Kubernetes client library in Rust.

## Examples

There are some examples in the `examples/` folder. To run them:
```bash
cd examples/
KUBE_TOKEN=$(k create token -n default default)
cargo run --example <example_name> --cluster-url <cluster_url> -t $KUBE_TOKEN [additional-flags-here]
```
