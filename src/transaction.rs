use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::{self, Visitor};
use serde::ser::SerializeStruct;
use sha2::{Digest, Sha256};
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Transaction {
    pub from: [u8; 32],      // Sender's address
    pub to: [u8; 32],        // Recipient's address
    pub amount: u64,         // Amount to transfer
    pub nonce: u64,          // Nonce to ensure uniqueness
    pub signature: [u8; 64], // Digital signature
}

// Manual implementation of Serialize for Transaction
impl Serialize for Transaction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Transaction", 5)?;
        state.serialize_field("from", &self.from)?;
        state.serialize_field("to", &self.to)?;
        state.serialize_field("amount", &self.amount)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("signature", &self.signature.as_slice())?;
        state.end()
    }
}

// Manual implementation of Deserialize for Transaction
impl<'de> Deserialize<'de> for Transaction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            From,
            To,
            Amount,
            Nonce,
            Signature,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("`from`, `to`, `amount`, `nonce`, or `signature`")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Field, E>
                    where
                        E: de::Error,
                    {
                        match value {
                            "from" => Ok(Field::From),
                            "to" => Ok(Field::To),
                            "amount" => Ok(Field::Amount),
                            "nonce" => Ok(Field::Nonce),
                            "signature" => Ok(Field::Signature),
                            _ => Err(de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct TransactionVisitor;

        impl<'de> Visitor<'de> for TransactionVisitor {
            type Value = Transaction;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Transaction")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Transaction, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut from = None;
                let mut to = None;
                let mut amount = None;
                let mut nonce = None;
                let mut signature: Option<Vec<u8>> = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::From => {
                            if from.is_some() {
                                return Err(de::Error::duplicate_field("from"));
                            }
                            from = Some(map.next_value()?);
                        }
                        Field::To => {
                            if to.is_some() {
                                return Err(de::Error::duplicate_field("to"));
                            }
                            to = Some(map.next_value()?);
                        }
                        Field::Amount => {
                            if amount.is_some() {
                                return Err(de::Error::duplicate_field("amount"));
                            }
                            amount = Some(map.next_value()?);
                        }
                        Field::Nonce => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        Field::Signature => {
                            if signature.is_some() {
                                return Err(de::Error::duplicate_field("signature"));
                            }
                            signature = Some(map.next_value()?);
                        }
                    }
                }

                let from = from.ok_or_else(|| de::Error::missing_field("from"))?;
                let to = to.ok_or_else(|| de::Error::missing_field("to"))?;
                let amount = amount.ok_or_else(|| de::Error::missing_field("amount"))?;
                let nonce = nonce.ok_or_else(|| de::Error::missing_field("nonce"))?;
                let signature = signature.clone().ok_or_else(|| de::Error::missing_field("signature"))?;

                // Convert the signature from Vec<u8> to [u8; 64]
                let signature: [u8; 64] = signature.clone()
                    .try_into()
                    .map_err(|_| de::Error::invalid_length(signature.len(), &"expected a Vec of length 64"))?;

                Ok(Transaction {
                    from,
                    to,
                    amount,
                    nonce,
                    signature,
                })
            }
        }

        const FIELDS: &'static [&'static str] = &["from", "to", "amount", "nonce", "signature"];
        deserializer.deserialize_struct("Transaction", FIELDS, TransactionVisitor)
    }
}

impl Default for Transaction {
    fn default() -> Self {
        Transaction {
            from: [0u8; 32],
            to: [0u8; 32],
            amount: 0,
            nonce: 0,
            signature: [0u8; 64],
        }
    }
}

impl Transaction {
    /// Computes a hash for the transaction using a chosen hash function.
    pub fn compute_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.from);
        hasher.update(&self.to);
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.nonce.to_le_bytes());
        hasher.update(&self.signature);
        hasher.finalize().into()
    }
}

pub struct TransactionBuilder {
    from: Option<[u8; 32]>,
    to: Option<[u8; 32]>,
    amount: Option<u64>,
    nonce: Option<u64>,
    signature: Option<[u8; 64]>,
}

impl TransactionBuilder {
    pub fn new() -> Self {
        Self {
            from: None,
            to: None,
            amount: None,
            nonce: None,
            signature: None,
        }
    }

    pub fn from(mut self, from: [u8; 32]) -> Self {
        self.from = Some(from);
        self
    }

    pub fn to(mut self, to: [u8; 32]) -> Self {
        self.to = Some(to);
        self
    }

    pub fn amount(mut self, amount: u64) -> Self {
        self.amount = Some(amount);
        self
    }

    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    pub fn signature(mut self, signature: [u8; 64]) -> Self {
        self.signature = Some(signature);
        self
    }

    pub fn build(self) -> Result<Transaction, String> {
        Ok(Transaction {
            from: self.from.ok_or("Sender address is missing")?,
            to: self.to.ok_or("Recipient address is missing")?,
            amount: self.amount.ok_or("Amount is missing")?,
            nonce: self.nonce.ok_or("Nonce is missing")?,
            signature: self.signature.ok_or("Signature is missing")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_builder_success() {
        let from = [1u8; 32];
        let to = [2u8; 32];
        let amount = 100;
        let nonce = 1;
        let signature = [0u8; 64];

        let tx = TransactionBuilder::new()
            .from(from)
            .to(to)
            .amount(amount)
            .nonce(nonce)
            .signature(signature)
            .build()
            .expect("Failed to build transaction");

        assert_eq!(tx.from, from);
        assert_eq!(tx.to, to);
        assert_eq!(tx.amount, amount);
        assert_eq!(tx.nonce, nonce);
        assert_eq!(tx.signature, signature);
    }

    #[test]
    fn test_transaction_builder_missing_fields() {
        let result = TransactionBuilder::new().build();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Sender address is missing");
    }
}
