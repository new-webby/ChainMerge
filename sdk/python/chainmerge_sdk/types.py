from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Literal, Mapping

Chain = Literal[
    "solana",
    "ethereum",
    "cosmos",
    "aptos",
    "sui",
    "polkadot",
    "bitcoin",
    "starknet",
]

SUPPORTED_CHAINS: tuple[str, ...] = (
    "solana",
    "ethereum",
    "cosmos",
    "aptos",
    "sui",
    "polkadot",
    "bitcoin",
    "starknet",
)

EventType = Literal["token_transfer", "unsupported"]

ActionType = Literal[
    "transfer",
    "swap",
    "nft_transfer",
    "stake",
    "bridge",
    "unknown",
]


@dataclass(frozen=True)
class HealthResponse:
    status: str
    service: str

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> "HealthResponse":
        return cls(
            status=str(value.get("status", "")),
            service=str(value.get("service", "")),
        )


@dataclass(frozen=True)
class ExampleTx:
    chain: str
    tx_hash: str
    note: str

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> "ExampleTx":
        return cls(
            chain=str(value.get("chain", "")),
            tx_hash=str(value.get("tx_hash", "")),
            note=str(value.get("note", "")),
        )


def _optional_string(value: Any) -> str | None:
    if value is None:
        return None
    if isinstance(value, str):
        return value
    return str(value)


@dataclass(frozen=True)
class NormalizedEvent:
    event_type: str
    token: str | None = None
    from_address: str | None = None
    to_address: str | None = None
    amount: str | None = None
    raw_program: str | None = None

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> "NormalizedEvent":
        return cls(
            event_type=str(value.get("event_type", "unsupported")),
            token=_optional_string(value.get("token")),
            from_address=_optional_string(value.get("from")),
            to_address=_optional_string(value.get("to")),
            amount=_optional_string(value.get("amount")),
            raw_program=_optional_string(value.get("raw_program")),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "event_type": self.event_type,
            "token": self.token,
            "from": self.from_address,
            "to": self.to_address,
            "amount": self.amount,
            "raw_program": self.raw_program,
        }


@dataclass(frozen=True)
class Action:
    action_type: str
    from_address: str | None = None
    to_address: str | None = None
    amount: str | None = None
    token: str | None = None
    metadata: Any = None

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> "Action":
        return cls(
            action_type=str(value.get("action_type", "unknown")),
            from_address=_optional_string(value.get("from")),
            to_address=_optional_string(value.get("to")),
            amount=_optional_string(value.get("amount")),
            token=_optional_string(value.get("token")),
            metadata=value.get("metadata"),
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "action_type": self.action_type,
            "from": self.from_address,
            "to": self.to_address,
            "amount": self.amount,
            "token": self.token,
            "metadata": self.metadata,
        }


@dataclass(frozen=True)
class NormalizedTransaction:
    chain: str
    tx_hash: str
    sender: str | None
    receiver: str | None
    value: str | None
    events: list[NormalizedEvent]
    actions: list[Action]

    @classmethod
    def from_dict(cls, value: Mapping[str, Any]) -> "NormalizedTransaction":
        events: list[NormalizedEvent] = []
        raw_events = value.get("events")
        if isinstance(raw_events, list):
            for item in raw_events:
                if isinstance(item, Mapping):
                    events.append(NormalizedEvent.from_dict(item))

        actions: list[Action] = []
        raw_actions = value.get("actions")
        if isinstance(raw_actions, list):
            for item in raw_actions:
                if isinstance(item, Mapping):
                    actions.append(Action.from_dict(item))

        return cls(
            chain=str(value.get("chain", "")),
            tx_hash=str(value.get("tx_hash", "")),
            sender=_optional_string(value.get("sender")),
            receiver=_optional_string(value.get("receiver")),
            value=_optional_string(value.get("value")),
            events=events,
            actions=actions,
        )

    def to_dict(self) -> dict[str, Any]:
        return {
            "chain": self.chain,
            "tx_hash": self.tx_hash,
            "sender": self.sender,
            "receiver": self.receiver,
            "value": self.value,
            "events": [event.to_dict() for event in self.events],
            "actions": [action.to_dict() for action in self.actions],
        }
