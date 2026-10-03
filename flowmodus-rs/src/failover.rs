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
}
