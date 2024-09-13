use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Account {
    pub address: [u8; 32], // Unique address for the account
    pub balance: u64,      // Account balance
    pub nonce: u64,        // Nonce to prevent replay attacks
}

impl Account {
    /// Creates a new account with the given address and initial balance.
    pub fn new(address: [u8; 32], initial_balance: u64) -> Self {
        Self {
            address,
            balance: initial_balance,
            nonce: 0, // Start nonce at 0
        }
    }

    /// Transfers an amount from the account, reducing its balance.
    /// Returns an error if the balance is insufficient.
    pub fn transfer(&mut self, amount: u64) -> Result<(), String> {
        if self.balance >= amount {
            self.balance -= amount;
            self.nonce += 1; // Increment nonce after a successful transfer
            Ok(())
        } else {
            Err("Insufficient balance".to_string())
        }
    }

    /// Credits an amount to the account, increasing its balance.
    pub fn credit(&mut self, amount: u64) {
        self.balance = self.balance.saturating_add(amount); // Use saturating_add to prevent overflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let address = [1u8; 32];
        let account = Account::new(address, 100);
        assert_eq!(account.address, address);
        assert_eq!(account.balance, 100);
        assert_eq!(account.nonce, 0);
    }

    #[test]
    fn test_transfer_success() {
        let mut account = Account::new([1u8; 32], 100);
        assert!(account.transfer(50).is_ok());
        assert_eq!(account.balance, 50);
        assert_eq!(account.nonce, 1);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let mut account = Account::new([1u8; 32], 100);
        assert!(account.transfer(150).is_err());
        assert_eq!(account.balance, 100); // Balance should remain unchanged
        assert_eq!(account.nonce, 0); // Nonce should not increment
    }

    #[test]
    fn test_credit() {
        let mut account = Account::new([1u8; 32], 100);
        account.credit(50);
        assert_eq!(account.balance, 150);
    }
}
