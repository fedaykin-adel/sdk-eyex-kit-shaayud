use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize)]
pub struct RuleSet {
    pub version: u32,
    pub default: i64,
    pub rules: Vec<Rule>,
}

#[derive(Debug, Deserialize)]
pub struct Rule {
    pub id: String,
    pub when: Cond,
    pub score: i64,
    pub desc: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "lowercase")]
pub enum Op {
    Eq(String, Value),
    Gt(String, f64),
    Regex(String, String),
    All(Vec<Cond>),
    Any(Vec<Cond>),
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum Cond {
    // atalho: {"eq": ["path","value"]}, {"gt":["path", 10]}, {"regex":["path","re"]}
    Eq { eq: (String, Value) },
    Gt { gt: (String, f64) },
    Regex { regex: (String, String) },
    All { all: Vec<Cond> },
    Any { any: Vec<Cond> },
}

#[derive(Debug, Serialize)]
pub struct ScoreBreakdown {
    pub total: i64,
    pub matched: Vec<(String, i64, Option<String>)>, // (rule_id, score, desc)
}

fn get_path<'a>(bag: &'a Value, path: &str) -> Option<&'a Value> {
    let mut cur = bag;
    for part in path.split('.') {
        cur = cur.get(part)?;
    }
    Some(cur)
}
fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        Value::String(s) => s.parse::<f64>().ok(),
        _ => None,
    }
}
fn matches_cond(bag: &Value, c: &Cond) -> bool {
    match c {
        Cond::Eq { eq: (path, val) } => {
            if let Some(v) = get_path(bag, path) {
                v == val
            } else {
                false
            }
        }
        Cond::Gt { gt: (path, thr) } => {
            if let Some(v) = get_path(bag, path).and_then(as_f64) {
                v > *thr
            } else {
                false
            }
        }
        Cond::Regex { regex: (path, re) } => {
            if let Some(Value::String(s)) = get_path(bag, path) {
                Regex::new(re).map(|r| r.is_match(s)).unwrap_or(false)
            } else {
                false
            }
        }
        Cond::All { all } => all.iter().all(|cc| matches_cond(bag, cc)),
        Cond::Any { any } => any.iter().any(|cc| matches_cond(bag, cc)),
    }
}
pub fn eval_score(bag: &Value, rules: &RuleSet) -> ScoreBreakdown {
    let mut total = rules.default;
    let mut matched = Vec::new();
    for r in &rules.rules {
        if matches_cond(bag, &r.when) {
            total += r.score;
            matched.push((r.id.clone(), r.score, r.desc.clone()));
        }
    }
    ScoreBreakdown { total, matched }
}
