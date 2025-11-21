"""Test Phase 3 advanced type features."""

import pytest
from typthon import (
    type, infer, validate,
    # Effect types
    effect, EffectType, IO, Network, Async, Random,
    # Refinement types
    refine, RefinementType,
    Positive, Negative, NonNegative, NonZero, NonEmpty, Even, Odd, Bounded,
    # Dependent types
    dependent, DependentType,
    # Nominal types
    newtype, NominalType,
    # Recursive types
    recursive, RecursiveType,
    Union, Optional,
)


# ============================================================================
# EFFECT TYPE TESTS
# ============================================================================

class TestEffectTypes:
    """Test effect type system."""

    def test_pure_effect(self):
        """Test pure (no effects) type."""
        pure_int = EffectType(int)
        assert pure_int.is_pure()
        assert not pure_int.has_effect('io')

    def test_single_effect(self):
        """Test type with single effect."""
        io_str = IO(str)
        assert not io_str.is_pure()
        assert io_str.has_effect('io')
        assert not io_str.has_effect('network')

    def test_multiple_effects(self):
        """Test type with multiple effects."""
        multi = EffectType(int, 'io', 'network')
        assert not multi.is_pure()
        assert multi.has_effect('io')
        assert multi.has_effect('network')
        assert not multi.has_effect('random')

    def test_effect_repr(self):
        """Test effect type string representation."""
        io_int = IO(int)
        assert 'int' in str(io_int)
        assert 'io' in str(io_int)

    def test_effect_decorator(self):
        """Test effect type decorator."""
        @type("() -> IO[str]")
        def read_input():
            return "test"

        assert read_input() == "test"

    def test_network_effect(self):
        """Test network effect type."""
        net = Network(dict)
        assert net.has_effect('network')
        assert not net.is_pure()


# ============================================================================
# REFINEMENT TYPE TESTS
# ============================================================================

class TestRefinementTypes:
    """Test refinement type system."""

    def test_positive_int(self):
        """Test positive integer refinement."""
        pos = Positive()
        assert pos.validate(1)
        assert pos.validate(100)
        assert not pos.validate(0)
        assert not pos.validate(-1)

    def test_negative_int(self):
        """Test negative integer refinement."""
        neg = Negative()
        assert neg.validate(-1)
        assert neg.validate(-100)
        assert not neg.validate(0)
        assert not neg.validate(1)

    def test_non_negative_int(self):
        """Test non-negative integer refinement."""
        nn = NonNegative()
        assert nn.validate(0)
        assert nn.validate(1)
        assert nn.validate(100)
        assert not nn.validate(-1)

    def test_non_zero(self):
        """Test non-zero refinement."""
        nz = NonZero()
        assert nz.validate(1)
        assert nz.validate(-1)
        assert not nz.validate(0)

    def test_even_odd(self):
        """Test even/odd refinements."""
        even = Even()
        odd = Odd()

        assert even.validate(0)
        assert even.validate(2)
        assert even.validate(4)
        assert not even.validate(1)
        assert not even.validate(3)

        assert odd.validate(1)
        assert odd.validate(3)
        assert not odd.validate(0)
        assert not odd.validate(2)

    def test_bounded(self):
        """Test bounded range refinement."""
        bounded = Bounded(0, 100)
        assert bounded.validate(0)
        assert bounded.validate(50)
        assert bounded.validate(100)
        assert not bounded.validate(-1)
        assert not bounded.validate(101)

    def test_non_empty_str(self):
        """Test non-empty string refinement."""
        ne = NonEmpty()
        assert ne.validate("hello")
        assert ne.validate("a")
        assert not ne.validate("")

    def test_custom_refinement(self):
        """Test custom refinement predicate."""
        age_type = refine('value >= 0 and value <= 150')(int)
        assert age_type.validate(0)
        assert age_type.validate(25)
        assert age_type.validate(150)
        assert not age_type.validate(-1)
        assert not age_type.validate(151)

    def test_refinement_repr(self):
        """Test refinement type string representation."""
        pos = Positive()
        assert 'int' in str(pos)
        assert 'value > 0' in str(pos)


# ============================================================================
# DEPENDENT TYPE TESTS
# ============================================================================

class TestDependentTypes:
    """Test dependent type system."""

    def test_fixed_length(self):
        """Test fixed-length dependent type."""
        arr5 = dependent('len=5')(list)
        assert isinstance(arr5, DependentType)
        assert 'len=5' in str(arr5)

    def test_length_range(self):
        """Test length range dependent type."""
        bounded_list = dependent('0<=len<=10')(list)
        assert isinstance(bounded_list, DependentType)
        assert '0<=len<=10' in str(bounded_list)

    def test_dependent_function(self):
        """Test function with dependent type."""
        def FixedArray(n):
            return dependent(f'len={n}')(list)

        arr3 = FixedArray(3)
        assert f'len=3' in str(arr3)


# ============================================================================
# NOMINAL TYPE TESTS
# ============================================================================

class TestNominalTypes:
    """Test nominal type system."""

    def test_newtype_creation(self):
        """Test creating nominal types."""
        UserId = newtype('UserId')(int)
        assert isinstance(UserId, NominalType)
        assert UserId.name == 'UserId'
        assert UserId.base_type == int

    def test_nominal_distinctness(self):
        """Test that nominal types are distinct."""
        UserId = newtype('UserId')(int)
        OrderId = newtype('OrderId')(int)

        assert UserId.name != OrderId.name
        assert isinstance(UserId, NominalType)
        assert isinstance(OrderId, NominalType)

    def test_nominal_constructor(self):
        """Test nominal type constructor."""
        UserId = newtype('UserId')(int)
        uid = UserId(12345)
        assert uid == 12345

    def test_nominal_repr(self):
        """Test nominal type string representation."""
        UserId = newtype('UserId')(int)
        assert str(UserId) == 'UserId'


# ============================================================================
# RECURSIVE TYPE TESTS
# ============================================================================

class TestRecursiveTypes:
    """Test recursive type system."""

    def test_recursive_creation(self):
        """Test creating recursive types."""
        JSON = recursive('JSON', lambda self: Union[None, bool, int, str, list, dict])
        assert isinstance(JSON, RecursiveType)
        assert JSON.name == 'JSON'

    def test_recursive_repr(self):
        """Test recursive type string representation."""
        IntList = recursive('IntList', lambda self: Union[None, tuple])
        assert 'IntList' in str(IntList)

    def test_nested_recursive(self):
        """Test nested recursive structures."""
        def LinkedList(T):
            return recursive('List', lambda self: Union[None, tuple])

        int_list = LinkedList(int)
        assert isinstance(int_list, RecursiveType)


# ============================================================================
# COMBINED FEATURE TESTS
# ============================================================================

class TestCombinedFeatures:
    """Test combinations of advanced features."""

    def test_effect_with_refinement(self):
        """Test combining effects with refinements."""
        # IO effect returning positive int
        io_pos = IO(Positive())
        assert io_pos.has_effect('io')

    def test_nominal_with_refinement(self):
        """Test combining nominal with refinements."""
        UserId = newtype('UserId')(int)
        # In practice, would combine: PositiveUserId = refine('value > 0')(UserId)
        assert isinstance(UserId, NominalType)

    def test_multiple_effects(self):
        """Test multiple effects composition."""
        multi = EffectType(int, 'io', 'network', 'async')
        assert multi.has_effect('io')
        assert multi.has_effect('network')
        assert multi.has_effect('async')

    def test_complex_refinement(self):
        """Test complex refinement predicates."""
        percentage = Bounded(0, 100)
        assert percentage.validate(0)
        assert percentage.validate(50)
        assert percentage.validate(100)
        assert not percentage.validate(-1)
        assert not percentage.validate(101)


# ============================================================================
# TYPE DECORATOR TESTS
# ============================================================================

class TestTypeDecorators:
    """Test advanced types with @type decorator."""

    def test_effect_annotation(self):
        """Test function with effect annotation."""
        @type("() -> IO[int]")
        def read_number():
            return 42

        assert read_number() == 42

    def test_refinement_annotation(self):
        """Test function with refinement annotation."""
        @type("(Positive) -> Positive")
        def square(x):
            return x * x

        assert square(5) == 25

    def test_nominal_annotation(self):
        """Test function with nominal type annotation."""
        UserId = newtype('UserId')(int)

        @type("(UserId) -> dict")
        def get_user(uid):
            return {"id": uid}

        assert get_user(123) == {"id": 123}


# ============================================================================
# TYPE INFERENCE TESTS
# ============================================================================

class TestTypeInference:
    """Test type inference with advanced types."""

    def test_infer_effect(self):
        """Test inferring effect types."""
        @infer
        def with_io():
            # Would be inferred to have IO effect
            return 42

        assert hasattr(with_io, '__typthon_inferred__')

    def test_infer_refinement(self):
        """Test inferring refinement types."""
        @infer
        def always_positive(x):
            # Would be inferred to return positive int
            return abs(x) + 1

        assert hasattr(always_positive, '__typthon_inferred__')


# ============================================================================
# VALIDATION TESTS
# ============================================================================

class TestValidation:
    """Test runtime validation of advanced types."""

    def test_validate_positive(self):
        """Test validating positive integers."""
        pos = Positive()
        assert pos.validate(5)
        assert not pos.validate(-5)

    def test_validate_even(self):
        """Test validating even numbers."""
        even = Even()
        assert even.validate(4)
        assert not even.validate(5)

    def test_validate_bounded(self):
        """Test validating bounded values."""
        bounded = Bounded(0, 100)
        assert bounded.validate(50)
        assert not bounded.validate(150)

    def test_validate_non_empty(self):
        """Test validating non-empty strings."""
        ne = NonEmpty()
        assert ne.validate("hello")
        assert not ne.validate("")


# ============================================================================
# ERROR CASES
# ============================================================================

class TestErrorCases:
    """Test error handling for advanced types."""

    def test_invalid_refinement(self):
        """Test invalid refinement predicate."""
        pos = Positive()
        # Should handle invalid values gracefully
        assert not pos.validate("not a number")

    def test_invalid_effect(self):
        """Test checking for non-existent effect."""
        io_int = IO(int)
        assert not io_int.has_effect('nonexistent')


if __name__ == "__main__":
    pytest.main([__file__, "-v"])

