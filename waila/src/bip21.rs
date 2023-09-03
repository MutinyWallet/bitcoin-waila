use core::str::FromStr;
use std::borrow::Cow;
use std::convert::TryFrom;

use ::bip21::de::*;
use ::bip21::*;
use lightning_invoice::{Bolt11Invoice, ParseOrSemanticError};

/// This lets us parse a `lightning` parameter from a BIP21 URI.
pub type UnifiedUri<'a> = Uri<'a, LightningExtras>;

#[derive(Debug, Default, Eq, PartialEq, Clone, Hash)]
pub struct LightningExtras {
    pub lightning: Option<Bolt11Invoice>,
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum LightningParseError {
    MultipleParams,
    InvoiceParsingError,
}

impl From<ParseOrSemanticError> for LightningParseError {
    fn from(_e: ParseOrSemanticError) -> Self {
        LightningParseError::InvoiceParsingError
    }
}

impl DeserializationError for LightningExtras {
    type Error = LightningParseError;
}

impl<'a> DeserializeParams<'a> for LightningExtras {
    type DeserializationState = LightningExtras;
}

impl<'a> DeserializationState<'a> for LightningExtras {
    type Value = LightningExtras;

    fn is_param_known(&self, param: &str) -> bool {
        matches!(param, "lightning")
    }

    fn deserialize_temp(
        &mut self,
        key: &str,
        value: Param<'_>,
    ) -> Result<ParamKind, <Self::Value as DeserializationError>::Error> {
        match key {
            "lightning" if self.lightning.is_none() => {
                let str =
                    Cow::try_from(value).map_err(|_| LightningParseError::InvoiceParsingError)?;
                let invoice = Bolt11Invoice::from_str(&str)?;
                self.lightning = Some(invoice);

                Ok(ParamKind::Known)
            }
            "lightning" => Err(LightningParseError::MultipleParams),
            _ => Ok(ParamKind::Unknown),
        }
    }

    fn finalize(self) -> Result<Self::Value, <Self::Value as DeserializationError>::Error> {
        Ok(self)
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;
    use std::convert::TryFrom;

    use lightning_invoice::Bolt11Invoice;

    use crate::bip21::LightningExtras;

    type UnifiedUri<'a> = bip21::Uri<'a, LightningExtras>;

    #[test]
    fn test_ln_uri() {
        let input = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";
        let expected_invoice = Bolt11Invoice::from_str("LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6").unwrap();

        assert!(UnifiedUri::try_from(input).is_ok());
        let uri = UnifiedUri::from_str(input).unwrap();
        assert_eq!(uri.extras.lightning, Some(expected_invoice));
    }

    #[test]
    fn test_no_ln_uri() {
        let input = "bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd";

        assert!(UnifiedUri::try_from(input).is_ok());
        let uri = UnifiedUri::from_str(input).unwrap();
        assert_eq!(uri.extras.lightning, None);
    }
}
