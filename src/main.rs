use butterfly_guard::{parse_config, Config, Debouncer};
use evdev::{raw_stream::RawDevice, uinput::VirtualDevice, EventType, InputEvent};
use std::{
    fs, io,
    os::fd::AsRawFd,
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant, UNIX_EPOCH},
};

const CONFIG: &str = "/etc/butterfly-guard.json";
const STATUS: &str = "/run/butterfly-guard/status.json";
const UNIT: &str = "butterfly-guard.service";
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
nix::ioctl_write_ptr!(set_clock, b'E', 0xa0, libc::c_int);

fn load_config(path: &str) -> Result<Config> {
    let config: Config = serde_json::from_slice(&fs::read(path)?)?;
    config.validate()?;
    Ok(config)
}
fn write_status(state: &str, delay: u16, dropped: u64) -> Result<()> {
    let text =
        serde_json::to_vec(&serde_json::json!({"state":state,"delay_ms":delay,"dropped":dropped}))?;
    fs::write(format!("{STATUS}.tmp"), text)?;
    fs::rename(format!("{STATUS}.tmp"), STATUS)?;
    Ok(())
}
fn set_nonblocking(fd: i32) -> io::Result<()> {
    // SAFETY: fcntl operates on a live owned fd; neither command uses pointers.
    let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
    if flags < 0 || unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}
fn find_keyboard() -> io::Result<Option<RawDevice>> {
    for entry in fs::read_dir("/dev/input")? {
        let entry = entry?;
        if !entry.file_name().to_string_lossy().starts_with("event") {
            continue;
        }
        // Do not open unrelated input devices: inspect sysfs first.
        let sys = Path::new("/sys/class/input")
            .join(entry.file_name())
            .join("device");
        if fs::read_to_string(sys.join("name"))
            .unwrap_or_default()
            .trim()
            != "Apple SPI Keyboard"
            || fs::read_to_string(sys.join("phys"))
                .unwrap_or_default()
                .trim()
                != "applespi/input0"
        {
            continue;
        }
        let device = RawDevice::open(entry.path())?;
        if device.name() == Some("Apple SPI Keyboard")
            && device.physical_path() == Some("applespi/input0")
        {
            return Ok(Some(device));
        }
    }
    Ok(None)
}

struct Session {
    device: RawDevice,
    output: VirtualDevice,
    debounce: Debouncer,
}
impl Drop for Session {
    fn drop(&mut self) {
        let releases: Vec<_> = self
            .debounce
            .pressed()
            .map(|k| InputEvent::new(EventType::KEY.0, k, 0))
            .collect();
        let _ = self.output.emit(&releases);
        let _ = self.device.ungrab();
    }
}
fn attach(mut device: RawDevice, config: &Config) -> Result<Session> {
    let keys = device
        .supported_keys()
        .ok_or("Keyboard has no key capabilities")?;
    let builder = VirtualDevice::builder()?
        .name("Butterfly Guard Keyboard")
        .with_keys(keys)?;
    // EV_REP is intentionally omitted: preserve physical repeats without a second kernel timer.
    let output = builder.build()?;
    set_nonblocking(device.as_raw_fd())?;
    set_nonblocking(output.as_raw_fd())?;
    // SAFETY: ioctl receives a live input fd and a correctly typed clock-id pointer.
    unsafe {
        set_clock(device.as_raw_fd(), &libc::CLOCK_MONOTONIC)?;
    }
    // udev/compositor discovery happens before exclusive access is taken.
    thread::sleep(Duration::from_millis(500));
    // Wait for idle before grabbing, so a modifier pressed before startup cannot stick
    // on the original device. A busy keyboard retries on the next attachment attempt.
    if device.get_key_state()?.iter().next().is_some() {
        return Err("Waiting for held keys to be released".into());
    }
    device.grab()?;
    let mut session = Session {
        device,
        output,
        debounce: Debouncer::new(config),
    };
    // Discard input queued before the grab, then snapshot held keys.
    loop {
        match session.device.fetch_events() {
            Ok(events) => for _ in events {},
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
            Err(error) => return Err(error.into()),
        }
    }
    for key in session.device.get_key_state()?.iter() {
        session.debounce.block_held(key.code());
    }
    Ok(session)
}
fn pump(session: &mut Session, config: &Config, stop: &AtomicBool) -> Result<()> {
    let mut next_status = Instant::now();
    let mut frame = Vec::with_capacity(32);
    while !stop.load(Ordering::Relaxed) {
        let mut fds = [
            libc::pollfd {
                fd: session.device.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
            libc::pollfd {
                fd: session.output.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            },
        ];
        // SAFETY: the array has exactly two initialized pollfd values and lives through poll.
        let result = unsafe { libc::poll(fds.as_mut_ptr(), 2, 250) };
        if result < 0 {
            let error = io::Error::last_os_error();
            if error.kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err(error.into());
        }
        if fds
            .iter()
            .any(|fd| fd.revents & (libc::POLLERR | libc::POLLHUP | libc::POLLNVAL) != 0)
        {
            return Err("Input device disconnected".into());
        }
        if fds[0].revents & libc::POLLIN != 0 {
            match session.device.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == EventType::SYNCHRONIZATION {
                            if event.code() == 3 {
                                return Err("Input queue overflow; reattaching".into());
                            }
                            if event.code() == 0 && !frame.is_empty() {
                                session.output.emit(&frame)?;
                                frame.clear();
                            }
                        } else if event.event_type() == EventType::KEY {
                            let us =
                                event.timestamp().duration_since(UNIX_EPOCH)?.as_micros() as u64;
                            if session.debounce.accept(event.code(), event.value(), us) {
                                frame.push(event);
                            }
                        }
                        if frame.len() > 1024 {
                            return Err("Input frame exceeded limit".into());
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
        if fds[1].revents & libc::POLLIN != 0 {
            match session.output.fetch_events() {
                Ok(events) => {
                    for event in events {
                        if event.event_type() == EventType::LED {
                            session.device.send_events(&[event])?;
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {}
                Err(error) => return Err(error.into()),
            }
        }
        if Instant::now() >= next_status {
            write_status("active", config.delay_ms, session.debounce.dropped)?;
            next_status = Instant::now() + Duration::from_secs(1);
        }
    }
    Ok(())
}
fn daemon() -> Result<()> {
    let config = load_config(CONFIG)?;
    let stop = Arc::new(AtomicBool::new(false));
    for signal in [signal_hook::consts::SIGINT, signal_hook::consts::SIGTERM] {
        signal_hook::flag::register(signal, Arc::clone(&stop))?;
    }
    while !stop.load(Ordering::Relaxed) {
        write_status("waiting for keyboard", config.delay_ms, 0)?;
        if let Some(device) = find_keyboard()? {
            match attach(device, &config) {
                Ok(mut session) => {
                    if let Err(error) = pump(&mut session, &config, &stop) {
                        eprintln!("{error}");
                    }
                }
                Err(error) => eprintln!("{error}"),
            }
        }
        if !stop.load(Ordering::Relaxed) {
            thread::sleep(Duration::from_secs(1));
        }
    }
    write_status("stopped", config.delay_ms, 0)
}
fn systemctl(args: &[&str]) -> Result<()> {
    if !Command::new("/usr/bin/systemctl")
        .args(args)
        .arg(UNIT)
        .status()?
        .success()
    {
        return Err("systemctl failed".into());
    }
    Ok(())
}
fn control(args: &[String]) -> Result<()> {
    // SAFETY: geteuid has no arguments or memory access requirements.
    if unsafe { libc::geteuid() } != 0 {
        return Err("Run controls with sudo or pkexec".into());
    }
    match args
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["start"] => systemctl(&["enable", "--now"]),
        ["stop"] => systemctl(&["disable", "--now"]),
        ["toggle"] => {
            let active = Command::new("/usr/bin/systemctl")
                .args(["is-active", "--quiet", UNIT])
                .status()?
                .success();
            systemctl(&[if active { "disable" } else { "enable" }, "--now"])
        }
        ["set-config", json, expected] => {
            let config = parse_config(json)?;
            let previous = parse_config(expected)?;
            let lock = fs::OpenOptions::new()
                .create(true)
                .truncate(false)
                .write(true)
                .open("/run/butterfly-guard-config.lock")?;
            lock.lock()?;
            if load_config(CONFIG)? != previous {
                return Err("Settings changed elsewhere. Reload settings before saving.".into());
            }
            fs::write(format!("{CONFIG}.tmp"), serde_json::to_vec_pretty(&config)?)?;
            fs::rename(format!("{CONFIG}.tmp"), CONFIG)?;
            systemctl(&["try-restart"])
        }
        ["cycle"] | ["set-delay", _] => {
            let mut config = load_config(CONFIG)?;
            config.delay_ms = if args[0] == "cycle" {
                match config.delay_ms {
                    40 => 60,
                    60 => 80,
                    _ => 40,
                }
            } else {
                args[1].parse()?
            };
            config.validate()?;
            fs::write(format!("{CONFIG}.tmp"), serde_json::to_vec_pretty(&config)?)?;
            fs::rename(format!("{CONFIG}.tmp"), CONFIG)?;
            systemctl(&["try-restart"])
        }
        _ => Err("Usage: butterfly-guardctl start|stop|toggle|cycle|set-delay MILLISECONDS".into()),
    }
}
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("daemon") if args.len() == 1 => daemon(),
        Some("check-config") if args.len() == 2 => {
            load_config(&args[1]).map(|_| println!("Configuration valid"))
        }
        Some("--help") | None => {
            println!("Butterfly Guard 1.2.0\nCommands: daemon, check-config PATH, start, stop, toggle, cycle, set-delay MS, set-config JSON EXPECTED_JSON");
            Ok(())
        }
        _ => control(&args),
    };
    if let Err(error) = result {
        eprintln!("Butterfly Guard: {error}");
        std::process::exit(1);
    }
}
