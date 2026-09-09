//! Compile the 5 FlowModus protobuf contracts into Rust types.
//! Contracts are the immutable schema boundary (DNA iron law 5).
//! All messages derive serde so registry JSON (SupplierDeclaration files)
//! round-trips deterministically (度量衡: JSON file == typed record).

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/gossip.proto",
        "proto/metrics.proto",
        "proto/registry.proto",
        "proto/routing.proto",
        "proto/supplier.proto",
    ];
    prost_build::Config::new()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&protos, &["proto"])?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
