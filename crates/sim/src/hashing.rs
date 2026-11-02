//! Canonical state serialization and BLAKE3 state hashes.

use crate::state::GalaxyState;

pub fn canonical_state_bytes(state: &GalaxyState) -> Result<Vec<u8>, serde_json::Error> {
    // State is represented only by ordered dense vectors and fixed-point
    // integers, so serde's struct field order is canonical for this schema.
    serde_json::to_vec(state)
}

pub fn state_hash(state: &GalaxyState) -> Result<[u8; 32], serde_json::Error> {
    let bytes = canonical_state_bytes(state)?;
    Ok(*blake3::hash(&bytes).as_bytes())
}

pub fn state_hash_hex(state: &GalaxyState) -> Result<String, serde_json::Error> {
    Ok(state_hash(state)?
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_state_has_byte_identical_canonical_form_and_hash() {
        let state = GalaxyState::default();
        assert_eq!(
            canonical_state_bytes(&state).unwrap(),
            canonical_state_bytes(&state).unwrap()
        );
        assert_eq!(state_hash(&state).unwrap(), state_hash(&state).unwrap());
        assert_eq!(state_hash_hex(&state).unwrap().len(), 64);
    }
}
