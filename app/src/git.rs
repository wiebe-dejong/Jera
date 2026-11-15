//! De branches van een repo, voor het git-paneel.
//! Zonder `gh`: de branch van een PR staat in de merge-commit.

use std::{
    collections::{HashMap, HashSet},
    path::Path,
    process::Command,
    sync::{LazyLock, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

/// Branches die blijven bestaan. De eerste die er is, is de hoofdbranch.
const VAST: [&str; 6] = ["main", "master", "staging", "develop", "development", "acceptatie"];

pub struct Commit {
    pub h: String,
    pub ts: i64,
    /// De branch waar een merge vandaan kwam.
    pub van: Option<String>,
    /// Bij een merge: hoeveel commits hij meebracht.
    pub n: u32,
}

pub struct Punt {
    pub h: String,
    pub ts: i64,
}

pub struct Origin {
    pub h: String,
    pub ts: i64,
    pub ahead: u32,
    pub behind: u32,
}

pub struct Branch {
    pub naam: String,
    pub h: String,
    pub ts: i64,
    pub vast: bool,
    /// De worktree van deze branch is de map van de tab.
    pub hier: bool,
    /// De worktree van deze branch heeft wijzigingen.
    pub gewijzigd: bool,
    pub alleen_origin: bool,
    pub origin: Option<Origin>,
    pub basis: Option<String>,
    pub mb: Option<Punt>,
    pub merged_into: Option<Punt>,
    pub eigen: Vec<Commit>,
    pub op_origin: Vec<Commit>,
    pub oud: Vec<Commit>,
    /// Commits die ook als losse commit op de basis staan.
    pub dubbel: HashSet<String>,
    pub uniek: u32,
}

pub struct Repo {
    pub nu: i64,
    pub opgehaald: Option<i64>,
    pub hoofd: String,
    pub branches: Vec<Branch>,
}

/// Hoeveel commits een merge meebracht, per merge-commit. Een commit verandert nooit, dus dit
/// hoeft maar één keer geteld te worden.
static MEEGEBRACHT: LazyLock<Mutex<HashMap<String, u32>>> = LazyLock::new(Mutex::default);

fn git(map: &Path, args: &[&str]) -> String {
    Command::new("git")
        // geen slot op de index: anders kan een commit in een sessie mislukken met "index.lock: File exists"
        .env("GIT_OPTIONAL_LOCKS", "0")
        .arg("-C")
        .arg(map)
        .args(args)
        .output()
        .map(|uit| String::from_utf8_lossy(&uit.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Of de repo een remote heeft. `None` als de map geen git-repo is.
pub fn heeft_remote(map: &Path) -> Option<bool> {
    (git(map, &["rev-parse", "--is-inside-work-tree"]) == "true").then(|| !git(map, &["remote"]).is_empty())
}

/// Haalt op bij origin, zonder ooit te wachten op een wachtwoord: dan hangt het paneel.
pub fn haal_op(map: &Path) {
    let _ = Command::new("git").env("GIT_TERMINAL_PROMPT", "0").arg("-C").arg(map).args(["fetch", "-q", "origin"]).output();
}

/// Leest de branches. `None` als de map geen git-repo is.
pub fn lees(map: &Path) -> Option<Repo> {
    let g = |args: &[&str]| git(map, args);
    if g(&["rev-parse", "--is-inside-work-tree"]) != "true" {
        return None;
    }
    let count = |van: &str, tot: &str, extra: &[&str]| -> u32 {
        let bereik = format!("{van}..{tot}");
        let mut args = vec!["rev-list", "--count"];
        args.extend_from_slice(extra);
        args.push(&bereik);
        g(&args).parse().unwrap_or(0)
    };
    let info = |r: &str| -> Punt {
        let uit = g(&["log", "-1", "--format=%H|%ct", r]);
        let (h, ts) = uit.split_once('|').unwrap_or_default();
        Punt { h: h.to_string(), ts: ts.parse().unwrap_or(0) }
    };

    // de worktrees: welke branch waar staat, en of er wijzigingen zijn
    let hier_pad = std::fs::canonicalize(map).ok();
    let mut worktrees: HashMap<String, (bool, bool)> = HashMap::new();
    let mut pad = String::new();
    for regel in g(&["worktree", "list", "--porcelain"]).lines() {
        if let Some(p) = regel.strip_prefix("worktree ") {
            pad = p.to_string();
        } else if let Some(b) = regel.strip_prefix("branch refs/heads/") {
            let hier = std::fs::canonicalize(&pad).ok() == hier_pad;
            let gewijzigd = git(Path::new(&pad), &["status", "--porcelain"]).lines().any(|l| !l.starts_with("??") && !l.is_empty());
            worktrees.insert(b.to_string(), (hier, gewijzigd));
        }
    }

    let lokaal: Vec<String> = g(&["for-each-ref", "--format=%(refname:short)", "refs/heads"]).lines().map(str::to_string).collect();
    let remote: Vec<String> = g(&["for-each-ref", "--format=%(refname:short)", "refs/remotes/origin"])
        .lines()
        .filter_map(|r| r.strip_prefix("origin/"))
        .filter(|r| *r != "HEAD" && !r.is_empty())
        .map(str::to_string)
        .collect();
    let hoofd = VAST.iter().find(|n| lokaal.iter().any(|l| l == *n)).map(|n| n.to_string()).or_else(|| lokaal.first().cloned())?;
    let best = |n: &str| if remote.iter().any(|r| r == n) { format!("origin/{n}") } else { n.to_string() };
    let vast_index = |n: &str| VAST.iter().position(|v| *v == n).unwrap_or(99);

    let mut namen = lokaal.clone();
    namen.sort_by_key(|n| (*n != hoofd, vast_index(n), n.clone()));
    namen.extend(remote.iter().filter(|n| VAST.contains(&n.as_str()) && !lokaal.contains(n)).cloned());
    let hoofdlijn: HashMap<&str, HashSet<String>> = namen
        .iter()
        .map(|n| (n.as_str(), g(&["rev-list", "--first-parent", "-n4000", &best(n)]).lines().map(str::to_string).collect()))
        .collect();

    // commits langs de hoofdlijn: in b maar niet in a, of (zonder b) de laatste n vanaf a
    let lijst = |zelf: &str, a: &str, b: Option<&str>, n: usize| -> Vec<Commit> {
        let bereik = match b {
            Some(b) => format!("{a}..{b}"),
            None => a.to_string(),
        };
        let aantal = format!("-n{n}");
        let uit = g(&["log", "--first-parent", &aantal, "--format=%H%x00%ct%x00%P%x00%s%x1e", &bereik]);
        uit.split('\x1e')
            .filter_map(|rec| {
                let mut v = rec.trim_start_matches('\n').splitn(4, '\0');
                let (h, ts, p, s) = (v.next()?, v.next()?, v.next()?, v.next()?);
                if h.is_empty() {
                    return None;
                }
                let ouders: Vec<&str> = p.split_whitespace().collect();
                // "Merge pull request #12 from eigenaar/branch"
                let pr_merge = s.strip_prefix("Merge pull request #").and_then(|rest| {
                    let (nr, rest) = rest.split_once(" from ")?;
                    Some((nr.parse::<u32>().ok()?, rest.split_once('/').map_or(rest, |(_, b)| b).to_string()))
                });
                let mut van = pr_merge.map(|(_, b)| b);
                if ouders.len() > 1 {
                    if let Some(c) = namen.iter().find(|c| *c != zelf && hoofdlijn[c.as_str()].contains(ouders[1])) {
                        van = Some(c.clone());
                    }
                }
                let n = if ouders.len() > 1 {
                    let bekend = MEEGEBRACHT.lock().unwrap().get(h).copied();
                    bekend.unwrap_or_else(|| {
                        let n = count(ouders[0], h, &[]).saturating_sub(1);
                        MEEGEBRACHT.lock().unwrap().insert(h.to_string(), n);
                        n
                    })
                } else {
                    0
                };
                Some(Commit { h: h.to_string(), ts: ts.parse().unwrap_or(0), van, n })
            })
            .collect()
    };

    // de basis: de branch waar n het dichtst tegenaan loopt
    let kies_basis = |n: &str| -> Option<String> {
        if n == hoofd {
            return None;
        }
        let mut beste: Option<(i64, &str)> = None;
        for c in &namen {
            let c_vast = VAST.contains(&c.as_str());
            if c == n || (VAST.contains(&n) && !c_vast) || (!c_vast && !remote.contains(c)) {
                continue;
            }
            let mb = g(&["merge-base", &best(c), &best(n)]);
            if !mb.is_empty() && hoofdlijn[c.as_str()].contains(&mb) {
                let ts = info(&mb).ts;
                if beste.is_none_or(|(t, _)| ts > t) {
                    beste = Some((ts, c));
                }
            }
        }
        Some(beste.map_or(hoofd.clone(), |(_, c)| c.to_string()))
    };
    let mut basis_van: HashMap<String, Option<String>> = namen.iter().map(|n| (n.clone(), kies_basis(n))).collect();
    // geen kringetjes: wijzen twee branches naar elkaar, dan valt de latere terug op de hoofdbranch
    for n in &namen {
        let mut gezien = vec![n.clone()];
        let mut c = basis_van[n].clone();
        while let Some(x) = c.clone().filter(|x| !gezien.contains(x)) {
            gezien.push(x.clone());
            c = basis_van[&x].clone();
        }
        if c.as_deref() == Some(n) {
            basis_van.insert(n.clone(), Some(hoofd.clone()));
        }
    }

    let mut branches = Vec::new();
    for n in &namen {
        let alleen_origin = !lokaal.contains(n);
        let r = if alleen_origin { format!("origin/{n}") } else { n.clone() };
        let eind = info(&r);
        let eigen_origin = remote.contains(n).then(|| format!("origin/{n}"));
        let origin = eigen_origin.as_ref().map(|o| {
            let p = info(o);
            Origin { h: p.h, ts: p.ts, ahead: count(o, &r, &[]), behind: count(&r, o, &[]) }
        });
        let achter = origin.as_ref().is_some_and(|o| o.behind > 0);
        let top = if achter { eigen_origin.clone().unwrap() } else { r.clone() };
        let mut basis = basis_van[n].clone();
        if basis.as_ref().is_some_and(|b| g(&["merge-base", &best(b), &top]).is_empty()) {
            basis = None;
        }
        let op_origin = if achter { lijst(n, &r, eigen_origin.as_deref(), 40) } else { Vec::new() };
        let (mut mb, mut merged_into, mut eigen, mut dubbel, mut uniek) = (None, None, Vec::new(), HashSet::new(), 0);
        if let Some(b) = &basis {
            let m = info(&g(&["merge-base", &best(b), &top]));
            eigen = lijst(n, &m.h, Some(&r), 300);
            for regel in g(&["log", "--first-parent", "--merges", "-n60", "--format=%H|%ct|%P", &best(b)]).lines() {
                let v: Vec<&str> = regel.split('|').collect();
                if v.len() == 3 && v[2].split_whitespace().nth(1) == Some(m.h.as_str()) {
                    merged_into = Some(Punt { h: v[0].to_string(), ts: v[1].parse().unwrap_or(0) });
                    break;
                }
            }
            // dezelfde wijziging aan beide kanten, als losse commit op elke branch gezet
            let bereik = format!("{}...{top}", best(b));
            let hier: HashSet<String> = g(&["rev-list", &format!("{}..{top}", best(b))]).lines().map(str::to_string).collect();
            for regel in g(&["log", "--no-merges", "--cherry-mark", "--left-right", "--format=%m|%H", &bereik]).lines() {
                match regel.split_once('|') {
                    Some(("=", h)) if hier.contains(h) => {
                        dubbel.insert(h.to_string());
                    }
                    Some((">", _)) => uniek += 1,
                    _ => {}
                }
            }
            mb = Some(m);
        }
        let gezien: HashSet<&str> = eigen.iter().chain(&op_origin).map(|c| c.h.as_str()).collect();
        let oud: Vec<Commit> =
            lijst(n, &top, None, 150 + gezien.len()).into_iter().filter(|c| !gezien.contains(c.h.as_str()) && c.h != eind.h).collect();
        let (hier, gewijzigd) = worktrees.get(n).copied().unwrap_or((false, false));
        branches.push(Branch {
            naam: n.clone(),
            h: eind.h,
            ts: eind.ts,
            vast: VAST.contains(&n.as_str()),
            hier,
            gewijzigd,
            alleen_origin,
            origin,
            basis,
            mb,
            merged_into,
            eigen,
            op_origin,
            oud,
            dubbel,
            uniek,
        });
    }

    let common = g(&["rev-parse", "--path-format=absolute", "--git-common-dir"]);
    let opgehaald = std::fs::metadata(Path::new(&common).join("FETCH_HEAD"))
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
        .map(|d| d.as_secs() as i64);
    let nu = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    Some(Repo { nu, opgehaald, hoofd, branches })
}

