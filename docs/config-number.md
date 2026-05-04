`<Number>` element
==================

```xml
<Number [id="string"]
        [min="integer"]
        [max="integer"]
        [style="string"] />
```

Attributes
----------

- `min` (optional): The minimum number in the range. Default value is `1`.
- `max` (optional): The maximum number in the range. Default value is `99`.
- `style` (optional): The formatting style for the output number. Available styles
  are:
  - `Dec` (default): Standard decimal representation (e.g. `42`).
  - `Hex`: Hexadecimal representation (e.g. `2A`).
  - `HexLower`: Hexadecimal representation with lowercase letters (e.g. `2a`).
  - `Oct`: Octal representation (e.g. `52`).
  - `Bin`: Binary representation (e.g. `101010`).
  - `Roman`: Roman numeral representation (e.g. `XLII`).
  - `RomanLower`: Roman numeral representation with lowercase letters (e.g.
    `xlii`).

Description
-----------

The `<Number>` generator selects a random number from the specified range and
formats it according to the specified style. The range is inclusive of both `min`
and `max`. If `min` is greater than `max`, then a parse error occurs.

Constraining
------------

The `<Number>` generator can be constrained by specifying an exact value to
output. The constraint should be a string that can be parsed into an integer.
If the constraint cannot be parsed into an integer, or if the parsed integer is
outside the specified range, then an error occurs.
