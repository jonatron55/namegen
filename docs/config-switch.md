`<Switch>` element
==================

```xml
<Switch [id="string"]>
  <!-- Two or more child generators or combiners -->
</Switch>
```

Description
-----------

The `<Switch>` element randomly selects only one of its child elements to run.
It can have two or more child elements, which can be any generators or
combiners. It selects from its children with equal likelihood.

Constraining
------------

The `<Switch>` element can be constrained by specifying the index of a
particular child element to select. Children are indexed starting from `0` in
the order they appear in the configuration file. The constraint should be a
string that can be parsed into an integer. If the constraint cannot be parsed
into an integer, or if the parsed integer exceeds the number of child elements,
then an error occurs.
