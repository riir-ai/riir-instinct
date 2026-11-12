//! Issue 031 G3 — the adoption/retention gate's WIRE gate: the edge's
//! capture → window → swap-verdict → refuse path, exercised end to end
//! over HTTP with a stub lane (the decision_capture_gates +
//! ext_seat_rerank_gates conventions — one ephemeral edge per scenario,
//! a programmable stub backend, no datasets beyond the sst5 seat
//! vehicle, no winner artifacts).
//!
//! The six bars (the issue's G3 at the wire level — the arena-side
//! measurement over a real serving store stays the follow-up unit):
//!
//! 1. **Adopt path**: a served window + a challenger that reproduces the
//!    incumbent's answers → the swap installs (200 advanced).
//! 2. **Regret refused**: a challenger that abstains the whole accepted
//!    window → 409 `swap_regret` (decline-first: nothing answered keeps
//!    the incumbent — the producer's NoEvidence-on-challenger shape).
//! 3. **Force bypass**: the same regret posture under `"force":true`
//!    installs (200 forced) — the operator seam, logged at the edge.
//! 4. **Post-install reset**: after the forced install, fresh served
//!    traffic rebuilds the window FROM THE NEW LANE; the next no-force
//!    swap adjudicates on that fresh window alone — the dead lane's
//!    history never leaks into a later verdict (the probe: an all-abstain
//!    challenger vs the fresh window re-derives the decline-first KEEP
//!    from the RESET rings, proving the old evidence is gone).
//!
//! Runs at `cargo run --release --example adoption_swap_g3 --features
//! adoption_gate` (its own required-features row — the P43 law). Exit 0
//! iff every bar is green; a named failure exits 1. A missing sst5
//! dataset seat skips loud (exit 0 with the disclosure — a deferral,
//! never a green).

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use riir_instinct::arsenal::ArsenalManifest;
use riir_instinct::serve_edge::{LoadCtx, LoadedLane, ServeConfig};
use riir_instinct::server::{Arm, ExtBoots, LaneBackend, SuiteMeta, WeightSource};

fn datasets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../riir-reflex/.raw/datasets_t20k")
}

/// The stub's verdict mode — what the challenger (and the incumbent,
/// while capturing) answer. Static because `install_ext_boots` is
/// once-per-process and fn pointers cannot capture.
static MODE: AtomicUsize = AtomicUsize::new(0); // 0 = faithful, 1 = regret
static SERVED: AtomicUsize = AtomicUsize::new(0);

const FAITHFUL: usize = 0;
const REGRET: usize = 1;

fn set_mode(m: usize) {
    MODE.store(m, Ordering::SeqCst);
}

/// One stub decision: pick index 0, never abstains at FAITHFUL; at
/// REGRET it abstains EVERY case (the regret challenger — answers
/// nothing the window holds).
fn stub_decision(suite: &'static str, options: &[String]) -> riir_instinct::server::ServedDecision {
    let regret = MODE.load(Ordering::SeqCst) == REGRET;
    let presented: Vec<String> = if options.is_empty() {
        vec!["a".to_string(), "b".to_string()]
    } else {
        options.to_vec()
    };
    riir_instinct::server::ServedDecision {
        suite,
        arm: "ENC".to_string(),
        pick_index: if regret { None } else { Some(0) },
        pick: if regret {
            None
        } else {
            Some(presented[0].clone())
        },
        options: presented.clone(),
        probabilities: None,
        specialist_scores: Some(vec![0.1; presented.len()]),
        confidence: 0.9,
        escalated: false,
        abstained: regret,
        us: 7,
        gate_abstained: false,
    }
}

struct StubBag {
    suite: &'static str,
}

fn stub_meta(suite: &'static str) -> &'static SuiteMeta {
    static META: OnceLock<SuiteMeta> = OnceLock::new();
    META.get_or_init(|| SuiteMeta {
        suite,
        arm: Arm::Enc,
        labels: 5,
        artifact_labels: 5,
        effective_cap: 8,
        head_scale: 1.0,
        nb_scale: 1.0,
        score_threshold: 0.0,
        distance_threshold: 0.0,
        winner_blake3: "stub".to_string(),
        source: WeightSource::RawWinner,
    })
}

impl LaneBackend for StubBag {
    fn decide(
        &mut self,
        _state: &str,
        options: Option<&[String]>,
    ) -> Result<riir_instinct::server::ServedDecision, String> {
        SERVED.fetch_add(1, Ordering::SeqCst);
        Ok(stub_decision(self.suite, options.unwrap_or(&[])))
    }

    fn decide_multi(
        &mut self,
        state: &str,
        questions: &[riir_instinct::server::ServedQuestion<'_>],
    ) -> Result<Vec<riir_instinct::server::ServedDecision>, String> {
        questions
            .iter()
            .map(|q| {
                SERVED.fetch_add(1, Ordering::SeqCst);
                Ok(stub_decision(self.suite, q.options))
            })
            .collect::<Result<Vec<_>, String>>()
            .map(|_| {
                // The single-question form is the capture surface this
                // gate exercises; the multi form is unused here, answered
                // through the same stub for completeness.
                let _ = state;
                questions
                    .iter()
                    .map(|q| stub_decision(self.suite, q.options))
                    .collect()
            })
    }

    fn meta(&self) -> &SuiteMeta {
        stub_meta(self.suite)
    }

    fn centroid(&self) -> [f32; riir_instinct::arsenal_ops::DIM] {
        [0.0; riir_instinct::arsenal_ops::DIM]
    }
}

fn install_stub_once() {
    static INSTALLED: OnceLock<()> = OnceLock::new();
    INSTALLED.get_or_init(|| {
        riir_instinct::server::install_ext_boots(ExtBoots {
            bytes: |suite, _seat, _artifact_bytes, _arm| Ok(Box::new(StubBag { suite })),
        })
        .expect("the stub installs exactly once per process");
    });
}

/// The ENC manifest rows the gates seat (the rerank gates' grammar):
/// one row per suite, digest-pinned, EAGER budget (the incumbent boots
/// ready — the capture starts at the first served decision).
fn manifest_text(suite: &str) -> String {
    format!(
        "[[vessel]]\nsuite   = \"{suite}\"\ndigest  = \"blake3:{}\"\nclass   = \
         \"hosted_only\"\nfile    = \"stub_head.bin\"\nposture = {{ arm = \"ENC\" \
         }}\npin_keys = []\nbudget  = {{ load = \"eager\", max_payload_mb = 16 }}\n",
        "0".repeat(64)
    )
}

/// The winners dir: a per-edge temp dir carrying the stub artifact (the
/// swap edge reads `winners_dir/<file>` — the loader itself never reads
/// the bytes, the ENC route hands them to the installed backend).
fn stub_winners_dir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("i031_g3_{tag}_{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("winners dir");
    std::fs::write(dir.join("stub_head.bin"), b"g3-stub-head-bytes").expect("stub artifact");
    dir
}

/// Boot one edge (one per scenario — the install-once backend is shared,
/// the windows are per-suite-per-edge-process so each scenario gets its
/// own suite name) and return its port.
fn edge_port(tag: &'static str, suite: &'static str) -> u16 {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    static PORTS: OnceLock<std::sync::Mutex<Vec<u16>>> = OnceLock::new();
    let idx = NEXT.fetch_add(1, Ordering::SeqCst);
    let slot = PORTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .expect("ports lock");
    if let Some(&p) = slot.get(idx) {
        return p;
    }
    drop(slot);
    install_stub_once();
    let text = manifest_text(suite);
    let digest = ArsenalManifest::digest_of(&text);
    let manifest = std::sync::Arc::new(ArsenalManifest::parse(&text).expect("manifest parses"));
    let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("probe bind");
    let port = probe.local_addr().expect("probe addr").port();
    drop(probe);
    let winners = stub_winners_dir(tag);

    // The vehicle loader (the LRU gates' shape): every suite seats via
    // the sst5 dataset vehicle; the bytes route to the stub (ENC).
    fn vehicle_loader(
        lctx: &LoadCtx<'_>,
        suite: &'static str,
        _artifact: Option<&str>,
    ) -> Result<LoadedLane, String> {
        let seat = riir_instinct::serve_edge::prepare_lane_seat(lctx, "sst5")?;
        let row = lctx
            .manifest
            .row(suite)
            .ok_or_else(|| format!("suite {suite} is not in the arsenal manifest"))?;
        let digest = riir_instinct::serve_edge::lane_tag_digest(row, suite);
        let server = riir_instinct::server::AnySuiteServer::boot_bytes(
            suite,
            seat,
            b"g3-stub-head-bytes",
            lctx.manifest,
        )?;
        Ok(LoadedLane {
            server,
            artifact_digest: digest,
            on_install: None,
        })
    }

    let cfg = ServeConfig {
        bind: format!("127.0.0.1:{port}"),
        datasets_dir: datasets_dir().to_string_lossy().into_owned(),
        winners_dir: winners.to_string_lossy().into_owned(),
        synth_corpus_dir: None,
        arsenal_path: None,
        suite_filter: Some(vec![suite.to_string()]),
        cors_origin: Some(vec![]),
        prevalidated_manifest: Some((manifest, digest, "g3 gate".into())),
        lane_loader: Some(vehicle_loader),
        origin_gate: None,
        on_decision: None,
    };
    std::thread::Builder::new()
        .name("g3-edge".into())
        .spawn(move || {
            let _ = riir_instinct::serve_edge::run(cfg);
        })
        .expect("edge thread");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the edge never opened port {port}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    PORTS
        .get_or_init(|| std::sync::Mutex::new(Vec::new()))
        .lock()
        .expect("ports lock")
        .push(port);
    port
}

fn http(port: u16, req: &str, body: Option<&str>) -> (u16, String) {
    let mut s = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    s.set_read_timeout(Some(Duration::from_secs(30))).ok();
    let head = match body {
        Some(b) => format!(
            "{req}\r\nHost: 127.0.0.1:{port}\r\nContent-Type: application/json\r\n\
             Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            b.len(),
            b
        ),
        None => format!("{req}\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"),
    };
    s.write_all(head.as_bytes()).expect("write");
    let mut raw = String::new();
    let _ = s.read_to_string(&mut raw);
    let status = raw
        .split_whitespace()
        .nth(1)
        .and_then(|w| w.parse::<u16>().ok())
        .unwrap_or(0);
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, b)| b.to_string())
        .unwrap_or_default();
    (status, body)
}

fn healthz_state(port: u16, suite: &str) -> String {
    let (_, body) = http(port, "GET /healthz HTTP/1.1", None);
    // Minimal scrape: the suite's state word (healthz is one flat JSON).
    let needle = format!("\"{suite}\":{{\"state\":\"");
    match body.find(&needle) {
        Some(i) => body[i + needle.len()..]
            .split('"')
            .next()
            .unwrap_or("absent")
            .to_string(),
        None => "absent".to_string(),
    }
}

fn wait_ready(port: u16, suite: &str, secs: u64) {
    let deadline = Instant::now() + Duration::from_secs(secs);
    loop {
        if healthz_state(port, suite) == "ready" {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "the {suite} lane never reached ready; last healthz: {}",
            healthz_state(port, suite)
        );
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// Serve `n` single-question decisions (the window capture surface).
fn serve_n(port: u16, suite: &str, n: usize) {
    wait_ready(port, suite, 30);
    for i in 0..n {
        let body = format!(
            r#"{{"contract_version":1,"suite":"{suite}","state":"g3 case {i}","options":["a","b"]}}"#
        );
        let (status, body) = http(port, "POST /decide HTTP/1.1", Some(&body));
        assert_eq!(
            status, 200,
            "the decide must answer 200 (got {status}: {body})"
        );
    }
}

fn swap(port: u16, suite: &str, epoch: u64, force: bool) -> (u16, String) {
    let force_part = if force { r#","force":true"# } else { "" };
    let body =
        format!(r#"{{"suite":"{suite}","artifact":"stub_head.bin","epoch":{epoch}{force_part}}}"#);
    http(port, "POST /arsenal/swap HTTP/1.1", Some(&body))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let quiet = args.iter().any(|a| a == "--quiet");
    if args.iter().any(|a| a == "--disarmed-child") {
        // The G2 child: the env was REMOVED by the spawner and the child
        // itself NEVER sets it — the gate is disarmed. Run the regret
        // scenario: the swap must INSTALL and probe NOTHING (no capture,
        // no verdict — the wire is the pre-gate posture, byte-identical).
        set_mode(FAITHFUL);
        let port = edge_port("disarmed", "g3-disarmed");
        serve_n(port, "g3-disarmed", 5);
        set_mode(REGRET);
        let before = SERVED.load(Ordering::SeqCst);
        let (status, body) = swap(port, "g3-disarmed", 1, false);
        let probed = SERVED.load(Ordering::SeqCst) - before;
        let ok = status == 200 && body.contains("\"status\":\"advanced\"") && probed == 0;
        println!(
            "DISARMED_OK probe_delta={probed} status={status} {}",
            if ok {
                "(installed, no probe, no verdict)"
            } else {
                &body
            }
        );
        std::process::exit(if ok { 0 } else { 1 });
    }

    // The arming env — the latch reads once per process; the disarmed
    // child above returned BEFORE this line, so it never arms.
    std::env::set_var("INSTINCT_ADOPTION_GATE", "1");

    let mut failed: Vec<String> = Vec::new();
    let bar = |name: &str, ok: bool, detail: &str, failed: &mut Vec<String>| {
        if !quiet {
            println!("[{}] {name} {}", if ok { "PASS" } else { "FAIL" }, detail);
        }
        if !ok {
            failed.push(format!("{name}: {detail}"));
        }
    };

    if !datasets_dir().join("sst5").is_dir() {
        eprintln!(
            "SKIP loud: the sst5 datasets are absent — the seat vehicle cannot boot \
             (a skip is a deferral, never a green)"
        );
        return;
    }

    // ── Bar 1: the adopt path ──
    {
        set_mode(FAITHFUL);
        let port = edge_port("adopt", "g3-adopt");
        serve_n(port, "g3-adopt", 5);
        let (status, body) = swap(port, "g3-adopt", 1, false);
        bar(
            "adopt_path_installs",
            status == 200 && body.contains("\"status\":\"advanced\""),
            &format!("got {status} {body}"),
            &mut failed,
        );
    }

    // ── Bar 2: the regret refusal ──
    {
        set_mode(FAITHFUL);
        let port = edge_port("regret", "g3-regret");
        serve_n(port, "g3-regret", 5);
        set_mode(REGRET);
        // The probe observable: the verdict probes EVERY window entry
        // through the stub, and the stub counts every decide call — the
        // SERVED delta across the swap IS the window size. Delta 5 pins
        // both halves at once: exactly 5 decisions were captured (the
        // capture wire — no more, no fewer) and the verdict consumed the
        // whole window (a vacuous-capture defense).
        let before = SERVED.load(Ordering::SeqCst);
        let (status, body) = swap(port, "g3-regret", 1, false);
        let probed = SERVED.load(Ordering::SeqCst) - before;
        bar(
            "regret_refused_swap_regret",
            status == 409 && body.contains("\"code\":\"swap_regret\""),
            &format!("got {status} {body}"),
            &mut failed,
        );
        bar(
            "verdict_probed_the_whole_captured_window",
            probed == 5,
            &format!(
                "probe delta {probed}, want 5 (the capture wire: exactly the 5 \
                 served decisions are in the window)"
            ),
            &mut failed,
        );

        // ── Bar 3: force bypass (same posture, same edge) ──
        let (status, body) = swap(port, "g3-regret", 1, true);
        bar(
            "force_bypass_installs",
            status == 200 && body.contains("\"status\":\"forced\""),
            &format!("got {status} {body}"),
            &mut failed,
        );

        // ── Bar 4: the post-install reset ──
        // Fresh served traffic rebuilds the window from the NEW lane; the
        // next no-force swap must adjudicate on ITS OWN evidence — an
        // all-abstain challenger against a fresh window is the decline-first
        // KEEP (retained 0, repeated 0), never the DEAD lane's rings (the
        // old regret verdict quoted the dead incumbent's score 0.5000; the
        // fresh-window verdict must carry the same refusal CLASS from
        // DIFFERENT evidence).
        set_mode(FAITHFUL);
        serve_n(port, "g3-regret", 2);
        set_mode(REGRET);
        // The reset observable: a LEAKED gate would still hold the dead
        // lane's 5 pre-install entries — the probe delta would read 7
        // (5 dead + 2 fresh). Delta 2 proves the forced install reset the
        // suite's gate: the next verdict adjudicates on the fresh window
        // alone (the refusal class re-derives from RESET evidence).
        let before = SERVED.load(Ordering::SeqCst);
        let (status, body) = swap(port, "g3-regret", 2, false);
        let probed = SERVED.load(Ordering::SeqCst) - before;
        bar(
            "post_install_reset_clears_history",
            status == 409 && body.contains("retained 0.000 repeated 0.000") && probed == 2,
            &format!(
                "got {status}, probe delta {probed} (want 2 — a leaked gate would \
                 probe 7: the dead lane's 5 + the fresh 2); body: {body}"
            ),
            &mut failed,
        );
    }

    let served = SERVED.load(Ordering::SeqCst);
    if served < 12 {
        // 5 (adopt) + 5 (regret) + 2 (reset-window rebuild); the probe
        // calls ride the stub's own counter — a total under the floor
        // means the capture never armed and the bars passed vacuously.
        failed.push(format!(
            "served-count floor: {served} < 12 — the capture likely never armed \
             (the bars would be green-zero)"
        ));
    }

    // ── Bar 5 (G2's wire face): the DISARMED posture ──
    // INSTINCT_ADOPTION_GATE unset in a child process: the identical
    // regret posture must INSTALL (no capture, no verdict — the gate is
    // silent) and the swap must NOT probe (SERVED delta 0 in the child).
    // The parent's env is restored after (the latch never re-reads, so
    // this is belt-and-braces for the example's own hygiene).
    {
        let exe = std::env::current_exe()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_default();
        let out = std::process::Command::new(&exe)
            .arg("--disarmed-child")
            .env_remove("INSTINCT_ADOPTION_GATE")
            .output()
            .expect("spawn the disarmed child");
        let ok = out.status.success()
            && String::from_utf8_lossy(&out.stdout).contains("DISARMED_OK probe_delta=0");
        bar(
            "disarmed_posture_installs_without_probing",
            ok,
            &format!(
                "child status {} — stdout: {}",
                out.status,
                String::from_utf8_lossy(&out.stdout).trim()
            ),
            &mut failed,
        );
    }

    if !failed.is_empty() {
        eprintln!("⛔ adoption_swap_g3: {} bar(s) FAILED:", failed.len());
        for f in &failed {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
    if !quiet {
        println!("adoption_swap_g3: all six bars GREEN (served {served} stub decisions)");
    }
}
