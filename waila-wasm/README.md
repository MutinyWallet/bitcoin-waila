# bitcoin-waila

"What am I looking at?" A tool for decoding bitcoin-related strings.

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
- Node Pubkey
- LNURL
- Lightning Address
- Payjoin URI
- RGB Invoice


## Examples

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
