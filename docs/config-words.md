`<Words>` element
=================

```xml
<Words [id="string"]>
  <!-- Words separated by whitespace -->
</Words>
```

Description
-----------

The `<Words>` generator selects a random word from a list. The element should be
populated with a list of words to select from, separated by whitespace.

Constraining
------------

The `<Words>` generator can be constrained by providing a prefix string. If this
is done, then the generator will only select from words that start with the
given prefix. If no words match the prefix, an error occurs.
