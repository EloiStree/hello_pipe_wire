use pipewire as pw;
use std::{cell::Cell, rc::Rc};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    pw::init();

    let mainloop = pw::main_loop::MainLoopRc::new(None)?;
    let context = pw::context::ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry()?;

    println!("Connected to PipeWire.");
    println!("Monitoring PipeWire for Steam Remote Play streams...\n");

    let initial_scan_complete = Rc::new(Cell::new(false));
    let initial_scan_complete_clone = initial_scan_complete.clone();

    // IMPORTANT: install the registry listener BEFORE sync().
    let _registry_listener = registry
        .add_listener_local()
        .global(move |global| {
            let Some(props) = global.props else {
                return;
            };

            // These properties identify the application and describe the stream.
            let interesting = [
                "node.name",
                "node.description",
                "media.name",
                "media.class",
                "media.role",
                "media.type",
                "media.category",
                "application.name",
                "application.id",
                "application.process.id",
                "application.process.binary",
                "application.process.user",
                "node.state",
                "node.latency",
                "node.rate",
                "audio.format",
                "audio.rate",
                "audio.channels",
                "video.format",
                "video.width",
                "video.height",
                "video.framerate",
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

            // Build a searchable string from all properties we care about.
            let searchable = interesting
                .iter()
                .filter_map(|key| props.get(key))
                .collect::<Vec<_>>()
                .join(" ")
                .to_ascii_lowercase();

            let remote_play = searchable.contains("remote play")
                || searchable.contains("steam remote")
                || searchable.contains("steam streaming")
                || searchable.contains("steam")
                    && searchable.contains("stream");

            if !remote_play {
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

            println!();
            println!("  >>> POSSIBLE STEAM REMOTE PLAY STREAM <<<");
            println!("  reason: Steam/Remote Play related property");

            println!();
        })
        .register();

    // Synchronize AFTER installing the registry listener, then keep monitoring
    // so streams created after startup are also reported.
    let pending = core.sync(0)?;

    let _core_listener = core
        .add_listener_local()
        .done(move |id, seq| {
            if id == pw::core::PW_ID_CORE && seq == pending {
                initial_scan_complete_clone.set(true);
                println!("\nInitial PipeWire registry scan complete. Continuing to monitor...\n");
            }
        })
        .register();

    mainloop.run();

    Ok(())
}
