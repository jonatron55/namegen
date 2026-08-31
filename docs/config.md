Configuration file format
=========================

The name generator application combines a variety of generator types to produce
random names. These generators are created using an XML configuration file that
defines the structure of the generator tree.

The root element of the configuration file must be `<NameGen>`. An XML header is
optional but recommended. The file is best saved using UTF-8 encoding with the
extension `.xml`, but the application will make a best effort to parse other
encodings and extensions.

A schema file is provided in the repository at [`/configs/namegen.xsd`] and can
optionally be added to `<NameGen>` as an `xsi:schemaLocation` attribute to
enable validation in compatible editors.

Example configuration files are provided in the [`/configs/`] directory. A
typical configuration file follows this structure:

```xml
<?xml version="1.0" encoding="UTF-8"?>

<!--
  Root element with a schema declaration.
  The xmlns:xsi and xsi:noNamespaceSchemaLocation attributes are optional.
-->
<NameGen
  xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
  xsi:noNamespaceSchemaLocation="namegen.xsd">
  <!-- If you want to provide a description, it should appear first. -->
  <Description display_name="Example Generator">
    <!--
      If any of your generators have an ID, you can provide a display name for
      it here.
    -->
    <Param id="firstname" display_name="First name" />
    <Param id="surname" display_name="Surname" />

    <!--
      The rest of the description is plain text, and provides general
      information about the configuration.
    -->
    An example configuration file that generates a random first name and
    surname, separated by a space.
  </Description>

  <!-- There must be a single root generator or combiner element. -->
  <Join separator=" ">
    <!--
      These generators include an ID, which has a display name provided in the
      description. The ID can be used as a target for constraints.
    -->
    <Words id="firstname">
      Alice Bob Eve Jane John
    </Words>
    <Words id="surname">
      Brown Johnson Jones Smith Williams
    </Words>
  </Join>
</NameGen>
```

Elements
--------

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

[`/configs/`]: /configs/
[`/configs/namegen.xsd`]: /configs/namegen.xsd
[`<Capitalize>`]: ./config-capitalize.md
[`<Join>`]: ./config-join.md
[`<Literal>`]: ./config-literal.md
[`<Markov>`]: ./config-markov.md
[`<Match>`]: ./config-match.md
[`<Number>`]: ./config-number.md
[`<Option>`]: ./config-option.md
[`<Repeat>`]: ./config-repeat.md
[`<Switch>`]: ./config-switch.md
[`<Words>`]: ./config-words.md

