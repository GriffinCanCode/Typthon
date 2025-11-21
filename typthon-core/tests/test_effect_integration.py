"""
Test end-to-end effect system integration.

Tests the complete integration of:
1. Effect inference in Rust TypeChecker
2. Effect annotation on function types
3. Runtime validation via @type decorator
4. Signature parsing for complex types
"""

import pytest
from typthon.decorators.type import type, TypeValidationError
from typthon.core.signature_parser import parse_signature


class TestBasicValidation:
    """Test basic runtime type validation."""

    def test_valid_int_function(self):
        @type("(int, int) -> int")
        def add(x, y):
            return x + y

        assert add(5, 3) == 8
        assert add(0, 0) == 0
        assert add(-5, 10) == 5

    def test_valid_string_function(self):
        @type("(str, str) -> str")
        def concat(a, b):
            return a + b

        assert concat("Hello", " World") == "Hello World"
        assert concat("", "test") == "test"

    def test_invalid_type_warning(self):
        @type("(int, int) -> int", runtime=True, strict=False)
        def add(x, y):
            return x + y

        # Should warn but not raise
        with pytest.warns(UserWarning):
            result = add("5", 3)

    def test_invalid_type_strict(self):
        @type("(int, int) -> int", runtime=True, strict=True)
        def add(x, y):
            return x + y

        # Should raise in strict mode
        with pytest.raises(TypeValidationError):
            add("5", 3)

    def test_return_type_validation(self):
        @type("(int, int) -> str", runtime=True, strict=True)
        def bad_add(x, y):
            return x + y  # Returns int, not str

        with pytest.raises(TypeValidationError, match="return.*type mismatch"):
            bad_add(5, 3)


class TestRefinementValidation:
    """Test refinement type validation."""

    def test_positive_int_valid(self):
        @type("(int[value > 0]) -> int", runtime=True, strict=True)
        def process_positive(x):
            return x * 2

        assert process_positive(5) == 10
        assert process_positive(1) == 2

    def test_positive_int_invalid(self):
        @type("(int[value > 0]) -> int", runtime=True, strict=True)
        def process_positive(x):
            return x * 2

        with pytest.raises(TypeValidationError, match="refinement"):
            process_positive(0)

        with pytest.raises(TypeValidationError, match="refinement"):
            process_positive(-5)

    def test_non_empty_string(self):
        @type("(str[len(value) > 0]) -> int", runtime=True, strict=True)
        def count_chars(s):
            return len(s)

        assert count_chars("hello") == 5

        with pytest.raises(TypeValidationError, match="refinement"):
            count_chars("")

    def test_bounded_int(self):
        @type("(int[value >= 0 and value <= 100]) -> int", runtime=True, strict=True)
        def process_percentage(x):
            return x

        assert process_percentage(0) == 0
        assert process_percentage(50) == 50
        assert process_percentage(100) == 100

        with pytest.raises(TypeValidationError, match="refinement"):
            process_percentage(101)

        with pytest.raises(TypeValidationError, match="refinement"):
            process_percentage(-1)


class TestCollectionValidation:
    """Test collection type validation."""

    def test_list_int(self):
        @type("(list[int]) -> int", runtime=True, strict=True)
        def sum_list(numbers):
            return sum(numbers)

        assert sum_list([1, 2, 3, 4, 5]) == 15
        assert sum_list([]) == 0

        with pytest.raises(TypeValidationError):
            sum_list([1, "2", 3])

    def test_dict_str_int(self):
        @type("(dict[str, int]) -> int", runtime=True, strict=True)
        def sum_values(d):
            return sum(d.values())

        assert sum_values({"a": 1, "b": 2, "c": 3}) == 6

        with pytest.raises(TypeValidationError):
            sum_values({"a": 1, "b": "2"})

    def test_nested_list(self):
        @type("(list[list[int]]) -> int", runtime=True, strict=True)
        def flatten_sum(matrix):
            return sum(sum(row) for row in matrix)

        assert flatten_sum([[1, 2], [3, 4], [5, 6]]) == 21

        with pytest.raises(TypeValidationError):
            flatten_sum([[1, 2], ["3", 4]])

    def test_tuple_validation(self):
        @type("(tuple[int, str, bool]) -> str", runtime=True, strict=True)
        def format_tuple(t):
            return f"{t[0]}, {t[1]}, {t[2]}"

        assert format_tuple((42, "hello", True)) == "42, hello, True"

        with pytest.raises(TypeValidationError):
            format_tuple((42, 100, True))


class TestUnionValidation:
    """Test union type validation."""

    def test_int_or_str(self):
        @type("(int | str) -> str", runtime=True, strict=True)
        def to_string(value):
            return str(value)

        assert to_string(42) == "42"
        assert to_string("hello") == "hello"

        with pytest.raises(TypeValidationError):
            to_string([1, 2, 3])

    def test_list_union_elements(self):
        @type("(list[int | float]) -> float", runtime=True, strict=True)
        def average(numbers):
            return sum(numbers) / len(numbers) if numbers else 0.0

        assert average([1, 2, 3]) == 2.0
        assert average([1.5, 2.5, 3.0]) == 2.333333333333333
        assert average([1, 2.5, 3]) == 2.1666666666666665

        with pytest.raises(TypeValidationError):
            average([1, 2, "3"])


class TestSignatureParsing:
    """Test signature parsing functionality."""

    def test_simple_signature(self):
        sig = parse_signature("(int, int) -> int")
        assert len(sig.params) == 2
        assert sig.params[0].base_type == "int"
        assert sig.params[1].base_type == "int"
        assert sig.return_type.base_type == "int"
        assert sig.effects == []

    def test_signature_with_effects(self):
        sig = parse_signature("(int) -> int ! {io, exception}")
        assert len(sig.params) == 1
        assert sig.return_type.base_type == "int"
        assert set(sig.effects) == {"io", "exception"}

    def test_signature_with_refinement(self):
        sig = parse_signature("(int[value > 0]) -> int[value > 0]")
        assert sig.params[0].is_refinement
        assert sig.params[0].predicate == "value > 0"
        assert sig.return_type.is_refinement
        assert sig.return_type.predicate == "value > 0"

    def test_signature_with_generics(self):
        sig = parse_signature("(list[int]) -> int")
        assert sig.params[0].base_type == "list"
        assert len(sig.params[0].args) == 1
        assert sig.params[0].args[0].base_type == "int"

    def test_signature_with_dict(self):
        sig = parse_signature("(dict[str, int]) -> list[str]")
        assert sig.params[0].base_type == "dict"
        assert len(sig.params[0].args) == 2
        assert sig.params[0].args[0].base_type == "str"
        assert sig.params[0].args[1].base_type == "int"
        assert sig.return_type.base_type == "list"

    def test_signature_with_union(self):
        sig = parse_signature("(int | str) -> str")
        assert sig.params[0].base_type == "Union"
        assert len(sig.params[0].args) == 2

    def test_empty_params(self):
        sig = parse_signature("() -> int")
        assert len(sig.params) == 0
        assert sig.return_type.base_type == "int"


class TestMetadata:
    """Test metadata storage on decorated functions."""

    def test_metadata_stored(self):
        @type("(int, int) -> int", runtime=True, strict=True)
        def add(x, y):
            return x + y

        assert hasattr(add, "__typthon_type__")
        assert add.__typthon_type__ == "(int, int) -> int"
        assert hasattr(add, "__typthon_signature__")

    def test_runtime_disabled(self):
        @type("(int, int) -> int", runtime=False)
        def add(x, y):
            return x + y

        # Should not validate when runtime=False
        result = add("5", 3)  # Would normally fail, but runtime disabled
        # Result might be weird but no error should be raised


class TestComplexScenarios:
    """Test complex real-world scenarios."""

    def test_nested_refinements(self):
        @type("(list[int[value > 0]]) -> int[value > 0]", runtime=True, strict=True)
        def sum_positive(numbers):
            return sum(numbers)

        assert sum_positive([1, 2, 3]) == 6

        with pytest.raises(TypeValidationError):
            sum_positive([1, -2, 3])

    def test_effect_with_refinement(self):
        @type("() -> int[value > 0] ! {io}", runtime=True, strict=False)
        def read_positive():
            # In real scenario would read from input
            return 42

        result = read_positive()
        assert result == 42

    def test_multiple_args_with_different_types(self):
        @type("(str, int, list[str], dict[str, int]) -> str", runtime=True, strict=True)
        def complex_func(name, age, hobbies, scores):
            return f"{name}, {age}, {hobbies}, {scores}"

        result = complex_func("Alice", 30, ["reading", "coding"], {"math": 95, "science": 92})
        assert "Alice" in result
        assert "30" in result


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

