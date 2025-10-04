// JDWP client library for Java debugging
//
// Implements a subset of the JDWP protocol focused on practical debugging scenarios:
// - Connection management
// - Breakpoint operations
// - Stack inspection
// - Variable evaluation
// - Execution control

pub mod connection;
pub mod protocol;
pub mod commands;
pub mod events;
pub mod eventloop;
pub mod types;
pub mod reader;
pub mod vm;
pub mod reftype;
pub mod method;
pub mod eventrequest;
pub mod thread;
pub mod stackframe;
pub mod string;
pub mod object;

pub use connection::JdwpConnection;
pub use eventloop::{EventLoopHandle, spawn_event_loop};
pub use events::EventSet;
pub use protocol::{JdwpError, JdwpResult};
pub use eventrequest::SuspendPolicy;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
