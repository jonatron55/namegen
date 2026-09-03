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

| Type                                    | Usage     | Description                                                                           |
| --------------------------------------- | --------- | ------------------------------------------------------------------------------------- |
| [`<Description>`](#description-element) | Metadata  | Provides a description of the configuration and display names for any IDs.            |
| [`<Literal>`](#literal-element)         | Generator | Inserts a literal, nonrandom string  into the output.                                 |
| [`<Markov>`](#markov-element)           | Generator | Generates words based on a Markov chain built from a list of words.                   |
| [`<Number>`](#number-element)           | Generator | Selects a random number from a range with a variety of formatting options.            |
| [`<Words>`](#words-element)             | Generator | Selects a random word from a list.                                                    |
| [`<Capitalize>`](#capitalize-element)   | Combiner  | Changes the capitalization of the output of its child.                                |
| [`<Join>`](#join-element)               | Combiner  | Join the output of two or more children together with am optional separator.          |
| [`<Match>`](#match-element)             | Combiner  | Runs a child generator, then selects another child generator to run based its output. |
| [`<Option>`](#option-element)           | Combiner  | Runs its child only a certain percentage of the time.                                 |
| [`<Repeat>`](#repeat-element)           | Combiner  | Runs its child a random number of times within a specified range.                     |
| [`<Switch>`](#switch-element)           | Combiner  | Randomly selects only one of its children to run.                                     |

### Element IDs and constraints ###

Each element may have an optional `id` attribute, which allows it to be used as
a target for constraints using the `--constrain` option. The behavior of
constraints differs based on the type of generator and is detailed in the
documentation for each generator type. Ids do not need to be unique, and it is
often desirable to reuse the same id for children of [`<Switch>`] and
[`<Option>`] elements. Not all elements support constraints, however it is not
an error to provide an id for an element that does not support constraints. IDs
may be any string, but it is recommended to use only alphanumeric characters,
hyphens, and underscores.

`<Description>` element
-----------------------

```xml
<Description [display_name="string"]>
  <!-- Optional <Param> children to provide display names for IDs -->
  <Param id="string" display_name="string" />
  <!-- Optional text content to provide a description of the configuration -->
</Description>
```

### Attributes ###

- `display_name` (optional): A human-readable name to display for the
  configuration.

### Description ###

The optional `<Description>` element provides a human-readable description of
the configuration and display names for use in a user interface. If provided, it
should be the first child of the root `<NameGen>` element. User interfaces
should handle the absence of a `<Description>` element gracefully, for example
by using the filename and `id` attributes where display names would normally be
used.

### Parameter display names ###

```xml
<Param id="string" display_name="string" />
```

#### Attributes ####

- `id` (required): The id of a generator or combiner element that can be
  constrained.
- `display_name` (required): A human-readable name to display for the element
  with the given id.

#### Description ####

Within a `<Description>` element, the optional `<Param>` element provides a
display name for a generator or combiner element with the given id. User
interfaces can use these to label input fields for constraints. As discussed in
[Element IDs and constraints](#element-ids-and-constraints), multiple elements
may share the same id. Regardless of how many times the same id is used, only
one `<Param>` element should be provided for that id.

It is possible that not every id used in the configuration will have a
corresponding `<Param>` element and it is also possible that a `<Param>` element
will be provided for an id that is not used. User interfaces should handle both
of these cases gracefully. Neither should be considered an error.

`<Literal>` element
-------------------

```xml
<Literal [id="string"] text="string"/>
```

### Attributes ###

- `text` (required): The literal string to insert into the output. Whitespace
  is preserved, including leading and trailing whitespace.

### Description ###

This inserts a nonrandom, literal string into the output. It is useful for
adding fixed components that such as punctuation or whitespace that would be
incorrectly handled by [`<Words>`](#words), which treats whitespace as a
delimiter.

### Constraining ###

This element cannot be constrained and attempting to do so will produce an
error.

`<Markov>` element
------------------

```xml
<Markov [id="string"]
        [target_len="integer"]
        [cutoff_len="integer"]
        [uniform="boolean"]
        [reject_training="boolean"]>

  <!-- Optional tokenizer. Either absent or exactly one of: -->
  <ChunkTokenizer chunk_len="integer" />
  <SplitTokenizer [split_chars="string"] />
  <SspTokenizer>
    <!-- Optional character classes -->
  </SspTokenizer>

  <!-- Training data separated by whitespace -->

  <Reject>
     <!-- Optional list of words to reject, separated by whitespace -->
  </Reject>
</Markov>
```

### Attributes ###

- `target_len` (optional): The desired minimum length of generated words. If the
  generator reaches a possible halting state before reaching this length when
  other options are available, it will skip halting as a possibility. This does
  not guarantee that generated words will be at least this long, but it prevents
  the generator from halting too early when it has other options.

  If not specified, then there will be no minimum length.

- `cutoff_len` (optional): The desired maximum length of generated words. If the
  generator reaches a possible halting state after reaching this length when it
  will take it regardless of other options. Like with `target_len`, this does is
  not a guarantee of maximum length, but rather a point at which the generator
  will be more likely to halt.

  If not specified, then there will be no maximum length.

- `uniform` (optional): If `true`, then Markov states will be created with equal
  probabilities instead of probabilities based on their frequency in the
  training data. This increases the "temperature" of the generator, making it
  more likely to generate less common outputs.

  Default value is `false`.

- `reject_training` (optional): If `true`, then the generator will reject any
  generated word that is in the training data. This will cause the generator to
  to retry until it generates a word that is not rejected. This requires the
  training data to have a sufficiently high perplexity to allow for generation
  of new words. If more than 100 consecutive rejections occur, then the
  generator aborts.

  Default value is `false`.

### Description ###

The Markov generator creates novel words based on its training data. The
element should be populated with a list of words to train on, separated by
whitespace. If `reject_training` is `false` or not specified, then it is
possible (and likely) that the generator will produce words from the training
as well as novel words.

The training data is case-sensitive, which is usually desirable for name
generation since capitalization can be an important part of the structure of
names (for example, we want to preserve the capitalization "McDonald"). If
case-insensitivity is desired, then the training data should be converted to
either all uppercase or all lowercase and the the `<Markov>` element wrapped
with a [`<Capitalize>`](#capitalize) element to produce the desired
capitalization in the output.

#### Input tokenization ####

A critical step in the creating a Markov generator is breaking the input words
into smaller pieces called "tokens". The choice of tokenizer and its parameters
can have a significant impact on the output of the generator. There are three
tokenizers available:

```xml
<ChunkTokenizer chunk_len="integer" />
```

The simplest option, which breaks words into substrings of a fixed length. This
can be surprisingly effective for chunks of length 3 to 5, especially for made-
up words and languages.

```xml
<SplitTokenizer [split_chars="string"] />
```

The manual option, which breaks words at specified characters. By default, it
splits on `/`. This requires the user to have some intuition about the structure
of the language and place the split characters accordingly.

```xml
<SspTokenizer>
  <!-- Optional character classes with ranks from 1 to 5. -->
  <Class rank="5">...</Class>
  <Class rank="4">...</Class>
  <Class rank="3">...</Class>
  <Class rank="2">...</Class>
  <Class rank="1">...</Class>
</SspTokenizer>
```

The Sonority Sequencing Principle tokenizer is the most sophisticated option. It
breaks words into their syllable-like based on phonetic rules. This requires
some understanding of the phonetics of the language being trained on, but it can
produce more natural-sounding results.

This tokenizer requires that all characters in the training data be assigned
character classes, which are specified as `<Class rank="integer">` children of
the `<SspTokenizer>`. The `rank` attribute determines sonority hierarchy:

| Rank | Use                                                                   |
| ----:| --------------------------------------------------------------------- |
|    5 | Vowels (e.g. `a`, `e`, `i`, `o`, `u`)                                 |
|    4 | Glides (e.g. `w` and `y` when not used as a vowel)                    |
|    3 | Liquids (e.g. `l` and `r`)                                            |
|    2 | Nasals and fricatives (e.g. `m`, `n`, `s`, and `z`)                   |
|    1 | Stops and affricates (e.g. `p`, `t`, and `k`)                         |

If both upper- and lowercase versions of a character are present in the training
data, then both versions should be included in the character classes. Any
character encountered in the training data that is not included in the character
classes will be treated as a token boundary, which correctly handles punctuation
in names like "O'Neill" and "Mary-Jane" but will cause problems if the training
data contains characters alphabetic characters that are not assigned to a class.

If no classes are specified, then a default set of classes will be used for the
Latin alphabet, with ranks assigned for the most common English phonemes (though
they are feasibly applicable to many other languages as well):

```xml
<Class rank="5">aAáÁàÀâÂåÅäÄãÃæÆeEéÉèÈêÊëËiIíÍìÌîÎïÏoOóÓòÒôÔöÖõÕøØuUúÚùÙûÛůŮüÜyYýÝÿŸ</Class>
<Class rank="4">wW</Class>
<Class rank="3">lLrRřŘ</Class>
<Class rank="2">çÇðÐfFhHmMnNňŇñÑsSšŠßvVzZžŽþÞ</Class>
<Class rank="1">bBcCčČdDďĎgGjJkKpPqQtTťŤxX</Class>
```

If no tokenizer is specified, then the default is the `<SspTokenizer>` with the
above character classes.

The `<Markov>` element may also contain a `<Reject>` child element, which
specifies a list of words that should be rejected if generated (regardless of
the `reject_training` setting). This is useful for filtering out undesirable
results. As with `reject_training`, if more than 100 consecutive rejections
occur, then the generator aborts.

### Constraining ###

The Markov generator can be constrained by providing a prefix string. The
generator will break the prefix into tokens using the configured tokenizer and
match only states that are consistent with those tokens until the prefix is
exhausted. In simpler terms, this forces the output to start with the given
prefix.

If the prefix cannot be matched after a certain number of attempts, then the
generator aborts.

`<Number>` element
------------------

```xml
<Number [id="string"] [min="integer"] [max="integer"] [style="string"] />
```

### Attributes ###

- `min` (optional): The minimum number in the range. Default value is `1`.
- `max` (optional): The maximum number in the range. Default value is `99`.
- `style` (optional): The formatting style for the output number. Available
  styles are:
  - `Dec` (default): Standard decimal representation (e.g. `42`).
  - `Hex`: Hexadecimal representation (e.g. `2A`).
  - `HexLower`: Hexadecimal representation with lowercase letters (e.g. `2a`).
  - `Oct`: Octal representation (e.g. `52`).
  - `Bin`: Binary representation (e.g. `101010`).
  - `Roman`: Roman numeral representation (e.g. `XLII`).
  - `RomanLower`: Roman numeral representation with lowercase letters (e.g.
    `xlii`).

### Description ###

The `<Number>` generator selects a random number from the specified range and
formats it according to the specified style. The range is inclusive of both
`min` and `max`. If `min` is greater than `max`, then a parse error occurs.

### Constraining ###

The `<Number>` generator can be constrained by specifying an exact value to
output. The constraint should be a string that can be parsed into an integer.
If the constraint cannot be parsed into an integer, or if the parsed integer is
outside the specified range, then an error occurs.

`<Words>` element
-----------------

```xml
<Words [id="string"]>
  <!-- Words separated by whitespace -->
</Words>
```

### Description ###

The `<Words>` generator selects a random word from a list. The element should be
populated with a list of words to select from, separated by whitespace.

### Constraining ###

The `<Words>` generator can be constrained by providing a prefix string. If this
is done, then the generator will only select from words that start with the
given prefix. If no words match the prefix, an error occurs.

`<Capitalize>` element
----------------------

```xml
<Capitalize [id="string"] [mode="string"]>
  <!-- Exactly one child generator or combiner -->
</Capitalize>
```

### Attributes ###

- `mode` (optional): The capitalization mode to apply to the output of the child
  element. Available modes are:
  - `FirstUpper` (default): Capitalizes the first letter of the output and
    lowercases the rest (e.g. `Smith`).
  - `AllUpper`: Capitalizes all letters of the output (e.g. `SMITH`).
  - `AllLower`: Lowercases all letters of the output (e.g. `smith`).

### Description ###

The `<Capitalize>` element changes the capitalization of the output of its child
element according to the specified mode. It should have exactly one child
element, which can be any generator or combiner.

### Constraining ###

This element cannot be constrained and attempting to do so will produce an
error.

`<Join>` element
----------------

```xml
<Join [id="string"] [sep="string"]>
  <!-- Two or more child generators or combiners -->
  <Reject>
    <!-- Optional list of combinations to reject, separated by whitespace -->
  </Reject>
</Join>
```

### Attributes ###

- `sep` (optional): A string to intersperse between the outputs of the child
  elements. If not specified, then the outputs will be concatenated with no
  separator.

### Description ###

The `<Join>` element concatenates (joins together) the outputs of its child
elements, optionally with a separator character in between. It can have two or
more child elements, which can be any generators or combiners.

An optional `<Reject>` element can be included as a child of `<Join>`. If
present, any combination of outputs from the child elements that matches one of
the strings in the `<Reject>` element will be rejected and regenerated.

### Constraining ###

This element cannot be constrained and attempting to do so will produce an
error.

`<Match>` element
------------------

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

### Description ###

The `<Match>` element runs its child generator, and then selects one of `<Case>`
elements that matches the output of the child generator using regular expression
matching. If a match is found, the corresponding child generator of the `<Case>`
element runs. If no match is found and a `<Default>` element is present, then
the child generator of the `<Default>` element runs. This element is best placed
in a [`<Join>`](#join-element) in order to combine the output of the child
generator with the output of the selected case.

### Constraining ###

This element cannot be constrained and attempting to do so will produce an
error.

`<Option>` element
------------------

```xml
<Option [id="string"] probability="real">
  <!-- Exactly one child generator or combiner -->
</Option>
```

### Attributes ###

- `probability` (required): A number between 0.0 and 1.0 that specifies the
  probability that the child element will run. An error occurs if this attribute
  is not between 0.0 and 1.0.

### Description ###

The `<Option>` element runs its child element only a certain percentage of the
time, as determined by the `probability` attribute. It should have exactly one
child element, which can be any generator or combiner.

### Constraining ###

The `<Option>` element can be constrained with the values `true` or `false`. If
a constraint is present, it overrides the specified probability, and the child
is either always run (if `true`) or never run (if `false`).

`<Repeat>` element
------------------

```xml
<Repeat [id="string"] [min="integer"] [max="integer"]>
  <!-- Exactly one child generator or combiner -->
</Repeat>
```

### Attributes ###

- `min` (optional): The minimum number of times to run the child element.
  Default value is `1`.
- `max` (optional): The maximum number of times to run the child element.
  Default value is `2`.

### Description ###

The `<Repeat>` element runs its child element a random number of times within
the specified range. It should be placed in a [`<Join>`](#join-element) in order
to be effective. The range is inclusive of both `min` and `max`. If `min` is
greater than `max`, then a parse error occurs. It should have exactly one child
element, which can be any generator or combiner.


### Constraining ###

The `<Repeat>` element can be constrained by specifying an exact number of
repetitions. The constraint should be a string that can be parsed into an
integer. If the constraint cannot be parsed into an integer, or if the parsed
integer is outside the specified range, then an error occurs.

`<Switch>` element
------------------

```xml
<Switch [id="string"]>
  <!-- Two or more child generators or combiners -->
</Switch>
```

### Description ###

The `<Switch>` element randomly selects only one of its child elements to run.
It can have two or more child elements, which can be any generators or
combiners. It selects from its children with equal likelihood.

### Constraining ###

The `<Switch>` element can be constrained by specifying the index of a
particular child element to select. Children are indexed starting from `0` in
the order they appear in the configuration file. The constraint should be a
string that can be parsed into an integer. If the constraint cannot be parsed
into an integer, or if the parsed integer exceeds the number of child elements,
then an error occurs.

[`/configs/`]: https://github.com/jonatron55/namegen/configs/
[`/configs/namegen.xsd`]: https://github.com/jonatron55/namegen/configs/namegen.xsd
