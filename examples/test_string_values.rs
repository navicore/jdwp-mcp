// Test string value retrieval
//
// This example demonstrates fetching actual string values from String objects

use jdwp_client::{JdwpConnection, spawn_event_loop};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Connecting to JVM at localhost:5005...");

    let mut connection = JdwpConnection::connect("localhost", 5005).await?;
    let (_event_loop, mut event_rx) = spawn_event_loop(connection.event_loop_handle());

    println!("✅ Connected!");

    // Get VM version
    let version = connection.version().await?;
    println!("📦 JVM Version: {}", version.vm_version);

    // Find HelloController class
    println!("\n🔍 Finding HelloController class...");
    let classes = connection.get_classes_by_signature("Lcom/example/probedemo/HelloController;").await?;

    if classes.is_empty() {
        eprintln!("❌ HelloController class not found. Make sure the app is running.");
        return Ok(());
    }

    let class_id = classes[0].type_id;
    println!("✅ Found class ID: {:x}", class_id);

    // Get methods to find debugTest
    println!("\n🔍 Finding debugTest method...");
    let methods = connection.get_methods(class_id).await?;

    let debug_test_method = methods.iter()
        .find(|m| m.name == "debugTest")
        .expect("debugTest method not found");

    println!("✅ Found debugTest method ID: {:x}", debug_test_method.method_id);

    // Get line table to find line 148
    let line_table = connection.get_line_table(class_id, debug_test_method.method_id).await?;

    // Find the line entry for line 148 (where we return the concatenated string)
    let line_entry = line_table.lines.iter()
        .find(|l| l.line_number == 148)
        .expect("Line 148 not found in line table");

    println!("✅ Line 148 is at code index: {}", line_entry.line_code_index);

    // Set breakpoint at line 148
    println!("\n⏸️  Setting breakpoint at line 148...");
    let _bp_id = connection.set_breakpoint(class_id, debug_test_method.method_id, line_entry.line_code_index).await?;
    println!("✅ Breakpoint set!");

    println!("\n📞 Trigger the breakpoint by running:");
    println!("   curl http://localhost:8080/debug-test");
    println!("\nWaiting for breakpoint to hit...");

    // Wait for breakpoint event
    let event_set = event_rx.recv().await
        .expect("No event received");

    println!("\n🎯 Breakpoint hit! Event: {:?}", event_set.suspend_policy);

    if let Some(event) = event_set.events.first() {
        if let jdwp_client::events::EventKind::Breakpoint { thread, .. } = event {
            println!("   Thread ID: {:x}", thread);

            // Get stack frames for this thread
            println!("\n📚 Getting stack frames...");
            let frames = connection.get_frames(thread, 0, -1).await?;

            if frames.is_empty() {
                println!("❌ No frames found");
                return Ok(());
            }

            let frame = &frames[0];
            println!("✅ Got {} frame(s)", frames.len());
            println!("   Frame 0: class={:x}, method={:x}", frame.location.class_id, frame.location.method_id);

            // Get variable table for debugTest method
            println!("\n🔍 Getting variable table...");
            let var_table = connection.get_variable_table(class_id, debug_test_method.method_id).await?;

            println!("✅ Found {} variables:", var_table.len());
            for var in &var_table {
                println!("   - {} ({})", var.name, var.signature);
            }

            // Get variables that are active at current code index
            let current_index = frame.location.index;
            let active_vars: Vec<_> = var_table.iter()
                .filter(|v| current_index >= v.code_index && current_index < v.code_index + v.length as u64)
                .collect();

            println!("\n📊 Active variables at index {}:", current_index);

            // Prepare slots for getting values
            let slots: Vec<jdwp_client::stackframe::VariableSlot> = active_vars.iter()
                .map(|v| jdwp_client::stackframe::VariableSlot {
                    slot: v.slot as i32,
                    sig_byte: v.signature.as_bytes()[0],
                })
                .collect();

            // Get frame values
            let values = connection.get_frame_values(thread, frame.frame_id, slots).await?;

            println!("\n🎁 Variable values:");
            for (var, value) in active_vars.iter().zip(values.iter()) {
                print!("   {} = ", var.name);

                // Check if this is a string (tag 115 = 's')
                if value.tag == 115 {
                    if let jdwp_client::types::ValueData::Object(object_id) = &value.data {
                        if object_id != &0 {
                            // THIS IS THE KEY TEST: Can we get the actual string value?
                            match connection.get_string_value(*object_id).await {
                                Ok(string_val) => {
                                    println!("\"{}\" ✅ STRING VALUE RETRIEVED!", string_val);
                                }
                                Err(e) => {
                                    println!("(String) @{:x} ❌ Failed to get string: {}", object_id, e);
                                }
                            }
                        } else {
                            println!("(String) null");
                        }
                    }
                } else {
                    println!("{}", value.format());
                }
            }

            println!("\n✅ String value retrieval test complete!");

            // Resume execution
            println!("\n▶️  Resuming execution...");
            connection.resume_all().await?;
        }
    }

    sleep(Duration::from_secs(1)).await;

    Ok(())
}
