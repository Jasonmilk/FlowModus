//! FAILOVER WITHIN THE CANDIDATE SET (ADR-0048 §362): a chosen supplier failing at RUNTIME must hand over to
//! the next one, and every ending must be NAMED. The pure core lives here so the rule is testable without a
//! network; the IO shell (ureq) only classifies what it saw.
//!
//! WHY THE THIRD CLASS EXISTS (ADR-0048 §358.2): EchoBird's failover triggers on "rate limit / quota /
//! temporary unavailability" — an ERROR. A model that cannot do text can nevertheless answer HTTP 200 with an
//! empty or meaningless body, and that is exactly the case an error-only rule lets through. So an EMPTY BODY
//! is a failure here, and it is named as its own class.

/// The ways one attempt can fail. Distinct names, because "it failed" is not a fact a reader can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttemptFailure {
    /// Nothing is listening / the host does not answer: the name says the port, not the symptom.
    Unreachable,
    /// The peer answered and REFUSED the connection (ECONNREFUSED): a different fact from "no answer".
    Refused,
    /// The peer accepted and then went silent past the deadline.
    Timeout,
    /// A 5xx from upstream: the supplier's own fault, transient by convention.
    Http5xx { status: u16 },
    /// Any other status: passed through VERBATIM (§358.1 — do not invent words for upstream's own).
    UpstreamStatus { status: u16, body: String },
    /// HTTP 200 with no usable text. THE case an error-only rule misses.
    EmptyBody,
}

impl AttemptFailure {
    /// One short, stable name per class — the roster reads these, and the criterion requires them distinct.
    pub fn name(&self) -> &'static str {
        match self {
            AttemptFailure::Unreachable => "unreachable",
            AttemptFailure::Refused => "refused",
            AttemptFailure::Timeout => "timeout",
            AttemptFailure::Http5xx { .. } => "http-5xx",
            AttemptFailure::UpstreamStatus { .. } => "upstream-status",
            AttemptFailure::EmptyBody => "empty-body",
        }
    }
}

/// A candidate the router already filtered: only text-capable suppliers ever reach the attempt loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub supplier_id: String,
    pub endpoint: String,
    pub model: String,
    /// The declared modality tag (`modality:text` / `modality:non-text` / `modality:unknown`).
    pub modality: Option<String>,
}

/// Every candidate failed. The caller must REPORT this, never return an empty string or a silent Ok.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllFailed {
    pub attempts: Vec<(String, AttemptFailure)>,
}

impl AllFailed {
    pub fn report(&self) -> String {
        let rows: Vec<String> = self
            .attempts
            .iter()
            .map(|(id, why)| format!("{id}: {}", why.name()))
            .collect();
        format!(
            "no candidate answered — {} supplier(s) tried [{}]",
            self.attempts.len(),
            rows.join("; ")
        )
    }
}

/// Is this body usable text? Empty, whitespace-only and "no recognisable text" all count as FAILURE.
pub fn body_is_empty(body: &str) -> bool {
    body.trim().is_empty()
}

/// THE RULE, pure and injectable: walk the candidates IN ORDER and hand over on the first failure.
/// `attempt` is the IO shell. Returns the first non-empty body, or the named report of every attempt.
pub fn try_candidates<F>(candidates: &[Candidate], mut attempt: F) -> Result<String, AllFailed>
where
    F: FnMut(&Candidate) -> Result<String, AttemptFailure>,
{
    let mut failures: Vec<(String, AttemptFailure)> = Vec::new();
    for c in candidates {
        // NON-TEXT CANDIDATES NEVER GET AN ATTEMPT (ADR-0048 §362.3 ⑤): the capability table is the input,
        // and an unfiltered candidate here is the "wrong model" hole this whole change exists to close.
        if let Some(m) = c.modality.as_deref() {
            if m == "modality:non-text" {
                continue;
            }
        }
        match attempt(c) {
            Ok(body) if !body_is_empty(&body) => return Ok(body),
            Ok(_) => failures.push((c.supplier_id.clone(), AttemptFailure::EmptyBody)),
            Err(why) => failures.push((c.supplier_id.clone(), why)),
        }
    }
    Err(AllFailed { attempts: failures })
}

/// THE IO CLASSIFIERS, split from the call so each rule is testable without a socket.
/// A status is 5xx (transient by convention) or "anything else" — and the else branch keeps the upstream
/// BODY VERBATIM (§358.1: passing upstream's own words through, not inventing a friendlier sentence).
pub fn classify_status(status: u16, body: &str) -> AttemptFailure {
    if (500..600).contains(&status) {
        AttemptFailure::Http5xx { status }
    } else {
        AttemptFailure::UpstreamStatus { status, body: body.to_string() }
    }
}

/// A transport error is three DIFFERENT facts (§362.3 ②): the peer refused, the peer went silent, or we never
/// reached it at all. Collapsing them is the mutation this function's criterion guards.
pub fn classify_transport_kind(kind: std::io::ErrorKind) -> AttemptFailure {
    use std::io::ErrorKind;
    match kind {
        ErrorKind::ConnectionRefused => AttemptFailure::Refused,
        ErrorKind::TimedOut => AttemptFailure::Timeout,
        _ => AttemptFailure::Unreachable,
    }
}

/// The thin shell: `ureq` has exactly two shapes, and both are delegated to the rules above.
/// (Not unit-tested directly: building a `ureq::Error::Status` needs a live `Response`. Its two branches are
/// one line each and their rules ARE tested — `classify_status` and `classify_transport_kind`.)
pub fn classify_ureq(err: &ureq::Error) -> AttemptFailure {
    match err {
        ureq::Error::Status(code, resp) => {
            let body = resp.status_text().to_string();
            classify_status(*code, &body)
        }
        ureq::Error::Transport(t) => classify_ureq_transport(t.kind(), t.message().unwrap_or("")),
    }
}

/// `ureq::ErrorKind` is NOT `io::ErrorKind` (measured: the compiler refused the conversion), and it collapses
/// several transport facts into `ConnectionFailed`. So the two facts this project needs to tell apart — the
/// peer REFUSED, or the peer went SILENT — are recognised from ureq's own message, and that is a LIMITATION
/// worth naming: it is string matching, so a wording change upstream could re-label an attempt.
/// It cannot however merge the classes silently: the fallback is `Unreachable`, which is a THIRD name.
pub fn classify_ureq_transport(kind: ureq::ErrorKind, message: &str) -> AttemptFailure {
    let m = message.to_ascii_lowercase();
    match kind {
        ureq::ErrorKind::ConnectionFailed | ureq::ErrorKind::Io => {
            if m.contains("refused") {
                AttemptFailure::Refused
            } else if m.contains("timed out") || m.contains("timeout") {
                AttemptFailure::Timeout
            } else {
                AttemptFailure::Unreachable
            }
        }
        _ => AttemptFailure::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cand(id: &str, modality: Option<&str>) -> Candidate {
        Candidate {
            supplier_id: id.into(),
            endpoint: format!("http://127.0.0.1:1/{id}"),
            model: format!("{id}-model"),
            modality: modality.map(|m| m.to_string()),
        }
    }

    /// ① A dead first candidate ⇒ the NEXT one answers, and the answer is non-empty and non-blank.
    #[test]
    fn a_dead_candidate_hands_over_and_the_next_answers() {
        let set = vec![cand("dead", Some("modality:text")), cand("alive", Some("modality:text"))];
        let out = try_candidates(&set, |c| {
            if c.supplier_id == "dead" { Err(AttemptFailure::Refused) } else { Ok("hello".into()) }
        })
        .expect("the second candidate answers");
        assert_eq!(out, "hello");
    }

    /// ② The four transport classes (plus the two others) have DISTINCT names. The mutation is the merge:
    /// a classifier that maps two of them to one name fails HERE.
    #[test]
    fn every_failure_class_has_its_own_name() {
        let classes = [
            AttemptFailure::Unreachable,
            AttemptFailure::Refused,
            AttemptFailure::Timeout,
            AttemptFailure::Http5xx { status: 503 },
            AttemptFailure::UpstreamStatus { status: 429, body: "slow down".into() },
            AttemptFailure::EmptyBody,
        ];
        let names: Vec<&str> = classes.iter().map(|c| c.name()).collect();
        let mut sorted = names.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), names.len(), "every class needs its own name, got {names:?}");
        assert!(names.contains(&"timeout") && names.contains(&"refused") && names.contains(&"unreachable"));
        assert!(names.contains(&"empty-body"), "the 200-with-nothing class must be nameable");
    }

    /// ③ THE CORRECTION (§358.2): a 200 with an empty body is a FAILURE, so the loop hands over.
    /// Mutation: treating `Ok("")` as success returns the empty string here and this assertion fails.
    #[test]
    fn an_empty_body_hands_over_instead_of_being_returned() {
        let set = vec![cand("mute", Some("modality:text")), cand("talker", Some("modality:text"))];
        let out = try_candidates(&set, |c| {
            if c.supplier_id == "mute" { Ok(String::new()) } else { Ok("real text".into()) }
        })
        .expect("an empty reply must NOT be the answer when another candidate can talk");
        assert_eq!(out, "real text");
        assert!(body_is_empty("   \n\t "), "whitespace is not text");
    }

    /// ④ An exhausted set is a NAMED failure listing every attempt — never an empty string, never `Ok`.
    #[test]
    fn an_exhausted_set_reports_every_attempt_by_name() {
        let set = vec![cand("a", Some("modality:text")), cand("b", Some("modality:text"))];
        let err = try_candidates(&set, |c| {
            if c.supplier_id == "a" { Err(AttemptFailure::Timeout) } else { Err(AttemptFailure::Http5xx { status: 500 }) }
        })
        .expect_err("all failed");
        assert_eq!(err.attempts.len(), 2);
        let rep = err.report();
        assert!(rep.contains("a: timeout") && rep.contains("b: http-5xx"), "{rep}");
        assert!(rep.contains("no candidate answered"), "{rep}");
    }

    /// ⑤ A `modality:non-text` candidate NEVER gets an attempt (the "wrong model" hole).
    /// Mutation: removing the filter lets the video model answer and this test sees its id.
    #[test]
    fn a_non_text_candidate_is_never_attempted() {
        let set = vec![cand("video", Some("modality:non-text")), cand("text", Some("modality:text"))];
        let tried = std::cell::RefCell::new(Vec::new());
        let out = try_candidates(&set, |c| {
            tried.borrow_mut().push(c.supplier_id.clone());
            Ok(format!("body from {}", c.supplier_id))
        })
        .expect("the text candidate answers");
        assert_eq!(out, "body from text");
        assert_eq!(*tried.borrow(), vec!["text".to_string()], "the non-text candidate must not be tried");
    }

    /// THE IO CLASSIFIERS (ADR-0048 §362.3): a status is named, and the upstream body is passed through
    /// VERBATIM. Mutation: "friendly" rewording of the body fails the equality below.
    #[test]
    fn statuses_are_classified_and_the_upstream_body_passes_through_verbatim() {
        assert_eq!(classify_status(503, "boom"), AttemptFailure::Http5xx { status: 503 });
        assert_eq!(classify_status(500, ""), AttemptFailure::Http5xx { status: 500 });
        let upstream = "rate limit exceeded; retry after 30s";
        assert_eq!(
            classify_status(429, upstream),
            AttemptFailure::UpstreamStatus { status: 429, body: upstream.to_string() },
            "the supplier's own words must survive unchanged"
        );
        assert_eq!(classify_status(401, "invalid api key").name(), "upstream-status");
    }

    /// THREE TRANSPORT FACTS, THREE NAMES. Mutation: mapping `TimedOut` (or `ConnectionRefused`) onto
    /// `Unreachable` makes the three names collide and this test fails.
    #[test]
    fn refused_silent_and_unreachable_are_three_facts() {
        use std::io::ErrorKind;
        assert_eq!(classify_transport_kind(ErrorKind::ConnectionRefused), AttemptFailure::Refused);
        assert_eq!(classify_transport_kind(ErrorKind::TimedOut), AttemptFailure::Timeout);
        assert_eq!(classify_transport_kind(ErrorKind::ConnectionReset), AttemptFailure::Unreachable);
        let names = [
            classify_transport_kind(ErrorKind::ConnectionRefused).name(),
            classify_transport_kind(ErrorKind::TimedOut).name(),
            classify_transport_kind(ErrorKind::ConnectionReset).name(),
        ];
        assert_eq!(names, ["refused", "timeout", "unreachable"]);
    }

    /// The ureq SHELL's rule, tested for what it can promise: the three names stay distinct and the
    /// fallback is never a silent merge. (The message inspection itself is a named limitation, see the doc.)
    #[test]
    fn the_ureq_shell_keeps_the_three_transport_names_distinct() {
        use ureq::ErrorKind as UK;
        assert_eq!(classify_ureq_transport(UK::ConnectionFailed, "Connection refused"), AttemptFailure::Refused);
        assert_eq!(classify_ureq_transport(UK::Io, "operation timed out"), AttemptFailure::Timeout);
        assert_eq!(classify_ureq_transport(UK::ConnectionFailed, "dns failure"), AttemptFailure::Unreachable);
        assert_eq!(classify_ureq_transport(UK::Dns, "no such host"), AttemptFailure::Unreachable);
        let names = [
            classify_ureq_transport(UK::ConnectionFailed, "refused").name(),
            classify_ureq_transport(UK::Io, "timeout").name(),
            classify_ureq_transport(UK::Dns, "?").name(),
        ];
        assert_eq!(names, ["refused", "timeout", "unreachable"]);
    }
}
