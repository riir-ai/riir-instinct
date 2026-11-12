//! Issue 031 G3 — the ARENA-side real-store cell: the swap gate's
//! verdict read against REAL lanes and REAL labelled traffic (plan 633's
//! T3 shape transposed to the decision-serving domain; the producer
//! cells live in riir-refine Benches 148/149/150).
//!
//! # The lane pair (real, decisive, in-repo)
//!
//! One suite — sst5 (5 score levels, the seat's cal slice carries gold) —
//! two REAL lanes constructed at EXPLICIT arms (the `from_parts_opt`
//! widening names this cell; the manifest row cannot pin two postures):
//!
//! - **SPECIALIST**: the trained winner artifact at arm A1
//!   (`SuiteServer::<5>::from_bytes(.., Arm::A1)` — the receipt's
//!   `specialist_scores` is Some; the specialist decides).
//! - **MODELLESS**: `HybridLane::ReflexOnly` at arm A0
//!   (`from_parts_opt(spec: None, .., Arm::A0)` — the modelless tier;
//!   `probabilities` is Some; never a specialist consult).
//!
//! The measured pair on the sst5 cal front is a real, decisive
//! regression (the first run: specialist 0.375 vs modelless 0.000 — the
//! served modelless posture's fitted fused-gate abstains on the whole
//! cal front; the harness's own rows are FORCED reads, the serve lane's
//! gate abstains for real).
//!
//! # The cell (frozen read, deterministic)
//!
//! One edge, the loader's mode chooses the lane at boot/swap time:
//!
//! 1. Boot SPECIALIST, serve 32 real cal-slice decisions (the window
//!    cap) — the incumbent window + rings carry the specialist's
//!    outcomes.
//! 2. Swap epoch 1 → MODELLESS → **Keep expected** (the abstainer
//!    answers nothing the specialist accepted — retained 0; the true
//!    regression) → 409 `swap_regret`; then `force` installs the
//!    regression (the operator seam) and the post-install reset clears
//!    the history.
//! 3. Serve 32 fresh decisions — the modelless window (the degenerate
//!    all-abstain shape).
//! 4. Swap epoch 2 → SPECIALIST → the DEGENERATE-SHAPE leg: the
//!    all-abstain incumbent carries no accepted evidence; the verdict
//!    (either shape) is RECORDED VERBATIM, never folded into P1's
//!    polarity. A second, FRESH edge runs the same degenerate shape
//!    from a cold boot — the two orderings' verdicts are compared and
//!    any disagreement is disclosed as the order-dependence finding
//!    (the producer's α-tie / crossover class, Bench 150's mechanism —
//!    a producer follow-up, never silently pooled).
//!
//! Ground truth: per-lane accuracy over the SAME window cases from the
//! served picks (`pick_index == gold.idx`), so the verdict is read
//! against the MEASURED delta, never the board's numbers.
//!
//! # Pre-registered bars
//!
//! - **P1 (the regression read)**: leg 2 Keep (409, retained 0) AND the
//!   probe consumed the full window AND acc(specialist) > acc(modelless).
//! - **W (the wire)**: the regression force-installs; the probe counter
//!   (a counting LaneBackend wrapper — every decide through the lane,
//!   capture and probe alike) reads the full window on both verdict
//!   legs.
//! - **P2 (the degenerate shape)**: both degenerate legs run and their
//!   verdicts are recorded verbatim; a disagreement between the
//!   post-reset and fresh-boot orderings is DISCLOSED (the
//!   order-dependence finding), not asserted either way.
//!
//! A bar may fail honestly — the record names the contradiction (the
//! thin-window noise class is known from the producer's Bench 148/149).
//!
//! # The persisted-window design decision (recorded here)
//!
//! The window stays IN-MEMORY (32/suite, process-local): a swap verdict
//! only ever reads the lane that ACTUALLY served the recent traffic in
//! THIS process — the A2 install path needs no history across restarts
//! (the first swap after a boot sees an empty window = NoEvidence =
//! install, the designed cold-boot posture). Persisting windows would
//! add a data class at rest (a serving store) for a verdict that is
//! advisory beside A2 — declined. Re-open only if a consumer needs
//! swap protection across process restarts.
//!
//! Runs at `cargo run --release --example adoption_swap_g3_arena
//! --features adoption_gate`. Exit 0 iff every bar is green; a missing
//! dataset/winner seat skips loud.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use riir_instinct::arsenal::ArsenalManifest;
use riir_instinct::serve_edge::{LoadCtx, LoadedLane, ServeConfig};
use riir_instinct::server::{
    AnySuiteServer, Arm, LaneBackend, ServedDecision, ServedQuestion, SuiteMeta, SuiteServer,
    WeightSource,
};

fn datasets_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../riir-reflex/.raw/datasets_t20k")
}

fn winners_source() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../riir-train/data/instinct_specialists")
}

/// The loader's lane mode for the NEXT boot (the swap edge and the boot
/// path both read it — one mode per boot, set before each swap).
static MODE: AtomicUsize = AtomicUsize::new(0); // 0 = modelless, 1 = specialist
/// Every decide through the WRAPPED lane (capture + probe + serving) —
/// the probe observable: the delta across a swap IS the probe call count.
static DECIDES: AtomicUsize = AtomicUsize::new(0);

const MODELLESS: usize = 0;
const SPECIALIST: usize = 1;

fn set_mode(m: usize) {
    MODE.store(m, Ordering::SeqCst);
}

const SUITE: &str = "sst5";
const WINDOW: usize = 32; // the swap gate's WINDOW_CAP — fill it fully

/// The counting wrapper: the real lane behind a LaneBackend that counts
/// every decide (the probe observable; the serve edge and the swap
/// probe both dispatch through the trait).
struct CountingLane {
    inner: AnySuiteServer,
}

impl LaneBackend for CountingLane {
    fn decide(
        &mut self,
        state: &str,
        options: Option<&[String]>,
    ) -> Result<ServedDecision, String> {
        DECIDES.fetch_add(1, Ordering::SeqCst);
        self.inner.decide(state, options)
    }

    fn decide_multi(
        &mut self,
        state: &str,
        questions: &[ServedQuestion<'_>],
    ) -> Result<Vec<ServedDecision>, String> {
        DECIDES.fetch_add(questions.len(), Ordering::SeqCst);
        self.inner.decide_multi(state, questions)
    }

    fn meta(&self) -> &SuiteMeta {
        self.inner.meta()
    }

    fn centroid(&self) -> [f32; riir_instinct::arsenal_ops::DIM] {
        self.inner.centroid()
    }
}

fn boot_lane(
    suite: &'static str,
    seat: riir_reflex::harness::runner::seat::Seat,
) -> AnySuiteServer {
    let inner: Box<SuiteServer<5>> = match MODE.load(Ordering::SeqCst) {
        SPECIALIST => Box::new(
            SuiteServer::<5>::from_bytes(
                suite,
                seat,
                &{
                    std::fs::read(winners_source().join("sst5_winner_v1.bin"))
                        .expect("winner bytes")
                },
                Arm::A1,
            )
            .expect("the specialist lane boots"),
        ),
        _ => Box::new(
            SuiteServer::<5>::from_parts_opt(
                suite,
                seat,
                None,
                "a0-reflex-only".to_string(),
                WeightSource::ReflexOnly,
                Arm::A0,
            )
            .expect("the modelless lane boots"),
        ),
    };
    AnySuiteServer::Ext(Box::new(CountingLane {
        inner: AnySuiteServer::S5(inner),
    }))
}

/// One arena edge (the OnceLock ports are per-scenario; the loader is
/// the explicit-arm pair constructor behind the counting wrapper).
fn edge_port(tag: &'static str) -> u16 {
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

    let winner_bytes =
        std::fs::read(winners_source().join("sst5_winner_v1.bin")).expect("winner artifact");
    let digest_hex = blake3::hash(&winner_bytes).to_hex().to_string();
    // The row: arm A0 (any posture — the loader never consults it; A0
    // keeps the row honest for the ReflexOnly leg), digest = the real
    // winner bytes (the swap tag's digest half), eager budget.
    let text = format!(
        "[[vessel]]\nsuite   = \"{SUITE}\"\ndigest  = \"blake3:{digest_hex}\"\nclass   = \
         \"hosted_only\"\nfile    = \"sst5_winner_v1.bin\"\nposture = {{ arm = \"A0\" \
         }}\npin_keys = []\nbudget  = {{ load = \"eager\", max_payload_mb = 16 }}\n"
    );
    let manifest_digest = ArsenalManifest::digest_of(&text);
    let manifest = std::sync::Arc::new(ArsenalManifest::parse(&text).expect("manifest parses"));
    let probe = std::net::TcpListener::bind("127.0.0.1:0").expect("probe bind");
    let port = probe.local_addr().expect("probe addr").port();
    drop(probe);
    let winners = std::env::temp_dir().join(format!("i031_g3_arena_{tag}_{}", std::process::id()));
    std::fs::create_dir_all(&winners).expect("winners dir");
    std::fs::copy(
        winners_source().join("sst5_winner_v1.bin"),
        winners.join("sst5_winner_v1.bin"),
    )
    .expect("stage the winner");

    fn arena_loader(
        lctx: &LoadCtx<'_>,
        suite: &'static str,
        _artifact: Option<&str>,
    ) -> Result<LoadedLane, String> {
        let seat = riir_instinct::serve_edge::prepare_lane_seat(lctx, suite)?;
        let row = lctx
            .manifest
            .row(suite)
            .ok_or_else(|| format!("suite {suite} is not in the arsenal manifest"))?;
        let tag = riir_instinct::serve_edge::lane_tag_digest(row, suite);
        Ok(LoadedLane {
            server: boot_lane(suite, seat),
            artifact_digest: tag,
            on_install: None,
        })
    }

    let cfg = ServeConfig {
        bind: format!("127.0.0.1:{port}"),
        datasets_dir: datasets_dir().to_string_lossy().into_owned(),
        winners_dir: winners.to_string_lossy().into_owned(),
        synth_corpus_dir: None,
        arsenal_path: None,
        suite_filter: Some(vec![SUITE.to_string()]),
        cors_origin: Some(vec![]),
        prevalidated_manifest: Some((manifest, manifest_digest, "g3 arena".into())),
        lane_loader: Some(arena_loader),
        origin_gate: None,
        on_decision: None,
    };
    std::thread::Builder::new()
        .name("g3-arena-edge".into())
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
    s.set_read_timeout(Some(Duration::from_secs(60))).ok();
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

/// One window case: the state string to POST + the gold level index.
struct WindowCase {
    state: String,
    gold: usize,
}

/// Serve the window: POST each case, read the pick, score against gold.
/// Returns (correct count, abstain count).
fn serve_window(port: u16, cases: &[WindowCase]) -> (usize, usize) {
    let mut correct = 0usize;
    let mut abstained = 0usize;
    for (i, c) in cases.iter().enumerate() {
        let body = format!(
            r#"{{"contract_version":1,"suite":"{SUITE}","state":{} }}"#,
            serde_json::to_string(&c.state).expect("state json")
        );
        let (status, resp) = http(port, "POST /decide HTTP/1.1", Some(&body));
        assert_eq!(status, 200, "decide {i} failed: {resp}");
        let v: serde_json::Value =
            serde_json::from_str(&resp).unwrap_or_else(|e| panic!("decide {i} body: {e} — {resp}"));
        let pick_idx = v["pick_index"].as_u64();
        let abst = v["abstained"].as_bool().unwrap_or(false);
        if abst || pick_idx.is_none() {
            abstained += 1;
        } else if pick_idx == Some(c.gold as u64) {
            correct += 1;
        }
    }
    (correct, abstained)
}

fn swap(port: u16, epoch: u64, force: bool) -> (u16, String) {
    let force_part = if force { r#","force":true"# } else { "" };
    let body = format!(
        r#"{{"suite":"{SUITE}","artifact":"sst5_winner_v1.bin","epoch":{epoch}{force_part}}}"#
    );
    http(port, "POST /arsenal/swap HTTP/1.1", Some(&body))
}

/// The probe delta across one swap HTTP call (nothing else serves during
/// it): the verdict's probe calls through the counting wrapper.
fn swap_probed(port: u16, epoch: u64, force: bool) -> ((u16, String), u64) {
    let before = DECIDES.load(Ordering::SeqCst);
    let r = swap(port, epoch, force);
    let delta = (DECIDES.load(Ordering::SeqCst) - before) as u64;
    (r, delta)
}

fn main() {
    std::env::set_var("INSTINCT_ADOPTION_GATE", "1");
    let args: Vec<String> = std::env::args().collect();
    let quiet = args.iter().any(|a| a == "--quiet");

    if !datasets_dir().join(SUITE).is_dir()
        || !winners_source().join("sst5_winner_v1.bin").is_file()
    {
        eprintln!(
            "SKIP loud: the sst5 datasets or the winner artifact are absent — the \
             arena cell cannot boot a real lane (a skip is a deferral, never a green)"
        );
        return;
    }

    // The window cases: the seat's cal slice front (gold labelled), the
    // state strings serialized the harness's own way.
    let seat = riir_reflex::harness::runner::seat::prepare_seat(SUITE, &datasets_dir())
        .expect("prepare the sst5 seat");
    assert!(
        seat.cal_cases.len() >= WINDOW,
        "the cal slice ({}) must cover the window cap {WINDOW}",
        seat.cal_cases.len()
    );
    let window_cases: Vec<WindowCase> = seat.cal_cases[..WINDOW]
        .iter()
        .map(|c| WindowCase {
            state: riir_reflex::pyjson::serialize_state(&c.state),
            gold: c.gold[0].idx,
        })
        .collect();

    let mut failed: Vec<String> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let bar = |name: &str, ok: bool, detail: &str, failed: &mut Vec<String>| {
        if !quiet {
            println!("[{}] {name} {detail}", if ok { "PASS" } else { "FAIL" });
        }
        if !ok {
            failed.push(format!("{name}: {detail}"));
        }
    };

    // ── Leg 1: the SPECIALIST incumbent, the window fills ──
    set_mode(SPECIALIST);
    let port = edge_port("main");
    wait_ready(port, SUITE, 120);
    let t0 = Instant::now();
    let (correct_spec, abst_spec) = serve_window(port, &window_cases);
    let acc_spec = correct_spec as f64 / WINDOW as f64;
    record.push(format!(
        "leg1 incumbent=specialist: acc {correct_spec}/{WINDOW} = {acc_spec:.3} \
         (abstained {abst_spec}) in {:.1}s",
        t0.elapsed().as_secs_f64()
    ));

    // ── Leg 2: swap to the MODELLESS challenger — Keep expected (the
    // regression: the abstainer answers nothing the specialist accepted)
    // → 409, then force installs the regression (the operator seam).
    set_mode(MODELLESS);
    let ((status, body), probed) = swap_probed(port, 1, false);
    bar(
        "P1_verdict_keep_on_the_regression",
        status == 409
            && body.contains("\"code\":\"swap_regret\"")
            && body.contains("retained 0.000"),
        &format!("got {status} {body}"),
        &mut failed,
    );
    bar(
        "P1_probe_consumed_the_full_window",
        probed == WINDOW as u64,
        &format!("probe delta {probed}, want {WINDOW}"),
        &mut failed,
    );
    let (status, body) = swap(port, 1, true);
    bar(
        "W_force_bypass_installs_the_regression",
        status == 200 && body.contains("\"status\":\"forced\""),
        &format!("got {status} {body}"),
        &mut failed,
    );

    // The regression lane's own accuracy on the SAME window (it is now
    // the installed incumbent; deterministic).
    wait_ready(port, SUITE, 120);
    let (correct0, abst0) = serve_window(port, &window_cases);
    let acc_mless = correct0 as f64 / WINDOW as f64;
    record.push(format!(
        "leg2 challenger=modelless: acc {correct0}/{WINDOW} = {acc_mless:.3} (abstained {abst0})"
    ));
    bar(
        "P1_measured_delta_agrees",
        acc_spec > acc_mless,
        &format!(
            "specialist {acc_spec:.3} vs modelless {acc_mless:.3} — the keep verdict \
             must track the measured regression"
        ),
        &mut failed,
    );

    // ── Leg 3: the modelless window fills (the degenerate all-abstain
    // shape — serve_window above captured its decisions) ──

    // ── Leg 4: the DEGENERATE-SHAPE leg (post-reset ordering): swap to
    // SPECIALIST over the all-abstain incumbent's window. The verdict is
    // recorded verbatim — no polarity expectation (the producer's tie
    // arithmetic on an empty accepted ring is Bench 150's crossover
    // class).
    set_mode(SPECIALIST);
    let ((status4, body4), probed4) = swap_probed(port, 2, false);
    let verdict4 = if status4 == 409 && body4.contains("\"code\":\"swap_regret\"") {
        format!("409 swap_regret ({})", extract_scores(&body4))
    } else if status4 == 200 {
        format!("200 {}", installed_status(&body4))
    } else {
        format!("{status4} UNPARSEABLE {body4}")
    };
    record.push(format!(
        "leg4 degenerate (post-reset ordering): {verdict4}; probe delta {probed4}"
    ));
    bar(
        "P2_post_reset_degenerate_leg_runs",
        (status4 == 409 || status4 == 200) && probed4 == WINDOW as u64,
        &format!("got {status4}, probe delta {probed4} (want {WINDOW})"),
        &mut failed,
    );

    // ── Leg 5: the SAME degenerate shape from a FRESH edge (the
    // cold-boot ordering): boot MODELLESS, serve the window, swap to
    // SPECIALIST. The two orderings' verdicts are compared — a
    // disagreement is the order-dependence finding (disclosed, never
    // pooled).
    set_mode(MODELLESS);
    let port2 = edge_port("fresh");
    wait_ready(port2, SUITE, 120);
    let _ = serve_window(port2, &window_cases);
    set_mode(SPECIALIST);
    let ((status5, body5), probed5) = swap_probed(port2, 1, false);
    let verdict5 = if status5 == 409 && body5.contains("\"code\":\"swap_regret\"") {
        format!("409 swap_regret ({})", extract_scores(&body5))
    } else if status5 == 200 {
        format!("200 {}", installed_status(&body5))
    } else {
        format!("{status5} UNPARSEABLE {body5}")
    };
    record.push(format!(
        "leg5 degenerate (fresh-boot ordering): {verdict5}; probe delta {probed5}"
    ));
    bar(
        "P2_fresh_boot_degenerate_leg_runs",
        (status5 == 409 || status5 == 200) && probed5 == WINDOW as u64,
        &format!("got {status5}, probe delta {probed5} (want {WINDOW})"),
        &mut failed,
    );
    let agreed = verdict4.split(' ').take(1).collect::<String>()
        == verdict5.split(' ').take(1).collect::<String>();
    if agreed {
        record.push(format!(
            "order-dependence: the two degenerate orderings AGREE ({})",
            verdict5.split(' ').next().unwrap_or("?")
        ));
    } else {
        record.push(format!(
            "order-dependence FINDING: the orderings DISAGREE (post-reset \
             {verdict4} vs fresh-boot {verdict5}) — the producer's tie/crossover \
             arithmetic on an empty accepted ring is order-sensitive; a producer \
             follow-up, never pooled into the polarity read"
        ));
    }

    if !record.is_empty() && !quiet {
        println!("── cell record ──");
        for r in &record {
            println!("  {r}");
        }
    }

    if !failed.is_empty() {
        eprintln!("⛔ adoption_swap_g3_arena: {} bar(s) FAILED:", failed.len());
        for f in &failed {
            eprintln!("  - {f}");
        }
        std::process::exit(1);
    }
    if !quiet {
        println!(
            "adoption_swap_g3_arena: all bars GREEN (specialist {acc_spec:.3} vs \
             modelless {acc_mless:.3} on the same 32-case window; the regression \
             verdict tracked the measured delta; the degenerate legs recorded)"
        );
    }
}

/// The verdict body's retained/repeated/score triple (the refusal
/// detail), for the record.
fn extract_scores(body: &str) -> String {
    let retained = body.split("retained ").nth(1).and_then(|s| {
        s.split_whitespace()
            .next()
            .map(|w| w.trim_end_matches(',').to_string())
    });
    let repeated = body.split("repeated ").nth(1).and_then(|s| {
        s.split_whitespace()
            .next()
            .map(|w| w.trim_end_matches(',').to_string())
    });
    let score = body
        .split("challenger score ")
        .nth(1)
        .and_then(|s| {
            s.split_whitespace()
                .next()
                .map(|w| w.trim_end_matches(',').to_string())
        })
        .or_else(|| {
            body.split("score ")
                .nth(1)
                .and_then(|s| s.split(' ').next().map(|w| w.to_string()))
        });
    format!(
        "retained {} repeated {} score {}",
        retained.as_deref().unwrap_or("?"),
        repeated.as_deref().unwrap_or("?"),
        score.as_deref().unwrap_or("?")
    )
}

/// The install body's status word.
fn installed_status(body: &str) -> String {
    body.split("\"status\":\"")
        .nth(1)
        .and_then(|s| s.split('"').next().map(str::to_string))
        .unwrap_or_else(|| "unknown".to_string())
}
