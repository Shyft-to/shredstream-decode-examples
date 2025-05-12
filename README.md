# Decode shredstreams

```
git clone https://github.com/Shyft-to/shredstream-decode-examples.git --recurse-submodules

cargo run -- --shredstream-uri <url> --x-token <authtoken>
```

| `x-token` _optional_

![screenshot-1](assets/usage-screenshot-1.png?raw=true "Screenshot")

## View Transactions
To view transactions remove comments from this part, save the file and rerun again.
```rust
                // entries.iter().for_each(|e| {
                //     e.transactions.iter().for_each(|t| {
                //         println!("Transaction: {:?}\n", t);
                //     });
                // });
```
### Preview:
![screenshot-2](assets/usage-screenshot-2.png?raw=true "Stream transactions from shredstream")

## Notes

Jito Shredstream Proxy: [https://github.com/jito-labs/shredstream-proxy]