fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Only build protobuf if grpc feature is enabled
    #[cfg(feature = "grpc")]
    {
        if std::path::Path::new("proto/orderbook_service.proto").exists() {
            // Try to compile protobuf, but don't fail the build if protoc is not available
            match tonic_build::configure()
                .build_server(true)
                .build_client(true)
                .compile(&["proto/orderbook_service.proto"], &["proto"])
            {
                Ok(_) => println!("cargo:warning=Successfully compiled protobuf files"),
                Err(e) => {
                    println!("cargo:warning=Failed to compile protobuf files: {}. Install protoc with: brew install protobuf", e);
                    println!("cargo:warning=Skipping gRPC server compilation");
                }
            }
        } else {
            println!("cargo:warning=Proto file not found, skipping protobuf compilation");
        }
    }

    #[cfg(not(feature = "grpc"))]
    {
        println!("cargo:warning=gRPC feature not enabled, skipping protobuf compilation");
    }

    Ok(())
}
