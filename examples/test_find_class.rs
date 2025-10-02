// Test finding classes by signature

use jdwp_client::JdwpConnection;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("jdwp_client=debug")
        .init();

    println!("Connecting to JDWP...");
    let mut conn = JdwpConnection::connect("localhost", 5005).await?;
    println!("✓ Connected\n");

    // Find HelloController class
    // Java class signature format: Lpackage/path/ClassName;
    let signature = "Lcom/example/probedemo/HelloController;";
    println!("Looking for class: {}", signature);

    let classes = conn.classes_by_signature(signature).await?;

    if classes.is_empty() {
        println!("❌ Class not found!");
        return Ok(());
    }

    let class = &classes[0];
    println!("✓ Found class:");
    println!("  Type ID: {:x}", class.type_id);
    println!("  Tag: {} (1=class, 2=interface, 3=array)", class.ref_type_tag);
    println!("  Status: {} (verified, prepared, initialized, error)\n", class.status);

    // Get methods
    println!("Getting methods...");
    let methods = conn.get_methods(class.type_id).await?;
    println!("✓ Found {} methods:", methods.len());

    for method in &methods {
        println!("  {} - {}{}", method.name, method.name, method.signature);
        println!("    Method ID: {:x}", method.method_id);
    }

    // Find the hello() method
    let hello_method = methods.iter().find(|m| m.name == "hello");

    if let Some(method) = hello_method {
        println!("\n✓ Found hello() method!");
        println!("  Getting line table...");

        let line_table = conn.get_line_table(class.type_id, method.method_id).await?;
        println!("  Line table: {} entries", line_table.lines.len());
        println!("  Bytecode range: {} - {}", line_table.start, line_table.end);

        for entry in &line_table.lines {
            println!("    Line {} → bytecode index {}", entry.line_number, entry.line_code_index);
        }

        // Find bytecode position for line 65 (inside hello method)
        if let Some(line_entry) = line_table.lines.iter().find(|e| e.line_number == 65) {
            println!("\n🎯 Line 65 maps to bytecode index: {}", line_entry.line_code_index);
        }
    }

    Ok(())
}
