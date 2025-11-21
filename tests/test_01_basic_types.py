"""Test basic type checking functionality.

This module tests fundamental type checking operations from a user's perspective.
Tests cover: primitives, basic operations, simple functions.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestPrimitiveTypes:
    """Test primitive type checking."""

    def test_int_literal(self, validator):
        """Test integer literal type checking."""
        code = "x: int = 42"
        assert validator.validate(code)

    def test_float_literal(self, validator):
        """Test float literal type checking."""
        code = "x: float = 3.14"
        assert validator.validate(code)

    def test_str_literal(self, validator):
        """Test string literal type checking."""
        code = 'x: str = "hello"'
        assert validator.validate(code)

    def test_bool_literal(self, validator):
        """Test boolean literal type checking."""
        code = "x: bool = True"
        assert validator.validate(code)

    def test_none_literal(self, validator):
        """Test None literal type checking."""
        code = "x: None = None"
        assert validator.validate(code)

    def test_int_type_error(self, validator):
        """Test int type mismatch detection."""
        code = 'x: int = "not an int"'
        assert not validator.validate(code)

    def test_float_type_error(self, validator):
        """Test float type mismatch detection."""
        code = 'x: float = "not a float"'
        assert not validator.validate(code)

    def test_str_type_error(self, validator):
        """Test string type mismatch detection."""
        code = "x: str = 42"
        assert not validator.validate(code)

    def test_bool_type_error(self, validator):
        """Test boolean type mismatch detection."""
        code = "x: bool = 42"
        assert not validator.validate(code)

    def test_multiple_assignments(self, validator):
        """Test multiple variable assignments."""
        code = """
x: int = 1
y: str = "hello"
z: float = 3.14
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestBasicOperations:
    """Test type checking of basic operations."""

    def test_int_addition(self, validator):
        """Test integer addition."""
        code = """
x: int = 1
y: int = 2
z: int = x + y
"""
        assert validator.validate(code)

    def test_int_subtraction(self, validator):
        """Test integer subtraction."""
        code = """
x: int = 5
y: int = 3
z: int = x - y
"""
        assert validator.validate(code)

    def test_int_multiplication(self, validator):
        """Test integer multiplication."""
        code = """
x: int = 3
y: int = 4
z: int = x * y
"""
        assert validator.validate(code)

    def test_int_division(self, validator):
        """Test integer division returns float."""
        code = """
x: int = 10
y: int = 2
z: float = x / y
"""
        assert validator.validate(code)

    def test_int_floor_division(self, validator):
        """Test integer floor division."""
        code = """
x: int = 10
y: int = 3
z: int = x // y
"""
        assert validator.validate(code)

    def test_int_modulo(self, validator):
        """Test integer modulo."""
        code = """
x: int = 10
y: int = 3
z: int = x % y
"""
        assert validator.validate(code)

    def test_int_power(self, validator):
        """Test integer power."""
        code = """
x: int = 2
y: int = 3
z: int = x ** y
"""
        assert validator.validate(code)

    def test_str_concatenation(self, validator):
        """Test string concatenation."""
        code = """
x: str = "hello"
y: str = "world"
z: str = x + y
"""
        assert validator.validate(code)

    def test_str_repetition(self, validator):
        """Test string repetition."""
        code = """
x: str = "ha"
y: int = 3
z: str = x * y
"""
        assert validator.validate(code)

    def test_mixed_arithmetic_error(self, validator):
        """Test arithmetic with incompatible types."""
        code = """
x: int = 1
y: str = "hello"
z: int = x + y  # Type error
"""
        assert not validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestComparisons:
    """Test comparison operations."""

    def test_int_equality(self, validator):
        """Test integer equality comparison."""
        code = """
x: int = 1
y: int = 1
result: bool = x == y
"""
        assert validator.validate(code)

    def test_int_inequality(self, validator):
        """Test integer inequality comparison."""
        code = """
x: int = 1
y: int = 2
result: bool = x != y
"""
        assert validator.validate(code)

    def test_int_less_than(self, validator):
        """Test integer less than comparison."""
        code = """
x: int = 1
y: int = 2
result: bool = x < y
"""
        assert validator.validate(code)

    def test_int_less_equal(self, validator):
        """Test integer less than or equal comparison."""
        code = """
x: int = 1
y: int = 2
result: bool = x <= y
"""
        assert validator.validate(code)

    def test_int_greater_than(self, validator):
        """Test integer greater than comparison."""
        code = """
x: int = 2
y: int = 1
result: bool = x > y
"""
        assert validator.validate(code)

    def test_int_greater_equal(self, validator):
        """Test integer greater than or equal comparison."""
        code = """
x: int = 2
y: int = 1
result: bool = x >= y
"""
        assert validator.validate(code)

    def test_str_comparison(self, validator):
        """Test string comparison."""
        code = """
x: str = "abc"
y: str = "def"
result: bool = x < y
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestSimpleFunctions:
    """Test simple function type checking."""

    def test_function_no_args(self, validator):
        """Test function with no arguments."""
        code = """
def get_value() -> int:
    return 42
"""
        assert validator.validate(code)

    def test_function_one_arg(self, validator):
        """Test function with one argument."""
        code = """
def double(x: int) -> int:
    return x * 2
"""
        assert validator.validate(code)

    def test_function_two_args(self, validator):
        """Test function with two arguments."""
        code = """
def add(x: int, y: int) -> int:
    return x + y
"""
        assert validator.validate(code)

    def test_function_multiple_args(self, validator):
        """Test function with multiple arguments."""
        code = """
def average(a: int, b: int, c: int) -> float:
    return (a + b + c) / 3
"""
        assert validator.validate(code)

    def test_function_call_valid(self, validator):
        """Test valid function call."""
        code = """
def add(x: int, y: int) -> int:
    return x + y

result: int = add(1, 2)
"""
        assert validator.validate(code)

    def test_function_call_invalid_arg_type(self, validator):
        """Test function call with invalid argument type."""
        code = """
def add(x: int, y: int) -> int:
    return x + y

result: int = add("1", "2")  # Type error
"""
        assert not validator.validate(code)

    def test_function_call_invalid_return_type(self, validator):
        """Test function call with invalid return type."""
        code = """
def get_str() -> str:
    return "hello"

result: int = get_str()  # Type error
"""
        assert not validator.validate(code)

    def test_function_wrong_return_type(self, validator):
        """Test function returning wrong type."""
        code = """
def get_int() -> int:
    return "not an int"  # Type error
"""
        assert not validator.validate(code)

    def test_void_function(self, validator):
        """Test function returning None."""
        code = """
def print_message(msg: str) -> None:
    print(msg)
"""
        assert validator.validate(code)

    def test_nested_function_calls(self, validator):
        """Test nested function calls."""
        code = """
def double(x: int) -> int:
    return x * 2

def quadruple(x: int) -> int:
    return double(double(x))

result: int = quadruple(5)
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestVariableScoping:
    """Test variable scoping rules."""

    def test_local_variable(self, validator):
        """Test local variable in function."""
        code = """
def func():
    x: int = 42
    return x
"""
        assert validator.validate(code)

    def test_global_variable(self, validator):
        """Test global variable access."""
        code = """
x: int = 42

def get_x() -> int:
    return x
"""
        assert validator.validate(code)

    def test_parameter_shadowing(self, validator):
        """Test parameter shadowing global variable."""
        code = """
x: int = 42

def func(x: str) -> str:
    return x
"""
        assert validator.validate(code)

    def test_local_shadowing(self, validator):
        """Test local variable shadowing."""
        code = """
def func():
    x: int = 1
    x: int = 2
    return x
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestControlFlow:
    """Test control flow type checking."""

    def test_if_statement(self, validator):
        """Test if statement."""
        code = """
def check(x: int) -> str:
    if x > 0:
        return "positive"
    return "non-positive"
"""
        assert validator.validate(code)

    def test_if_else_statement(self, validator):
        """Test if-else statement."""
        code = """
def check(x: int) -> str:
    if x > 0:
        return "positive"
    else:
        return "non-positive"
"""
        assert validator.validate(code)

    def test_if_elif_else(self, validator):
        """Test if-elif-else statement."""
        code = """
def check(x: int) -> str:
    if x > 0:
        return "positive"
    elif x < 0:
        return "negative"
    else:
        return "zero"
"""
        assert validator.validate(code)

    def test_while_loop(self, validator):
        """Test while loop."""
        code = """
def countdown(n: int) -> int:
    while n > 0:
        n = n - 1
    return n
"""
        assert validator.validate(code)

    def test_for_loop_range(self, validator):
        """Test for loop with range."""
        code = """
def sum_range(n: int) -> int:
    total: int = 0
    for i in range(n):
        total = total + i
    return total
"""
        assert validator.validate(code)

    def test_for_loop_list(self, validator):
        """Test for loop over list."""
        code = """
def sum_list(items: list[int]) -> int:
    total: int = 0
    for item in items:
        total = total + item
    return total
"""
        assert validator.validate(code)

    def test_break_statement(self, validator):
        """Test break statement."""
        code = """
def find_first_positive(items: list[int]) -> int:
    for item in items:
        if item > 0:
            break
    return item
"""
        assert validator.validate(code)

    def test_continue_statement(self, validator):
        """Test continue statement."""
        code = """
def sum_positive(items: list[int]) -> int:
    total: int = 0
    for item in items:
        if item <= 0:
            continue
        total = total + item
    return total
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.type_checking
@pytest.mark.unit
class TestExceptionHandling:
    """Test exception handling type checking."""

    def test_try_except(self, validator):
        """Test try-except block."""
        code = """
def safe_divide(x: int, y: int) -> float:
    try:
        return x / y
    except ZeroDivisionError:
        return 0.0
"""
        assert validator.validate(code)

    def test_try_except_finally(self, validator):
        """Test try-except-finally block."""
        code = """
def process() -> int:
    try:
        return 42
    except Exception:
        return 0
    finally:
        pass
"""
        assert validator.validate(code)

    def test_raise_exception(self, validator):
        """Test raising exception."""
        code = """
def validate(x: int) -> None:
    if x < 0:
        raise ValueError("x must be non-negative")
"""
        assert validator.validate(code)

