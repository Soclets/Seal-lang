//! An example host: a small game that embeds Seal the way D48 sketches it.
//!
//! The game keeps its own world, opens doors onto it as plain Rust closures, loads `goblin.seal` with those doors
//! and its limits, and calls the script's procedures as its own events happen.

use std::sync::{Arc, Mutex};

use seal::{Called, Doors, Error, Limits, MessageKind, Pause, Script, Seal, labelled};

/// A record that crosses the doors as a Rust struct (D27).
#[derive(Seal, Clone, Debug, PartialEq)]
struct Enemy {
    name: String,
    hp: i64,
}

/// A `one_of` that crosses the doors as a Rust enum (D35).
#[derive(Seal, Clone, Debug, PartialEq)]
enum Event {
    Idle,
    Hurt { amount: i64 },
    Healed { amount: i64, by: String },
}

/// A door's failures, as a Rust enum (D44).
#[derive(Seal, Clone, Copy, Debug)]
enum SaveFailure {
    DiskFull,
    NoSlot,
}

/// The game's own data. A script never touches it except through the doors (D11, D15).
#[derive(Default)]
struct World {
    shown: Vec<String>,
    enemies: Vec<Enemy>,
    waits: Vec<i64>,
}

const SOURCE: &str = include_str!("goblin.seal");
const LIMITS: Limits = Limits { steps: 100_000, memory: 1024 * 1024 };

type Shared = Arc<Mutex<World>>;

/// The world, even after a door's closure panicked while holding it.
fn lock(shared: &Shared) -> std::sync::MutexGuard<'_, World> {
    shared.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn open_doors() -> Result<(Doors, Shared), seal::Message> {
    let world: Shared = Arc::new(Mutex::new(World::default()));
    let mut doors = Doors::new();

    let shared = Arc::clone(&world);
    doors.add("show_msg", move |text: String| lock(&shared).shown.push(text))?;

    let shared = Arc::clone(&world);
    doors
        .add("damage", move |id: i64, amount: i64| {
            let mut world = lock(&shared);
            if let Some(enemy) = usize::try_from(id).ok().and_then(|id| world.enemies.get_mut(id)) {
                enemy.hp -= amount;
            }
        })?
        .labels_after_first(["by"])?;

    let shared = Arc::clone(&world);
    doors.add("nearest_enemy", move || -> Option<i64> {
        let count = lock(&shared).enemies.len();
        i64::try_from(count).ok().filter(|count| *count > 0).map(|count| count - 1)
    })?;

    let shared = Arc::clone(&world);
    doors.add("enemy_info", move |id: i64| -> Enemy {
        let world = lock(&shared);
        usize::try_from(id)
            .ok()
            .and_then(|id| world.enemies.get(id).cloned())
            .unwrap_or(Enemy { name: "nobody".to_owned(), hp: 0 })
    })?;

    let shared = Arc::clone(&world);
    doors.add("spawn", move |enemy: Enemy| -> i64 {
        let mut world = lock(&shared);
        world.enemies.push(enemy);
        i64::try_from(world.enemies.len()).unwrap_or(0) - 1
    })?;

    // A door that pauses the script (D49): the host notes how long, and resumes the task when it likes.
    let shared = Arc::clone(&world);
    doors
        .add("wait", move |seconds: i64| {
            lock(&shared).waits.push(seconds);
            Pause::<()>::new()
        })?
        .labels(["seconds"])?;

    doors.add("save", |slot: i64| -> Result<(), SaveFailure> {
        match slot {
            1 => Ok(()),
            2 => Err(SaveFailure::DiskFull),
            _ => Err(SaveFailure::NoSlot),
        }
    })?;
    // A type only the host's calls into the script use is opened by itself (D85).
    doors.add_type::<Event>()?;
    Ok((doors, world))
}

fn load() -> Result<(Script, Shared), Box<dyn std::error::Error>> {
    let (doors, world) = open_doors()?;
    let script = Script::load("goblin.seal", SOURCE, &doors, LIMITS)?;
    Ok((script, world))
}

fn shown(world: &Shared) -> Vec<String> {
    lock(world).shown.clone()
}

// trace: D4, D48
#[test]
fn d48_loading_runs_the_top_level_and_the_host_calls_procedures() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, world) = load()?;
    assert_eq!(shown(&world), ["loaded"]);
    assert_eq!(script.call::<()>("on_hit", (10,))?, Called::Finished(()));
    assert_eq!(script.call::<String>("hp_left", (" HP".to_owned(),))?, Called::Finished("20 HP".to_owned()));
    assert_eq!(shown(&world), ["loaded", "hit for 10, 20 left"]);
    Ok(())
}

// trace: D5, D6, D48
// trace: D60
#[test]
fn d60_a_procedure_that_takes_nothing_is_called_with_nothing() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, _world) = load()?;
    script.call::<()>("on_tick", ())?;
    assert_eq!(script.call::<String>("hp_left", (" hp".to_owned(),))?, Called::Finished("31 hp".to_owned()));
    assert_eq!(script.call::<i64>("full_hp", ())?, Called::Finished(30));
    // Arguments it doesn't take are refused before anything runs.
    assert!(matches!(script.call::<()>("on_tick", (1i64,)), Err(Error::Refused(_))));
    Ok(())
}

#[test]
fn d48_a_call_is_checked_by_name_and_types() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, _world) = load()?;
    assert!(matches!(script.call::<()>("on_miss", (10,)), Err(Error::Refused(_))));
    assert!(matches!(script.call::<()>("on_hit", ("ten".to_owned(),)), Err(Error::Refused(_))));
    assert!(matches!(script.call::<()>("on_hit", (1, 2)), Err(Error::Refused(_))));
    assert!(matches!(script.call::<i64>("on_hit", (10,)), Err(Error::Refused(_))));
    // A refused call leaves the script as it was.
    assert_eq!(script.call::<String>("hp_left", (String::new(),))?, Called::Finished("30".to_owned()));
    Ok(())
}

// trace: D41, D42
#[test]
fn d42_the_host_writes_labels_and_can_leave_defaults_out() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, _world) = load()?;
    script.call::<()>("heal", (labelled("by", 5),))?;
    script.call::<()>("heal", (labelled("by", 2), labelled("times", 3)))?;
    assert_eq!(script.call::<String>("hp_left", (String::new(),))?, Called::Finished("41".to_owned()));
    assert!(matches!(script.call::<()>("heal", (5,)), Err(Error::Refused(_))));
    Ok(())
}

// trace: D27, D35, D38, D48
#[test]
fn d48_records_one_ofs_and_maybes_cross_the_doors() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, world) = load()?;
    assert_eq!(script.call::<i64>("handle", (Event::Hurt { amount: 5 },))?, Called::Finished(25));
    assert_eq!(
        script.call::<i64>("handle", (Event::Healed { amount: 3, by: "Mia".to_owned() },))?,
        Called::Finished(28)
    );
    assert_eq!(script.call::<i64>("handle", (Event::Idle,))?, Called::Finished(28));
    assert_eq!(script.call::<i64>("target", (-1,))?, Called::Finished(-1));
    assert_eq!(script.call::<i64>("add_goblin", (12,))?, Called::Finished(0));
    assert_eq!(lock(&world).enemies, [Enemy { name: "Goblin".to_owned(), hp: 12 }]);
    assert_eq!(script.call::<String>("strongest", (0,))?, Called::Finished("Goblin".to_owned()));
    assert_eq!(script.call::<i64>("target", (-1,))?, Called::Finished(0));
    Ok(())
}

// trace: D44
#[test]
fn d44_a_door_can_fail_and_a_procedure_can_fail_to_its_host() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, world) = load()?;
    script.call::<()>("save_game", (1,))?;
    script.call::<()>("save_game", (2,))?;
    assert_eq!(shown(&world).last().map(String::as_str), Some("the disk is full"));
    assert_eq!(script.call::<()>("save_game", (3,)), Err(Error::Failed("no_slot".to_owned())));
    // A failure isn't a stop: the script carries on.
    script.call::<()>("on_hit", (1,))?;
    Ok(())
}

// trace: D49, D51, D52, D53, D54
#[test]
fn d49_a_door_pauses_a_task_and_the_host_resumes_or_cancels_it() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, world) = load()?;
    script.call::<i64>("add_goblin", (10,))?;
    assert_eq!(script.can_pause("patrol"), Some(true));
    assert_eq!(script.can_pause("on_hit"), Some(false));
    let before = script.memory_in_use();

    let Called::Paused(patrol) = script.call::<()>("patrol", (0,))? else {
        panic!("the patrol should pause at its wait")
    };
    assert_eq!(lock(&world).waits, [2]);
    // While the patrol waits, the host calls the script again, as another task (D51).
    script.call::<()>("on_hit", (5,))?;
    let Called::Paused(patrol) = script.resume::<()>(patrol, ())? else { panic!("the patrol should pause again") };
    assert_eq!(shown(&world).last().map(String::as_str), Some("pass 1 of 2"));
    assert_eq!(lock(&world).enemies.first().map(|enemy| enemy.hp), Some(8));
    // Each resume starts a fresh step budget, so a loop that pauses every pass can run for ever (D53).
    let mut task = patrol;
    for _ in 0..50_000 {
        let Called::Paused(next) = script.resume::<()>(task, ())? else { panic!("the patrol never ends by itself") };
        task = next;
    }
    // The guard dies: its patrol is cancelled, ending at its pause, and what it held is freed (D54).
    script.cancel(task)?;
    assert_eq!(script.memory_in_use(), before);
    assert!(matches!(script.resume::<()>(task, ()), Err(Error::Refused(_))));
    script.call::<()>("on_hit", (1,))?;
    Ok(())
}

// trace: D14, D53, D54
#[test]
fn d14_a_script_past_its_step_budget_stops_and_the_host_carries_on() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, _world) = load()?;
    let Err(Error::Stopped(message)) = script.call::<()>("spin", (0,)) else {
        panic!("the loop should run out of steps")
    };
    assert_eq!(message.kind, MessageKind::LimitReached);
    assert_eq!(message.what.as_deref(), Some("the script ran out of steps"));
    assert_eq!(message.place.line, 44);
    // A stopped script is over: every later call says why (D54).
    assert!(matches!(script.call::<()>("on_hit", (1,)), Err(Error::Stopped(_))));
    assert!(script.stopped().is_some());
    Ok(())
}

// trace: D14, D53
#[test]
fn d14_a_script_past_its_memory_cap_stops() -> Result<(), Box<dyn std::error::Error>> {
    let (doors, _world) = open_doors()?;
    let mut script = Script::load("goblin.seal", SOURCE, &doors, Limits { steps: u64::MAX, memory: 200_000 })?;
    let Err(Error::Stopped(message)) = script.call::<()>("hoard", ("gold".to_owned(),)) else {
        panic!("the pile should pass the cap")
    };
    assert_eq!(message.kind, MessageKind::LimitReached);
    assert_eq!(message.what.as_deref(), Some("the script ran out of memory"));
    assert_eq!(script.memory_in_use(), 0);
    Ok(())
}

// trace: D17, D26
#[test]
fn d17_a_failed_check_stops_the_script_at_its_exact_line() -> Result<(), Box<dyn std::error::Error>> {
    let (mut script, _world) = load()?;
    let Err(Error::Stopped(message)) = script.call::<i64>("overflow", (2,)) else { panic!("the sum should overflow") };
    assert_eq!(
        message.text(),
        "check failed: goblin.seal:57:12, line 57 column 12\n 57 |     return n * 9223372036854775807\n    |            ^^^^^^^^^^^^^^^^^^^^^^^\n \
         what:    the result is too big for a `Whole`\n when:    `2 * 9223372036854775807`\n \
         why:     a `Whole` holds whole numbers from -9223372036854775808 to 9223372036854775807.\n"
    );
    Ok(())
}

// trace: D17, D26
#[test]
fn d17_a_refused_script_does_not_load_and_the_host_gets_the_messages() -> Result<(), Box<dyn std::error::Error>> {
    let (doors, _world) = open_doors()?;
    let source =
        "mut tries = 0\nmut won = false\nwhile tries < 7 and not won\n    tires = tries + 1\n    won = true\nend\n";
    let Err(Error::Refused(messages)) = Script::load("goblin.seal", source, &doors, LIMITS) else {
        panic!("the typo should be refused")
    };
    assert_eq!(
        messages.text(),
        "refused: goblin.seal:4:5, line 4 column 5\n 4 |     tires = tries + 1\n   |     ^^^^^\n what:    `tires` is never used\n"
    );
    Ok(())
}

// trace: D13, D24
#[test]
fn d24_a_doors_name_follows_seals_spelling() {
    let mut doors = Doors::new();
    assert!(doors.add("GetService", |name: String| name).is_err());
    assert!(doors.add("get_service", |name: String| name).is_ok());
    assert!(doors.add("get_service.cached", |name: String| name).is_ok());
    assert!(doors.add("get_service", |name: String| name).is_err());
    let unknown = Script::load("a.seal", "x = get_service.fresh(\"a\")\nshow_msg(x)\n", &doors, LIMITS);
    let Err(Error::Refused(messages)) = unknown else { panic!("an unknown narrowing is refused") };
    // trace: D13: the door's real narrowings are listed.
    assert!(messages.text().contains("`get_service`'s narrowings are `get_service.cached`."), "{messages}");
}

// trace: D49
#[test]
fn d49_scripts_are_separate_and_move_between_threads() -> Result<(), Box<dyn std::error::Error>> {
    fn assert_send<T: Send>() {}
    assert_send::<Script>();
    let (doors, _world) = open_doors()?;
    let mut first = Script::load("goblin.seal", SOURCE, &doors, LIMITS)?;
    let mut second = Script::load("goblin.seal", SOURCE, &doors, LIMITS)?;
    first.call::<()>("on_hit", (10,))?;
    let moved = std::thread::spawn(move || second.call::<String>("hp_left", (String::new(),)))
        .join()
        .map_err(|_| "the thread failed")?;
    assert_eq!(moved?, Called::Finished("30".to_owned()));
    assert_eq!(first.call::<String>("hp_left", (String::new(),))?, Called::Finished("20".to_owned()));
    Ok(())
}

// The compiler's shortcuts for a list's items and for float maths give what the plain way gives.
#[test]
fn shortcuts_for_list_items_give_the_plain_answers() -> Result<(), Box<dyn std::error::Error>> {
    let source = "record Ball
    x: Float
    v: Float
    hits: Whole
end

proc settle[count: Whole] -> Float
    mut balls = [Ball[x: 0.0, v: 1.5, hits: 0], Ball[x: 4.0, v: -0.5, hits: 0], Ball[x: 9.0, v: 0.25, hits: 0]]
    kept = balls
    for each unused round in range[from: 1, to: count]
        for each i in range[from: 1, to: balls.len]
            for each j in range[from: i + 1, before: balls.len + 1]
                gap = balls[pos: j].x - balls[pos: i].x
                balls[pos: i].v = balls[pos: i].v + gap * 0.125 - balls[pos: j].v * 0.5
                balls[pos: j].v = balls[pos: j].v - gap * 0.125 * balls[pos: i].v
                balls[pos: j].hits = balls[pos: j].hits + balls[pos: i].hits + 1
                balls[pos: 1].hits = balls[offset: 0].hits + 1
            end
            balls[pos: i].x = balls[pos: i].x + balls[pos: i].v * 0.5
        end
    end
    mut total = kept[pos: 1].v
    for each ball in balls
        total = total + ball.x * 0.5 + ball.v + be.Float(ball.hits)
    end
    return total
end
";
    let doors = Doors::new();
    let limits = Limits { steps: 10_000_000, memory: 1 << 20 };
    let mut usual = Script::load("balls.seal", source, &doors, limits)?;
    let mut plain = Script::load_plainly("balls.seal", source, &doors, limits)?;
    for count in [1i64, 2, 3, 5, 12] {
        // A run that stops, as on a number too big for a `Float`, must stop the same way both ways.
        let with_shortcuts = usual.call::<f64>("settle", (count,)).map_err(|error| error.to_string());
        let plainly = plain.call::<f64>("settle", (count,)).map_err(|error| error.to_string());
        assert_eq!(with_shortcuts, plainly, "after {count} rounds");
        assert!(with_shortcuts.is_ok(), "after {count} rounds: {with_shortcuts:?}");
    }
    assert_eq!(usual.memory_in_use(), plain.memory_in_use());
    Ok(())
}
