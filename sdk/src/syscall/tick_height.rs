//! This account contains the current cluster tick height
//!
use crate::account::Account;
use crate::syscall;
use bincode::serialized_size;

/// "Sysca11TickHeight11111111111111111111111111"
///  tick_height account pubkey
const ID: [u8; 32] = [
    6, 167, 211, 138, 69, 219, 242, 63, 162, 206, 168, 232, 212, 90, 152, 107, 220, 251, 113, 215,
    208, 229, 34, 163, 11, 168, 45, 109, 60, 0, 0, 0,
];

crate::solana_name_id!(ID, "Sysca11TickHeight11111111111111111111111111");

#[repr(C)]
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct TickHeight(pub u64);

impl TickHeight {
    pub fn from(account: &Account) -> Option<Self> {
        account.deserialize_data().ok()
    }
    pub fn to(&self, account: &mut Account) -> Option<()> {
        account.serialize_data(self).ok()
    }

    pub fn size_of() -> usize {
        serialized_size(&TickHeight::default()).unwrap() as usize
    }
}

pub fn create_account(lamports: u64, tick_height: u64) -> Account {
    Account::new_data(lamports, &TickHeight(tick_height), &syscall::id()).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tick_height_create_account() {
        let account = create_account(1, 1);
        let tick_height = TickHeight::from(&account).unwrap();
        assert_eq!(tick_height.0, 1);
    }
}
