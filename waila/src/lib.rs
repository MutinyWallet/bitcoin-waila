use bech32::Variant;
use fedimint_mint_client::OOBNotes;
use std::convert::{TryFrom, TryInto};
use std::str::FromStr;

use bitcoin::blockdata::constants::ChainHash;
use bitcoin::key::XOnlyPublicKey;
use bitcoin::secp256k1::PublicKey;
use bitcoin::{Address, Amount, Network};
use lightning::offers::offer;
use lightning::offers::offer::Offer;
use lightning::offers::refund::Refund;
use lightning_invoice::{Bolt11Invoice, Bolt11InvoiceDescription};
use lnurl::lightning_address::LightningAddress;
use lnurl::lnurl::LnUrl;
use moksha_core::model::TokenV3;
use nostr::FromBech32;

#[cfg(feature = "rgb")]
use rgbstd::Chain;
#[cfg(feature = "rgb")]
use rgbwallet::RgbInvoice;
use url::Url;

use crate::bip21::UnifiedUri;
use crate::nwa::NIP49URI;

mod bip21;
mod nwa;

#[derive(Debug, Clone)]
pub enum PaymentParams<'a> {
    OnChain(Address),
    Bip21(Box<UnifiedUri<'a>>),
    Bolt11(Bolt11Invoice),
    Bolt12(Offer),
    Bolt12Refund(Refund),
    NodePubkey(PublicKey),
    LnUrl(LnUrl),
    LightningAddress(LightningAddress),
    Nostr(XOnlyPublicKey),
    FedimintInvite(String),
    NostrWalletAuth(NIP49URI),
    CashuToken(TokenV3),
    FedimintOOBNotes(OOBNotes),
    #[cfg(feature = "rgb")]
    Rgb(RgbInvoice),
}

#[cfg(feature = "rgb")]
fn map_chain_to_network(chain: Chain) -> Option<Network> {
    Network::from_str(&chain.to_string()).ok()
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
                Bolt11InvoiceDescription::Direct(desc) => Some(desc.to_string()),
                Bolt11InvoiceDescription::Hash(_) => None,
            },
            PaymentParams::Bolt12(offer) => Some(offer.description().to_string()),
            PaymentParams::Bolt12Refund(refund) => Some(refund.description().to_string()),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn network(&self) -> Option<Network> {
        match self {
            PaymentParams::OnChain(address) => Some(address.network),
            PaymentParams::Bip21(uri) => Some(uri.address.network),
            PaymentParams::Bolt11(invoice) => Some(Network::from(invoice.currency())),
            PaymentParams::Bolt12(o) => o.chains().first().cloned().and_then(|c| c.try_into().ok()),
            PaymentParams::Bolt12Refund(refund) => refund.chain().try_into().ok(),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(invoice) => invoice.chain.and_then(map_chain_to_network),
        }
    }

    /// Given the network, determine if the payment params are valid for that network
    /// Returns None if the network is unknown
    pub fn valid_for_network(&self, network: Network) -> Option<bool> {
        match self {
            PaymentParams::OnChain(address) => Some(address.network == network),
            PaymentParams::Bip21(uri) => Some(uri.address.is_valid_for_network(network)),
            PaymentParams::Bolt11(invoice) => Some(Network::from(invoice.currency()) == network),
            PaymentParams::Bolt12(offer) => {
                Some(offer.supports_chain(ChainHash::using_genesis_block(network)))
            }
            PaymentParams::Bolt12Refund(refund) => {
                Some(refund.chain() == ChainHash::using_genesis_block(network))
            }
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(invoice) => invoice
                .chain
                .and_then(map_chain_to_network)
                .map(|n| n == network),
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
            PaymentParams::Bolt12Refund(refund) => Some(refund.amount_msats()),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(token) => Some(token.total_amount() * 1000),
            PaymentParams::FedimintOOBNotes(oob_notes) => Some(oob_notes.total_amount().msats),
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn address(&self) -> Option<Address> {
        match self {
            PaymentParams::OnChain(address) => Some(address.clone()),
            PaymentParams::Bip21(uri) => Some(uri.address.clone().assume_checked()),
            PaymentParams::Bolt11(invoice) => invoice.fallback_addresses().first().cloned(),
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn invoice(&self) -> Option<Bolt11Invoice> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.extras.lightning.clone(),
            PaymentParams::Bolt11(invoice) => Some(invoice.clone()),
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn offer(&self) -> Option<Offer> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(uri) => uri.extras.b12.clone(),
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(offer) => Some(offer.clone()),
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn refund(&self) -> Option<Refund> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(refund) => Some(refund.clone()),
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
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
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(pubkey) => Some(*pubkey),
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn lnurl(&self) -> Option<LnUrl> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(lnurl) => Some(lnurl.clone()),
            PaymentParams::LightningAddress(ln_addr) => Some(LnUrl::from_url(ln_addr.lnurlp_url())),
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn is_lnurl_auth(&self) -> bool {
        self.lnurl()
            .map(|lnurl| lnurl.is_lnurl_auth())
            .unwrap_or(false)
    }

    pub fn lightning_address(&self) -> Option<LightningAddress> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(l) => l.lightning_address(),
            PaymentParams::LightningAddress(ln_addr) => Some(ln_addr.clone()),
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn nostr_pubkey(&self) -> Option<XOnlyPublicKey> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(key) => Some(*key),
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn fedimint_invite_code(&self) -> Option<String> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(i) => Some(i.clone()),
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn nostr_wallet_auth(&self) -> Option<NIP49URI> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(a) => Some(a.clone()),
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn cashu_token(&self) -> Option<TokenV3> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(a) => Some(a.clone()),
            PaymentParams::FedimintOOBNotes(_) => None,
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn fedimint_oob_notes(&self) -> Option<OOBNotes> {
        match self {
            PaymentParams::OnChain(_) => None,
            PaymentParams::Bip21(_) => None,
            PaymentParams::Bolt11(_) => None,
            PaymentParams::Bolt12(_) => None,
            PaymentParams::Bolt12Refund(_) => None,
            PaymentParams::NodePubkey(_) => None,
            PaymentParams::LnUrl(_) => None,
            PaymentParams::LightningAddress(_) => None,
            PaymentParams::Nostr(_) => None,
            PaymentParams::FedimintInvite(_) => None,
            PaymentParams::NostrWalletAuth(_) => None,
            PaymentParams::CashuToken(_) => None,
            PaymentParams::FedimintOOBNotes(a) => Some(a.clone()),
            #[cfg(feature = "rgb")]
            PaymentParams::Rgb(_) => None,
        }
    }

    pub fn payjoin_endpoint(&self) -> Option<Url> {
        if let PaymentParams::Bip21(uri) = self {
            uri.extras.pj.clone()
        } else {
            None
        }
    }

    pub fn disable_output_substitution(&self) -> Option<bool> {
        if let PaymentParams::Bip21(uri) = self {
            Some(uri.extras.disable_output_substitution())
        } else {
            None
        }
    }

    pub fn payjoin_supported(&self) -> bool {
        self.payjoin_endpoint().is_some()
    }
}

// just checks if it has correct HRP and variant
fn parse_fedi_invite_code(str: &str) -> Result<String, ()> {
    bech32::decode(str)
        .map_err(|_| ())
        .and_then(|(hrp, _, variant)| {
            if hrp == "fed1" && variant == Variant::Bech32m {
                Ok(str.to_string())
            } else {
                Err(())
            }
        })
}

impl FromStr for PaymentParams<'_> {
    type Err = ();

    fn from_str(str: &str) -> Result<Self, Self::Err> {
        let lower = str.to_lowercase();
        if lower.starts_with("lightning:") {
            let str = lower.strip_prefix("lightning:").unwrap();
            return Bolt11Invoice::from_str(str)
                .map(PaymentParams::Bolt11)
                .or_else(|_| LnUrl::from_str(str).map(PaymentParams::LnUrl))
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .or_else(|_| Offer::from_str(str).map(PaymentParams::Bolt12))
                .or_else(|_| Refund::from_str(str).map(PaymentParams::Bolt12Refund))
                .map_err(|_| ());
        } else if lower.starts_with("lnurl:") {
            let str = lower.strip_prefix("lnurl:").unwrap();
            return LnUrl::from_str(str)
                .map(PaymentParams::LnUrl)
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .map_err(|_| ());
        } else if lower.starts_with("lnurlp:") {
            let str = lower.strip_prefix("lnurlp:").unwrap();
            return LnUrl::from_str(str)
                .map(PaymentParams::LnUrl)
                .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
                .map_err(|_| ());
        } else if lower.starts_with("nostr:") {
            let str = lower.strip_prefix("nostr:").unwrap();
            return XOnlyPublicKey::from_str(str)
                .map(PaymentParams::Nostr)
                .or_else(|_| XOnlyPublicKey::from_bech32(str).map(PaymentParams::Nostr))
                .map_err(|_| ());
        } else if lower.starts_with("fedimint:") {
            let str = lower.strip_prefix("fedimint:").unwrap();
            return parse_fedi_invite_code(str).map(PaymentParams::FedimintInvite);
        }

        #[cfg(feature = "rgb")]
        if lower.starts_with("rgb:") {
            return RgbInvoice::from_str(str)
                .map(PaymentParams::Rgb)
                .map_err(|_| ());
        }

        Address::from_str(str)
            .map(|a| PaymentParams::OnChain(a.assume_checked()))
            .or_else(|_| Bolt11Invoice::from_str(str).map(PaymentParams::Bolt11))
            .or_else(|_| UnifiedUri::from_str(str).map(|u| PaymentParams::Bip21(Box::new(u))))
            .or_else(|_| LightningAddress::from_str(str).map(PaymentParams::LightningAddress))
            .or_else(|_| LnUrl::from_str(str).map(PaymentParams::LnUrl))
            .or_else(|_| PublicKey::from_str(str).map(PaymentParams::NodePubkey))
            .or_else(|_| Offer::from_str(str).map(PaymentParams::Bolt12))
            .or_else(|_| Refund::from_str(str).map(PaymentParams::Bolt12Refund))
            .or_else(|_| XOnlyPublicKey::from_str(str).map(PaymentParams::Nostr))
            .or_else(|_| XOnlyPublicKey::from_bech32(str).map(PaymentParams::Nostr))
            .or_else(|_| NIP49URI::from_str(str).map(PaymentParams::NostrWalletAuth))
            .or_else(|_| parse_fedi_invite_code(str).map(PaymentParams::FedimintInvite))
            .or_else(|_| TokenV3::try_from(str.to_string()).map(PaymentParams::CashuToken))
            .or_else(|_| OOBNotes::from_str(str).map(PaymentParams::FedimintOOBNotes))
            .map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use lightning_invoice::Bolt11Invoice;
    use std::str::FromStr;

    use super::*;

    const SAMPLE_PUBKEY: &str =
        "03e7156ae33b0a208d0744199163177e909e80176e55d97a2f221ede0f934dd9ad";
    const SAMPLE_INVOICE: &str = "lnbc20m1pvjluezsp5zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zyg3zygspp5qqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqqqsyqcyq5rqwzqfqypqhp58yjmdan79s6qqdhdzgynm4zwqd5d7xmw5fk98klysy043l2ahrqsfpp3qjmp7lwpagxun9pygexvgpjdc4jdj85fr9yq20q82gphp2nflc7jtzrcazrra7wwgzxqc8u7754cdlpfrmccae92qgzqvzq2ps8pqqqqqqpqqqqq9qqqvpeuqafqxu92d8lr6fvg0r5gv0heeeqgcrqlnm6jhphu9y00rrhy4grqszsvpcgpy9qqqqqqgqqqqq7qqzq9qrsgqdfjcdk6w3ak5pca9hwfwfh63zrrz06wwfya0ydlzpgzxkn5xagsqz7x9j4jwe7yj7vaf2k9lqsdk45kts2fd0fkr28am0u4w95tt2nsq76cqw0";
    const SAMPLE_OFFER: &str = "lno1qgs0v8hw8d368q9yw7sx8tejk2aujlyll8cp7tzzyh5h8xyppqqqqqqgqvqcdgq2qenxzatrv46pvggrv64u366d5c0rr2xjc3fq6vw2hh6ce3f9p7z4v4ee0u7avfynjw9q";
    const SAMPLE_REFUND: &str = "lnr1qqsqzqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqgpqyqszqg2qdnx7m6jqgp7skppq0n326hr8v9zprg8gsvezcch06gfaqqhde2aj730yg0durunfhv66";
    const SAMPLE_BIP21: &str = "bitcoin:1andreas3batLhQa2FawWjeyjCqyBzypd?amount=50&label=Luke-Jr&message=Donation%20for%20project%20xyz";
    const SAMPLE_BIP21_WITH_INVOICE: &str = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";
    const SAMPLE_BIP21_WITH_INVOICE_AND_LABEL: &str = "bitcoin:tb1p0vztr8q25czuka5u4ta5pqu0h8dxkf72mam89cpg4tg40fm8wgmqp3gv99?amount=0.000001&label=yooo&lightning=lntbs1u1pjrww6fdq809hk7mcnp4qvwggxr0fsueyrcer4x075walsv93vqvn3vlg9etesx287x6ddy4xpp5a3drwdx2fmkkgmuenpvmynnl7uf09jmgvtlg86ckkvgn99ajqgtssp5gr3aghgjxlwshnqwqn39c2cz5hw4cnsnzxdjn7kywl40rru4mjdq9qyysgqcqpcxqrpwurzjqfgtsj42x8an5zujpxvfhp9ngwm7u5lu8lvzfucjhex4pq8ysj5q2qqqqyqqv9cqqsqqqqlgqqqqqqqqfqzgl9zq04nzpxyvdr8vj3h98gvnj3luanj2cxcra0q2th4xjsxmtj8k3582l67xq9ffz5586f3nm5ax58xaqjg6rjcj2vzvx2q39v9eqpn0wx54";
    const SAMPLE_LNURL: &str = "LNURL1DP68GURN8GHJ7UM9WFMXJCM99E3K7MF0V9CXJ0M385EKVCENXC6R2C35XVUKXEFCV5MKVV34X5EKZD3EV56NYD3HXQURZEPEXEJXXEPNXSCRVWFNV9NXZCN9XQ6XYEFHVGCXXCMYXYMNSERXFQ5FNS";
    const SAMPLE_FEDI_INVITE_CODE: &str = "fed11jpr3lgm8tuhcky2r3g287tgk9du7dd7kr95fptdsmkca7cwcvyu0lyqeh0e6rgp4u0shxsfaxycpwqpfwaehxw309askcurgvyhx6at5d9h8jmn9wsknqvfwv3jhvtnxv4jxjcn5vvhxxmmd9udpnpn49yg9w98dejw9u76hmm9";
    const SAMPLE_NWA: &str = "nostr+walletauth://b889ff5b1513b641e2a139f661a661364979c5beee91842f8f0ef42ab558e9d4?relay=wss%3A%2F%2Frelay.damus.io&secret=b8a30fafa48d4795b6c0eec169a383de&required_commands=pay_invoice&optional_commands=get_balance&budget=10000%2Fdaily";
    const SAMPLE_CASHU_TOKEN: &str = "cashuAeyJ0b2tlbiI6W3sibWludCI6Imh0dHBzOi8vODMzMy5zcGFjZTozMzM4IiwicHJvb2ZzIjpbeyJhbW91bnQiOjIsImlkIjoiMDA5YTFmMjkzMjUzZTQxZSIsInNlY3JldCI6IjQwNzkxNWJjMjEyYmU2MWE3N2UzZTZkMmFlYjRjNzI3OTgwYmRhNTFjZDA2YTZhZmMyOWUyODYxNzY4YTc4MzciLCJDIjoiMDJiYzkwOTc5OTdkODFhZmIyY2M3MzQ2YjVlNDM0NWE5MzQ2YmQyYTUwNmViNzk1ODU5OGE3MmYwY2Y4NTE2M2VhIn0seyJhbW91bnQiOjgsImlkIjoiMDA5YTFmMjkzMjUzZTQxZSIsInNlY3JldCI6ImZlMTUxMDkzMTRlNjFkNzc1NmIwZjhlZTBmMjNhNjI0YWNhYTNmNGUwNDJmNjE0MzNjNzI4YzcwNTdiOTMxYmUiLCJDIjoiMDI5ZThlNTA1MGI4OTBhN2Q2YzA5NjhkYjE2YmMxZDVkNWZhMDQwZWExZGUyODRmNmVjNjlkNjEyOTlmNjcxMDU5In1dfV0sInVuaXQiOiJzYXQiLCJtZW1vIjoiVGhhbmsgeW91LiJ9";
    const SAMPLE_FEDIMINT_OOB_NOTES: &str = "AgEEyNQjlgD9AaMFEAGPoosRshrR37QwoMzyQtjRqIOw+zqlqJUlMP4tY8PmLkQwDzZxOIqvBRwdWLR7ZR4hCh5CH4pgBDDxJoKh9FSHFuVfaicAF4a2xc8QNYlwtv0BAAGxQ4CfvfXB6XAaMPyVlWjt7a2Z1bvh18bKx9i0NX0KmC/KAwzo7nzxe5aISrcKYw2qheA65rSoOA6oAYs1YegPWIAcKWl4YfPaROIdlv8zfP0CAAGzD8GzMknXfXv102IzMADaL/ZGs9351HPbZMkOxrdB4WeyhEy5bnOFI0YIBUHs/ESKeDVm1Yv9j19y7mDIyXDmvFIwtCXDjFqWE4i0qzrdzv0EAAGsB8LTXGGZyW7KZDE3CtMbWXTgIuBa3A/nll/foeD5VOACUraOkeRMeNIiZvTellBa9CHtIRpWXlt46hKSFWjpQRh4Jk/ga+t0WlJ//Mxihv0gAAGSm+bQkczA4F1lvg9Vh2yJmgGTtElL4U3uhW+xuP5lsxz+kPwR3qUMX0KJfOE4oN5XpwYDQVoPRroiXAcnakM9thPeMyycDMENeNSKQ1LBmA==";
    #[cfg(feature = "rgb")]
    const SAMPLE_RGB_INVOICE: &str = "rgb:Cbw1h3zbHgRhA6sxb4FS3Z7GTpdj9MLb7Do88qh5TUH1/RGB20/1+utxob0KPoUVTWL3WqyY6zsJY5giaugWHt5n4hEeWMQymQJmPRFPXL2n";

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
        let address = Address::from_str("1andreas3batLhQa2FawWjeyjCqyBzypd")
            .unwrap()
            .assume_checked();
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
            Some(
                Address::from_str("1RustyRX2oai4EYYDpQGWvEL62BBGqN9T")
                    .unwrap()
                    .assume_checked()
            )
        );
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_offer() {
        let parsed = PaymentParams::from_str(SAMPLE_OFFER).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(100)));
        assert_eq!(parsed.amount_msats(), Some(100_000));
        assert!(parsed.valid_for_network(Network::Signet).unwrap_or(false));
        assert_eq!(parsed.offer().unwrap().to_string(), SAMPLE_OFFER);
        assert_eq!(parsed.memo().as_deref(), Some("faucet"));
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_refund() {
        let parsed = PaymentParams::from_str(SAMPLE_REFUND).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(1)));
        assert_eq!(parsed.amount_msats(), Some(1_000));
        assert!(parsed.valid_for_network(Network::Bitcoin).unwrap_or(false));
        assert_eq!(parsed.refund().unwrap().to_string(), SAMPLE_REFUND);
        assert_eq!(parsed.memo().as_deref(), Some("foo"));
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
            Some(
                Address::from_str("1RustyRX2oai4EYYDpQGWvEL62BBGqN9T")
                    .unwrap()
                    .assume_checked()
            )
        );
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_invoice_with_prefix_capital() {
        let parsed =
            PaymentParams::from_str(&format!("LIGHTNING:{}", SAMPLE_INVOICE.to_uppercase()))
                .unwrap();
        let expected_pubkey = PublicKey::from_str(SAMPLE_PUBKEY).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_sat(2_000_000)));
        assert_eq!(parsed.amount_msats(), Some(2_000_000_000));
        assert_eq!(parsed.node_pubkey(), Some(expected_pubkey));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(
            parsed.address(),
            Some(
                Address::from_str("1RustyRX2oai4EYYDpQGWvEL62BBGqN9T")
                    .unwrap()
                    .assume_checked()
            )
        );
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.lnurl(), None);
    }

    #[test]
    fn parse_bip_21() {
        let parsed = PaymentParams::from_str(SAMPLE_BIP21).unwrap();

        assert_eq!(parsed.amount(), Some(Amount::from_btc(50_f64).unwrap()));
        assert_eq!(
            parsed.address(),
            Some(
                Address::from_str("1andreas3batLhQa2FawWjeyjCqyBzypd")
                    .unwrap()
                    .assume_checked()
            )
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
            Some(
                Address::from_str("BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U")
                    .unwrap()
                    .assume_checked()
            )
        );
        assert_eq!(parsed.memo(), Some("For lunch Tuesday".to_string()));
        assert_eq!(parsed.network(), Some(Network::Bitcoin));
        assert_eq!(parsed.invoice(), Some(Bolt11Invoice::from_str("LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6").unwrap()));
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
                    .assume_checked()
            )
        );
        assert_eq!(parsed.memo(), Some("yooo".to_string()));
        assert_eq!(parsed.network(), Some(Network::Testnet));
        assert_eq!(parsed.invoice(), Some(Bolt11Invoice::from_str("lntbs1u1pjrww6fdq809hk7mcnp4qvwggxr0fsueyrcer4x075walsv93vqvn3vlg9etesx287x6ddy4xpp5a3drwdx2fmkkgmuenpvmynnl7uf09jmgvtlg86ckkvgn99ajqgtssp5gr3aghgjxlwshnqwqn39c2cz5hw4cnsnzxdjn7kywl40rru4mjdq9qyysgqcqpcxqrpwurzjqfgtsj42x8an5zujpxvfhp9ngwm7u5lu8lvzfucjhex4pq8ysj5q2qqqqyqqv9cqqsqqqqlgqqqqqqqqfqzgl9zq04nzpxyvdr8vj3h98gvnj3luanj2cxcra0q2th4xjsxmtj8k3582l67xq9ffz5586f3nm5ax58xaqjg6rjcj2vzvx2q39v9eqpn0wx54").unwrap()));
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
        let str = "ben@opreturnbot.com";
        let parsed = PaymentParams::from_str(str).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.lightning_address(),
            Some(LightningAddress::from_str(str).unwrap())
        );
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap()));
    }

    #[test]
    fn parse_lightning_address_with_prefix() {
        let str = "ben@opreturnbot.com";
        let parsed = PaymentParams::from_str(&format!("lightning:{str}")).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.lightning_address(),
            Some(LightningAddress::from_str(str).unwrap())
        );
        assert_eq!(parsed.lnurl(), Some(LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap()));
    }

    #[test]
    fn parse_nostr_key() {
        let parsed = PaymentParams::from_str(
            "npub1u8lnhlw5usp3t9vmpz60ejpyt649z33hu82wc2hpv6m5xdqmuxhs46turz",
        )
        .unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.nostr_pubkey(),
            Some(
                XOnlyPublicKey::from_str(
                    "e1ff3bfdd4e40315959b08b4fcc8245eaa514637e1d4ec2ae166b743341be1af"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn parse_nostr_key_with_prefix() {
        let parsed = PaymentParams::from_str(
            "nostr:npub1u8lnhlw5usp3t9vmpz60ejpyt649z33hu82wc2hpv6m5xdqmuxhs46turz",
        )
        .unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.nostr_pubkey(),
            Some(
                XOnlyPublicKey::from_str(
                    "e1ff3bfdd4e40315959b08b4fcc8245eaa514637e1d4ec2ae166b743341be1af"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn parse_fedimint_invite_code() {
        let parsed = PaymentParams::from_str(SAMPLE_FEDI_INVITE_CODE).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.fedimint_invite_code(),
            Some(SAMPLE_FEDI_INVITE_CODE.to_string())
        );
    }

    #[test]
    fn parse_fedimint_invite_code_with_prefix() {
        let str = format!("fedimint:{SAMPLE_FEDI_INVITE_CODE}");
        let parsed = PaymentParams::from_str(&str).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.fedimint_invite_code(),
            Some(SAMPLE_FEDI_INVITE_CODE.to_string())
        );
    }

    #[test]
    fn parse_cashu_token() {
        let parsed = PaymentParams::from_str(SAMPLE_CASHU_TOKEN).unwrap();

        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.amount(), Some(Amount::from_sat(10)));
        assert_eq!(
            parsed.cashu_token(),
            Some(TokenV3::try_from(SAMPLE_CASHU_TOKEN.to_string()).unwrap())
        )
    }

    #[test]
    fn parse_fedimint_oob_notes() {
        let parsed = PaymentParams::from_str(SAMPLE_FEDIMINT_OOB_NOTES).unwrap();

        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.amount(), Some(Amount::from_sat(10)));
        // NOTE: (@leonardo) there is not `Eq` implementation for `fedimint-mint-client::OOBNotes`
        assert_eq!(
            parsed.fedimint_oob_notes().unwrap().to_string(),
            OOBNotes::from_str(SAMPLE_FEDIMINT_OOB_NOTES)
                .unwrap()
                .to_string()
        )
    }

    #[test]
    fn parse_nwa() {
        let parsed = PaymentParams::from_str(SAMPLE_NWA).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(
            parsed.nostr_wallet_auth(),
            Some(NIP49URI::from_str(SAMPLE_NWA).unwrap())
        );
    }

    #[cfg(feature = "rgb")]
    #[test]
    fn parse_rgb_invoice() {
        let parsed = PaymentParams::from_str(SAMPLE_RGB_INVOICE).unwrap();

        assert_eq!(parsed.amount(), None);
        assert_eq!(parsed.address(), None);
        assert_eq!(parsed.memo(), None);
        assert_eq!(parsed.network(), None);
        assert_eq!(parsed.invoice(), None);
        assert_eq!(parsed.node_pubkey(), None);
        assert_eq!(parsed.nostr_pubkey(), None);
        assert!(matches!(parsed, PaymentParams::Rgb(_)));
    }
}
