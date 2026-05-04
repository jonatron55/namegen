
`<Literal>` element
===================

```xml
<Literal [id="string"]
         value="string"/>
```

Attributes
----------

- `value` (required): The literal string to insert into the output. Whitespace
  is preserved, including leading and trailing whitespace.

Description
-----------

This inserts a nonrandom, literal string into the output. It is useful for
adding fixed components that such as punctuation or whitespace that would be
incorrectly handled by [`<Words>`](#words), which treats whitespace as a
delimiter.

Constraining
------------

This element cannot be constrained and attempting to do so will produce an
error.
