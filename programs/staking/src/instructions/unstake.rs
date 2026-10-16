//! Withdraw principal allocated to a character.
//! TODO: Settle rewards before reducing stake, transfer principal to its owner,
//! and recompute the character level from the remaining stake.
//! Falling below a threshold downgrades the level; below level 1 locks the character.
//! Previously earned rewards remain claimable. Lock/cooldown policy is undecided.
