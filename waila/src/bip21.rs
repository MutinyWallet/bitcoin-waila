use core::str::FromStr;
use std::borrow::Cow;
use std::convert::TryFrom;

use ::bip21::de::*;
use ::bip21::*;
use bitcoin::address::NetworkUnchecked;
use lightning_invoice::{Bolt11Invoice, ParseOrSemanticError};
use url::Url;

/// This lets us parse `lightning` and payjoin parameters from a BIP21 URI.
pub type UnifiedUri<'a> = Uri<'a, NetworkUnchecked, WailaExtras>;

#[derive(Debug, Default, Eq, PartialEq, Clone, Hash)]
pub struct WailaExtras {
    pub lightning: Option<Bolt11Invoice>,
    pub pj: Option<Url>,
    pjos: Option<bool>,
}

impl WailaExtras {
    pub fn disable_output_substitution(&self) -> bool {
        self.pjos.unwrap_or(false)
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ExtraParamsParseError {
    MultipleParams(String),
    InvoiceParsingError,
    MissingEndpoint,
    NotUtf8(core::str::Utf8Error),
    BadEndpoint(url::ParseError),
    UnsecureEndpoint,
    BadPjOs,
}

impl From<ParseOrSemanticError> for ExtraParamsParseError {
    fn from(_e: ParseOrSemanticError) -> Self {
        ExtraParamsParseError::InvoiceParsingError
    }
}

impl DeserializationError for WailaExtras {
    type Error = ExtraParamsParseError;
}

impl<'a> DeserializeParams<'a> for WailaExtras {
    type DeserializationState = WailaExtras;
}

impl<'a> DeserializationState<'a> for WailaExtras {
    type Value = WailaExtras;

    fn is_param_known(&self, param: &str) -> bool {
        matches!(param, "lightning" | "pj" | "pjos")
    }

    fn deserialize_temp(
        &mut self,
        key: &str,
        value: Param<'_>,
    ) -> Result<ParamKind, <Self::Value as DeserializationError>::Error> {
        match key {
            "pj" if self.pj.is_none() => {
                let endpoint = Cow::try_from(value).map_err(ExtraParamsParseError::NotUtf8)?;
                let url = Url::parse(&endpoint).map_err(ExtraParamsParseError::BadEndpoint)?;
                self.pj = Some(url);

                Ok(ParamKind::Known)
            }
            "pj" => Err(ExtraParamsParseError::MultipleParams(key.to_string())),
            "pjos" if self.pjos.is_none() => {
                match &*Cow::try_from(value).map_err(|_| ExtraParamsParseError::BadPjOs)? {
                    "0" => self.pjos = Some(false),
                    "1" => self.pjos = Some(true),
                    _ => return Err(ExtraParamsParseError::BadPjOs),
                }
                Ok(ParamKind::Known)
            }
            "pjos" => Err(ExtraParamsParseError::MultipleParams(key.to_string())),
            "lightning" if self.lightning.is_none() => {
                let str =
                    Cow::try_from(value).map_err(|_| ExtraParamsParseError::InvoiceParsingError)?;
                let invoice = Bolt11Invoice::from_str(&str)?;
                self.lightning = Some(invoice);

                Ok(ParamKind::Known)
            }
            "lightning" => Err(ExtraParamsParseError::MultipleParams(key.to_string())),
            _ => Ok(ParamKind::Unknown),
        }
    }

    fn finalize(self) -> Result<Self::Value, <Self::Value as DeserializationError>::Error> {
        match (self.pj.as_ref(), self.pjos) {
            (None, None) => Ok(self),
            (None, Some(_)) => Err(ExtraParamsParseError::MissingEndpoint),
            (Some(endpoint), _) => {
                if endpoint.scheme() == "https"
                    || endpoint.scheme() == "http"
                        && endpoint.domain().unwrap_or_default().ends_with(".onion")
                {
                    Ok(self)
                } else {
                    Err(ExtraParamsParseError::UnsecureEndpoint)
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use core::str::FromStr;
    use std::convert::TryFrom;

    use lightning_invoice::Bolt11Invoice;

    use crate::bip21::UnifiedUri;

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
