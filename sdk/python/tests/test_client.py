from __future__ import annotations

import json
import unittest

from chainmerge_sdk import (
    ChainMergeAPIError,
    ChainMergeClient,
    ChainMergeTransportError,
)


class ChainMergeClientTests(unittest.TestCase):
    def test_decode_tx_success(self) -> None:
        captured: dict[str, object] = {}

        def transport(url: str, headers: dict[str, str], timeout: float) -> tuple[int, str]:
            captured["url"] = url
            captured["headers"] = headers
            captured["timeout"] = timeout
            return (
                200,
                json.dumps(
                    {
                        "decoded": {
                            "chain": "polkadot",
                            "tx_hash": "0xhash",
                            "sender": "alice",
                            "receiver": "bob",
                            "value": "1.23",
                            "events": [
                                {
                                    "event_type": "token_transfer",
                                    "token": "DOT",
                                    "from": "alice",
                                    "to": "bob",
                                    "amount": "1.23",
                                    "raw_program": "substrate_balances",
                                }
                            ],
                            "actions": [
                                {
                                    "action_type": "transfer",
                                    "from": "alice",
                                    "to": "bob",
                                    "amount": "1.23",
                                    "token": "DOT",
                                }
                            ],
                        }
                    }
                ),
            )

        client = ChainMergeClient(
            base_url="http://127.0.0.1:8080/",
            api_key="secret",
            timeout=9.0,
            transport=transport,
        )

        tx = client.decode_tx(
            chain="polkadot",
            tx_hash=" 0xhash ",
            rpc_url="https://polkadot.api.subscan.io",
        )

        self.assertEqual(tx.chain, "polkadot")
        self.assertEqual(tx.tx_hash, "0xhash")
        self.assertEqual(tx.events[0].event_type, "token_transfer")
        self.assertEqual(tx.events[0].from_address, "alice")
        self.assertEqual(tx.events[0].to_address, "bob")
        self.assertEqual(tx.actions[0].action_type, "transfer")

        url = str(captured["url"])
        self.assertIn("/api/decode?", url)
        self.assertIn("chain=polkadot", url)
        self.assertIn("hash=0xhash", url)
        self.assertIn("rpc_url=https%3A%2F%2Fpolkadot.api.subscan.io", url)
        self.assertEqual(captured["headers"], {"accept": "application/json", "x-api-key": "secret"})
        self.assertEqual(captured["timeout"], 9.0)

    def test_decode_tx_raises_api_error(self) -> None:
        def transport(url: str, headers: dict[str, str], timeout: float) -> tuple[int, str]:
            return (
                422,
                json.dumps(
                    {
                        "error": {
                            "code": "invalid_transaction_hash",
                            "message": "invalid hash",
                            "retryable": False,
                        }
                    }
                ),
            )

        client = ChainMergeClient(base_url="http://127.0.0.1:8080", transport=transport)

        with self.assertRaises(ChainMergeAPIError) as raised:
            client.decode_tx(chain="ethereum", tx_hash="bad")

        error = raised.exception
        self.assertEqual(error.status_code, 422)
        self.assertEqual(error.code, "invalid_transaction_hash")
        self.assertEqual(error.retryable, False)
        self.assertEqual(str(error), "invalid hash")

    def test_decode_tx_raises_transport_error_on_invalid_json(self) -> None:
        def transport(url: str, headers: dict[str, str], timeout: float) -> tuple[int, str]:
            return 200, "not-json"

        client = ChainMergeClient(base_url="http://127.0.0.1:8080", transport=transport)

        with self.assertRaises(ChainMergeTransportError):
            client.decode_tx(chain="ethereum", tx_hash="0xabc")

    def test_decode_tx_rejects_unsupported_chain(self) -> None:
        client = ChainMergeClient(base_url="http://127.0.0.1:8080", transport=lambda *_: (200, "{}"))

        with self.assertRaises(ValueError):
            client.decode_tx(chain="dogecoin", tx_hash="abc")

    def test_decode_tx_alias_decodeTx(self) -> None:
        def transport(url: str, headers: dict[str, str], timeout: float) -> tuple[int, str]:
            return (
                200,
                json.dumps(
                    {
                        "decoded": {
                            "chain": "ethereum",
                            "tx_hash": "0xtest",
                            "events": [],
                            "actions": [],
                        }
                    }
                ),
            )

        client = ChainMergeClient(base_url="http://127.0.0.1:8080", transport=transport)
        tx = client.decodeTx(chain="ethereum", hash="0xtest")
        self.assertEqual(tx.tx_hash, "0xtest")


if __name__ == "__main__":
    unittest.main()
