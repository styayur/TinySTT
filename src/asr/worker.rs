//! Long-lived single ASR worker.
//!
//! The model is created once and kept inside this thread. Requests are handled
//! serially, so the UI never creates concurrent inference jobs.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Instant;

use crate::asr::sensevoice::SenseVoiceRecognizer;
use crate::asr::{clean_transcript, RecognitionStats, SpeechRecognizer};
use crate::error::{Result, TinySttError};
use crate::paths::Paths;
use crate::settings::Settings;

#[derive(Debug, Clone, PartialEq)]
pub enum WorkerEvent {
    ModelReady {
        model_dir: String,
    },
    ModelError(String),
    RecognitionStarted {
        audio_seconds: f64,
    },
    RecognitionCompleted {
        text: String,
        stats: RecognitionStats,
    },
    RecognitionError(String),
}

enum WorkerCommand {
    Recognize { samples: Vec<f32>, sample_rate: u32 },
    Shutdown,
}

pub struct AsrWorker {
    tx: Sender<WorkerCommand>,
    events: Receiver<WorkerEvent>,
}

impl AsrWorker {
    pub fn start(paths: Paths, settings: Settings) -> Self {
        let (command_tx, command_rx) = mpsc::channel();
        let (event_tx, event_rx) = mpsc::channel();
        let worker_event_tx = event_tx.clone();

        if let Err(error) = thread::Builder::new()
            .name("tinystt-asr".to_string())
            .spawn(move || run_worker(paths, settings, command_rx, worker_event_tx))
        {
            let _ = event_tx.send(WorkerEvent::ModelError(format!(
                "failed to start ASR worker: {error}"
            )));
            crate::log::error(format_args!("failed to spawn ASR worker: {error}"));
        }

        Self {
            tx: command_tx,
            events: event_rx,
        }
    }
    pub fn recognize(&self, samples: Vec<f32>, sample_rate: u32) -> Result<()> {
        self.tx
            .send(WorkerCommand::Recognize {
                samples,
                sample_rate,
            })
            .map_err(|_| TinySttError::Recognition("ASR worker is not running".to_string()))
    }

    pub fn try_recv(&self) -> Option<WorkerEvent> {
        self.events.try_recv().ok()
    }
}

impl Drop for AsrWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(WorkerCommand::Shutdown);
    }
}

fn run_worker(
    paths: Paths,
    settings: Settings,
    commands: Receiver<WorkerCommand>,
    events: Sender<WorkerEvent>,
) {
    let model_paths = match paths.resolve_model(&settings) {
        Ok(model_paths) => model_paths,
        Err(error) => {
            let _ = events.send(WorkerEvent::ModelError(error.to_string()));
            return;
        }
    };

    let load_start = Instant::now();
    let mut recognizer =
        match SenseVoiceRecognizer::new(&model_paths, settings.num_threads, &settings.language) {
            Ok(recognizer) => recognizer,
            Err(error) => {
                let _ = events.send(WorkerEvent::ModelError(error.to_string()));
                return;
            }
        };
    crate::log::info(format_args!(
        "SenseVoice model loaded from {} in {:.3}s",
        model_paths.directory.display(),
        load_start.elapsed().as_secs_f64()
    ));
    let _ = events.send(WorkerEvent::ModelReady {
        model_dir: model_paths.directory.display().to_string(),
    });

    while let Ok(command) = commands.recv() {
        match command {
            WorkerCommand::Recognize {
                samples,
                sample_rate,
            } => {
                let audio_seconds = samples.len() as f64 / sample_rate.max(1) as f64;
                let _ = events.send(WorkerEvent::RecognitionStarted { audio_seconds });
                crate::log::info(format_args!(
                    "ASR start: audio_duration={audio_seconds:.3}s"
                ));
                let inference_start = Instant::now();
                match recognizer
                    .recognize(&samples, sample_rate)
                    .and_then(|text| clean_transcript(&text))
                {
                    Ok(text) => {
                        let inference_seconds = inference_start.elapsed().as_secs_f64();
                        let rtf = if audio_seconds > 0.0 {
                            inference_seconds / audio_seconds
                        } else {
                            0.0
                        };
                        crate::log::info(format_args!(
                            "ASR success: audio_duration={audio_seconds:.3}s inference_duration={inference_seconds:.3}s rtf={rtf:.3}"
                        ));
                        let _ = events.send(WorkerEvent::RecognitionCompleted {
                            text,
                            stats: RecognitionStats {
                                audio_seconds,
                                inference_seconds,
                                rtf,
                            },
                        });
                    }
                    Err(error) => {
                        crate::log::error(format_args!("ASR failure: {error}"));
                        let _ = events.send(WorkerEvent::RecognitionError(error.to_string()));
                    }
                }
            }
            WorkerCommand::Shutdown => break,
        }
    }
}
