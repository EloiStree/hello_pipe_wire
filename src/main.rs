use pipewire as pw;
use std::{cell::Cell, rc::Rc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize PipeWire.
    pw::init();

    // Create the PipeWire event loop.
    let mainloop = pw::main_loop::MainLoopRc::new(None)?;

    // Create a context associated with the event loop.
    let context = pw::context::ContextRc::new(&mainloop, None)?;

    // Connect to the user's PipeWire server.
    let core = context.connect_rc(None)?;

    // Get the registry containing PipeWire objects.
    let registry = core.get_registry()?;

    println!("Connected to PipeWire!");
    println!("PipeWire objects:");

    // We'll quit after the initial registry has been received.
    let done = Rc::new(Cell::new(false));

    let done_clone = done.clone();
    let loop_clone = mainloop.clone();

    // Ask PipeWire to synchronize with the server.
    let pending = core.sync(0)?;

    // Listen for the sync completion.
    let _core_listener = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pw::core::PW_ID_CORE && seq == pending {
                done_clone.set(true);
                loop_clone.quit();
            }
        })
        .register();

    // Listen for objects appearing in the registry.
    let _registry_listener = registry
        .add_listener_local()
        .global(|global| {
            println!(
                "  id={} type={} version={}",
                global.id,
                global.type_,
                global.version
            );
        })
        .register();

    // Process PipeWire events.
    while !done.get() {
        mainloop.run();
    }

    println!("Done.");

    Ok(())
}
