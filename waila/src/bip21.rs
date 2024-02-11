use core::str::FromStr;
use std::borrow::Cow;
use std::convert::TryFrom;

use ::bip21::de::*;
use ::bip21::*;
use bitcoin::address::NetworkUnchecked;
use lightning::offers::offer::Offer;
use lightning::offers::parse::Bolt12ParseError;
use lightning_invoice::{Bolt11Invoice, ParseOrSemanticError};
use url::Url;

/// This lets us parse `lightning`, bolt12, and payjoin parameters from a BIP21 URI.
pub type UnifiedUri<'a> = Uri<'a, NetworkUnchecked, WailaExtras>;

#[derive(Debug, Default, Clone)]
pub struct WailaExtras {
    pub lightning: Option<Bolt11Invoice>,
    pub b12: Option<Offer>,
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
    Bolt12ParsingError,
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

impl From<Bolt12ParseError> for ExtraParamsParseError {
    fn from(_e: Bolt12ParseError) -> Self {
        ExtraParamsParseError::Bolt12ParsingError
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
            "b12" if self.b12.is_none() => {
                let str =
                    Cow::try_from(value).map_err(|_| ExtraParamsParseError::InvoiceParsingError)?;
                let offer = Offer::from_str(&str)?;
                self.b12 = Some(offer);

                Ok(ParamKind::Known)
            }
            "b12" => Err(ExtraParamsParseError::MultipleParams(key.to_string())),
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
    use lightning::offers::offer::Offer;
    use lightning::util::ser::Writeable;
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
    fn test_offer_uri() {
        let input = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&b12=lno1qsgqmqvgm96frzdg8m0gc6nzeqffvzsqzrxqy32afmr3jn9ggkwg3egfwch2hy0l6jut6vfd8vpsc3h89l6u3dm4q2d6nuamav3w27xvdmv3lpgklhg7l5teypqz9l53hj7zvuaenh34xqsz2sa967yzqkylfu9xtcd5ymcmfp32h083e805y7jfd236w9afhavqqvl8uyma7x77yun4ehe9pnhu2gekjguexmxpqjcr2j822xr7q34p078gzslf9wpwz5y57alxu99s0z2ql0kfqvwhzycqq45ehh58xnfpuek80hw6spvwrvttjrrq9pphh0dpydh06qqspp5uq4gpyt6n9mwexde44qv7lstzzq60nr40ff38u27un6y53aypmx0p4qruk2tf9mjwqlhxak4znvna5y";
        let offer = Offer::from_str("lno1qsgqmqvgm96frzdg8m0gc6nzeqffvzsqzrxqy32afmr3jn9ggkwg3egfwch2hy0l6jut6vfd8vpsc3h89l6u3dm4q2d6nuamav3w27xvdmv3lpgklhg7l5teypqz9l53hj7zvuaenh34xqsz2sa967yzqkylfu9xtcd5ymcmfp32h083e805y7jfd236w9afhavqqvl8uyma7x77yun4ehe9pnhu2gekjguexmxpqjcr2j822xr7q34p078gzslf9wpwz5y57alxu99s0z2ql0kfqvwhzycqq45ehh58xnfpuek80hw6spvwrvttjrrq9pphh0dpydh06qqspp5uq4gpyt6n9mwexde44qv7lstzzq60nr40ff38u27un6y53aypmx0p4qruk2tf9mjwqlhxak4znvna5y").unwrap();

        let uri = UnifiedUri::from_str(input).unwrap();
        assert!(uri.extras.lightning.is_none());
        assert_eq!(uri.extras.b12.map(|i| i.encode()), Some(offer.encode()));
    }

    #[test]
    fn test_no_ln_uri() {
        let input = "bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd";

        assert!(UnifiedUri::try_from(input).is_ok());
        let uri = UnifiedUri::from_str(input).unwrap();
        assert_eq!(uri.extras.lightning, None);
    }
}
