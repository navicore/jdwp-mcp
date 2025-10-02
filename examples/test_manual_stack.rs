// Simpler stack inspection: manually suspend and inspect

use jdwp_client::JdwpConnection;
use jdwp_client::stackframe::VariableSlot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("jdwp_client=info")
        .init();

    println!("🔍 Manual Stack Inspection Test\n");
    println!("📌 This test will:");
    println!("   1. Suspend the JVM");
    println!("   2. Inspect all thread stacks");
    println!("   3. Show variables from frames");
    println!("   4. Resume execution\n");

    let mut conn = JdwpConnection::connect("localhost", 5005).await?;
    println!("✓ Connected to JVM\n");

    // First, make a request to ensure hello() is on the stack
    println!("💡 Making request to trigger hello()...");
    tokio::task::spawn(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        let _ = reqwest::get("http://localhost:8080/").await;
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    println!("⏸️  Suspending all threads...");
    conn.suspend_all().await?;
    println!("✓ JVM suspended\n");

    // Get all threads
    let threads = conn.get_all_threads().await?;
    println!("📋 Found {} threads\n", threads.len());

    // Find HelloController class for comparison
    let classes = conn.classes_by_signature("Lcom/example/probedemo/HelloController;").await?;
    let hello_class_id = if !classes.is_empty() { Some(classes[0].type_id) } else { None };

    // Inspect each thread
    for (idx, thread_id) in threads.iter().enumerate().take(5) {
        println!("🧵 Thread {} (ID: {:x})", idx + 1, thread_id);

        match conn.get_frames(*thread_id, 0, 3).await {
            Ok(frames) if !frames.is_empty() => {
                for (fidx, frame) in frames.iter().enumerate() {
                    println!("  Frame {}: class_id={:x}, method_id={:x}, index={}",
                        fidx, frame.location.class_id, frame.location.method_id, frame.location.index);

                    // If this is HelloController, inspect variables
                    if Some(frame.location.class_id) == hello_class_id {
                        println!("    ⭐ This is HelloController!");

                        // Get methods to find method name
                        let methods = conn.get_methods(frame.location.class_id).await?;
                        if let Some(method) = methods.iter().find(|m| m.method_id == frame.location.method_id) {
                            println!("       Method: {}", method.name);

                            // Try to get variable table
                            match conn.get_variable_table(frame.location.class_id, frame.location.method_id).await {
                                Ok(var_table) => {
                                    let current_index = frame.location.index;
                                    let active_vars: Vec<_> = var_table
                                        .iter()
                                        .filter(|v| {
                                            current_index >= v.code_index
                                                && current_index < v.code_index + v.length as u64
                                        })
                                        .collect();

                                    if !active_vars.is_empty() {
                                        println!("       Variables:");

                                        let slots: Vec<VariableSlot> = active_vars
                                            .iter()
                                            .map(|v| VariableSlot {
                                                slot: v.slot as i32,
                                                sig_byte: v.signature.as_bytes()[0],
                                            })
                                            .collect();

                                        match conn.get_frame_values(*thread_id, frame.frame_id, slots).await {
                                            Ok(values) => {
                                                for (var, value) in active_vars.iter().zip(values.iter()) {
                                                    println!("         {} = {}", var.name, value.format());
                                                }
                                            }
                                            Err(e) => println!("         Error getting values: {}", e),
                                        }
                                    }
                                }
                                Err(e) => println!("       No variable table: {}", e),
                            }
                        }
                    }
                }
                println!();
            }
            Ok(_) => println!("  (no frames)\n"),
            Err(e) => println!("  Error: {}\n", e),
        }
    }

    println!("▶️  Resuming execution...");
    conn.resume_all().await?;
    println!("✓ JVM resumed\n");

    println!("✅ Stack inspection complete!");

    Ok(())
}
