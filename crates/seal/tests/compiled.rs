//! Scripts compiled ahead of time into `.sealc` files, single and bundled, loaded by name, and kept in a cache
//! (D141, D153, D154).

use std::sync::{Arc, Mutex};

use seal::{Cache, Doors, Error, Limits, Script};

const LIMITS: Limits = Limits { steps: 1_000_000, memory: 1 << 20 };

/// Doors that keep what `show_msg` is given, and what it was given.
fn doors() -> (Doors, Arc<Mutex<Vec<String>>>) {
    let shown = Arc::new(Mutex::new(Vec::new()));
    let mut doors = Doors::new();
    let into = Arc::clone(&shown);
    let _ = doors.add("show_msg", move |text: String| {
        into.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(text);
    });
    (doors, shown)
}

fn taken(shown: &Mutex<Vec<String>>) -> Vec<String> {
    std::mem::take(&mut *shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner))
}

const GOBLIN: &str = "record Enemy\n    name: Text\n    hp: Whole\n\n    proc take_damage[mut enemy: Enemy, amount: Whole]\n        enemy.hp = max(_ - amount, 0)\n    end\nend\n\nmut goblin = Enemy[name: \"Goblin\", hp: 30]\ngoblin.take_damage(12)\nshow_msg(\"{goblin.name} {goblin.hp}\")\n";
/// A unit group of the script's own, after Seal's, which a file writes and reads back (D123, D171).
const COINS: &str = "group.unit coins\n    ::bits base\n    ::stacks = 12 bits\nend\n\nheap = 3 stacks\nbits = heap to bits\nshow_msg(\"{bits}\")\n";
const COUNTER: &str =
    "mut total: HugeWhole = 1\nfor n from 1 to 30\n    total = total * n\nend\nshow_msg(\"{total}\")\n";

// trace: D141, D153
#[test]
fn d141_d153_a_bundle_loads_each_script_by_name_running_as_its_text_does() {
    let (doors, shown) = doors();
    let scripts = [("goblin.seal", GOBLIN), ("counter.seal", COUNTER), ("coins.seal", COINS)];
    let Ok(bytes) = seal::build(&scripts, &doors) else {
        panic!("the scripts didn't build");
    };
    let names = scripts.iter().map(|(name, _)| (*name).to_owned()).collect::<Vec<_>>();
    assert_eq!(seal::names(&bytes, &doors).ok(), Some(names));
    for (name, source) in scripts {
        assert!(Script::load(name, source, &doors, LIMITS).is_ok());
        let from_text = taken(&shown);
        assert!(Script::load_compiled(&bytes, name, &doors, LIMITS).is_ok(), "{name} didn't load");
        assert_eq!(taken(&shown), from_text, "{name}");
    }
    // trace: D176. The same file loaded again reuses what was verified, and runs the same.
    for _ in 0..3 {
        assert!(Script::load_compiled(&bytes, "counter.seal", &doors, LIMITS).is_ok());
        assert_eq!(taken(&shown), ["265252859812191058636308480000000"]);
    }
    let missing = Script::load_compiled(&bytes, "orc.seal", &doors, LIMITS);
    assert!(
        matches!(&missing, Err(Error::Refused(messages)) if messages.text().contains("no script named `orc.seal`"))
    );
    // Built for other doors.
    let other = Doors::new();
    let refused = Script::load_compiled(&bytes, "counter.seal", &other, LIMITS);
    assert!(matches!(&refused, Err(Error::Refused(messages)) if messages.text().contains("other than the ones")));
    // Every byte changed in turn is found.
    for at in (0..bytes.len()).step_by(7) {
        let mut damaged = bytes.clone();
        if let Some(byte) = damaged.get_mut(at) {
            *byte ^= 0x5a;
        }
        assert!(
            matches!(Script::load_compiled(&damaged, "goblin.seal", &doors, LIMITS), Err(Error::Refused(_))),
            "byte {at}"
        );
    }
    // A script with a mistake doesn't build, with the messages loading it would give.
    let built = seal::build(&[("bad.seal", "x = 1 +\n")], &doors);
    let loaded = Script::load("bad.seal", "x = 1 +\n", &doors, LIMITS);
    match (built, loaded) {
        (Err(Error::Refused(built)), Err(Error::Refused(loaded))) => assert_eq!(built.text(), loaded.text()),
        _ => panic!("a script with a mistake built"),
    }
}

// trace: D154
#[test]
fn d154_a_cache_keeps_compiled_scripts_and_never_changes_what_they_mean() {
    let folder = std::env::temp_dir().join(format!("seal-cache-test-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&folder);
    let cache = Cache::new(&folder);
    let (doors, shown) = doors();
    assert!(Script::load_cached("goblin.seal", GOBLIN, &doors, LIMITS, &cache).is_ok());
    assert_eq!(taken(&shown), ["Goblin 18"]);
    let files: Vec<_> =
        std::fs::read_dir(&folder).map(|entries| entries.filter_map(Result::ok).collect()).unwrap_or_default();
    assert_eq!(files.len(), 1, "the first load keeps one file");
    assert!(Script::load_cached("goblin.seal", GOBLIN, &doors, LIMITS, &cache).is_ok());
    assert_eq!(taken(&shown), ["Goblin 18"]);
    // A ruined cache file is never used: the text is loaded, and the file written again.
    let path = files.first().map(std::fs::DirEntry::path).unwrap_or_default();
    let _ = std::fs::write(&path, b"not a sealc file");
    assert!(Script::load_cached("goblin.seal", GOBLIN, &doors, LIMITS, &cache).is_ok());
    assert_eq!(taken(&shown), ["Goblin 18"]);
    assert!(std::fs::read(&path).is_ok_and(|bytes| bytes.starts_with(b"SEALC")), "the cache file is written again");
    // Another text is kept under a key of its own.
    assert!(Script::load_cached("counter.seal", COUNTER, &doors, LIMITS, &cache).is_ok());
    assert_eq!(taken(&shown), ["265252859812191058636308480000000"]);
    assert_eq!(std::fs::read_dir(&folder).map_or(0, Iterator::count), 2);
    let _ = std::fs::remove_dir_all(&folder);
}

// trace: D169, D175
#[test]
fn d169_a_host_can_require_files_signed_with_its_key() {
    use seal::host::signing::{keys_from_seed, sign};
    let (public, secret) = keys_from_seed(&[7; 32]);
    let (other_public, _) = keys_from_seed(&[8; 32]);
    let (mut doors, shown) = doors();
    let Ok(built) = seal::build(&[("goblin.seal", GOBLIN), ("counter.seal", COUNTER)], &doors) else {
        panic!("the scripts didn't build");
    };
    let Ok(signed) = sign(&built, &secret, &[1; 32]) else { panic!("the file didn't sign") };
    assert!(sign(&signed, &secret, &[1; 32]).is_err(), "a file is signed once");
    assert!(sign(&built, &public, &[1; 32]).is_err(), "a public key doesn't sign");
    // A host that doesn't require signatures loads a signed file as it is.
    assert!(Script::load_compiled(&signed, "counter.seal", &doors, LIMITS).is_ok());
    assert_eq!(seal::names(&signed, &doors).ok().map(|names| names.len()), Some(2));
    assert!(doors.require_signature(&public[..100]).is_err(), "a key of the wrong size is refused");
    assert!(doors.require_signature(&public).is_ok());
    let _ = taken(&shown);
    for name in ["goblin.seal", "counter.seal", "goblin.seal"] {
        assert!(Script::load_compiled(&signed, name, &doors, LIMITS).is_ok(), "{name} didn't load");
    }
    assert_eq!(taken(&shown).len(), 3);
    let refusal = |bytes: &[u8], doors: &Doors| match Script::load_compiled(bytes, "counter.seal", doors, LIMITS) {
        Err(Error::Refused(messages)) => messages.text(),
        _ => String::new(),
    };
    assert!(refusal(&built, &doors).contains("isn't signed"), "{}", refusal(&built, &doors));
    // Any byte changed, in the file or its signature, is refused.
    for at in [10, built.len() / 2, built.len() + 5, signed.len() - 9] {
        let mut changed = signed.clone();
        if let Some(byte) = changed.get_mut(at) {
            *byte ^= 1;
        }
        let refused = refusal(&changed, &doors);
        assert!(
            refused.contains("doesn't match the host's key") || refused.contains("isn't signed"),
            "{at}: {refused}"
        );
    }
    let mut elsewhere = doors.clone();
    assert!(elsewhere.require_signature(&other_public).is_ok());
    assert!(refusal(&signed, &elsewhere).contains("doesn't match the host's key"));
}
