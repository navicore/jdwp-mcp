// Test VirtualMachine commands (Version and IDSizes)

use jdwp_client::JdwpConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Enable tracing
    tracing_subscriber::fmt()
        .with_env_filter("jdwp_client=debug")
        .init();

    println!("Connecting to JDWP at localhost:5005...");
    let mut connection = JdwpConnection::connect("localhost", 5005).await?;
    println!("✓ Connected\n");

    // Get version info
    println!("Fetching VM version...");
    let version = connection.get_version().await?;
    println!("✓ Version received:");
    println!("  Description: {}", version.description);
    println!("  JDWP: {}.{}", version.jdwp_major, version.jdwp_minor);
    println!("  VM Version: {}", version.vm_version);
    println!("  VM Name: {}", version.vm_name);
    println!();

    // Get ID sizes
    println!("Fetching ID sizes...");
    let id_sizes = connection.get_id_sizes().await?;
    println!("✓ ID sizes received:");
    println!("  Field ID: {} bytes", id_sizes.field_id_size);
    println!("  Method ID: {} bytes", id_sizes.method_id_size);
    println!("  Object ID: {} bytes", id_sizes.object_id_size);
    println!("  ReferenceType ID: {} bytes", id_sizes.reference_type_id_size);
    println!("  Frame ID: {} bytes", id_sizes.frame_id_size);

    println!("\n🎉 All VM commands working!");

    Ok(())
}
