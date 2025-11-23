//! Desktop application entry point for AC PCAP Parser
//!
//! This binary provides a native desktop GUI that shares the same UI code
//! as the web version, with additional desktop-specific features like
//! native file dialogs.

fn main() -> eframe::Result<()> {
    // Initialize logging for desktop
    env_logger::init();

    // Call the library's main function
    web::main()
}
