`<Option>` element
==================

```xml
<Option [id="string"]
        probability="real">
  <!-- Exactly one child generator or combiner -->
</Option>
```

Attributes
----------

- `probability` (required): A number between 0.0 and 1.0 that specifies the
  probability that the child element will run. An error occurs if this attribute
  is not between 0.0 and 1.0.

Description
-----------

The `<Option>` element runs its child element only a certain percentage of the
time, as determined by the `probability` attribute. It should have exactly one
child element, which can be any generator or combiner.

Constraining
------------

The `<Option>` element can be constrained with the values `true` or `false`. If
a constraint is present, it overrides the specified probability, and the child
is either always run (if `true`) or never run (if `false`).
