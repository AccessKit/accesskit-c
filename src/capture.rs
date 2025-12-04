use accesskit::TreeUpdate;
use std::{
    env,
    fs::{remove_file, File, OpenOptions},
    io::{BufWriter, Write},
    path::PathBuf,
    sync::Mutex,
};

const ENV_VAR: &str = "ACCESSKIT_CAPTURE_PATH";

struct CaptureState {
    writer: BufWriter<File>,
    last_update: Option<TreeUpdate>,
}

impl CaptureState {
    fn new(path: PathBuf) -> std::io::Result<Self> {
        let _ = remove_file(&path);
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        Ok(Self {
            writer: BufWriter::new(file),
            last_update: None,
        })
    }

    fn capture(&mut self, update: &TreeUpdate) {
        if let Some(ref last) = self.last_update {
            if update == last {
                return;
            }
        }

        let json = match serde_json::to_string(update) {
            Ok(json) => json,
            Err(e) => {
                eprintln!("accesskit capture: failed to serialize tree update: {e}");
                return;
            }
        };

        if let Err(e) = writeln!(self.writer, "{json}") {
            eprintln!("accesskit capture: failed to write tree update: {e}");
            return;
        }

        if let Err(e) = self.writer.flush() {
            eprintln!("accesskit capture: failed to flush: {e}");
            return;
        }

        if let Err(e) = self.writer.get_ref().sync_data() {
            eprintln!("accesskit capture: failed to sync: {e}");
        }

        self.last_update = Some(update.clone());
    }
}

enum CaptureInit {
    Uninitialized,
    Disabled,
    Enabled(CaptureState),
}

static CAPTURE: Mutex<CaptureInit> = Mutex::new(CaptureInit::Uninitialized);

fn init_capture() -> Option<CaptureState> {
    let path = env::var(ENV_VAR).ok()?;
    if path.is_empty() {
        return None;
    }

    match CaptureState::new(PathBuf::from(&path)) {
        Ok(state) => {
            eprintln!("accesskit capture: capturing to {path}");
            Some(state)
        }
        Err(e) => {
            eprintln!("accesskit capture: failed to open {path}: {e}");
            None
        }
    }
}

pub(crate) fn capture_tree_update(update: &TreeUpdate) {
    let mut guard = match CAPTURE.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("accesskit capture: failed to acquire lock: {e}");
            return;
        }
    };

    if matches!(*guard, CaptureInit::Uninitialized) {
        *guard = match init_capture() {
            Some(state) => CaptureInit::Enabled(state),
            None => CaptureInit::Disabled,
        };
    }

    if let CaptureInit::Enabled(state) = &mut *guard {
        state.capture(update);
    }
}
