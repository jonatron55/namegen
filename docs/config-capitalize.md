`<Capitalize>` element
======================

```xml
<Capitalize [id="string"]
            [mode="string"]>
  <!-- Exactly one child generator or combiner -->
</Capitalize>
```

Attributes
----------

- `mode` (optional): The capitalization mode to apply to the output of the child
  element. Available modes are:
  - `FirstUpper` (default): Capitalizes the first letter of the output and lowercases
    the rest (e.g. `Smith`).
  - `AllUpper`: Capitalizes all letters of the output (e.g. `SMITH`).
  - `AllLower`: Lowercases all letters of the output (e.g. `smith`).

Description
-----------

The `<Capitalize>` element changes the capitalization of the output of its child
element according to the specified mode. It should have exactly one child
element, which can be any generator or combiner.

Constraining
------------

This element cannot be constrained and attempting to do so will produce an error.
