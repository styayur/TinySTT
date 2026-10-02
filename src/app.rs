//! Main application window and state management.

use std::path::Path;
use std::time::Duration;

use eframe::egui;

use crate::asr::{AsrWorker, WorkerEvent};
use crate::audio::capture::CaptureEvent;
use crate::audio::{write_wav_16k_mono, MAX_RECORDING_SECONDS, MIN_RECORDING_SECONDS};
use crate::clipboard;
use crate::error::Result;
use crate::hotkey::{GlobalHotkey, HotkeyEvent, HotkeySpec};
use crate::paste;
use crate::paths::Paths;
use crate::pipeline::RecordingPipeline;
use crate::settings::{self, Settings};
use crate::state::{AppState, StateMachine};
use crate::tray::{SystemTray, TrayAction};

pub struct TinySttApp {
    paths: Paths,
    settings: Settings,
    state: StateMachine,
    status: String,
    model_ready: bool,
    model_dir: Option<String>,
    model_error: Option<String>,
    worker: Option<AsrWorker>,
    recording: Option<RecordingPipeline>,
    hotkey: Option<GlobalHotkey>,
    hotkey_enabled: bool,
    hotkey_error: Option<String>,
    transcript: String,
    last_audio: Option<Vec<f32>>,
    last_audio_seconds: f64,
    last_stats: Option<crate::asr::RecognitionStats>,
    tray: Option<SystemTray>,
    tray_error: Option<String>,
    allow_exit: bool,
}

impl TinySttApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, paths: Paths) -> Self {
        let settings = Settings::load(&paths.config_file()).unwrap_or_else(|error| {
            crate::log::error(format_args!("failed to load settings: {error}"));
            Settings::default()
        });

        let mut app = Self {
            paths: paths.clone(),
            settings,
            state: StateMachine::default(),
            status: "Loading model...".to_string(),
            model_ready: false,
            model_dir: None,
            model_error: None,
            worker: None,
            recording: None,
            hotkey: None,
            hotkey_enabled: true,
            hotkey_error: None,
            transcript: String::new(),
            last_audio: None,
            last_audio_seconds: 0.0,
            last_stats: None,
            tray: None,
            tray_error: None,
            allow_exit: false,
        };

        app.worker = Some(AsrWorker::start(app.paths.clone(), app.settings.clone()));
        app.reload_hotkey();
        match SystemTray::new(app.hotkey_enabled, _cc.egui_ctx.clone()) {
            Ok(tray) => app.tray = Some(tray),
            Err(error) => {
                app.tray_error = Some(error.to_string());
                crate::log::error(format_args!("tray initialization failed: {error}"));
            }
        }
        crate::log::info(format_args!("TinySTT started"));
        app
    }

    fn poll_background(&mut self, ctx: &egui::Context) {
        self.poll_asr_worker();
        self.poll_capture();
        self.poll_hotkey();
        self.poll_tray(ctx);
        if self.state.state() == AppState::Recording
            || self.state.state() == AppState::Recognizing
            || self.state.state() == AppState::LoadingModel
        {
            ctx.request_repaint_after(Duration::from_millis(80));
        }
    }

    fn poll_asr_worker(&mut self) {
        let mut events = Vec::new();
        if let Some(worker) = &self.worker {
            while let Some(event) = worker.try_recv() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                WorkerEvent::ModelReady { model_dir } => {
                    self.model_ready = true;
                    self.model_error = None;
                    self.model_dir = Some(model_dir);
                    if self.state.state() == AppState::LoadingModel {
                        let _ = self.state.transition(AppState::Idle);
                    }
                    self.status = format!("Ready. Hold {} and speak.", self.settings.hotkey);
                    crate::log::info(format_args!("model ready"));
                }
                WorkerEvent::ModelError(error) => {
                    self.model_ready = false;
                    self.model_error = Some(error.clone());
                    self.status = error;
                    let _ = self.state.transition(AppState::Error);
                    crate::log::error(format_args!("model initialization failed"));
                }
                WorkerEvent::RecognitionStarted { audio_seconds } => {
                    self.status = format!("Recognizing {audio_seconds:.1}s of audio...");
                }
                WorkerEvent::RecognitionCompleted { text, stats } => {
                    self.transcript = text;
                    self.last_stats = Some(stats);
                    let copy_result = clipboard::write_text(&self.transcript);
                    let mut status =
                        "Recognition completed and copied to the clipboard.".to_string();
                    if let Err(error) = &copy_result {
                        status = error.to_string();
                    } else if self.settings.auto_paste {
                        if let Err(error) = paste::send_ctrl_v() {
                            status =
                                format!("Recognition completed, but auto paste failed: {error}");
                        } else {
                            status = "Recognition completed, copied, and pasted.".to_string();
                        }
                    }
                    self.status = status;
                    let _ = self.state.transition(AppState::Completed);
                }
                WorkerEvent::RecognitionError(error) => {
                    self.status = error;
                    let _ = self.state.transition(AppState::Error);
                }
            }
        }
    }

    fn poll_capture(&mut self) {
        let mut events = Vec::new();
        if let Some(capture) = self.recording.as_ref().and_then(RecordingPipeline::capture) {
            while let Some(event) = capture.poll_event() {
                events.push(event);
            }
        }

        for event in events {
            match event {
                CaptureEvent::MaxReached => {
                    self.status = "Maximum recording length reached.".to_string();
                    self.finish_recording();
                }
                CaptureEvent::Error(error) => {
                    if let Some(recording) = self.recording.take() {
                        recording.cancel();
                    }
                    self.status = format!("Audio capture failed: {error}");
                    let _ = self.state.transition(AppState::Error);
                }
            }
        }
    }

    fn poll_hotkey(&mut self) {
        let mut events = Vec::new();
        if let Some(hotkey) = &self.hotkey {
            while let Some(event) = hotkey.try_recv() {
                events.push(event);
            }
        }
        for event in events {
            match event {
                HotkeyEvent::Pressed => self.handle_hotkey_pressed(),
                HotkeyEvent::Released => self.handle_hotkey_released(),
            }
        }
    }

    fn poll_tray(&mut self, ctx: &egui::Context) {
        let action = self.tray.as_ref().and_then(SystemTray::poll);
        match action {
            Some(TrayAction::Open) => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
            }
            Some(TrayAction::ToggleHotkey) => self.toggle_hotkey(),
            Some(TrayAction::Exit) => {
                self.allow_exit = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            None => {}
        }
    }

    fn handle_hotkey_pressed(&mut self) {
        match self.state.state() {
            AppState::Recording => {}
            AppState::Recognizing | AppState::LoadingModel => {
                self.status = "Busy. Wait for the current operation to finish.".to_string();
            }
            AppState::Idle | AppState::Completed | AppState::Cancelled | AppState::Error => {
                self.start_recording();
            }
        }
    }

    fn handle_hotkey_released(&mut self) {
        if self.state.state() == AppState::Recording {
            self.finish_recording();
        }
    }

    fn start_recording(&mut self) {
        if !self.model_ready {
            self.status = self
                .model_error
                .clone()
                .unwrap_or_else(|| "The model is not ready yet.".to_string());
            let _ = self.state.transition(AppState::Error);
            return;
        }

        if self.state.state() != AppState::Idle {
            if let Err(error) = self.state.transition(AppState::Idle) {
                self.status = error.to_string();
                return;
            }
        }

        match RecordingPipeline::start(&self.settings) {
            Ok(recording) => {
                let device = recording
                    .capture()
                    .map(|capture| capture.device_name().to_string())
                    .unwrap_or_else(|| "default microphone".to_string());
                self.recording = Some(recording);
                if let Err(error) = self.state.transition(AppState::Recording) {
                    self.status = error.to_string();
                } else {
                    self.status = format!("Recording from {device}...");
                    crate::log::info(format_args!("recording start: device={device}"));
                }
            }
            Err(error) => {
                self.status = error.to_string();
                let _ = self.state.transition(AppState::Error);
                crate::log::error(format_args!("recording start failed: {error}"));
            }
        }
    }

    fn finish_recording(&mut self) {
        let Some(recording) = self.recording.take() else {
            return;
        };
        let captured = recording.stop();
        crate::log::info(format_args!(
            "recording stop: duration={:.3}s device={}",
            captured.duration_seconds, captured.device_name
        ));

        if captured.duration_seconds < MIN_RECORDING_SECONDS {
            self.status = format!(
                "Recording too short (minimum {:.0} ms).",
                MIN_RECORDING_SECONDS * 1000.0
            );
            let _ = self.state.transition(AppState::Cancelled);
            return;
        }

        self.last_audio_seconds = captured.duration_seconds as f64;
        self.last_audio = Some(captured.samples.clone());
        if captured.max_reached {
            self.status = "Maximum recording length reached.".to_string();
        } else {
            self.status = "Recognizing...".to_string();
        }

        if let Some(worker) = &self.worker {
            if let Err(error) = worker.recognize(captured.samples, captured.sample_rate) {
                self.status = error.to_string();
                let _ = self.state.transition(AppState::Error);
                return;
            }
        } else {
            self.status = "ASR worker is unavailable.".to_string();
            let _ = self.state.transition(AppState::Error);
            return;
        }

        let _ = self.state.transition(AppState::Recognizing);
    }

    fn cancel_recording(&mut self) {
        if let Some(recording) = self.recording.take() {
            recording.cancel();
            self.status = "Recording cancelled.".to_string();
            let _ = self.state.transition(AppState::Cancelled);
            crate::log::info(format_args!("recording cancelled"));
        }
    }

    fn reload_hotkey(&mut self) {
        self.hotkey = None;
        self.hotkey_error = None;
        if !self.hotkey_enabled {
            return;
        }

        let registration =
            HotkeySpec::parse(&self.settings.hotkey).and_then(GlobalHotkey::register);
        match registration {
            Ok(hotkey) => {
                self.hotkey = Some(hotkey);
                crate::log::info(format_args!(
                    "global hotkey registered: {}",
                    self.settings.hotkey
                ));
            }
            Err(error) => {
                self.hotkey_error = Some(error.to_string());
                self.status = error.to_string();
                crate::log::error(format_args!("global hotkey registration failed: {error}"));
            }
        }
    }

    fn toggle_hotkey(&mut self) {
        self.hotkey_enabled = !self.hotkey_enabled;
        self.reload_hotkey();
        self.status = if self.hotkey_enabled {
            format!("Hotkey enabled: {}", self.settings.hotkey)
        } else {
            "Hotkey disabled.".to_string()
        };
        if let Some(tray) = &self.tray {
            tray.set_hotkey_enabled(self.hotkey_enabled);
        }
    }

    fn save_settings(&mut self) {
        self.settings = self.settings.clone().normalized();
        match self.settings.save(&self.paths.config_file()) {
            Ok(()) => {
                self.status =
                    "Settings saved. Thread and language changes apply after restart.".to_string();
            }
            Err(error) => {
                self.status = error.to_string();
            }
        }
        self.reload_hotkey();
    }

    fn copy_transcript(&mut self) {
        if self.transcript.trim().is_empty() {
            self.status = "Nothing to copy.".to_string();
            return;
        }
        self.status = match clipboard::write_text(&self.transcript) {
            Ok(()) => "Transcript copied to the clipboard.".to_string(),
            Err(error) => error.to_string(),
        };
    }

    fn save_debug_wav(&mut self) {
        let Some(samples) = &self.last_audio else {
            self.status = "No recording is available to export.".to_string();
            return;
        };
        let path = self.paths.debug_wav_file();
        self.status = match write_wav_16k_mono(&path, samples) {
            Ok(()) => format!("Debug WAV saved: {}", path.display()),
            Err(error) => error.to_string(),
        };
    }

    fn recording_controls(&mut self, ui: &mut egui::Ui) {
        let idle = self.state.state().is_idle_like() && self.model_ready;
        let recording = self.state.state() == AppState::Recording;
        ui.horizontal(|ui| {
            if ui.add_enabled(idle, egui::Button::new("Record")).clicked() {
                self.start_recording();
            }
            if ui
                .add_enabled(recording, egui::Button::new("Stop"))
                .clicked()
            {
                self.finish_recording();
            }
            if ui
                .add_enabled(recording, egui::Button::new("Cancel"))
                .clicked()
            {
                self.cancel_recording();
            }
        });
    }

    fn settings_ui(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Settings")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("settings-grid")
                    .num_columns(2)
                    .spacing([12.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Microphone");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.settings.microphone)
                                .hint_text("Default device"),
                        );
                        ui.end_row();

                        ui.label("Model threads");
                        egui::ComboBox::from_id_salt("threads")
                            .selected_text(self.settings.num_threads.to_string())
                            .show_ui(ui, |ui| {
                                for threads in settings::ALLOWED_THREADS {
                                    ui.selectable_value(
                                        &mut self.settings.num_threads,
                                        threads,
                                        threads.to_string(),
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Language");
                        egui::ComboBox::from_id_salt("language")
                            .selected_text(self.settings.language.clone())
                            .show_ui(ui, |ui| {
                                for language in ["auto", "zh", "en", "yue", "ja", "ko"] {
                                    ui.selectable_value(
                                        &mut self.settings.language,
                                        language.to_string(),
                                        language,
                                    );
                                }
                            });
                        ui.end_row();

                        ui.label("Hotkey");
                        ui.add(egui::TextEdit::singleline(&mut self.settings.hotkey));
                        ui.end_row();

                        ui.label("Auto paste");
                        ui.checkbox(
                            &mut self.settings.auto_paste,
                            "Insert into the active window",
                        );
                        ui.end_row();
                    });

                ui.label(
                    egui::RichText::new(
                        "Thread, language, and model-directory changes apply after restart.",
                    )
                    .small()
                    .weak(),
                );

                if ui.button("Save settings").clicked() {
                    self.save_settings();
                }

                if let Some(model_dir) = &self.model_dir {
                    ui.label(format!("Model: {model_dir}"));
                }
                if let Some(error) = &self.hotkey_error {
                    ui.colored_label(egui::Color32::from_rgb(180, 50, 40), error);
                }
                if let Some(error) = &self.tray_error {
                    ui.label(egui::RichText::new(error).small().weak());
                }
            });
    }
}

impl eframe::App for TinySttApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poll_background(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("TinySTT");
            ui.label("Tiny, offline speech-to-text for Windows.");
            ui.add_space(4.0);

            ui.label(
                egui::RichText::new(format!(
                    "Hold {} -> Speak -> Release -> Text.",
                    self.settings.hotkey
                ))
                .strong(),
            );
            ui.label(
                egui::RichText::new(
                    "TinySTT does not send your audio or transcript to any server.",
                )
                .small()
                .weak(),
            );

            ui.separator();
            ui.label(format!("Status: {}", self.status));
            if self.state.state() == AppState::Recording {
                if let Some(capture) = self.recording.as_ref().and_then(RecordingPipeline::capture)
                {
                    ui.label(format!(
                        "Recording {:.1}s / {}s",
                        capture.duration_seconds(),
                        MAX_RECORDING_SECONDS
                    ));
                }
            }

            self.recording_controls(ui);
            ui.separator();

            ui.label("Recognition result:");
            egui::ScrollArea::vertical()
                .max_height(190.0)
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut self.transcript)
                            .desired_rows(7)
                            .desired_width(f32::INFINITY)
                            .hint_text("Your transcript will appear here."),
                    );
                });

            ui.horizontal(|ui| {
                if ui.button("Copy").clicked() {
                    self.copy_transcript();
                }
                if ui.button("Clear").clicked() {
                    self.transcript.clear();
                    self.status = "Result cleared.".to_string();
                }
                if ui
                    .add_enabled(self.last_audio.is_some(), egui::Button::new("Save WAV"))
                    .clicked()
                {
                    self.save_debug_wav();
                }
            });

            if let Some(stats) = &self.last_stats {
                ui.label(
                    egui::RichText::new(format!(
                        "Last run: {:.2}s audio, {:.2}s inference, RTF {:.2}",
                        stats.audio_seconds, stats.inference_seconds, stats.rtf
                    ))
                    .small()
                    .weak(),
                );
            }

            ui.separator();
            self.settings_ui(ui);

            if let Some(error) = &self.model_error {
                ui.separator();
                ui.colored_label(egui::Color32::from_rgb(180, 50, 40), error);
            }
        });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(error) = self.settings.save(&self.paths.config_file()) {
            crate::log::error(format_args!("settings save on exit failed: {error}"));
        }
        if let Some(recording) = self.recording.take() {
            recording.cancel();
        }
        crate::log::info(format_args!("TinySTT exiting"));
    }
}

pub fn load_model_for_cli(paths: &Paths, settings: &Settings) -> Result<crate::paths::ModelPaths> {
    paths.resolve_model(settings)
}

pub fn model_file_exists(path: &Path) -> bool {
    path.is_file()
}
