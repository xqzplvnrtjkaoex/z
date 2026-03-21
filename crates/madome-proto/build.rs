fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_files = &[
        "../../proto/auth.proto",
        "../../proto/catalog.proto",
        "../../proto/user.proto",
    ];

    let include_dirs = &["../../proto"];

    tonic_prost_build::configure()
        .build_server(true)
        .build_client(true)
        .compile_protos(proto_files, include_dirs)?;

    // Rerun if any proto file changes
    for proto in proto_files {
        println!("cargo:rerun-if-changed={proto}");
    }

    Ok(())
}
