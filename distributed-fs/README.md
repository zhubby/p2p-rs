```
cargo run -- \
          --listen-address /ip4/127.0.0.1/tcp/40837 \
          --secret-key-seed 1 \
          provide \
          --path /var/tmp/sharing_file.txt \
          --name sharing_file
```

```
cargo run -- \
          --peer /ip4/127.0.0.1/tcp/40837/p2p/12D3KooWPjceQrSwdWXPyLLeABRXmuqt69Rg3sBYbU1Nft9HyQ6X \
          get \
          --name sharing_file
```