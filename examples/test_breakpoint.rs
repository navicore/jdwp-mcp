// Test setting a breakpoint on HelloController.hello()

use jdwp_client::{JdwpConnection, SuspendPolicy};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("jdwp_client=debug")
        .init();

    println!("🔧 Setting up breakpoint test...\n");

    let mut conn = JdwpConnection::connect("localhost", 5005).await?;
    println!("✓ Connected to JVM\n");

    // Find HelloController class
    let signature = "Lcom/example/probedemo/HelloController;";
    let classes = conn.classes_by_signature(signature).await?;
    let class = &classes[0];
    println!("✓ Found HelloController (type_id: {:x})", class.type_id);

    // Get methods
    let methods = conn.get_methods(class.type_id).await?;
    let hello_method = methods.iter().find(|m| m.name == "hello").unwrap();
    println!("✓ Found hello() method (method_id: {:x})", hello_method.method_id);

    // Get line table
    let line_table = conn.get_line_table(class.type_id, hello_method.method_id).await?;

    // Find line 64 (helloCounter.increment())
    let line_64 = line_table.lines.iter().find(|e| e.line_number == 64).unwrap();
    println!("✓ Line 64 → bytecode index: {}", line_64.line_code_index);

    // Set breakpoint!
    println!("\n🎯 Setting breakpoint at HelloController.java:64...");
    let request_id = conn.set_breakpoint(
        class.type_id,
        hello_method.method_id,
        line_64.line_code_index,
        SuspendPolicy::All,  // Suspend all threads when hit
    ).await?;

    println!("✅ Breakpoint set! Request ID: {}", request_id);
    println!("\n📍 Breakpoint is active at:");
    println!("   Class: com.example.probedemo.HelloController");
    println!("   Method: hello()");
    println!("   Line: 64");
    println!("   Bytecode index: {}", line_64.line_code_index);

    println!("\n💡 Try hitting the endpoint:");
    println!("   curl http://localhost:30080/");
    println!("\n   The JVM should pause when the breakpoint is hit!");
    println!("   (Press Ctrl+C to stop this test)");

    // Keep the connection alive
    tokio::signal::ctrl_c().await?;

    println!("\n🧹 Cleaning up...");
    conn.clear_breakpoint(request_id).await?;
    println!("✓ Breakpoint cleared");

    Ok(())
}
