mod bip21;

use std::str::FromStr;

use crate::bip21::UnifiedUri;
use bitcoin::{Address, Amount, Network, PublicKey};
use lightning::offers::offer;
use lightning::offers::offer::Offer;
use lightning_invoice::{Currency, Invoice, InvoiceDescription};
use lnurl::lightning_address::LightningAddress;
use lnurl::lnurl::LnUrl;

#[derive(Debug)]
pub enum PaymentParams<'a> {
    OnChain(Address),
    Bip21(UnifiedUri<'a>),
    Bolt11(Invoice),
    Bolt12(Offer),
    NodePubkey(PublicKey),
    LnUrl(LnUrl),
    LightningAddress(LightningAddress),
}

impl PaymentParams<'_> {
    pub fn memo(&self) -> Option<String> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.message.clone().and_then(|m| m.try_into().ok()),
            PaymentParams::Bolt11(invoice) => match invoice.description() {
                InvoiceDescription::Direct(desc) => Some(desc.to_string()),
                InvoiceDescription::Hash(_) => None,
            },
            PaymentParams::Bolt12(offer) => Some(offer.description().to_string()),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn network(&self) -> Option<Network> {
        match self {
            PaymentParams::OnChain(address) => Some(address.network),
            PaymentParams::Bip21(uri) => Some(uri.address.network),
            PaymentParams::Bolt11(invoice) => match invoice.currency() {
                Currency::Bitcoin => Some(Network::Bitcoin),
                Currency::BitcoinTestnet => Some(Network::Testnet),
                Currency::Regtest => Some(Network::Regtest),
                Currency::Simnet => Some(Network::Regtest),
                Currency::Signet => Some(Network::Signet),
            },
            PaymentParams::Bolt12(_) => None, // todo fix after https://github.com/rust-bitcoin/rust-bitcoin/pull/1675
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn amount(&self) -> Option<Amount> {
        self.amount_msats()
            .map(|msats| Amount::from_sat(msats / 1_000))
    }

    pub fn amount_msats(&self) -> Option<u64> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.amount.map(|amount| amount.to_sat() * 1_000),
            PaymentParams::Bolt11(invoice) => invoice.amount_milli_satoshis(),
            PaymentParams::Bolt12(offer) => offer.amount().and_then(|amt| match amt {
                offer::Amount::Bitcoin { amount_msats } => Some(*amount_msats),
                offer::Amount::Currency { .. } => None,
            }),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn address(&self) -> Option<Address> {
        match self {
            PaymentParams::OnChain(address) => Some(address.clone()),
            PaymentParams::Bip21(uri) => Some(uri.address.clone()),
            PaymentParams::Bolt11(_) => None, // todo update after https://github.com/lightningdevkit/rust-lightning/pull/2023
            PaymentParams::Bolt12(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn invoice(&self) -> Option<Invoice> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.extras.clone().lightning,
            PaymentParams::Bolt11(invoice) => Some(invoice.clone()),
            PaymentParams::Bolt12(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn node_pubkey(&self) -> Option<PublicKey> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.extras.clone().lightning.map(|invoice| {
                let secp = invoice.recover_payee_pub_key();
                PublicKey::new(secp)
            }),
            PaymentParams::Bolt11(invoice) => {
                let secp = invoice.recover_payee_pub_key();
                Some(PublicKey::new(secp))
            }
            PaymentParams::Bolt12(_) => None,
            PaymentParams::NodePubkey(pubkey) => Some(*pubkey),
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn lnurl(&self) -> Option<LnUrl> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(lnurl) => Some(lnurl.clone()),
            PaymentParams::LightningAddress(ln_addr) => LnUrl::from_url(ln_addr.lnurlp_url()).ok(),
        }
    }
}

impl FromStr for PaymentParams<'_> {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        Address::from_str(str)
            .map(PaymentParams::OnChain)
            .or_else(|_| Invoice::from_str(str).map(PaymentParams::Bolt11))
            .or_else(|_| PublicKey::from_str(str).map(PaymentParams::NodePubkey))
            .or_else(|_| LnUrl::from_str(str).map(PaymentParams::LnUrl))
            .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
            .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
            .or_else(|_| UnifiedUri::from_str(str).map(PaymentParams::Bip21))
            .or_else(|_| Offer::from_str(str).map(PaymentParams::Bolt12))
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const SAMPLE_INVOICE: &str = "lnbc20m1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqhp58yjmdan79s6qqdhdzgynm4zwqd5d7xmw5fk98klysy043l2ahrqsfpp3qjmp7lwpagxun9pygexvgpjdc4jdj85fr9yq20q82gphp2nflc7jtzrcazrra7wwgzxqc8u7754cdlpfrmccae92qgzqvzq2ps8pqqqqqqpqqqqq9qqqvpeuqafqxu92d8lr6fvg0r5gv0heeeqgcrqlnm6jhphu9y00rrhy4grqszsvpcgpy9qqqqqqgqqqqq7qqzq9qrsgqdfjcdk6w3ak5pca9hwfwfh63zrrz06wwfya0ydlzpgzxkn5xagsqz7x9j4jwe7yj7vaf2k9lqsdk45kts2fd0fkr28am0u4w95tt2nsq76cqw0";
    const SAMPLE_BIP21: &str = "bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz";
    const SAMPLE_BIP21_WITH_INVOICE: &str = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";
    const SAMPLE_LNURL: &str = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";

    #[test]
    fn parse_node_pubkey() {
        let pubkey = PublicKey::from_str(
            "03e7156ae33b0a208d0744199163177e909e80176e55d97a2f221ede0f934dd9ad",
        )
            .unwrap();
        let parsed = PaymentParams::from_str(&pubkey.to_string()).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), Some(pubkey));
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_address() {
        let address = Address::from_str("1andreas3batLhQa2FawWjeyjCqyBzypd").unwrap();
        let parsed = PaymentParams::from_str(&address.to_string()).unwrap();

        assert_eq!(parsed.address(), Some(address));
        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_invoice() {
        let parsed = PaymentParams::from_str(SAMPLE_INVOICE).unwrap();

        let expected_pubkey = PublicKey::from_str(
            "03e7156ae33b0a208d0744199163177e909e80176e55d97a2f221ede0f934dd9ad",
        )
            .unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(2_000_000)));
        assert_eq!(parsed.amount_msats(), Some(2_000_000_000));
        assert_eq!(parsed.node_pubkey(), Some(expected_pubkey));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(parsed.address(), None); // todo: add fallback address
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_bip_21() {
        let parsed = PaymentParams::from_str(SAMPLE_BIP21).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_btc(50 as f64).unwrap()));
        assert_eq!(
            parsed.address(),
            Some(Address::from_str("1andreas3batLhQa2FawWjeyjCqyBzypd").unwrap())
        );
        assert_eq!(parsed.memo(), Some("Donation for project xyz".to_string()));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_bip_21_with_invoice() {
        let parsed = PaymentParams::from_str(SAMPLE_BIP21_WITH_INVOICE).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_btc(0.00001).unwrap()));
        assert_eq!(
            parsed.address(),
            Some(Address::from_str("BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U").unwrap())
        );
        assert_eq!(parsed.memo(), Some("For lunch Tuesday".to_string()));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(parsed.invoice(), Some(Invoice::from_str("LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6").unwrap()));
        assert_eq!(parsed.node_pubkey(), Some(PublicKey::from_str("037cc5f9f1da20ac0d60e83989729a204a33cc2d8e80438969fadf35c1c5f1233b").unwrap()));
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_lnurl() {
        let parsed = PaymentParams::from_str(SAMPLE_LNURL).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str(SAMPLE_LNURL).unwrap()));
    }

    #[test]
    fn parse_lightning_address() {
        let parsed = PaymentParams::from_str("ben@opreturnbot.com").unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap()));
    }
}
