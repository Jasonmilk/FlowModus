//! Compile the 5 FlowModus protobuf contracts into Rust types.
//! Contracts are the immutable schema boundary (DNA iron law 5).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/gossip.proto",
        "proto/metrics.proto",
        "proto/registry.proto",
        "proto/routing.proto",
        "proto/supplier.proto",
    ];
    prost_build::Config::new()
        .compile_protos(&protos, &["proto"])?;
    Ok(())
}
