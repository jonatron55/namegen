`<Join>` element
================

```xml
<Join [id="string"]
      [sep="string"]>
  <!-- Two or more child generators or combiners -->
  <Reject>
    <!-- Optional list of combinations to reject, separated by whitespace -->
  </Reject>
</Join>
```

Attributes
----------

- `sep` (optional): A string to intersperse between the outputs of the child
  elements. If not specified, then the outputs will be concatenated with no
  separator.

Description
-----------

The `<Join>` element concatenates (joins together) the outputs of its child
elements, optionally with a separator character in between. It can have two or
more child elements, which can be any generators or combiners.

An optional `<Reject>` element can be included as a child of `<Join>`. If
present, any combination of outputs from the child elements that matches one of
the strings in the `<Reject>` element will be rejected and regenerated.

Constraining
------------

This element cannot be constrained and attempting to do so will produce an error.
