/*
 * Copyright (c) 2026 Bastiaan van der Plaat
 *
 * SPDX-License-Identifier: MIT
 */

//! Follows the tempo and beat of the music with the default microphone

use std::collections::VecDeque;
use std::f32::consts::TAU;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use log::{info, warn};

use crate::dmx::{DMX_STATE, Tempo};
use crate::ipc::{self, IpcMessage};

// MARK: Detector
/// Onset envelope frames per second
const ENVELOPE_RATE: f64 = 100.0;
/// Seconds of music the tempo is estimated from
const HISTORY_SECONDS: f64 = 8.0;
/// Seconds of music before the first estimate
const MIN_HISTORY_SECONDS: f64 = 4.0;
/// Seconds of music the beat phase is estimated from
const PHASE_SECONDS: f64 = 4.0;
const MIN_BPM: f64 = 60.0;
const MAX_BPM: f64 = 200.0;
/// Number of beats the onsets are correlated on
const COMB_BEATS: usize = 4;
/// Resolution of the beat period search in envelope frames
const LAG_STEP: f64 = 0.1;
/// Triangular kernel the onset envelope is smoothed with
const SMOOTHING: [f64; 5] = [1.0 / 9.0, 2.0 / 9.0, 3.0 / 9.0, 2.0 / 9.0, 1.0 / 9.0];
/// Center and width in octaves of the tempo preference, this resolves half and double tempo
const PREFERRED_BPM: f64 = 128.0;
const PREFERENCE_OCTAVES: f64 = 1.0;
/// Minimum autocorrelation of the onsets at the beat period, below it there is no clear beat
const MIN_CONFIDENCE: f64 = 0.15;
/// The tempo is the median of the last estimates, so a single wrong estimate is ignored
const MEDIAN_ESTIMATES: usize = 5;
/// The bass band follows the kick drum
const BASS_CUTOFF_HZ: f32 = 150.0;

/// Detected tempo and beat phase
#[derive(Debug, Clone, Copy)]
pub(crate) struct Beat {
    pub bpm: f32,
    /// Seconds from the last beat to the end of the processed samples
    pub since: f64,
}

/// Estimates the tempo from the autocorrelation of an onset envelope of the bass and full band
pub(crate) struct Detector {
    sample_rate: f64,
    hop: usize,
    bass_coefficient: f32,
    bass: f32,
    hop_samples: usize,
    /// Energy of the current frame and log energy of the last frame, for the bass and full band
    energy: [f32; 2],
    level: [f32; 2],
    envelope: VecDeque<f32>,
    estimates: VecDeque<f64>,
}

impl Detector {
    pub(crate) fn new(sample_rate: u32) -> Self {
        let sample_rate = sample_rate as f64;
        Self {
            sample_rate,
            hop: (sample_rate / ENVELOPE_RATE).round().max(1.0) as usize,
            bass_coefficient: 1.0 - (-TAU * BASS_CUTOFF_HZ / sample_rate as f32).exp(),
            bass: 0.0,
            hop_samples: 0,
            energy: [0.0; 2],
            level: [0.0; 2],
            envelope: VecDeque::new(),
            estimates: VecDeque::new(),
        }
    }

    /// Process mono samples
    pub(crate) fn process(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.bass += self.bass_coefficient * (sample - self.bass);
            self.energy[0] += self.bass * self.bass;
            self.energy[1] += sample * sample;
            self.hop_samples += 1;
            if self.hop_samples == self.hop {
                self.push_frame();
            }
        }
    }

    /// Onset strength is the rise of the compressed energy of each band
    fn push_frame(&mut self) {
        let mut onset = 0.0;
        for (energy, level) in self.energy.iter_mut().zip(&mut self.level) {
            let new_level = (1.0 + 1000.0 * *energy / self.hop as f32).ln();
            onset += (new_level - *level).max(0.0);
            *level = new_level;
            *energy = 0.0;
        }
        self.hop_samples = 0;
        self.envelope.push_back(onset);
        if self.envelope.len() > (HISTORY_SECONDS * ENVELOPE_RATE) as usize {
            self.envelope.pop_front();
        }
    }

    /// Estimate the tempo and beat phase, `None` while there is no clear beat
    pub(crate) fn estimate(&mut self) -> Option<Beat> {
        if self.envelope.len() < (MIN_HISTORY_SECONDS * ENVELOPE_RATE) as usize {
            return None;
        }
        // Smooth the onsets, beats of a period between whole frames land a frame apart
        let envelope: Vec<f64> = self.envelope.iter().map(|&x| x as f64).collect();
        let smoothed: Vec<f64> = envelope
            .windows(SMOOTHING.len())
            .map(|window| window.iter().zip(SMOOTHING).map(|(x, w)| x * w).sum())
            .collect();
        let mean = smoothed.iter().sum::<f64>() / smoothed.len() as f64;
        let onsets: Vec<f64> = smoothed.iter().map(|x| x - mean).collect();
        // Autocorrelation at a lag between frames, so every tempo correlates at its exact period
        let autocorrelation = |lag: f64| {
            let (whole, fraction) = (lag as usize, lag.fract());
            let count = onsets.len() - whole - 1;
            let sum = (0..count).map(|i| {
                let delayed =
                    onsets[i + whole] * (1.0 - fraction) + onsets[i + whole + 1] * fraction;
                onsets[i] * delayed
            });
            sum.sum::<f64>() / count as f64
        };
        let power = autocorrelation(0.0);
        if power < 1e-9 {
            return None;
        }

        // Beat period in envelope frames, correlating on multiple beats and weighted by the tempo
        // preference to pick between half and double tempo
        // Converts a lag to BPM and a BPM to lag
        let bpm_of = |lag: f64| 60.0 * ENVELOPE_RATE / lag;
        let max_multiple_lag = onsets.len() as f64 / 2.0;
        let multiples =
            |lag: f64| (1..=COMB_BEATS).take_while(move |&k| k as f64 * lag <= max_multiple_lag);
        let score = |lag: f64| {
            let comb = multiples(lag).map(|k| autocorrelation(k as f64 * lag));
            let octaves = (bpm_of(lag) / PREFERRED_BPM).log2() / PREFERENCE_OCTAVES;
            comb.sum::<f64>() / multiples(lag).count() as f64 * (-0.5 * octaves * octaves).exp()
        };
        let lags = |from: f64, to: f64| {
            (0..=((to - from) / LAG_STEP) as usize).map(move |step| from + step as f64 * LAG_STEP)
        };
        let lag = lags(bpm_of(MAX_BPM), bpm_of(MIN_BPM))
            .max_by(|&a, &b| score(a).total_cmp(&score(b)))?;
        if autocorrelation(lag) / power < MIN_CONFIDENCE {
            return None;
        }

        // Refine the period on the furthest beat, there the frame rounding is smallest
        let k = multiples(lag).last()? as f64;
        let lag = lags(k * lag - 1.0, k * lag + 1.0)
            .max_by(|&a, &b| autocorrelation(a).total_cmp(&autocorrelation(b)))?
            / k;
        self.estimates.push_back(bpm_of(lag));
        if self.estimates.len() > MEDIAN_ESTIMATES {
            self.estimates.pop_front();
        }
        let mut estimates: Vec<f64> = self.estimates.iter().copied().collect();
        estimates.sort_by(f64::total_cmp);
        let bpm = estimates[estimates.len() / 2];

        // Beat phase with the most onset strength on the recent beats
        let period = 60.0 * ENVELOPE_RATE / bpm;
        let last = self.envelope.len() - 1;
        let beats = (PHASE_SECONDS * ENVELOPE_RATE / period) as usize;
        let offset = (0..period.ceil() as usize).max_by(|&a, &b| {
            let strength = |offset: usize| {
                (0..beats)
                    .filter_map(|beat| {
                        let back = (offset as f64 + beat as f64 * period).round() as usize;
                        last.checked_sub(back).map(|index| self.envelope[index])
                    })
                    .sum::<f32>()
            };
            strength(a).total_cmp(&strength(b))
        })?;

        // An onset lands in the middle of its frame on average
        let since_samples = self.hop_samples as f64 + (offset as f64 + 0.5) * self.hop as f64;
        Some(Beat {
            bpm: bpm as f32,
            since: since_samples / self.sample_rate,
        })
    }
}

// MARK: Microphone
/// Seconds between tempo estimates
const ESTIMATE_INTERVAL: Duration = Duration::from_millis(500);
/// Tempo changes below this are ignored, so the beat clock stays steady
const BPM_TOLERANCE: f32 = 0.5;
/// Beat phase drift below this fraction of a beat is ignored
const PHASE_TOLERANCE: f64 = 0.1;

/// Stop flag of the running detector thread
static DETECTOR: Mutex<Option<Arc<AtomicBool>>> = Mutex::new(None);

pub(crate) fn is_enabled() -> bool {
    DETECTOR
        .lock()
        .expect("Failed to lock BPM detector")
        .is_some()
}

/// Start or stop following the tempo with the microphone
pub(crate) fn set_enabled(enabled: bool) {
    let mut detector = DETECTOR.lock().expect("Failed to lock BPM detector");
    if enabled == detector.is_some() {
        return;
    }
    if let Some(stop) = detector.take() {
        stop.store(true, Ordering::Relaxed);
        return;
    }
    let stop = Arc::new(AtomicBool::new(false));
    match thread::Builder::new().name("bpm".to_string()).spawn({
        let stop = stop.clone();
        move || detector_thread(&stop)
    }) {
        Ok(_) => *detector = Some(stop),
        Err(error) => warn!("Failed to spawn BPM thread: {error}"),
    }
}

fn detector_thread(stop: &Arc<AtomicBool>) {
    if let Err(error) = listen(stop) {
        warn!("Can't follow the tempo with the microphone: {error}");
        let mut detector = DETECTOR.lock().expect("Failed to lock BPM detector");
        if detector
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, stop))
        {
            *detector = None;
            ipc::broadcast(&IpcMessage::SetAutoBpm { auto_bpm: false });
        }
    }
}

fn listen(stop: &AtomicBool) -> Result<(), cpal::Error> {
    let device = cpal::default_host()
        .default_input_device()
        .ok_or(cpal::ErrorKind::DeviceNotAvailable)?;
    let config = device.default_input_config()?.config();
    let channels = config.channels as usize;

    let (sender, receiver) = mpsc::channel::<Result<Vec<f32>, cpal::Error>>();
    let error_sender = sender.clone();
    let stream = device.build_input_stream(
        config,
        move |data: &[f32], _: &cpal::InputCallbackInfo| {
            let mono = data
                .chunks(channels)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect();
            _ = sender.send(Ok(mono));
        },
        move |error| _ = error_sender.send(Err(error)),
        None,
    )?;
    stream.play()?;
    info!("Following the tempo with the microphone");

    let mut detector = Detector::new(config.sample_rate);
    let mut next_estimate = Instant::now() + ESTIMATE_INTERVAL;
    while !stop.load(Ordering::Relaxed) {
        match receiver.recv_timeout(ESTIMATE_INTERVAL) {
            Ok(samples) => detector.process(&samples?),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if Instant::now() >= next_estimate {
            next_estimate += ESTIMATE_INTERVAL;
            if let Some(beat) = detector.estimate() {
                follow(beat);
            }
        }
    }
    info!("Stopped following the tempo");
    Ok(())
}

/// Move the beat clock to the detected beat, while keeping the beat count so scripts don't
/// restart their bars
fn follow(beat: Beat) {
    let Some(last_beat) = Instant::now().checked_sub(Duration::from_secs_f64(beat.since)) else {
        return;
    };
    let mut dmx_state = DMX_STATE.lock().expect("Failed to lock DMX state");
    let tempo = dmx_state.tempo;
    let beats = tempo
        .downbeat
        .map(|downbeat| tempo.beats(last_beat.saturating_duration_since(downbeat)));
    let in_phase = beats.is_some_and(|beats| (beats - beats.round()).abs() < PHASE_TOLERANCE);
    if in_phase && (beat.bpm - tempo.bpm).abs() < BPM_TOLERANCE {
        return;
    }

    let beat_count = beats.map_or(0.0, f64::round);
    let since_downbeat = Duration::from_secs_f64(beat_count * 60.0 / beat.bpm as f64);
    dmx_state.tempo = Tempo {
        bpm: beat.bpm,
        downbeat: Some(last_beat.checked_sub(since_downbeat).unwrap_or(last_beat)),
    };
    drop(dmx_state);
    if beat.bpm.round() != tempo.bpm.round() {
        ipc::broadcast(&IpcMessage::SetBpm { bpm: beat.bpm });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_RATE: u32 = 48_000;
    /// Seconds before the first kick drum
    const OFFSET: f64 = 0.137;

    /// Kick drums on the beat and hi-hats between them over a noise floor
    fn track(bpm: f64, seconds: f64, offset: f64) -> Vec<f32> {
        let mut seed = 0x1234_5678u32;
        let mut noise = move || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            seed as f32 / u32::MAX as f32 * 2.0 - 1.0
        };
        let period = 60.0 / bpm;
        (0..(seconds * SAMPLE_RATE as f64) as usize)
            .map(|i| {
                let t = i as f64 / SAMPLE_RATE as f64 - offset;
                let since_beat = t.rem_euclid(period);
                let since_half = (t + period / 2.0).rem_euclid(period);
                let kick = (TAU as f64 * 60.0 * since_beat).sin() * (-since_beat * 25.0).exp();
                let hihat = noise() as f64 * 0.2 * (-since_half * 80.0).exp();
                (0.8 * kick + hihat + 0.02 * noise() as f64) as f32
            })
            .collect()
    }

    #[test]
    fn detects_tempo_and_phase() {
        for bpm in [90.0, 128.0, 174.0] {
            let mut detector = Detector::new(SAMPLE_RATE);
            let samples = track(bpm, 10.0, OFFSET);
            let mut beat = None;
            for chunk in samples.chunks(SAMPLE_RATE as usize / 2) {
                detector.process(chunk);
                beat = detector.estimate().or(beat);
            }
            let beat = beat.expect("No beat detected");
            assert!(
                (beat.bpm as f64 - bpm).abs() < 1.0,
                "Detected {} BPM for {bpm} BPM",
                beat.bpm
            );

            // The last beat before the end of the track
            let period = 60.0 / bpm;
            let since = (samples.len() as f64 / SAMPLE_RATE as f64 - OFFSET).rem_euclid(period);
            let error = (beat.since - since)
                .abs()
                .min(period - (beat.since - since).abs());
            assert!(error < 0.03, "Beat {}s ago instead of {since}s", beat.since);
        }
    }

    #[test]
    fn ignores_noise_and_silence() {
        let mut detector = Detector::new(SAMPLE_RATE);
        detector.process(&vec![0.0; SAMPLE_RATE as usize * 10]);
        assert!(detector.estimate().is_none());

        let mut seed = 42u32;
        let noise: Vec<f32> = (0..SAMPLE_RATE * 10)
            .map(|_| {
                seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (seed >> 8) as f32 / (1 << 24) as f32 - 0.5
            })
            .collect();
        let mut detector = Detector::new(SAMPLE_RATE);
        detector.process(&noise);
        assert!(detector.estimate().is_none());
    }
}
