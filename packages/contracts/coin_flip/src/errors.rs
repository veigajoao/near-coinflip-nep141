// contract errors
pub const ERR_001: &str = "ERR_001: Account is not registered";
pub const ERR_002: &str = "ERR_002: No partner registered for this address";
pub const ERR_003: &str = "ERR_003: Partner already registered for this address";
pub const ERR_004: &str = "ERR_004: Only partner game owner can call this method";
pub const ERR_005: &str = "ERR_005: ft_on_transfer msg parameter could not be parsed";
pub const ERR_006: &str = "ERR_006: Only owner can call this method";
pub const ERR_007: &str = "ERR_007: Panic mode is on, all non owner tasks are suspended";


// storage errors
pub const ERR_101: &str = "ERR_101: Insufficient storage deposit";
pub const ERR_102: &str = "ERR_102: Must attach at least the minimum deposit value";
pub const ERR_103: &str = "ERR_103: Cannot unregister storage while user still has token balances to withdraw";

// owner actions errors
pub const ERR_201: &str = "ERR_201: No owner funds to withdraw";
pub const ERR_202: &str = "ERR_202: No NFT funds to withdraw";
pub const ERR_203: &str = "ERR_203: Balance for this token is 0";
pub const ERR_204: &str = "ERR_204: Token index out of bouds";
pub const ERR_205: &str = "ERR_205: Fee parameters must be <= FRACTION_BASE";
pub const ERR_206: &str = "ERR_206: max parameter must be greater than min parameter";


// partnered game errors
pub const ERR_301: &str = "ERR_301: Token sent is not the registered token type for game";

// player actions errors
pub const ERR_401: &str = "ERR_401: Not enough balance for this withdraw";
pub const ERR_402: &str = "ERR_402: Not enough balance for this bet size";
pub const ERR_403: &str = "ERR_403: Minimum bet size not respected";
pub const ERR_404: &str = "ERR_404: Maximum bet size not respected";
pub const ERR_405: &str = "ERR_405: Minimum odds not respected";
pub const ERR_406: &str = "ERR_406: Maximum odds not respected";
pub const ERR_407: &str = "ERR_407: Bet denied, house_funds are not enough to cover your possible win value";