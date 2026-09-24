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
    // gRPC Reason 契约（anaphase 同款）：独立输出到 OUT_DIR/grpc，
    // 与 prost_build 的 5 个文件互不污染（单一职责：一套 proto 一套生成物）。
    let grpc_out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("grpc");
    std::fs::create_dir_all(&grpc_out)?;
    tonic_build::configure()
        .out_dir(&grpc_out)
        .build_server(true)
        .build_client(false)
        .compile(&["proto/flowmodus.proto"], &["proto"])?;
    println!("cargo:rerun-if-changed=proto");
    Ok(())
}
