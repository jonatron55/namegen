`<Match>` element
==================

```xml
<Match [id="string"]>
  <!-- Exactly one child generator or combiner -->

  <Case expr="regex">
    <!-- Exactly one child generator or combiner -->
  </Case>

  <!-- Additional <Case> elements as needed -->

  <!-- Optional fallback if no case is matched -->
  <Default>
    <!-- Exactly one child generator or combiner -->
  </Default>
</Match>
```

Description
-----------

The `<Match>` element runs its child generator, and then selects one of `<Case>`
elements that matches the output of the child generator using regular expression
matching. If a match is found, the corresponding child generator of the `<Case>`
element runs. If no match is found and a `<Default>` element is present, then
the child generator of the `<Default>` element runs. This element is best placed
in a [`<Join>`] in order to combine the output of the child generator with the
output of the selected case.


Constraining
------------

This element cannot be constrained and attempting to do so will produce an
error.

[`<Join>`]: ./config-join.md
