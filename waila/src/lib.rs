use std::convert::TryInto;
use std::str::FromStr;

use bitcoin::secp256k1::PublicKey;
use bitcoin::{Address, Amount, Network};
use lightning::offers::offer;
use lightning::offers::offer::Offer;
use lightning_invoice::{Invoice, InvoiceDescription};
use lnurl::lightning_address::LightningAddress;
use lnurl::lnurl::LnUrl;

use crate::bip21::UnifiedUri;

mod bip21;

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
            PaymentParams::Bip21(uri) => uri
                .message
                .clone()
                .and_then(|m| m.try_into().ok())
                .or_else(|| uri.label.clone().and_then(|l| l.try_into().ok())),
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
            PaymentParams::Bolt11(invoice) => Some(Network::from(invoice.currency())),
            PaymentParams::Bolt12(_) => None, // todo fix after https://github.com/rust-bitcoin/rust-bitcoin/pull/1675
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    /// Given the network, determine if the payment params are valid for that network
    /// Returns None if the network is unknown
    pub fn valid_for_network(&self, network: Network) -> Option<bool> {
        match self {
            PaymentParams::OnChain(address) => Some(address.is_valid_for_network(network)),
            PaymentParams::Bip21(uri) => Some(uri.address.is_valid_for_network(network)),
            PaymentParams::Bolt11(invoice) => Some(Network::from(invoice.currency()) == network),
            PaymentParams::Bolt12(_) => None,
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
            PaymentParams::Bolt11(invoice) => invoice.fallback_addresses().first().cloned(),
            PaymentParams::Bolt12(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
        }
    }

    pub fn invoice(&self) -> Option<Invoice> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.extras.lightning.clone(),
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
            PaymentParams::Bip21(uri) => uri
                .extras
                .lightning
                .clone()
                .map(|invoice| invoice.recover_payee_pub_key()),
            PaymentParams::Bolt11(invoice) => Some(invoice.recover_payee_pub_key()),
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
            PaymentParams::LightningAddress(ln_addr) => Some(LnUrl::from_url(ln_addr.lnurlp_url())),
        }
    }

    pub fn is_lnurl_auth(&self) -> bool {
        self.lnurl()
            .map(|lnurl| lnurl.is_lnurl_auth())
            .unwrap_or(false)
    }
}

impl FromStr for PaymentParams<'_> {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let lower = str.to_lowercase();
        if lower.starts_with("lightning:") {
            let str = str.strip_prefix("lightning:").unwrap();
            return Invoice::from_str(str)
                .map(PaymentParams::Bolt11)
                .or_else(|_| LnUrl::from_str(str).map(PaymentParams::LnUrl))
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .or_else(|_| Offer::from_str(str).map(PaymentParams::Bolt12))
                .map_err(|_| ());
        } else if lower.starts_with("lnurl:") {
            let str = str.strip_prefix("lnurl:").unwrap();
            return LnUrl::from_str(str)
                .map(PaymentParams::LnUrl)
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .map_err(|_| ());
        } else if lower.starts_with("lnurlp:") {
            let str = str.strip_prefix("lnurlp:").unwrap();
            return LnUrl::from_str(str)
                .map(PaymentParams::LnUrl)
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .map_err(|_| ());
        }

        Address::from_str(str)
            .map(PaymentParams::OnChain)
            .or_else(|_| Invoice::from_str(str).map(PaymentParams::Bolt11))
            .or_else(|_| UnifiedUri::from_str(str).map(PaymentParams::Bip21))
            .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
            .or_else(|_| LnUrl::from_str(str).map(PaymentParams::LnUrl))
            .or_else(|_| PublicKey::from_str(str).map(PaymentParams::NodePubkey))
            .or_else(|_| Offer::from_str(str).map(PaymentParams::Bolt12))
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    const SAMPLE_PUBKEY: &str =
        "03e7156ae33b0a208d0744199163177e909e80176e55d97a2f221ede0f934dd9ad";
    const SAMPLE_INVOICE: &str = "lnbc20m1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqhp58yjmdan79s6qqdhdzgynm4zwqd5d7xmw5fk98klysy043l2ahrqsfpp3qjmp7lwpagxun9pygexvgpjdc4jdj85fr9yq20q82gphp2nflc7jtzrcazrra7wwgzxqc8u7754cdlpfrmccae92qgzqvzq2ps8pqqqqqqpqqqqq9qqqvpeuqafqxu92d8lr6fvg0r5gv0heeeqgcrqlnm6jhphu9y00rrhy4grqszsvpcgpy9qqqqqqgqqqqq7qqzq9qrsgqdfjcdk6w3ak5pca9hwfwfh63zrrz06wwfya0ydlzpgzxkn5xagsqz7x9j4jwe7yj7vaf2k9lqsdk45kts2fd0fkr28am0u4w95tt2nsq76cqw0";
    const SAMPLE_BIP21: &str = "bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz";
    const SAMPLE_BIP21_WITH_INVOICE: &str = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";
    const SAMPLE_BIP21_WITH_INVOICE_AND_LABEL: &str = "bitcoin:tb1p0vztr8q25czuka5u4ta5pqu0h8dxkf72mam89cpg4tg40fm8wgmqp3gv99?amount=0.000001&label=yooo&lightning=lntbs1u1pjrww6fdq809hk7mcnp4qvwggxr0fsueyrcer4x075walsv93vqvn3vlg9etesx287x6ddy4xpp5a3drwdx2fmkkgmuenpvmynnl7uf09jmgvtlg86ckkvgn99ajqgtssp5gr3aghgjxlwshnqwqn39c2cz5hw4cnsnzxdjn7kywl40rru4mjdq9qyysgqcqpcxqrpwurzjqfgtsj42x8an5zujpxvfhp9ngwm7u5lu8lvzfucjhex4pq8ysj5q2qqqqyqqv9cqqsqqqqlgqqqqqqqqfqzgl9zq04nzpxyvdr8vj3h98gvnj3luanj2cxcra0q2th4xjsxmtj8k3582l67xq9ffz5586f3nm5ax58xaqjg6rjcj2vzvx2q39v9eqpn0wx54";
    const SAMPLE_LNURL: &str = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";

    #[test]
    fn parse_node_pubkey() {
        let pubkey = PublicKey::from_str(SAMPLE_PUBKEY).unwrap();
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
        let expected_pubkey = PublicKey::from_str(SAMPLE_PUBKEY).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(2_000_000)));
        assert_eq!(parsed.amount_msats(), Some(2_000_000_000));
        assert_eq!(parsed.node_pubkey(), Some(expected_pubkey));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(
            parsed.address(),
            Some(Address::from_str("1RustyRX2oai4EYYDpQGWvEL62BBGqN9T").unwrap())
        );
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_invoice_with_prefix() {
        let parsed = PaymentParams::from_str(&format!("lightning:{SAMPLE_INVOICE}")).unwrap();
        let expected_pubkey = PublicKey::from_str(SAMPLE_PUBKEY).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(2_000_000)));
        assert_eq!(parsed.amount_msats(), Some(2_000_000_000));
        assert_eq!(parsed.node_pubkey(), Some(expected_pubkey));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(
            parsed.address(),
            Some(Address::from_str("1RustyRX2oai4EYYDpQGWvEL62BBGqN9T").unwrap())
        );
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
        assert_eq!(
            parsed.node_pubkey(),
            Some(
                PublicKey::from_str(
                    "037cc5f9f1da20ac0d60e83989729a204a33cc2d8e80438969fadf35c1c5f1233b"
                )
                .unwrap()
            )
        );
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_bip_21_with_invoice_with_label() {
        let parsed = PaymentParams::from_str(SAMPLE_BIP21_WITH_INVOICE_AND_LABEL).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_btc(0.000001).unwrap()));
        assert_eq!(
            parsed.address(),
            Some(
                Address::from_str("tb1p0vztr8q25czuka5u4ta5pqu0h8dxkf72mam89cpg4tg40fm8wgmqp3gv99")
                    .unwrap()
            )
        );
        assert_eq!(parsed.memo(), Some("yooo".to_string()));
        assert_eq!(parsed.network(), Some(Network::Testnet));
        assert_eq!(parsed.invoice(), Some(Invoice::from_str("lntbs1u1pjrww6fdq809hk7mcnp4qvwggxr0fsueyrcer4x075walsv93vqvn3vlg9etesx287x6ddy4xpp5a3drwdx2fmkkgmuenpvmynnl7uf09jmgvtlg86ckkvgn99ajqgtssp5gr3aghgjxlwshnqwqn39c2cz5hw4cnsnzxdjn7kywl40rru4mjdq9qyysgqcqpcxqrpwurzjqfgtsj42x8an5zujpxvfhp9ngwm7u5lu8lvzfucjhex4pq8ysj5q2qqqqyqqv9cqqsqqqqlgqqqqqqqqfqzgl9zq04nzpxyvdr8vj3h98gvnj3luanj2cxcra0q2th4xjsxmtj8k3582l67xq9ffz5586f3nm5ax58xaqjg6rjcj2vzvx2q39v9eqpn0wx54").unwrap()));
        assert_eq!(
            parsed.node_pubkey(),
            Some(
                PublicKey::from_str(
                    "031c84186f4c39920f191d4cff51ddfc1858b00c9c59f4172bcc0ca3f8da6b4953"
                )
                .unwrap()
            )
        );
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
    fn parse_lnurl_with_prefix() {
        let parsed = PaymentParams::from_str(&format!("lnurl:{SAMPLE_LNURL}")).unwrap();
        let parsed_lnurlp = PaymentParams::from_str(&format!("lnurlp:{SAMPLE_LNURL}")).unwrap();
        let parsed_lightning =
            PaymentParams::from_str(&format!("lightning:{SAMPLE_LNURL}")).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str(SAMPLE_LNURL).unwrap()));
        assert_eq!(parsed.lnurl(), parsed_lnurlp.lnurl());
        assert_eq!(parsed.lnurl(), parsed_lightning.lnurl());
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

    #[test]
    fn parse_lightning_address_with_prefix() {
        let parsed = PaymentParams::from_str("lightning:ben@opreturnbot.com").unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap()));
    }
}
