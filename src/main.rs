//! TinySTT entry point.
//!
//! GUI: launch without arguments.
//! CLI: `TinySTT.exe --file test.wav`

#![cfg_attr(
    all(windows, not(debug_assertions), feature = "native"),
    windows_subsystem = "windows"
)]

#[cfg(feature = "native")]
#[cfg(feature = "native")]
use std::path::PathBuf;

#[cfg(feature = "native")]
use tinystt::asr::sensevoice::SenseVoiceRecognizer;
#[cfg(feature = "native")]
use tinystt::asr::SpeechRecognizer;
#[cfg(feature = "native")]
use tinystt::audio::read_wav_as_16k_mono;
#[cfg(feature = "native")]
use tinystt::error::{Result, TinySttError};
#[cfg(feature = "native")]
use tinystt::paths::Paths;
#[cfg(feature = "native")]
use tinystt::settings::Settings;

#[cfg(feature = "native")]
enum Command {
    Gui,
    File {
        path: PathBuf,
        model_dir: Option<String>,
        num_threads: Option<u8>,
        language: Option<String>,
    },
    Help,
    Version,
}

#[cfg(feature = "native")]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        attach_parent_console();
    }

    match parse_command(&args) {
        Ok(Command::Gui) => {
            if let Err(error) = run_gui() {
                log_or_dialog(&error.to_string());
            }
        }
        Ok(Command::File {
            path,
            model_dir,
            num_threads,
            language,
        }) => {
            if let Err(error) = run_cli(path, model_dir, num_threads, language) {
                emit_line(&format!("TinySTT error: {error}"), true);
                std::process::exit(1);
            }
        }
        Ok(Command::Help) => {
            emit_line(HELP, false);
        }
        Ok(Command::Version) => {
            emit_line(concat!("TinySTT ", env!("CARGO_PKG_VERSION")), false);
        }
        Err(error) => {
            emit_line(&error, true);
            std::process::exit(2);
        }
    }
}

#[cfg(feature = "native")]
const HELP: &str = "TinySTT - offline Windows speech-to-text

Usage:
  TinySTT.exe
      Start the push-to-talk desktop app.

  TinySTT.exe --file <test.wav> [--model-dir <dir>] [--num-threads <1|2|4>] [--language <auto|zh|en|yue|ja|ko>]
      Convert a WAV file to text and print the transcript to stdout.

Options:
  -h, --help       Show this help.
  -V, --version    Show the version.
";

#[cfg(feature = "native")]
fn parse_command(args: &[String]) -> std::result::Result<Command, String> {
    if args.is_empty() {
        return Ok(Command::Gui);
    }
    if args.iter().any(|arg| arg == "-h" || arg == "--help") {
        return Ok(Command::Help);
    }
    if args.iter().any(|arg| arg == "-V" || arg == "--version") {
        return Ok(Command::Version);
    }

    let mut file = None;
    let mut model_dir = None;
    let mut num_threads = None;
    let mut language = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--file" | "-f" => {
                index += 1;
                file = Some(
                    args.get(index)
                        .ok_or_else(|| "missing value for --file".to_string())?
                        .clone(),
                );
            }
            "--model-dir" => {
                index += 1;
                model_dir = Some(
                    args.get(index)
                        .ok_or_else(|| "missing value for --model-dir".to_string())?
                        .clone(),
                );
            }
            "--num-threads" => {
                index += 1;
                let value = args
                    .get(index)
                    .ok_or_else(|| "missing value for --num-threads".to_string())?;
                num_threads = Some(
                    value
                        .parse::<u8>()
                        .map_err(|_| format!("invalid --num-threads value: {value}"))?,
                );
            }
            "--language" => {
                index += 1;
                language = Some(
                    args.get(index)
                        .ok_or_else(|| "missing value for --language".to_string())?
                        .clone(),
                );
            }
            other => return Err(format!("unknown argument: {other}")),
        }
        index += 1;
    }

    let path = file.ok_or_else(|| "CLI mode requires --file <test.wav>".to_string())?;
    Ok(Command::File {
        path: PathBuf::from(path),
        model_dir,
        num_threads,
        language,
    })
}

#[cfg(feature = "native")]
fn run_cli(
    wav_path: PathBuf,
    model_dir: Option<String>,
    num_threads: Option<u8>,
    language: Option<String>,
) -> Result<()> {
    let paths = Paths::from_exe()?;
    tinystt::log::init(&paths.log_file());

    let mut settings = Settings::load(&paths.config_file()).unwrap_or_default();
    if let Some(model_dir) = model_dir {
        settings.model_dir = model_dir;
    }
    if let Some(num_threads) = num_threads {
        settings.num_threads = num_threads;
    }
    if let Some(language) = language {
        settings.language = language;
    }
    let settings = settings.normalized();

    let model = paths.resolve_model(&settings)?;
    let samples = read_wav_as_16k_mono(&wav_path).map_err(|error| {
        TinySttError::Wav(format!("failed to read {}: {error}", wav_path.display()))
    })?;
    let audio_seconds = samples.len() as f64 / 16_000.0;
    let mut recognizer =
        SenseVoiceRecognizer::new(&model, settings.num_threads, &settings.language)?;
    let started = std::time::Instant::now();
    let text = recognizer.recognize(&samples, 16_000)?;
    let elapsed = started.elapsed().as_secs_f64();
    let rtf = if audio_seconds > 0.0 {
        elapsed / audio_seconds
    } else {
        0.0
    };
    tinystt::log_info!(
        "CLI success: audio_duration={audio_seconds:.3}s inference_duration={elapsed:.3}s rtf={rtf:.3}"
    );
    emit_line(text.trim(), false);
    Ok(())
}

#[cfg(feature = "native")]
fn run_gui() -> Result<()> {
    let paths = Paths::from_exe()?;
    tinystt::log::init(&paths.log_file());

    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([760.0, 620.0])
            .with_min_inner_size([560.0, 460.0])
            .with_title("TinySTT"),
        ..Default::default()
    };

    eframe::run_native(
        "TinySTT",
        native_options,
        Box::new(|cc| Ok(Box::new(tinystt::app::TinySttApp::new(cc, paths)))),
    )
    .map_err(|error| TinySttError::Settings(format!("GUI failed: {error}")))
}

#[cfg(feature = "native")]
fn log_or_dialog(message: &str) {
    tinystt::log::error(format_args!("{message}"));
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
        let title: Vec<u16> = "TinySTT error\0".encode_utf16().collect();
        let message: Vec<u16> = format!("{message}\0").encode_utf16().collect();
        unsafe {
            let _ = MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }
}

#[cfg(feature = "native")]
fn emit_line(text: &str, to_stderr: bool) {
    #[cfg(windows)]
    if emit_windows_line(text, to_stderr) {
        return;
    }

    if to_stderr {
        eprintln!("{text}");
    } else {
        println!("{text}");
    }
}

#[cfg(all(windows, feature = "native"))]
fn emit_windows_line(text: &str, to_stderr: bool) -> bool {
    use std::ptr;
    use windows_sys::Win32::Foundation::HANDLE;
    use windows_sys::Win32::Storage::FileSystem::WriteFile;
    use windows_sys::Win32::System::Console::{GetStdHandle, STD_ERROR_HANDLE, STD_OUTPUT_HANDLE};

    let std_handle = if to_stderr {
        STD_ERROR_HANDLE
    } else {
        STD_OUTPUT_HANDLE
    };
    let handle: HANDLE = unsafe { GetStdHandle(std_handle) };
    if handle.is_null() || handle as isize == -1 {
        return false;
    }

    let mut bytes = text.as_bytes().to_vec();
    bytes.extend_from_slice(b"\r\n");
    let mut written = 0u32;
    unsafe {
        WriteFile(
            handle,
            bytes.as_ptr(),
            bytes.len() as u32,
            &mut written,
            ptr::null_mut(),
        ) != 0
    }
}

#[cfg(all(windows, feature = "native"))]
fn attach_parent_console() {
    use windows_sys::Win32::System::Console::{
        AllocConsole, AttachConsole, GetStdHandle, SetConsoleOutputCP, ATTACH_PARENT_PROCESS,
        STD_OUTPUT_HANDLE,
    };
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        if handle.is_null() || handle as isize == -1 {
            let _ = AllocConsole();
        }
        let _ = SetConsoleOutputCP(65001);
    }
}

#[cfg(all(not(windows), feature = "native"))]
fn attach_parent_console() {}
#[cfg(not(feature = "native"))]
fn main() {
    eprintln!("TinySTT was compiled without the 'native' feature.");
    eprintln!("Run `cargo test --no-default-features` for core tests.");
    eprintln!("Build the full app with `cargo build --release` on Windows.");
}
