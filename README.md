# bitcoin-waila

"What am I looking at?" A tool for decoding bitcoin-related strings.

## Installing

`bitcoin-waila` is available as a Rust crate and as an npm package.

```
cargo add bitcoin-waila
```

```
npm i @mutinywallet/waila-wasm
```

---

## What is this?

This is a tool for decoding bitcoin-related strings.
The goal is to be able to give it any string, and it will decode it for you while giving you all the relevant payment
information.

Currently supported:

- Bitcoin address
- BIP-21 URI
- Lightning invoice
- Lightning Offer
- Bolt 12 Refund
- Node Pubkey
- LNURL
- Lightning Address
- Nostr Pubkey
- RGB invoice

## Examples

Bitcoin Address:

```rust
let string = "1andreas3batLhQa2FawWjeyjCqyBzypd";

let decoded = bitcoin_waila::PaymentParams::from_str(string).unwrap();

assert_eq!(decoded.address, Some(Address::from_str("1andreas3batLhQa2FawWjeyjCqyBzypd").unwrap()));
assert_eq!(parsed.network(), Some(Network::Bitcoin));
```

BIP 21:

```rust
let string = "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";

let decoded = bitcoin_waila::PaymentParams::from_str(string).unwrap();

assert_eq!(parsed.amount(), Some(Amount::from_btc(0.00001).unwrap()));
assert_eq!(parsed.address(), Some(Address::from_str("BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U").unwrap()));
assert_eq!(parsed.memo(), Some("For lunch Tuesday".to_string()));
assert_eq!(parsed.network(), Some(Network::Bitcoin));
assert_eq!(parsed.invoice(), Some(Invoice::from_str("LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6").unwrap()));
assert_eq!(parsed.node_pubkey(), Some(PublicKey::from_str("037cc5f9f1da20ac0d60e83989729a204a33cc2d8e80438969fadf35c1c5f1233b").unwrap()));
```

Lightning Address:

```rust
let parsed = bitcoin_waila::PaymentParams::from_str("ben@opreturnbot.com").unwrap();
assert_eq!(parsed.lnurl(), Some(LnUrl::from_str("lnurl1dp68gurn8ghj7mmswfjhgatjde3x7apwvdhk6tewwajkcmpdddhx7amw9akxuatjd3cz7cn9dc94s6d4").unwrap()));
```

Bolt 12:

```rust
let parsed = bitcoin_waila::PaymentParams::from_str("lno1qgs0v8hw8d368q9yw7sx8tejk2aujlyll8cp7tzzyh5h8xyppqqqqqqgqvqcdgq2qenxzatrv46pvggrv64u366d5c0rr2xjc3fq6vw2hh6ce3f9p7z4v4ee0u7avfynjw9q").unwrap();
assert_eq!(parsed.amount_msats(), Some(100_000));
assert_eq!(parsed.offer(), Some(Offer::from_str("lno1qgs0v8hw8d368q9yw7sx8tejk2aujlyll8cp7tzzyh5h8xyppqqqqqqgqvqcdgq2qenxzatrv46pvggrv64u366d5c0rr2xjc3fq6vw2hh6ce3f9p7z4v4ee0u7avfynjw9q").unwrap()));
```

RGB Invoice:

```rust
let parsed = bitcoin_waila::PaymentParams::from_str("rgb:Cbw1h3zbHgRhA6sxb4FS3Z7GTpdj9MLb7Do88qh5TUH1/RGB20/1+utxob0KPoUVTWL3WqyY6zsJY5giaugWHt5n4hEeWMQymQJmPRFPXL2n").unwrap();
assert!(matches!(parsed, PaymentParams::Rgb(_)));
```

JavaScript:

```js
// You need to initialize the wasm
// There's also an initSync() if you don't like async
const waila = await init();

const string =
  "bitcoin:BC1QYLH3U67J673H6Y6ALV70M0PL2YZ53TZHVXGG7U?amount=0.00001&label=sbddesign%3A%20For%20lunch%20Tuesday&message=For%20lunch%20Tuesday&lightning=LNBC10U1P3PJ257PP5YZTKWJCZ5FTL5LAXKAV23ZMZEKAW37ZK6KMV80PK4XAEV5QHTZ7QDPDWD3XGER9WD5KWM36YPRX7U3QD36KUCMGYP282ETNV3SHJCQZPGXQYZ5VQSP5USYC4LK9CHSFP53KVCNVQ456GANH60D89REYKDNGSMTJ6YW3NHVQ9QYYSSQJCEWM5CJWZ4A6RFJX77C490YCED6PEMK0UPKXHY89CMM7SCT66K8GNEANWYKZGDRWRFJE69H9U5U0W57RRCSYSAS7GADWMZXC8C6T0SPJAZUP6";

const params = new PaymentParams(string);

console.log(params.address);
console.log(params.invoice);
console.log(params.memo);
```
