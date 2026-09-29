use pipewire as pw;
use std::{cell::Cell, rc::Rc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pw::init();

    let mainloop = pw::main_loop::MainLoopRc::new(None)?;
    let context = pw::context::ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry()?;

    println!("Connected to PipeWire.");
    println!("Looking for Steam Remote Play / World of Warcraft streams...\n");

    let done = Rc::new(Cell::new(false));
    let done_clone = done.clone();
    let loop_clone = mainloop.clone();

    // IMPORTANT: install the registry listener BEFORE sync().
    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            let Some(props) = global.props else {
                return;
            };

            // Print the interesting properties of every object.
            let interesting = [
                "node.name",
                "node.description",
                "media.name",
                "media.class",
                "media.role",
                "application.name",
                "application.id",
                "application.process.id",
                "application.process.binary",
                "application.process.user",
                "pipewire.access",
                "pipewire.access.portal.app_id",
                "pipewire.access.portal.app_path",
                "window.title",
                "window.name",
                "window.id",
                "target.object",
                "object.path",
                "object.serial",
            ];

            let mut has_interesting_info = false;

            for key in interesting {
                if let Some(value) = props.get(key) {
                    if !value.is_empty() {
                        has_interesting_info = true;
                    }
                }
            }

            if !has_interesting_info {
                return;
            }

            println!("==================================================");
            println!("PipeWire object id={}", global.id);
            println!("type={} version={}", global.type_, global.version);

            for key in interesting {
                if let Some(value) = props.get(key) {
                    println!("  {key} = {value}");
                }
            }

            // Build a searchable string from all properties we care about.
            let searchable = interesting
                .iter()
                .filter_map(|key| props.get(key))
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase();

            let remote_play =
                searchable.contains("remote play")
                || searchable.contains("steam remote")
                || searchable.contains("steam streaming")
                || searchable.contains("steam")
                    && searchable.contains("stream");

            let world_of_warcraft =
                searchable.contains("world of warcraft")
                || searchable.contains("worldofwarcraft")
                || searchable.contains("wow.exe")
                || searchable.contains("wow");

            if remote_play || world_of_warcraft {
                println!();
                println!("  >>> POSSIBLE REMOTE PLAY / WOW STREAM <<<");

                if remote_play {
                    println!("  reason: Steam/Remote Play related property");
                }

                if world_of_warcraft {
                    println!("  reason: World of Warcraft related property");
                }

                println!();
            }
        })
        .register();

    // Synchronize AFTER installing the registry listener.
    let pending = core.sync(0)?;

    let _core_listener = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pw::core::PW_ID_CORE && seq == pending {
                done_clone.set(true);
                loop_clone.quit();
            }
        })
        .register();

    while !done.get() {
        mainloop.run();
    }

    println!("\nInitial PipeWire registry scan complete.");

    Ok(())
}
