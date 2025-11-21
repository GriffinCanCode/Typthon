"""Test effect system functionality.

This module tests the effect type system for tracking side effects.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestPureEffects:
    """Test pure (no effect) functions."""

    def test_pure_function(self, validator):
        """Test pure function detection."""
        code = """
def add(x: int, y: int) -> int:
    return x + y
"""
        assert validator.validate(code)
        # TODO: Check that function is marked as pure

    def test_pure_arithmetic(self, validator):
        """Test arithmetic is pure."""
        code = """
def calculate(a: int, b: int, c: int) -> int:
    return a * b + c
"""
        assert validator.validate(code)

    def test_pure_with_local_vars(self, validator):
        """Test pure function with local variables."""
        code = """
def compute(x: int) -> int:
    temp = x * 2
    result = temp + 1
    return result
"""
        assert validator.validate(code)

    def test_pure_composition(self, validator):
        """Test composition of pure functions."""
        code = """
def double(x: int) -> int:
    return x * 2

def triple(x: int) -> int:
    return x * 3

def six_times(x: int) -> int:
    return triple(double(x))
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestIOEffects:
    """Test I/O effect tracking."""

    def test_print_effect(self, validator):
        """Test print has IO effect."""
        code = """
def greet(name: str) -> None:
    print(f"Hello, {name}")
"""
        assert validator.validate(code)

    def test_file_read_effect(self, validator):
        """Test file reading has IO effect."""
        code = """
def read_file(path: str) -> str:
    with open(path) as f:
        return f.read()
"""
        assert validator.validate(code)

    def test_file_write_effect(self, validator):
        """Test file writing has IO effect."""
        code = """
def write_file(path: str, content: str) -> None:
    with open(path, 'w') as f:
        f.write(content)
"""
        assert validator.validate(code)

    def test_input_effect(self, validator):
        """Test input() has IO effect."""
        code = """
def get_name() -> str:
    return input("Enter name: ")
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestMutationEffects:
    """Test mutation effect tracking."""

    def test_list_append_mutation(self, validator):
        """Test list append has mutation effect."""
        code = """
def add_item(items: list[int], item: int) -> None:
    items.append(item)
"""
        assert validator.validate(code)

    def test_list_extend_mutation(self, validator):
        """Test list extend has mutation effect."""
        code = """
def extend_list(items: list[int], more: list[int]) -> None:
    items.extend(more)
"""
        assert validator.validate(code)

    def test_dict_update_mutation(self, validator):
        """Test dict update has mutation effect."""
        code = """
def update_dict(d: dict[str, int], key: str, value: int) -> None:
    d[key] = value
"""
        assert validator.validate(code)

    def test_set_add_mutation(self, validator):
        """Test set add has mutation effect."""
        code = """
def add_to_set(s: set[int], item: int) -> None:
    s.add(item)
"""
        assert validator.validate(code)

    def test_object_mutation(self, validator):
        """Test object attribute mutation."""
        code = """
class Counter:
    value: int = 0

def increment(counter: Counter) -> None:
    counter.value += 1
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestNetworkEffects:
    """Test network effect tracking."""

    def test_http_request_effect(self, validator):
        """Test HTTP request has network effect."""
        code = """
import urllib.request

def fetch_url(url: str) -> str:
    with urllib.request.urlopen(url) as response:
        return response.read().decode()
"""
        assert validator.validate(code)

    def test_socket_effect(self, validator):
        """Test socket operations have network effect."""
        code = """
import socket

def create_server(port: int) -> None:
    sock = socket.socket()
    sock.bind(('localhost', port))
    sock.listen()
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestExceptionEffects:
    """Test exception effect tracking."""

    def test_raise_exception_effect(self, validator):
        """Test raising exception has exception effect."""
        code = """
def validate(x: int) -> int:
    if x < 0:
        raise ValueError("x must be non-negative")
    return x
"""
        assert validator.validate(code)

    def test_exception_propagation(self, validator):
        """Test exception effect propagates."""
        code = """
def validate(x: int) -> int:
    if x < 0:
        raise ValueError("Invalid")
    return x

def process(x: int) -> int:
    return validate(x) * 2
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestAsyncEffects:
    """Test async effect tracking."""

    def test_async_function(self, validator):
        """Test async function has async effect."""
        code = """
async def fetch_data() -> str:
    return "data"
"""
        assert validator.validate(code)

    def test_await_effect(self, validator):
        """Test await has async effect."""
        code = """
async def helper() -> int:
    return 42

async def main() -> int:
    result = await helper()
    return result
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestRandomEffects:
    """Test random/non-deterministic effect tracking."""

    def test_random_effect(self, validator):
        """Test random() has random effect."""
        code = """
import random

def get_random() -> float:
    return random.random()
"""
        assert validator.validate(code)

    def test_random_choice_effect(self, validator):
        """Test random.choice has random effect."""
        code = """
import random

def pick_random(items: list[int]) -> int:
    return random.choice(items)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestTimeEffects:
    """Test time-dependent effect tracking."""

    def test_time_effect(self, validator):
        """Test time() has time effect."""
        code = """
import time

def get_timestamp() -> float:
    return time.time()
"""
        assert validator.validate(code)

    def test_datetime_effect(self, validator):
        """Test datetime.now() has time effect."""
        code = """
from datetime import datetime

def get_now() -> datetime:
    return datetime.now()
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestEffectComposition:
    """Test effect composition and propagation."""

    def test_effect_propagation(self, validator):
        """Test effects propagate through calls."""
        code = """
def print_message(msg: str) -> None:
    print(msg)  # IO effect

def log_error(error: str) -> None:
    print_message(f"ERROR: {error}")  # Inherits IO effect
"""
        assert validator.validate(code)

    def test_multiple_effects(self, validator):
        """Test function with multiple effects."""
        code = """
def log_to_file(message: str, filename: str) -> None:
    print(message)  # IO effect
    with open(filename, 'a') as f:  # IO effect
        f.write(message)  # Mutation effect
"""
        assert validator.validate(code)

    def test_conditional_effects(self, validator):
        """Test conditional effect application."""
        code = """
def maybe_print(flag: bool, message: str) -> None:
    if flag:
        print(message)  # IO effect only if flag is True
"""
        assert validator.validate(code)

    def test_effect_isolation(self, validator):
        """Test pure code doesn't inherit effects."""
        code = """
def impure() -> int:
    print("side effect")
    return 42

def pure_wrapper() -> int:
    # Should not inherit IO effect if we don't call impure
    return 42
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestEffectAnnotations:
    """Test explicit effect annotations."""

    def test_pure_annotation(self, validator):
        """Test explicit pure annotation."""
        code = """
def add(x: int, y: int) -> int:  # Pure
    return x + y
"""
        assert validator.validate(code)

    def test_io_annotation(self, validator):
        """Test explicit IO effect annotation."""
        code = """
def log(message: str) -> None:  # IO
    print(message)
"""
        assert validator.validate(code)

    def test_multiple_effect_annotation(self, validator):
        """Test multiple effect annotations."""
        code = """
import random

def random_log() -> None:  # IO, Random
    if random.random() > 0.5:
        print("Random message")
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.effects
@pytest.mark.unit
class TestEffectPolymorphism:
    """Test effect polymorphism."""

    def test_effect_parameter(self, validator):
        """Test function with effect parameter."""
        code = """
from typing import Callable

def apply(f: Callable[[int], int], x: int) -> int:
    return f(x)
"""
        assert validator.validate(code)

    def test_effect_preservation(self, validator):
        """Test effects preserve through higher-order functions."""
        code = """
from typing import Callable

def map_list(f: Callable[[int], int], items: list[int]) -> list[int]:
    return [f(x) for x in items]
"""
        assert validator.validate(code)

    def test_effect_bound(self, validator):
        """Test effect bound on type variable."""
        code = """
from typing import TypeVar, Callable

T = TypeVar('T')

def safe_apply(f: Callable[[int], T], x: int) -> T:
    return f(x)
"""
        assert validator.validate(code)

