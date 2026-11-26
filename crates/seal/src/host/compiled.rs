//! Scripts compiled ahead of time into `.sealc` files, and the cache that keeps them (D141, D153, D154).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::doors::{Doors, Loaded};
use super::script::{Error, Limits, Script};
use crate::compile::file::{self, FileError, ToWrite};
use crate::message::{Message, Messages, Place};

impl Doors {
    /// trace: D153, D154. What a compiled script was compiled against: every door, type and unit group the host
    /// opens. A file built for other doors is refused, as a call to a door that isn't there would be.
    pub(crate) fn file_key(&self) -> u64 {
        let described = format!("{:?}{:?}{:?}", self.shared_signatures(), self.shared_types(), self.unit_groups());
        file::hash(described.as_bytes())
    }
}

/// A file's problem as the refusal a host gets.
fn refused(name: &str, error: &FileError) -> Error {
    let mut message = Message::refused(Place::default(), error.to_string()).with_why(match error {
        FileError::OtherSeal | FileError::OtherDoors => {
            "a `.sealc` file is built by one Seal for one host's doors; build it again with `seal build`."
        }
        _ => "a `.sealc` file is checked every time it's loaded, and one that doesn't pass is never run.",
    });
    name.clone_into(&mut message.file);
    Error::Refused(Messages::one(message))
}

/// trace: D141, D153. Reads, checks and compiles scripts once, for a host's doors, into the bytes of a `.sealc`
/// file: one script, or a bundle of several, each loaded by its name with [`Script::load_compiled`].
///
/// # Errors
///
/// [`Error::Refused`] with the messages of the first script that doesn't pass checking, as loading it would give.
pub fn build(scripts: &[(&str, &str)], doors: &Doors) -> Result<Vec<u8>, Error> {
    let mut compiled = Vec::with_capacity(scripts.len());
    for &(name, source) in scripts {
        compiled.push(Script::compile_for_file(name, source, doors)?);
    }
    let to_write: Vec<ToWrite> =
        scripts.iter().zip(&compiled).map(|(&(name, source), compiled)| ToWrite { name, source, compiled }).collect();
    Ok(file::write(&to_write, doors.file_key()))
}

/// trace: D153. The names of the scripts a `.sealc` file holds, in order, checking that it was built by this Seal
/// for these doors and isn't damaged.
///
/// # Errors
///
/// [`Error::Refused`] when it wasn't, or is.
pub fn names(bytes: &[u8], doors: &Doors) -> Result<Vec<String>, Error> {
    file::names(bytes, doors.file_key()).map_err(|error| refused("the `.sealc` file", &error))
}

/// trace: D154. A folder of compiled scripts a host keeps, so a script loaded from its text is read, checked and
/// compiled once, then loaded from the folder after that. Seal writes nothing unless a host makes one (D11).
#[derive(Clone, Debug)]
pub struct Cache {
    folder: PathBuf,
}

impl Cache {
    /// A cache in `folder`, made when it isn't there.
    #[must_use]
    pub fn new(folder: impl AsRef<Path>) -> Cache {
        Cache { folder: folder.as_ref().to_path_buf() }
    }

    /// Where the compiled form of `source` is kept, by its text, this Seal and the host's doors.
    fn path(&self, source: &str, doors: &Doors) -> PathBuf {
        let key = file::hash(
            [file::fingerprint().to_le_bytes().as_slice(), &doors.file_key().to_le_bytes(), source.as_bytes()]
                .concat()
                .as_slice(),
        );
        self.folder.join(format!("{key:016x}.sealc"))
    }
}

impl Script {
    /// Reads, checks and compiles a script for a file.
    fn compile_for_file(name: &str, source: &str, doors: &Doors) -> Result<crate::compile::bytecode::Compiled, Error> {
        match catch_unwind(AssertUnwindSafe(|| Self::prepare_for_file(name, source, doors))) {
            Ok(Ok(compiled)) => Ok(compiled),
            Ok(Err(mut messages)) => {
                for message in &mut messages {
                    message.locate(name, source);
                }
                Err(Error::Refused(Messages { list: messages }))
            }
            Err(_) => {
                let mut message = Message::internal("compiling a script went wrong");
                name.clone_into(&mut message.file);
                Err(Error::Stopped(message))
            }
        }
    }

    /// trace: D141, D153. Loads the script named `name` from a `.sealc` file's bytes, a single script or a bundle,
    /// verifying it first, every time. It runs exactly as the same script loaded from its text does.
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the file isn't one this Seal built for these doors, is damaged, has no script of that
    /// name, or doesn't pass verifying; as [`Script::load`] once the script starts.
    pub fn load_compiled(bytes: &[u8], name: &str, doors: &Doors, limits: Limits) -> Result<Script, Error> {
        let door_count = doors.shared_signatures().len();
        let read = catch_unwind(AssertUnwindSafe(|| file::read(bytes, name, doors.file_key(), door_count)));
        let script = match read {
            Ok(Ok(script)) => script,
            Ok(Err(error)) => return Err(refused(name, &error)),
            Err(_) => return Err(refused(name, &FileError::Damaged("reading it went wrong"))),
        };
        let loaded = Loaded {
            source: Arc::from(script.source.as_str()),
            compiled: Arc::new(script.compiled),
            name: Arc::from(script.name.as_str()),
        };
        Self::from_loaded(loaded, name, doors, limits, true)
    }

    /// trace: D154. Loads a script from its text through a cache of compiled scripts: from the cache when it has
    /// this text compiled for this Seal and these doors, and from the text otherwise, keeping what it compiled for
    /// next time. A cache file that's damaged or stale is never used; the text is loaded instead and the file
    /// written again. The cache never changes what a script means.
    ///
    /// # Errors
    ///
    /// As [`Script::load`].
    pub fn load_cached(
        name: &str,
        source: &str,
        doors: &Doors,
        limits: Limits,
        cache: &Cache,
    ) -> Result<Script, Error> {
        let path = cache.path(source, doors);
        if let Ok(bytes) = std::fs::read(&path)
            && let Ok(script) = Self::load_compiled(&bytes, name, doors, limits)
            && script.source_text() == source
        {
            return Ok(script);
        }
        let script = Script::load(name, source, doors, limits)?;
        let bytes = file::write(&[ToWrite { name, source, compiled: script.compiled_script() }], doors.file_key());
        // Written beside its place first, then moved there, so a script loading at the same time never reads half
        // of it. A cache that can't be written is only slower.
        let _ = std::fs::create_dir_all(&cache.folder);
        let partial = path.with_extension(format!("sealc.{}", std::process::id()));
        if std::fs::write(&partial, &bytes).is_ok() && std::fs::rename(&partial, &path).is_err() {
            let _ = std::fs::remove_file(&partial);
        }
        Ok(script)
    }
}
