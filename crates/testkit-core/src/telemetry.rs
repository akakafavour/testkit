//! Process telemetry and runaway detection for hard-capped runs.
//!
//! A [`TelemetryWatcher`] samples one OS process and keeps a bounded history
//! of CPU and resident-memory readings. [`TelemetryWatcher::detect_runaway`]
//! turns that history into a human-readable [`RunawayFinding`] so the harness
//! can kill a process that is busy-looping or leaking before it takes the
//! host down.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Number of samples retained in memory (about two minutes at one sample per
/// second).
pub const HISTORY_CAPACITY: usize = 120;

/// A sustained CPU excursion must cover at least this many milliseconds before
/// it counts as runaway.
pub const CPU_LOOP_WINDOW_MS: i64 = 2_000;

/// Number of trailing samples that must be monotonic before memory growth is
/// treated as a leak.
pub const MEMORY_LEAK_MIN_SAMPLES: usize = 5;

/// Resource budget a single run may consume before it is considered hostile to
/// the host machine. Fully configurable to adapt dynamically to any developer
/// environment, device constraint, or cloud tier.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, serde::Deserialize)]
pub struct ResourceEnvelope {
    /// CPU ceiling in percent. Values above 100 are possible on multi-core
    /// hosts because sysinfo sums usage across cores.
    pub max_cpu_percent: f32,
    /// Resident set size ceiling in mebibytes.
    pub max_rss_mb: u64,
}

impl Default for ResourceEnvelope {
    fn default() -> Self {
        Self {
            max_cpu_percent: 95.0,
            max_rss_mb: 1536,
        }
    }
}

impl ResourceEnvelope {
    /// Create a fully custom envelope dynamically based on the project's target environment.
    pub fn custom(max_cpu_percent: f32, max_rss_mb: u64) -> Self {
        Self {
            max_cpu_percent,
            max_rss_mb,
        }
    }

    /// Budget for a constrained VPS (roughly 12 USD/mo): 95% CPU, 1.5 GiB RAM.
    pub fn twelve_dollar_server() -> Self {
        Self::default()
    }

    /// Micro-envelope for serverless / edge runtimes (AWS Lambda, Cloudflare Workers):
    /// 50% CPU, 128 MiB RAM.
    pub fn edge_serverless() -> Self {
        Self {
            max_cpu_percent: 50.0,
            max_rss_mb: 128,
        }
    }

    /// High-performance container / microservice envelope: 400% CPU (4 cores), 8 GiB RAM.
    pub fn high_throughput() -> Self {
        Self {
            max_cpu_percent: 400.0,
            max_rss_mb: 8192,
        }
    }
}

/// One point-in-time reading for the watched process.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TelemetrySample {
    /// Wall-clock time the reading was taken.
    pub timestamp: DateTime<Utc>,
    /// CPU usage in percent at the time of the sample.
    pub cpu_percent: f32,
    /// Resident set size in mebibytes at the time of the sample.
    pub rss_mb: u64,
}

/// The two runaway shapes the watcher can recognise.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum RunawayKind {
    /// Sustained CPU saturation.
    CpuLoop,
    /// Monotonically growing resident memory.
    MemoryLeak,
}

/// A detected runaway with a message suitable for a CLI or log line.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RunawayFinding {
    pub kind: RunawayKind,
    pub detail: String,
}

/// Samples a single process and classifies its behaviour against a
/// [`ResourceEnvelope`].
#[derive(Debug)]
pub struct TelemetryWatcher {
    system: System,
    pid: Pid,
    envelope: ResourceEnvelope,
    history: VecDeque<TelemetrySample>,
}

impl TelemetryWatcher {
    /// Starts watching `pid`.
    ///
    /// sysinfo needs two refreshes separated by at least
    /// [`sysinfo::MINIMUM_CPU_UPDATE_INTERVAL`] before process CPU usage is
    /// meaningful, so the constructor primes the counters once and waits that
    /// interval. Memory readings are valid immediately.
    pub fn new(pid: u32, envelope: ResourceEnvelope) -> Self {
        let pid = Pid::from_u32(pid);
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);
        Self {
            system,
            pid,
            envelope,
            history: VecDeque::with_capacity(HISTORY_CAPACITY),
        }
    }

    /// Refreshes the process counters, records a sample and returns it.
    ///
    /// If the process has already exited the sample reads `0.0` CPU and `0`
    /// MiB instead of failing.
    pub fn sample_now(&mut self) -> TelemetrySample {
        self.system
            .refresh_processes(ProcessesToUpdate::Some(&[self.pid]), true);
        let (cpu_percent, rss_mb) = match self.system.process(self.pid) {
            Some(process) => (process.cpu_usage(), process.memory() / (1024 * 1024)),
            None => (0.0, 0),
        };
        let sample = TelemetrySample {
            timestamp: Utc::now(),
            cpu_percent,
            rss_mb,
        };
        if self.history.len() == HISTORY_CAPACITY {
            self.history.pop_front();
        }
        self.history.push_back(sample.clone());
        sample
    }

    /// Read-only view of the retained history, oldest first.
    pub fn samples(&self) -> &VecDeque<TelemetrySample> {
        &self.history
    }

    /// The budget this watcher classifies against.
    pub fn envelope(&self) -> &ResourceEnvelope {
        &self.envelope
    }

    /// Highest CPU percent and highest RSS (MiB) seen so far.
    pub fn peak(&self) -> (f32, u64) {
        self.history
            .iter()
            .fold((0.0_f32, 0_u64), |(cpu_peak, rss_peak), sample| {
                (
                    cpu_peak.max(sample.cpu_percent),
                    rss_peak.max(sample.rss_mb),
                )
            })
    }

    /// Classifies the retained history, preferring CPU runaways when both
    /// patterns are present.
    pub fn detect_runaway(&self) -> Option<RunawayFinding> {
        self.detect_cpu_runaway()
            .or_else(|| self.detect_memory_growth())
    }

    /// Reports a CPU loop when every trailing sample sits above the envelope
    /// limit and those samples span at least [`CPU_LOOP_WINDOW_MS`].
    fn detect_cpu_runaway(&self) -> Option<RunawayFinding> {
        let limit = self.envelope.max_cpu_percent;
        let run_start = self
            .history
            .iter()
            .rposition(|sample| sample.cpu_percent <= limit)
            .map_or(0, |index| index + 1);
        let run: Vec<&TelemetrySample> = self.history.iter().skip(run_start).collect();
        let first = run.first()?;
        let last = run.last()?;
        let span_ms = last
            .timestamp
            .signed_duration_since(first.timestamp)
            .num_milliseconds();
        if span_ms < CPU_LOOP_WINDOW_MS {
            return None;
        }
        let minimum = run
            .iter()
            .map(|sample| sample.cpu_percent)
            .fold(f32::INFINITY, f32::min);
        let maximum = run
            .iter()
            .map(|sample| sample.cpu_percent)
            .fold(f32::NEG_INFINITY, f32::max);
        let average =
            run.iter().map(|sample| f64::from(sample.cpu_percent)).sum::<f64>() / run.len() as f64;
        Some(RunawayFinding {
            kind: RunawayKind::CpuLoop,
            detail: format!(
                "process held CPU above the {limit:.1}% envelope for {:.1}s across {} samples \
                 (min {minimum:.1}%, avg {average:.1}%, peak {maximum:.1}%); suspect an infinite \
                 loop or a retry storm",
                span_ms as f64 / 1000.0,
                run.len(),
            ),
        })
    }

    /// Reports a leak when the last [`MEMORY_LEAK_MIN_SAMPLES`] samples never
    /// decrease and the latest RSS exceeds the envelope.
    fn detect_memory_growth(&self) -> Option<RunawayFinding> {
        if self.history.len() < MEMORY_LEAK_MIN_SAMPLES {
            return None;
        }
        let window_start = self.history.len() - MEMORY_LEAK_MIN_SAMPLES;
        let window: Vec<&TelemetrySample> = self.history.iter().skip(window_start).collect();
        if !window
            .windows(2)
            .all(|pair| pair[0].rss_mb <= pair[1].rss_mb)
        {
            return None;
        }
        let earliest = window.first()?;
        let latest = window.last()?;
        if latest.rss_mb <= self.envelope.max_rss_mb {
            return None;
        }
        let growth = latest.rss_mb.saturating_sub(earliest.rss_mb);
        let seconds = latest
            .timestamp
            .signed_duration_since(earliest.timestamp)
            .num_milliseconds() as f64
            / 1000.0;
        Some(RunawayFinding {
            kind: RunawayKind::MemoryLeak,
            detail: format!(
                "resident memory rose monotonically over the last {} samples ({} MB -> {} MB, \
                 +{} MB in {:.1}s) and now exceeds the {} MB envelope; treat as an unbounded \
                 cache or a leaked allocation",
                window.len(),
                earliest.rss_mb,
                latest.rss_mb,
                growth,
                seconds,
                self.envelope.max_rss_mb,
            ),
        })
    }
}
