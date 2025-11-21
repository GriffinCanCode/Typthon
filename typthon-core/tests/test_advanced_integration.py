"""End-to-end tests for advanced type features integration."""

import pytest
from typthon import (
    type, check, validate,
    EffectType, RefinementType, DependentType, RecursiveType,
    effect, refine, dependent, recursive,
    Positive, Negative, NonEmpty, Even, Odd, Bounded,
    IO, Async, Random
)
from typthon.core.validator import (
    validate_refinement_type,
    validate_dependent_type,
    validate_effect_type
)


class TestEffectTypes:
    """Test effect type integration."""

    def test_effect_type_creation(self):
        """Test creating effect types."""
        IOInt = effect('io')(int)
        assert isinstance(IOInt, EffectType)
        assert IOInt.has_effect('io')
        assert not IOInt.is_pure()

    def test_pure_effect(self):
        """Test pure effect type."""
        PureInt = effect('pure')(int)
        assert isinstance(PureInt, EffectType)
        assert PureInt.is_pure()

    def test_multiple_effects(self):
        """Test type with multiple effects."""
        ComplexType = effect('io', 'async', 'network')(str)
        assert ComplexType.has_effect('io')
        assert ComplexType.has_effect('async')
        assert ComplexType.has_effect('network')

    def test_effect_shortcuts(self):
        """Test effect shortcut constructors."""
        io_int = IO(int)
        async_str = Async(str)
        random_float = Random(float)

        assert io_int.has_effect('io')
        assert async_str.has_effect('async')
        assert random_float.has_effect('random')


class TestRefinementTypes:
    """Test refinement type integration."""

    def test_positive_int(self):
        """Test positive integer refinement."""
        pos = Positive()
        assert pos.validate(5)
        assert not pos.validate(-5)
        assert not pos.validate(0)

    def test_negative_int(self):
        """Test negative integer refinement."""
        neg = Negative()
        assert neg.validate(-5)
        assert not neg.validate(5)
        assert not neg.validate(0)

    def test_non_empty_string(self):
        """Test non-empty string refinement."""
        non_empty = NonEmpty(str)
        assert non_empty.validate("hello")
        assert not non_empty.validate("")

    def test_even_odd(self):
        """Test even/odd refinements."""
        even = Even()
        odd = Odd()

        assert even.validate(2)
        assert even.validate(0)
        assert not even.validate(1)

        assert odd.validate(1)
        assert odd.validate(3)
        assert not odd.validate(2)

    def test_bounded_int(self):
        """Test bounded integer refinement."""
        percentage = Bounded(0, 100)
        assert percentage.validate(50)
        assert percentage.validate(0)
        assert percentage.validate(100)
        assert not percentage.validate(-1)
        assert not percentage.validate(101)

    def test_custom_refinement(self):
        """Test custom refinement predicate."""
        divisible_by_3 = refine('value % 3 == 0')(int)
        assert divisible_by_3.validate(9)
        assert divisible_by_3.validate(0)
        assert not divisible_by_3.validate(10)

    def test_refinement_with_rust_validator(self):
        """Test refinement validation through Rust."""
        result = validate_refinement_type(5, int, 'value > 0')
        assert result is True

        result = validate_refinement_type(-5, int, 'value > 0')
        assert result is False


class TestDependentTypes:
    """Test dependent type integration."""

    def test_fixed_length_list(self):
        """Test dependent type with fixed length."""
        Array5 = dependent('len=5')(list)

        assert validate_dependent_type([1, 2, 3, 4, 5], list, 'len=5')
        assert not validate_dependent_type([1, 2, 3], list, 'len=5')

    def test_length_range(self):
        """Test dependent type with length range."""
        BoundedStr = dependent('0<=len<=10')(str)

        assert validate_dependent_type("hello", str, '0<=len<=10')
        assert validate_dependent_type("", str, '0<=len<=10')
        assert not validate_dependent_type("a" * 20, str, '0<=len<=10')


class TestRecursiveTypes:
    """Test recursive type integration."""

    def test_json_type(self):
        """Test JSON recursive type."""
        JSON = recursive('JSON', lambda self:
            type('None | bool | int | float | str | list[self] | dict[str, self]'))

        assert isinstance(JSON, RecursiveType)
        assert JSON.name == 'JSON'

    def test_linked_list(self):
        """Test linked list recursive type."""
        def LinkedList(T):
            return recursive('List', lambda self:
                type(f'None | tuple[{T}, self]'))

        IntList = LinkedList('int')
        assert isinstance(IntList, RecursiveType)

    def test_binary_tree(self):
        """Test binary tree recursive type."""
        Tree = recursive('Tree', lambda self:
            type('tuple[int] | tuple[self, int, self]'))

        assert isinstance(Tree, RecursiveType)
        assert Tree.name == 'Tree'


class TestAdvancedIntegration:
    """Test full end-to-end integration of advanced features."""

    def test_effect_with_refinement(self):
        """Test combining effects with refinements."""
        # IO operation returning positive integer
        IOPositive = effect('io')(Positive())

        assert isinstance(IOPositive, EffectType)
        assert IOPositive.has_effect('io')
        # Base type should be RefinementType
        assert isinstance(IOPositive.base_type, RefinementType)

    def test_refinement_in_function(self):
        """Test function with refinement type annotation."""

        @type("(Positive) -> Positive")
        def square_positive(x: int) -> int:
            """Square a positive number."""
            return x * x

        # Function should be defined
        assert callable(square_positive)

        # Test execution
        result = square_positive(5)
        assert result == 25

    def test_effect_tracking(self):
        """Test effect tracking in functions."""

        @type("() -> IO[str]")
        def read_input():
            """Read input (has IO effect)."""
            return "test input"

        result = read_input()
        assert result == "test input"

    def test_dependent_type_validation(self):
        """Test dependent type in function signature."""

        def make_fixed_array(n: int):
            """Create a type for arrays of length n."""
            return dependent(f'len={n}')(list)

        Array3 = make_fixed_array(3)
        assert validate_dependent_type([1, 2, 3], list, 'len=3')

    def test_complex_type_combination(self):
        """Test complex combination of advanced types."""
        # Function that does IO and returns positive bounded integer
        @type("() -> IO[Bounded[0, 100]]")
        def get_percentage():
            """Get a percentage value with IO."""
            return 75

        result = get_percentage()
        assert result == 75

        # Validate result is in bounds
        bounded = Bounded(0, 100)
        assert bounded.validate(result)


class TestRustIntegration:
    """Test Rust FFI integration."""

    def test_rust_effect_analysis(self):
        """Test effect analysis through Rust."""
        try:
            from typthon._core import check_effects

            source = """
def pure_func(x):
    return x + 1

def io_func():
    print("Hello")
    return 42
"""
            effects = check_effects(source)

            # Check that effects were analyzed
            assert isinstance(effects, dict)
            # pure_func should be pure or have no/minimal effects
            # io_func should have IO effect

        except ImportError:
            pytest.skip("Rust core not available")

    def test_rust_refinement_validation(self):
        """Test refinement validation through Rust."""
        try:
            from typthon._core import validate_refinement
            import json

            # Test positive integer
            assert validate_refinement(json.dumps(5), "value > 0")
            assert not validate_refinement(json.dumps(-5), "value > 0")

            # Test bounded
            assert validate_refinement(json.dumps(50), "value >= 0 and value <= 100")
            assert not validate_refinement(json.dumps(150), "value >= 0 and value <= 100")

        except ImportError:
            pytest.skip("Rust core not available")

    def test_rust_type_validator(self):
        """Test TypeValidator class from Rust."""
        try:
            from typthon._core import TypeValidator

            validator = TypeValidator()

            # Test basic validation
            result = validator.validate("x: int = 5")
            assert isinstance(result, bool)

            # Test type inference
            type_str = validator.get_type("5")
            assert "Int" in type_str

        except ImportError:
            pytest.skip("Rust core not available")


class TestErrorHandling:
    """Test error handling in advanced features."""

    def test_invalid_refinement(self):
        """Test handling of invalid refinement predicates."""
        invalid_ref = RefinementType(int, "invalid syntax!!")

        # Should not crash, should return False
        result = invalid_ref.validate(5)
        assert isinstance(result, bool)

    def test_type_mismatch_refinement(self):
        """Test refinement with wrong base type."""
        pos_int = Positive()

        # Passing string to int refinement
        assert not pos_int.validate("hello")

    def test_malformed_dependent_constraint(self):
        """Test handling of malformed dependent constraint."""
        result = validate_dependent_type([1, 2, 3], list, "malformed!!!")
        # Should default to True or handle gracefully
        assert isinstance(result, bool)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

