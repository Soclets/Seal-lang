//! Signed `.sealc` files (D169, D175): a game's developer signs its compiled scripts with a secret key, and a host can
//! require a valid signature from the matching public key, so a file changed in any way, even one that stays a valid
//! program, is refused. The signature is ML-DSA-44 (FIPS 204), from the `fips204` crate; Seal writes no cryptography
//! of its own.
//!
//! A signed file is the file as `seal build` writes it, then the signature over all of it, then [`SIGNED`]. A host
//! that doesn't require signatures loads a signed file as it is, without checking.

use std::sync::{Arc, Mutex};

use fips204::ml_dsa_44;
use fips204::traits::{KeyGen, SerDes, Signer, Verifier};

use crate::message::{Message, Place};

/// What a signed file ends with, after its signature.
pub const SIGNED: &[u8; 8] = b"SEALSIG\x01";

/// How many bytes a public key, a secret key and a signature have.
pub const PUBLIC_KEY_LENGTH: usize = ml_dsa_44::PK_LEN;
pub const SECRET_KEY_LENGTH: usize = ml_dsa_44::SK_LEN;
pub const SIGNATURE_LENGTH: usize = ml_dsa_44::SIG_LEN;

/// Every signature says it's for a `.sealc` file, so one made for anything else never passes.
const CONTEXT: &[u8] = b"seal .sealc";

/// trace: D175. A key pair made from 32 random bytes the caller gives: the public key, for hosts, then the secret one,
/// for the build that signs. `seal keygen` gives the system's randomness.
#[must_use]
pub fn keys_from_seed(seed: &[u8; 32]) -> (Vec<u8>, Vec<u8>) {
    let (public, secret) = ml_dsa_44::KG::keygen_from_seed(seed);
    (public.into_bytes().to_vec(), secret.into_bytes().to_vec())
}

/// trace: D169. A `.sealc` file's bytes, signed with a secret key, given 32 random bytes for the signature.
///
/// # Errors
///
/// When the secret key isn't one, or the file is signed already.
pub fn sign(file: &[u8], secret_key: &[u8], randomness: &[u8; 32]) -> Result<Vec<u8>, Message> {
    if split(file).1.is_some() {
        return Err(Message::refused(Place::default(), "this `.sealc` file is signed already")
            .with_why("a file is signed once, by the build that makes it."));
    }
    let secret = <[u8; SECRET_KEY_LENGTH]>::try_from(secret_key)
        .ok()
        .and_then(|bytes| ml_dsa_44::PrivateKey::try_from_bytes(bytes).ok())
        .ok_or_else(|| not_a_key("secret", secret_key.len(), SECRET_KEY_LENGTH))?;
    let signature = secret
        .try_sign_with_seed(randomness, file, CONTEXT)
        .map_err(|_| Message::internal("signing a `.sealc` file went wrong"))?;
    let mut signed = Vec::with_capacity(file.len() + SIGNATURE_LENGTH + SIGNED.len());
    signed.extend_from_slice(file);
    signed.extend_from_slice(&signature);
    signed.extend_from_slice(SIGNED);
    Ok(signed)
}

fn not_a_key(which: &str, length: usize, wanted: usize) -> Message {
    Message::refused(
        Place::default(),
        format!("this isn't an ML-DSA-44 {which} key: it has {length} bytes, and one has {wanted}"),
    )
    .with_why("Seal signs `.sealc` files with ML-DSA-44, and `seal keygen` makes its keys.")
}

/// A file's bytes as built, and its signature when it has one.
#[must_use]
pub fn split(bytes: &[u8]) -> (&[u8], Option<&[u8]>) {
    let Some(body_end) = bytes.len().checked_sub(SIGNATURE_LENGTH + SIGNED.len()) else { return (bytes, None) };
    if bytes.get(bytes.len() - SIGNED.len()..) != Some(SIGNED.as_slice()) {
        return (bytes, None);
    }
    match (bytes.get(..body_end), bytes.get(body_end..bytes.len() - SIGNED.len())) {
        (Some(body), Some(signature)) => (body, Some(signature)),
        _ => (bytes, None),
    }
}

/// trace: D169. A host's public key, and the files already checked against it, so each file or bundle is checked
/// once in a run, however many scripts are loaded from it: a check costs far more than loading a script.
pub(crate) struct RequiredKey {
    key: ml_dsa_44::PublicKey,
    /// The files that passed, kept whole: a file is found among them only when it's the same, byte for byte.
    checked: Mutex<Vec<Arc<[u8]>>>,
}

impl RequiredKey {
    /// # Errors
    ///
    /// When the bytes aren't an ML-DSA-44 public key.
    pub(crate) fn new(public_key: &[u8]) -> Result<RequiredKey, Message> {
        let key = <[u8; PUBLIC_KEY_LENGTH]>::try_from(public_key)
            .ok()
            .and_then(|bytes| ml_dsa_44::PublicKey::try_from_bytes(bytes).ok())
            .ok_or_else(|| not_a_key("public", public_key.len(), PUBLIC_KEY_LENGTH))?;
        Ok(RequiredKey { key, checked: Mutex::default() })
    }

    /// The file's bytes as built, when it's signed with this key.
    ///
    /// # Errors
    ///
    /// The refusal when it isn't signed, or not with this key.
    pub(crate) fn check<'b>(&self, bytes: &'b [u8]) -> Result<&'b [u8], Message> {
        let why = "a host that requires signatures loads only files signed with its own key, so a file changed in any way is refused.";
        let (body, Some(signature)) = split(bytes) else {
            return Err(Message::refused(
                Place::default(),
                "this `.sealc` file isn't signed, and this host loads only signed ones",
            )
            .with_why(why));
        };
        let mut checked = self.checked.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if checked.iter().any(|known| **known == *bytes) {
            return Ok(body);
        }
        let passes = <[u8; SIGNATURE_LENGTH]>::try_from(signature)
            .is_ok_and(|signature| self.key.verify(body, &signature, CONTEXT));
        if !passes {
            return Err(Message::refused(
                Place::default(),
                "this `.sealc` file's signature doesn't match the host's key",
            )
            .with_why(why));
        }
        checked.push(Arc::from(bytes));
        Ok(body)
    }
}
