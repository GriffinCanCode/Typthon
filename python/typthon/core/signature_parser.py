"""Signature parser for runtime type validation."""

import re
from typing import List, Dict, Any, Optional, Union
from dataclasses import dataclass


@dataclass
class TypeSignature:
    """Parsed type signature."""
    params: List['ParsedType']
    return_type: 'ParsedType'
    effects: List[str]

    def __repr__(self) -> str:
        params_str = ", ".join(str(p) for p in self.params)
        effects_str = f" ! {{{', '.join(self.effects)}}}" if self.effects else ""
        return f"({params_str}) -> {self.return_type}{effects_str}"


@dataclass
class ParsedType:
    """Parsed type information."""
    base_type: str
    args: List['ParsedType']
    is_refinement: bool = False
    predicate: Optional[str] = None
    is_effect: bool = False
    effects: List[str] = None

    def __post_init__(self):
        if self.effects is None:
            self.effects = []

    def __repr__(self) -> str:
        if self.is_refinement and self.predicate:
            return f"{self.base_type}[{self.predicate}]"
        if self.is_effect and self.effects:
            return f"{self.base_type} ! {{{', '.join(self.effects)}}}"
        if self.args:
            args_str = ", ".join(str(arg) for arg in self.args)
            return f"{self.base_type}[{args_str}]"
        return self.base_type


class SignatureParser:
    """Parser for function type signatures."""

    # Built-in types
    BUILTIN_TYPES = {
        'int', 'float', 'str', 'bool', 'bytes', 'None',
        'list', 'tuple', 'dict', 'set', 'frozenset',
        'Any', 'Union', 'Optional'
    }

    # Effect types
    EFFECT_TYPES = {
        'pure', 'io', 'network', 'mutation', 'exception',
        'async', 'random', 'time'
    }

    def parse(self, signature: str) -> TypeSignature:
        """
        Parse a function signature string.

        Formats:
            - "(int, int) -> int"
            - "(str, bool) -> list[int]"
            - "(int) -> int ! {io, exception}"
            - "() -> str"
        """
        signature = signature.strip()

        # Extract effects if present
        effects = []
        if '!' in signature:
            sig_part, effect_part = signature.rsplit('!', 1)
            effects = self._parse_effects(effect_part)
            signature = sig_part.strip()

        # Split into params and return type
        if '->' not in signature:
            raise ValueError(f"Invalid signature format: missing '->': {signature}")

        params_str, return_str = signature.split('->', 1)
        params_str = params_str.strip()
        return_str = return_str.strip()

        # Parse parameters
        if not params_str.startswith('(') or not params_str.endswith(')'):
            raise ValueError(f"Invalid parameter format: {params_str}")

        params_content = params_str[1:-1].strip()
        params = self._parse_params(params_content) if params_content else []

        # Parse return type
        return_type = self._parse_type(return_str)

        return TypeSignature(params=params, return_type=return_type, effects=effects)

    def _parse_params(self, params_str: str) -> List[ParsedType]:
        """Parse comma-separated parameter types."""
        params = []
        current = ""
        depth = 0

        for char in params_str:
            if char in '[({':
                depth += 1
                current += char
            elif char in '])}':
                depth -= 1
                current += char
            elif char == ',' and depth == 0:
                if current.strip():
                    params.append(self._parse_type(current.strip()))
                current = ""
            else:
                current += char

        if current.strip():
            params.append(self._parse_type(current.strip()))

        return params

    def _parse_type(self, type_str: str) -> ParsedType:
        """Parse a single type annotation."""
        type_str = type_str.strip()

        # Check for effect type: "int ! {io}"
        if '!' in type_str:
            base_str, effect_str = type_str.split('!', 1)
            base_type = self._parse_type(base_str.strip())
            effects = self._parse_effects(effect_str)
            base_type.is_effect = True
            base_type.effects = effects
            return base_type

        # Check for refinement type: "int[value > 0]"
        refinement_match = re.match(r'(\w+)\[(.*)\]$', type_str)
        if refinement_match and refinement_match.group(1) in self.BUILTIN_TYPES:
            base = refinement_match.group(1)
            predicate = refinement_match.group(2)
            # Check if it's actually a generic type vs refinement
            if self._is_refinement_predicate(predicate):
                return ParsedType(
                    base_type=base,
                    args=[],
                    is_refinement=True,
                    predicate=predicate
                )

        # Check for generic type: "list[int]", "dict[str, int]"
        generic_match = re.match(r'(\w+)\[(.*)\]$', type_str)
        if generic_match:
            base = generic_match.group(1)
            args_str = generic_match.group(2)
            args = self._parse_params(args_str)
            return ParsedType(base_type=base, args=args)

        # Check for union type: "int | str"
        if '|' in type_str:
            types = [self._parse_type(t.strip()) for t in type_str.split('|')]
            return ParsedType(base_type='Union', args=types)

        # Simple type
        return ParsedType(base_type=type_str, args=[])

    def _parse_effects(self, effect_str: str) -> List[str]:
        """Parse effect set: "{io, network}" -> ['io', 'network']"""
        effect_str = effect_str.strip()
        if effect_str.startswith('{') and effect_str.endswith('}'):
            effect_str = effect_str[1:-1]

        effects = [e.strip() for e in effect_str.split(',') if e.strip()]
        return effects

    def _is_refinement_predicate(self, predicate: str) -> bool:
        """Check if string looks like a refinement predicate."""
        # Heuristic: contains comparison operators or 'value' keyword
        predicate_indicators = ['>', '<', '==', '!=', '>=', '<=', 'value', 'len(']
        return any(indicator in predicate for indicator in predicate_indicators)


# Singleton parser instance
_parser = SignatureParser()


def parse_signature(sig: str) -> TypeSignature:
    """Parse a function signature string."""
    return _parser.parse(sig)

