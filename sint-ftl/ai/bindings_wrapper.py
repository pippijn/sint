import json
from typing import Any, Dict, Optional, Type, TypeVar, List, cast, Tuple
from pydantic import BaseModel
import sint_core  # type: ignore
import sint_solver # type: ignore
from game_types import GameState, Action, GameAction

T = TypeVar("T", bound=BaseModel)

class SintBindings:
    """
    Abstraction layer for sint_core Rust bindings to handle data munging,
    Pydantic V2 interop, and type conversions.
    """

    @staticmethod
    def _string_keys_to_int(d: Any) -> Any:
        """
        Recursively converts string dictionary keys to integers where they represent
        SmallMap keys (RoomId, etc.) for Rust interop.
        """
        if isinstance(d, dict):
            new_dict = {}
            for k, v in d.items():
                if isinstance(k, str) and k.isdigit():
                    new_key: Any = int(k)
                else:
                    new_key = k
                new_dict[new_key] = SintBindings._string_keys_to_int(v)
            return new_dict
        elif isinstance(d, list):
            return [SintBindings._string_keys_to_int(x) for x in d]
        return d

    @staticmethod
    def _to_pydantic_friendly(data: Any) -> Any:
        """
        Ensures data from Rust (which might have integer keys) is compatible with
        Pydantic models (which expect string keys for dicts).
        """
        return json.loads(json.dumps(data))

    @staticmethod
    def _to_rust_friendly(model: Any) -> Any:
        """
        Converts a Pydantic model to a dict with integer keys where appropriate
        for Rust consumption.
        """
        if hasattr(model, "model_dump"):
            data = model.model_dump(mode='json', by_alias=True)
        else:
            data = model
        return SintBindings._string_keys_to_int(data)

    @classmethod
    def new_game(cls, player_ids: List[str], seed: int) -> GameState:
        raw_state = sint_core.new_game(player_ids, seed)
        friendly_state = cls._to_pydantic_friendly(raw_state)
        return GameState.model_validate(friendly_state)

    @classmethod
    def apply_action(cls, state: GameState, player_id: str, action: Action, seed: Optional[int] = None) -> GameState:
        rust_state = cls._to_rust_friendly(state)
        rust_action = action.model_dump(mode='json')
        new_state_raw = sint_core.apply_action_with_id(rust_state, player_id, rust_action, seed)
        friendly_state = cls._to_pydantic_friendly(new_state_raw)
        return GameState.model_validate(friendly_state)

    @classmethod
    def get_valid_actions(cls, state: GameState, player_id: str) -> List[Action]:
        rust_state = cls._to_rust_friendly(state)
        raw_actions = sint_core.get_valid_actions(rust_state, player_id)
        return [Action.model_validate(cls._to_pydantic_friendly(a)) for a in cast(List[Dict[str, Any]], raw_actions)]

    @staticmethod
    def get_schema() -> str:
        return cast(str, sint_core.get_schema_json())

class SolverBindings:
    """
    Abstraction layer for sint_solver Rust bindings.
    """

    @classmethod
    def verify_solution(cls, player_ids: List[str], seed: int, rounds: List[List[Tuple[str, Any]]], session_id: Optional[str] = None) -> Dict[str, Any]:
        # Convert GameAction models to dicts if they are Pydantic objects,
        # but keep them as is if they are strings or already dicts.
        rust_rounds = []
        for r in rounds:
            round_actions = []
            for pid, act in r:
                if hasattr(act, "model_dump"):
                    round_actions.append((pid, act.model_dump(mode='json', by_alias=True)))
                else:
                    round_actions.append((pid, act))
            rust_rounds.append(round_actions)
            
        result = sint_solver.verify_solution(player_ids, seed, rust_rounds, session_id)
        return cast(Dict[str, Any], result)

    @classmethod
    def get_trajectory_log(cls, initial_state: GameState, history: List[Tuple[str, Any]]) -> List[str]:
        rust_state = SintBindings._to_rust_friendly(initial_state)
        rust_history = []
        for pid, act in history:
            if hasattr(act, "model_dump"):
                rust_history.append((pid, act.model_dump(mode='json', by_alias=True)))
            else:
                rust_history.append((pid, act))
        return cast(List[str], sint_solver.get_trajectory_log(rust_state, rust_history))
