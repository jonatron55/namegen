Configuration file format
=========================

Name generator configuration files are written in XML with a root element of
`<NameGen>`. An XML header is optional but recommended and it is recommended to
save the file using UTF-8 encoding with the extension `.xml`.

The configuration defines a tree of either generators or combiners of different
types. This tree should have a single root generator or combiner, which,
depending on the type, may have one or more children. The following types are
available:

| Type             | Usage     | Description                                                                           |
| ---------------- | --------- | ------------------------------------------------------------------------------------- |
| [`<Literal>`]    | Generator | Inserts a literal, nonrandom string  into the output.                                 |
| [`<Markov>`]     | Generator | Generates words based on a Markov chain built from a list of words.                   |
| [`<Number>`]     | Generator | Selects a random number from a range with a variety of formatting options.            |
| [`<Words>`]      | Generator | Selects a random word from a list.                                                    |
| [`<Capitalize>`] | Combiner  | Changes the capitalization of the output of its child.                                |
| [`<Join>`]       | Combiner  | Join the output of two or more children together with am optional separator.          |
| [`<Option>`]     | Combiner  | Runs its child only a certain percentage of the time.                                 |
| [`<Repeat>`]     | Combiner  | Runs its child a random number of times within a specified range.                     |
| [`<Switch>`]     | Combiner  | Randomly selects only one of its children to run.                                     |
| [`<Match>`]      | Combiner  | Runs a child generator, then selects another child generator to run based its output. |

Element IDs and constraints
---------------------------

Each element may have an optional `id` attribute, which allows it to be used as
a target for constraints using the `--constrain` option. The behavior of
constraints differs based on the type of generator and is detailed in the
documentation for each generator type. Ids do not need to be unique, and it is
often desirable to reuse the same id for children of [`<Switch>`] and
[`<Option>`] elements. Not all elements support constraints, however it is not
an error to provide an id for an element that does not support constraints. IDs
may be any string, but it is recommended to use only alphanumeric characters,
hyphens, and underscores.

[`<Literal>`]: ./config-literal.md
[`<Markov>`]: ./config-markov.md
[`<Number>`]: ./config-number.md
[`<Words>`]: ./config-words.md
[`<Capitalize>`]: ./config-capitalize.md
[`<Join>`]: ./config-join.md
[`<Option>`]: ./config-option.md
[`<Repeat>`]: ./config-repeat.md
[`<Switch>`]: ./config-switch.md
[`<Match>`]: ./config-match.md
