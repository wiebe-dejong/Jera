//! De branches van een repo, voor het git-paneel, en wat het paneel ermee doet:
//! commit, push, update, checkout en verwijderen, per branch.
//! Zonder `gh`: de branch van een PR staat in de merge-commit.

use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

/// Branches die blijven bestaan. De eerste die er is, is de hoofdbranch.
const VAST: [&str; 6] = ["main", "master", "staging", "develop", "development", "acceptatie"];

pub struct Commit {
    pub h: String,
    pub ts: i64,
    /// De branch waar een merge vandaan kwam.
    pub van: Option<String>,
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
    /// De worktree van deze branch heeft wijzigingen, of Jera parkeerde ze bij een checkout.
    pub gewijzigd: bool,
    /// Waar de branch uitgecheckt staat: de map van de tab of een andere worktree.
    pub map: Option<PathBuf>,
    /// Hoeveel bestanden in die map nog niet gecommit zijn, nieuwe bestanden meegeteld.
    pub wijzigingen: u32,
    /// Hoeveel bestanden Jera bij een checkout bij deze branch parkeerde.
    pub geparkeerd: u32,
    pub alleen_origin: bool,
    pub origin: Option<Origin>,
    pub basis: Option<String>,
    pub mb: Option<Punt>,
    pub merged_into: Option<Punt>,
    pub eigen: Vec<Commit>,
    pub op_origin: Vec<Commit>,
    pub oud: Vec<Commit>,
    pub uniek: u32,
}

pub struct Repo {
    pub nu: i64,
    pub opgehaald: Option<i64>,
    pub hoofd: String,
    pub branches: Vec<Branch>,
}

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

/// De patch-id van een diff: gelijk als twee diffs dezelfde wijziging zijn.
fn patch_id(map: &Path, args: &[&str]) -> String {
    let diff = Command::new("git").env("GIT_OPTIONAL_LOCKS", "0").arg("-C").arg(map).args(args).output().map(|u| u.stdout).unwrap_or_default();
    let Ok(mut kind) = Command::new("git").arg("-C").arg(map).args(["patch-id", "--stable"]).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn() else {
        return String::new();
    };
    if let Some(mut invoer) = kind.stdin.take() {
        let _ = invoer.write_all(&diff);
    }
    kind.wait_with_output().map(|u| String::from_utf8_lossy(&u.stdout).split_whitespace().next().unwrap_or("").to_string()).unwrap_or_default()
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
    let mut worktrees: HashMap<String, (bool, u32, PathBuf)> = HashMap::new();
    let mut pad = String::new();
    for regel in g(&["worktree", "list", "--porcelain"]).lines() {
        if let Some(p) = regel.strip_prefix("worktree ") {
            pad = p.to_string();
        } else if let Some(b) = regel.strip_prefix("branch refs/heads/") {
            let hier = std::fs::canonicalize(&pad).ok() == hier_pad;
            let n = git(Path::new(&pad), &["status", "--porcelain", "-uall"]).lines().filter(|l| !l.is_empty()).count() as u32;
            worktrees.insert(b.to_string(), (hier, n, PathBuf::from(&pad)));
        }
    }
    // wat Jera bij een checkout bij zijn branch parkeerde
    let mut geparkeerd: HashMap<String, u32> = HashMap::new();
    for (i, regel) in g(&["stash", "list", "--format=%gs"]).lines().enumerate() {
        if let Some(b) = regel.strip_prefix("On ").and_then(|r| r.strip_suffix(PARKEER)) {
            let n = g(&["stash", "show", "--include-untracked", "--name-only", &format!("stash@{{{i}}}")]).lines().count() as u32;
            *geparkeerd.entry(b.to_string()).or_default() += n;
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
                Some(Commit { h: h.to_string(), ts: ts.parse().unwrap_or(0), van })
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
        let (mut mb, mut merged_into, mut eigen, mut uniek) = (None, None, Vec::new(), 0);
        if let Some(b) = &basis {
            let mut m = info(&g(&["merge-base", &best(b), &top]));
            eigen = lijst(n, &m.h, Some(&r), 300);
            for regel in g(&["log", "--first-parent", "--merges", "-n60", "--format=%H|%ct|%P", &best(b)]).lines() {
                let v: Vec<&str> = regel.split('|').collect();
                if v.len() == 3 && v[2].split_whitespace().nth(1) == Some(m.h.as_str()) {
                    merged_into = Some(Punt { h: v[0].to_string(), ts: v[1].parse().unwrap_or(0) });
                    break;
                }
            }
            // gemerged: de merge-base is dan het eindpunt zelf; het echte aftakpunt ligt onder de merge
            if let Some(mi) = &merged_into {
                m = info(&g(&["merge-base", &format!("{}^1", mi.h), &top]));
                eigen = lijst(n, &m.h, Some(&r), 300);
            } else if !eigen.is_empty() {
                // een squash-merge: een PR-commit op de basis met precies de wijziging van deze branch
                let eigen_id = patch_id(map, &["diff", &m.h, &top]);
                for regel in g(&["log", "--first-parent", "--no-merges", "-n60", "--format=%H|%ct|%s", &format!("{}..{}", m.h, best(b))]).lines() {
                    let v: Vec<&str> = regel.splitn(3, '|').collect();
                    if v.len() == 3
                        && v[2].trim_end().ends_with(')')
                        && v[2].contains("(#")
                        && !eigen_id.is_empty()
                        && patch_id(map, &["diff", &format!("{}^", v[0]), v[0]]) == eigen_id
                    {
                        merged_into = Some(Punt { h: v[0].to_string(), ts: v[1].parse().unwrap_or(0) });
                        break;
                    }
                }
            }
            // eigen werk: niet als losse commit ook op de basis gezet
            let bereik = format!("{}...{top}", best(b));
            uniek = g(&["log", "--no-merges", "--cherry-mark", "--left-right", "--format=%m", &bereik]).lines().filter(|m| *m == ">").count() as u32;
            if merged_into.is_some() {
                uniek = eigen.len() as u32;
            }
            mb = Some(m);
        }
        let gezien: HashSet<&str> = eigen.iter().chain(&op_origin).map(|c| c.h.as_str()).collect();
        let oud: Vec<Commit> =
            lijst(n, &top, None, 150 + gezien.len()).into_iter().filter(|c| !gezien.contains(c.h.as_str()) && c.h != eind.h).collect();
        let (hier, wijzigingen, map) = match worktrees.get(n) {
            Some((hier, w, pad)) => (*hier, *w, Some(pad.clone())),
            None => (false, 0, None),
        };
        let geparkeerd = geparkeerd.get(n).copied().unwrap_or(0);
        let gewijzigd = wijzigingen + geparkeerd > 0;
        branches.push(Branch {
            naam: n.clone(),
            h: eind.h,
            ts: eind.ts,
            vast: VAST.contains(&n.as_str()),
            hier,
            gewijzigd,
            map,
            wijzigingen,
            geparkeerd,
            alleen_origin,
            origin,
            basis,
            mb,
            merged_into,
            eigen,
            op_origin,
            oud,
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


/// Het bericht waarmee Jera wijzigingen bij een checkout parkeert: `git stash` maakt er "On branch: jera" van.
const PARKEER: &str = ": jera";

/// Een git-commando dat iets verandert. Bij een fout: de eerste regel van wat git zei.
fn doe(map: &Path, args: &[&str]) -> Result<(), String> {
    let uit = Command::new("git")
        // nooit wachten op een wachtwoord: dan hangt het paneel
        .env("GIT_TERMINAL_PROMPT", "0")
        .arg("-C")
        .arg(map)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if uit.status.success() {
        return Ok(());
    }
    let tekst = [uit.stderr, uit.stdout].map(|b| String::from_utf8_lossy(&b).into_owned()).join("\n");
    Err(tekst.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with("hint:")).unwrap_or("mislukt").to_string())
}

/// Een bestand dat nog niet gecommit is.
pub struct Wijziging {
    /// M gewijzigd, A nieuw, D verwijderd, R hernoemd.
    pub soort: char,
    /// Het pad vanaf de map.
    pub pad: String,
    /// Bij een hernoemd bestand: hoe het heette.
    pub oud: Option<String>,
}

/// Wat in de map nog niet gecommit is, nieuwe bestanden meegeteld.
pub fn wijzigingen(map: &Path) -> Vec<Wijziging> {
    // niet via `git()`: die knipt de spatie weg waarmee de eerste regel kan beginnen
    let uit = Command::new("git").env("GIT_OPTIONAL_LOCKS", "0").arg("-C").arg(map).args(["status", "--porcelain", "-uall", "-z"]).output();
    let uit = uit.map(|u| String::from_utf8_lossy(&u.stdout).into_owned()).unwrap_or_default();
    let mut delen = uit.split('\0');
    let mut lijst = Vec::new();
    while let Some(deel) = delen.next() {
        let (Some(xy), Some(pad)) = (deel.get(..2), deel.get(3..)) else { continue };
        let soort = match xy {
            "??" => 'A',
            _ if xy.contains('D') => 'D',
            _ => xy.chars().find(|c| *c != ' ').unwrap_or('M'),
        };
        // na een hernoemd bestand volgt hoe het heette
        let oud = if xy.contains(['R', 'C']) { delen.next().map(str::to_string) } else { None };
        lijst.push(Wijziging { soort, pad: pad.to_string(), oud });
    }
    lijst
}

/// Commit deze bestanden, ook nieuwe en verwijderde. Wat er niet bij staat, blijft zoals het was.
pub fn commit(map: &Path, bericht: &str, paden: &[String]) -> Result<(), String> {
    // letterlijk: `app/[id]/page.tsx` is een pad, geen patroon
    let paden = paden.iter().map(String::as_str);
    let add: Vec<&str> = ["--literal-pathspecs", "add", "-A", "--"].into_iter().chain(paden.clone()).collect();
    doe(map, &add)?;
    let commit: Vec<&str> = ["--literal-pathspecs", "commit", "-q", "-m", bericht, "--"].into_iter().chain(paden).collect();
    doe(map, &commit)
}

/// Pusht een branch; een branch die nog niet op GitHub staat krijgt daar een upstream.
pub fn push(map: &Path, branch: &str, nieuw: bool) -> Result<(), String> {
    if nieuw { doe(map, &["push", "-q", "-u", "origin", branch]) } else { doe(map, &["push", "-q", "origin", branch]) }
}

/// Haalt binnen wat op GitHub verder is. Staat de branch uitgecheckt, dan met een rebase in die map, die bij een
/// conflict wordt teruggedraaid. Anders alleen als het zonder samenvoegen kan.
pub fn update(root: &Path, branch: &str, map: Option<&Path>) -> Result<(), String> {
    let Some(map) = map else {
        return doe(root, &["fetch", "-q", "origin", &format!("{branch}:{branch}")]);
    };
    doe(map, &["pull", "-q", "--rebase", "--autostash", "origin", branch]).map_err(|fout| {
        let bezig = ["rebase-merge", "rebase-apply"].iter().any(|d| Path::new(&git(map, &["rev-parse", "--path-format=absolute", "--git-path", d])).exists());
        if bezig {
            let _ = doe(map, &["rebase", "--abort"]);
            crate::taal::t("conflict · vraag Claude om samen te voegen", "conflict · ask Claude to merge").to_string()
        } else {
            fout
        }
    })
}

/// Zet de map op een andere branch. Wat in de map niet gecommit is, blijft geparkeerd bij zijn branch;
/// wat bij de nieuwe branch geparkeerd stond, komt terug.
pub fn checkout(map: &Path, branch: &str) -> Result<(), String> {
    let gewijzigd = !git(map, &["status", "--porcelain", "-uall"]).is_empty();
    if gewijzigd {
        doe(map, &["stash", "push", "-q", "-u", "-m", "jera"])?;
    }
    if let Err(fout) = doe(map, &["switch", "-q", branch]) {
        if gewijzigd {
            let _ = doe(map, &["stash", "pop", "-q"]);
        }
        return Err(fout);
    }
    let terug = format!("On {branch}{PARKEER}");
    let stashes: Vec<String> = git(map, &["stash", "list", "--format=%gs"]).lines().map(str::to_string).collect();
    // van hoog naar laag, zodat de nummers van de rest niet verschuiven
    for (i, _) in stashes.iter().enumerate().filter(|(_, s)| **s == terug).collect::<Vec<_>>().into_iter().rev() {
        doe(map, &["stash", "pop", "-q", &format!("stash@{{{i}}}")])?;
    }
    Ok(())
}

/// Verwijdert een gemergde branch, hier en op GitHub.
pub fn verwijder(map: &Path, branch: &str, lokaal: bool, op_github: bool) -> Result<(), String> {
    if lokaal {
        // -D: een squash-merge ziet git niet als gemerged
        doe(map, &["branch", "-q", "-D", branch])?;
    }
    if op_github {
        doe(map, &["push", "-q", "origin", "--delete", branch])?;
    }
    Ok(())
}

/// Een commitbericht van één regel voor de wijzigingen in deze bestanden, geschreven door Claude (`claude -p`).
/// Claude draait buiten de repo, zodat er geen sessie bij dit project bijkomt.
pub fn bericht(map: &Path, paden: &[String]) -> Option<String> {
    let over = |args: &[&'static str]| {
        let args: Vec<&str> = ["--literal-pathspecs"].into_iter().chain(args.iter().copied()).chain(["--"]).chain(paden.iter().map(String::as_str)).collect();
        git(map, &args)
    };
    let mut diff = over(&["diff", "HEAD"]);
    if diff.is_empty() {
        diff = over(&["diff"]);
    }
    if diff.len() > 40_000 {
        let mut eind = 40_000;
        while !diff.is_char_boundary(eind) {
            eind -= 1;
        }
        diff.truncate(eind);
    }
    let prompt = format!(
        "Schrijf het commitbericht voor deze wijzigingen: één regel, in dezelfde taal en stijl als de eerdere berichten. \
         Geef alleen het bericht, zonder aanhalingstekens.\n\nEerdere berichten:\n{}\n\nGewijzigd:\n{}\n\nDiff:\n{diff}",
        git(map, &["log", "-8", "--format=%s"]),
        over(&["status", "--short", "-uall"]),
    );
    let eigen = std::env::home_dir().map(|h| h.join(".local/bin/claude")).filter(|p| p.exists());
    let mut cmd = match eigen {
        Some(pad) => Command::new(pad),
        None => {
            let mut c = Command::new("/bin/zsh");
            c.args(["-ilc", "claude \"$@\"", "claude"]);
            c
        }
    };
    let mut kind = cmd
        // kaal: zonder tools, MCP, skills en instellingen, en een korte systeemprompt; scheelt de helft
        .args(["-p", "--model", "haiku", "--tools", "", "--strict-mcp-config", "--disable-slash-commands", "--no-session-persistence"])
        .args(["--setting-sources", "", "--system-prompt", "Je schrijft commitberichten."])
        .current_dir(std::env::temp_dir())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    kind.stdin.take()?.write_all(prompt.as_bytes()).ok()?;
    let uit = kind.wait_with_output().ok()?;
    if !uit.status.success() {
        return None;
    }
    let regel = String::from_utf8_lossy(&uit.stdout).lines().map(str::trim).find(|l| !l.is_empty())?.trim_matches(['"', '`', '\'']).to_string();
    (!regel.is_empty()).then_some(regel)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Een lege repo in een eigen tijdelijke map, met één commit op main.
    fn repo(naam: &str) -> PathBuf {
        let map = std::env::temp_dir().join(format!("jera-git-{naam}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&map);
        std::fs::create_dir_all(&map).unwrap();
        for args in [&["init", "-q", "-b", "main"][..], &["config", "user.email", "t@t"], &["config", "user.name", "t"]] {
            doe(&map, args).unwrap();
        }
        std::fs::write(map.join("a.txt"), "a\n").unwrap();
        commit(&map, "eerste", &["a.txt".into()]).unwrap();
        map
    }

    fn branch<'a>(r: &'a Repo, naam: &str) -> &'a Branch {
        r.branches.iter().find(|b| b.naam == naam).unwrap()
    }

    #[test]
    fn checkout_parkeert_wijzigingen_bij_hun_branch() {
        let map = repo("parkeer");
        doe(&map, &["branch", "feat/x"]).unwrap();
        std::fs::write(map.join("a.txt"), "a\nwip\n").unwrap();
        std::fs::write(map.join("nieuw.txt"), "n\n").unwrap();
        assert_eq!(branch(&lees(&map).unwrap(), "main").wijzigingen, 2);

        checkout(&map, "feat/x").unwrap();
        assert_eq!(std::fs::read_to_string(map.join("a.txt")).unwrap(), "a\n");
        assert!(!map.join("nieuw.txt").exists());
        let r = lees(&map).unwrap();
        assert_eq!((branch(&r, "main").geparkeerd, branch(&r, "main").wijzigingen), (2, 0));
        assert!(branch(&r, "feat/x").hier && !branch(&r, "feat/x").gewijzigd);

        checkout(&map, "main").unwrap();
        assert_eq!(std::fs::read_to_string(map.join("a.txt")).unwrap(), "a\nwip\n");
        assert!(map.join("nieuw.txt").exists());
        let r = lees(&map).unwrap();
        assert_eq!((branch(&r, "main").geparkeerd, branch(&r, "main").wijzigingen), (0, 2));
        let _ = std::fs::remove_dir_all(&map);
    }

    #[test]
    fn commit_neemt_alleen_de_gekozen_bestanden_mee() {
        let map = repo("gekozen");
        std::fs::write(map.join("b.txt"), "b\n").unwrap();
        commit(&map, "b", &["b.txt".into()]).unwrap();
        std::fs::write(map.join("a.txt"), "a\nwip\n").unwrap();
        std::fs::remove_file(map.join("b.txt")).unwrap();
        std::fs::create_dir_all(map.join("app/[id]")).unwrap();
        std::fs::write(map.join("app/[id]/nieuw.txt"), "n\n").unwrap();
        std::fs::write(map.join("blijft.txt"), "x\n").unwrap();
        let staat = |map: &Path| wijzigingen(map).into_iter().map(|w| format!("{} {}", w.soort, w.pad)).collect::<Vec<_>>();
        assert_eq!(staat(&map), ["M a.txt", "D b.txt", "A app/[id]/nieuw.txt", "A blijft.txt"]);

        commit(&map, "deel", &["a.txt".into(), "b.txt".into(), "app/[id]/nieuw.txt".into()]).unwrap();
        assert_eq!(staat(&map), ["A blijft.txt"]);
        assert_eq!(git(&map, &["show", "--name-status", "--format=", "HEAD"]), "M\ta.txt\nA\tapp/[id]/nieuw.txt\nD\tb.txt");
        let _ = std::fs::remove_dir_all(&map);
    }

    #[test]
    fn verwijder_haalt_een_gemergde_branch_weg() {
        let map = repo("verwijder");
        doe(&map, &["switch", "-q", "-c", "feat/y"]).unwrap();
        std::fs::write(map.join("y.txt"), "y\n").unwrap();
        commit(&map, "y", &["y.txt".into()]).unwrap();
        doe(&map, &["switch", "-q", "main"]).unwrap();
        doe(&map, &["merge", "-q", "--no-ff", "-m", "Merge branch 'feat/y'", "feat/y"]).unwrap();
        assert!(branch(&lees(&map).unwrap(), "feat/y").merged_into.is_some());
        verwijder(&map, "feat/y", true, false).unwrap();
        assert!(!lees(&map).unwrap().branches.iter().any(|b| b.naam == "feat/y"));
        let _ = std::fs::remove_dir_all(&map);
    }
}
