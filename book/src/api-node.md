# API Node

Solana supports a fullnode type called an *api node*. This node type is intended
for applications that need to observe the data plane, without participating in
transaction validation or ledger replication.

An api node runs without a vote signer, and can optionally stream ledger entries
out to a Unix domain socket as they are processed. The JSON-RPC service still
functions as on any other node.

To run an api node, include the argument `no-signer` and (optional)
`entry-stream` socket location:

```bash
$ ./multinode-demo/fullnode-x.sh --no-signer --entry-stream <SOCKET>
```

The stream will output a series of JSON objects:
- An Entry event JSON object is sent when each ledger entry is processed, with
the following fields:

   * `dt`, the system datetime, as RFC3339-formatted string
   * `t`, the event type, always "entry"
   * `s`, the slot height, as unsigned 64-bit integer
   * `h`, the tick height, as unsigned 64-bit integer
   * `entry`, the entry, as JSON object


- A Block event JSON object is sent when a block is complete, with the
following fields:

   * `dt`, the system datetime, as RFC3339-formatted string
   * `t`, the event type, always "block"
   * `s`, the slot height, as unsigned 64-bit integer
   * `h`, the tick height, as unsigned 64-bit integer
   * `l`, the slot leader id, as base-58 encoded string
   * `id`, the block id, as base-58 encoded string