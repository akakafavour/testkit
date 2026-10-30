//! Process telemetry and runaway detection for hard-capped runs.
//!
//! A [`TelemetryWatcher`] samples one OS process for the entire lifetime of a
//! run: the running CPU/RAM peaks never decay, and the retained sample
//! history decimates instead of forgetting its head, so a leak that creeps in
//! minutes into a run still reaches the receipt. [`TelemetryWatcher::detect_runaway`]
//! turns that history into a human-readable [`RunawayFinding`] so the harness
//! can kill a process that is busy-looping or leaking before it takes the
//! host down.

use std::collections::VecDeque;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sysinfo::{Pid, ProcessesToUpdate, System};

/// Number of samples retained in memory before decimation kicks in. The
/// retained history always covers the whole run: when the cap is reached every
/// second sample is dropped, doubling the stored interval instead of
/// forgetting the run's head.
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

    /// Render free web service: 512 MiB RAM, shared (burst) CPU. Verify the
    /// number against the provider's current plan page before relying on it.
    pub fn render_free() -> Self {
        Self {
            max_cpu_percent: 100.0,
            max_rss_mb: 512,
        }
    }

    /// Default serverless function tier (Vercel/Netlify-class): 1 GiB RAM,
    /// short CPU bursts. Verify against the provider's current plan page.
    pub fn vercel_function() -> Self {
        Self {
            max_cpu_percent: 200.0,
            max_rss_mb: 1024,
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
    /// Highest CPU percent observed since the watcher started, including
    /// samples already decimated out of `history`.
    peak_cpu_percent: f32,
    /// Highest RSS (MiB) observed since the watcher started, including
    /// samples already decimated out of `history`.
    peak_rss_mb: u64,
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
            peak_cpu_percent: 0.0,
            peak_rss_mb: 0,
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
        self.peak_cpu_percent = self.peak_cpu_percent.max(cpu_percent);
        self.peak_rss_mb = self.peak_rss_mb.max(rss_mb);
        if self.history.len() == HISTORY_CAPACITY {
            // Keep the whole run represented: drop every second retained
            // sample so the stored interval doubles rather than the history
            // forgetting its beginning.
            let decimated: VecDeque<TelemetrySample> = self
                .history
                .iter()
                .enumerate()
                .filter(|&(index, _)| index % 2 == 1)
                .map(|(_, sample)| sample.clone())
                .collect();
            self.history = decimated;
        }
        self.history.push_back(sample.clone());
        sample
    }

    /// Read-only view of the retained history, oldest first. The history is
    /// decimated once it passes [`HISTORY_CAPACITY`], so it always spans the
    /// whole run at a coarser interval rather than only its recent tail.
    pub fn samples(&self) -> &VecDeque<TelemetrySample> {
        &self.history
    }

    /// The budget this watcher classifies against.
    pub fn envelope(&self) -> &ResourceEnvelope {
        &self.envelope
    }

    /// Highest CPU percent and highest RSS (MiB) observed over the entire
    /// watch, including samples already decimated out of the retained history.
    pub fn peak(&self) -> (f32, u64) {
        (self.peak_cpu_percent, self.peak_rss_mb)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peaks_cover_the_whole_run_even_after_decimation() {
        let mut watcher = TelemetryWatcher::new(std::process::id(), ResourceEnvelope::default());
        let mut max_rss = 0u64;
        let mut max_cpu = 0.0_f32;
        for _ in 0..(HISTORY_CAPACITY + 10) {
            let sample = watcher.sample_now();
            max_rss = max_rss.max(sample.rss_mb);
            max_cpu = max_cpu.max(sample.cpu_percent);
        }

        let (peak_cpu, peak_rss) = watcher.peak();
        assert_eq!(peak_rss, max_rss, "peak RSS must include evicted samples");
        assert_eq!(peak_cpu, max_cpu, "peak CPU must include evicted samples");
        assert!(watcher.samples().len() <= HISTORY_CAPACITY);
        // The live test process always has a nonzero resident footprint.
        assert!(max_rss > 0);
    }

    #[test]
    fn envelope_presets_match_their_documented_budgets() {
        let edge = ResourceEnvelope::edge_serverless();
        assert_eq!(edge.max_cpu_percent, 50.0);
        assert_eq!(edge.max_rss_mb, 128);

        let high = ResourceEnvelope::high_throughput();
        assert_eq!(high.max_cpu_percent, 400.0);
        assert_eq!(high.max_rss_mb, 8192);
    }
}
