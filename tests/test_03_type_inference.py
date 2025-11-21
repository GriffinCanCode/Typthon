"""Test type inference functionality.

This module tests the type inference engine's ability to infer types automatically.
"""

import pytest


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestBasicInference:
    """Test basic type inference."""

    def test_int_literal_inference(self, validator):
        """Test inference of int literal."""
        code = "x = 42"
        assert validator.validate(code)

    def test_float_literal_inference(self, validator):
        """Test inference of float literal."""
        code = "x = 3.14"
        assert validator.validate(code)

    def test_str_literal_inference(self, validator):
        """Test inference of string literal."""
        code = 'x = "hello"'
        assert validator.validate(code)

    def test_bool_literal_inference(self, validator):
        """Test inference of boolean literal."""
        code = "x = True"
        assert validator.validate(code)

    def test_list_inference(self, validator):
        """Test inference of list type."""
        code = "x = [1, 2, 3]"
        assert validator.validate(code)

    def test_dict_inference(self, validator):
        """Test inference of dict type."""
        code = "x = {'a': 1, 'b': 2}"
        assert validator.validate(code)

    def test_tuple_inference(self, validator):
        """Test inference of tuple type."""
        code = "x = (1, 'hello', 3.14)"
        assert validator.validate(code)

    def test_set_inference(self, validator):
        """Test inference of set type."""
        code = "x = {1, 2, 3}"
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestFunctionInference:
    """Test function return type inference."""

    def test_simple_return_inference(self, validator):
        """Test inference of simple return type."""
        code = """
def get_value():
    return 42
"""
        assert validator.validate(code)

    def test_arithmetic_return_inference(self, validator):
        """Test inference of arithmetic result."""
        code = """
def add(x: int, y: int):
    return x + y
"""
        assert validator.validate(code)

    def test_string_return_inference(self, validator):
        """Test inference of string return."""
        code = """
def greet(name: str):
    return f"Hello, {name}"
"""
        assert validator.validate(code)

    def test_list_return_inference(self, validator):
        """Test inference of list return."""
        code = """
def make_list():
    return [1, 2, 3]
"""
        assert validator.validate(code)

    def test_conditional_return_inference(self, validator):
        """Test inference with conditional returns."""
        code = """
def check(x: int):
    if x > 0:
        return "positive"
    else:
        return "non-positive"
"""
        assert validator.validate(code)

    def test_multiple_returns_same_type(self, validator):
        """Test inference with multiple returns of same type."""
        code = """
def classify(x: int):
    if x > 0:
        return "positive"
    elif x < 0:
        return "negative"
    else:
        return "zero"
"""
        assert validator.validate(code)

    def test_function_call_inference(self, validator):
        """Test inference from function call."""
        code = """
def get_int() -> int:
    return 42

x = get_int()
"""
        assert validator.validate(code)

    def test_chain_inference(self, validator):
        """Test inference through function chain."""
        code = """
def double(x: int) -> int:
    return x * 2

def triple(x: int) -> int:
    return x * 3

result = triple(double(5))
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestBinaryOperationInference:
    """Test type inference for binary operations."""

    def test_int_addition_inference(self, validator):
        """Test inference of int addition."""
        code = """
x = 1
y = 2
z = x + y
"""
        assert validator.validate(code)

    def test_float_addition_inference(self, validator):
        """Test inference of float addition."""
        code = """
x = 1.0
y = 2.0
z = x + y
"""
        assert validator.validate(code)

    def test_str_concatenation_inference(self, validator):
        """Test inference of string concatenation."""
        code = """
x = "hello"
y = "world"
z = x + y
"""
        assert validator.validate(code)

    def test_list_concatenation_inference(self, validator):
        """Test inference of list concatenation."""
        code = """
x = [1, 2]
y = [3, 4]
z = x + y
"""
        assert validator.validate(code)

    def test_division_returns_float(self, validator):
        """Test that division infers float."""
        code = """
x = 10
y = 2
z = x / y
"""
        assert validator.validate(code)

    def test_floor_division_returns_int(self, validator):
        """Test that floor division infers int."""
        code = """
x = 10
y = 3
z = x // y
"""
        assert validator.validate(code)

    def test_comparison_returns_bool(self, validator):
        """Test that comparison infers bool."""
        code = """
x = 5
y = 3
result = x > y
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestCollectionInference:
    """Test type inference for collections."""

    def test_list_comprehension_inference(self, validator):
        """Test inference of list comprehension."""
        code = """
numbers = [i * 2 for i in range(5)]
"""
        assert validator.validate(code)

    def test_dict_comprehension_inference(self, validator):
        """Test inference of dict comprehension."""
        code = """
squares = {i: i * i for i in range(5)}
"""
        assert validator.validate(code)

    def test_set_comprehension_inference(self, validator):
        """Test inference of set comprehension."""
        code = """
evens = {i for i in range(10) if i % 2 == 0}
"""
        assert validator.validate(code)

    def test_nested_list_inference(self, validator):
        """Test inference of nested lists."""
        code = """
matrix = [[1, 2], [3, 4], [5, 6]]
"""
        assert validator.validate(code)

    def test_list_index_inference(self, validator):
        """Test inference from list indexing."""
        code = """
numbers = [1, 2, 3, 4, 5]
first = numbers[0]
"""
        assert validator.validate(code)

    def test_dict_index_inference(self, validator):
        """Test inference from dict indexing."""
        code = """
data = {'a': 1, 'b': 2}
value = data['a']
"""
        assert validator.validate(code)

    def test_tuple_unpacking_inference(self, validator):
        """Test inference from tuple unpacking."""
        code = """
pair = (1, "hello")
x, y = pair
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestControlFlowInference:
    """Test type inference with control flow."""

    def test_if_branch_inference(self, validator):
        """Test inference in if branch."""
        code = """
x = 5
if x > 0:
    result = "positive"
else:
    result = "non-positive"
"""
        assert validator.validate(code)

    def test_loop_variable_inference(self, validator):
        """Test inference of loop variable."""
        code = """
for i in range(10):
    x = i * 2
"""
        assert validator.validate(code)

    def test_while_loop_inference(self, validator):
        """Test inference in while loop."""
        code = """
counter = 0
while counter < 10:
    counter = counter + 1
"""
        assert validator.validate(code)

    def test_accumulator_inference(self, validator):
        """Test inference of accumulator."""
        code = """
total = 0
for i in range(10):
    total = total + i
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestComplexInference:
    """Test complex type inference scenarios."""

    def test_lambda_inference(self, validator):
        """Test inference of lambda."""
        code = """
double = lambda x: x * 2
result = double(5)
"""
        assert validator.validate(code)

    def test_map_inference(self, validator):
        """Test inference with map."""
        code = """
numbers = [1, 2, 3]
doubled = list(map(lambda x: x * 2, numbers))
"""
        assert validator.validate(code)

    def test_filter_inference(self, validator):
        """Test inference with filter."""
        code = """
numbers = [1, 2, 3, 4, 5]
evens = list(filter(lambda x: x % 2 == 0, numbers))
"""
        assert validator.validate(code)

    def test_nested_function_inference(self, validator):
        """Test inference with nested functions."""
        code = """
def outer():
    def inner(x: int):
        return x * 2
    return inner(5)
"""
        assert validator.validate(code)

    def test_closure_inference(self, validator):
        """Test inference with closures."""
        code = """
def make_multiplier(factor: int):
    def multiplier(x: int):
        return x * factor
    return multiplier

double = make_multiplier(2)
"""
        assert validator.validate(code)

    def test_generator_inference(self, validator):
        """Test inference with generators."""
        code = """
def count_up_to(n: int):
    i = 0
    while i < n:
        yield i
        i = i + 1

gen = count_up_to(5)
"""
        assert validator.validate(code)

    def test_method_chain_inference(self, validator):
        """Test inference through method chain."""
        code = """
text = "  hello world  "
result = text.strip().upper()
"""
        assert validator.validate(code)

    def test_ternary_inference(self, validator):
        """Test inference with ternary operator."""
        code = """
x = 5
result = "positive" if x > 0 else "non-positive"
"""
        assert validator.validate(code)


@pytest.mark.requires_typhon
@pytest.mark.inference
@pytest.mark.unit
class TestBidirectionalInference:
    """Test bidirectional type inference."""

    def test_expected_type_propagation(self, validator):
        """Test that expected types propagate down."""
        code = """
def process(x: int) -> int:
    return x * 2

result: int = process(5)
"""
        assert validator.validate(code)

    def test_contextual_list_inference(self, validator):
        """Test contextual inference for lists."""
        code = """
def take_ints(numbers: list[int]) -> int:
    return sum(numbers)

result = take_ints([1, 2, 3])
"""
        assert validator.validate(code)

    def test_contextual_dict_inference(self, validator):
        """Test contextual inference for dicts."""
        code = """
def process_mapping(data: dict[str, int]) -> int:
    return sum(data.values())

result = process_mapping({'a': 1, 'b': 2})
"""
        assert validator.validate(code)

    def test_return_type_constraint(self, validator):
        """Test return type constraining inference."""
        code = """
def get_value() -> int:
    x = 42
    return x
"""
        assert validator.validate(code)

    def test_assignment_type_constraint(self, validator):
        """Test assignment constraining inference."""
        code = """
x: int = 42
y = x
"""
        assert validator.validate(code)

