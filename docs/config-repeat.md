`<Repeat>` element
==================

```xml
<Repeat [id="string"]
        [min="integer"]
        [max="integer"]>
  <!-- Exactly one child generator or combiner -->
</Repeat>
```

Attributes
----------

- `min` (optional): The minimum number of times to run the child element. Default
  value is `1`.
- `max` (optional): The maximum number of times to run the child element. Default
  value is `2`.

Description
-----------

The `<Repeat>` element runs its child element a random number of times within
the specified range. It should be placed in a [`<Join>`] in order to to be
effective. The range is inclusive of both `min` and `max`. If `min` is greater
than `max`, then a parse error occurs. It should have exactly one child element,
which can be any generator or combiner.


Constraining
------------

The `<Repeat>` element can be constrained by specifying an exact number of
repetitions. The constraint should be a string that can be parsed into an
integer. If the constraint cannot be parsed into an integer, or if the parsed
integer is outside the specified range, then an error occurs.

[`<Join>`]: ./config-join.md
