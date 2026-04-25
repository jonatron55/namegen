Configuration file format
=========================

Name generator configuration files are written in XML with a root element of
`<NameGen>`. An XML header is optional but recommended and it is recommended to
save the file using UTF-8 encoding with the extension `.xml`.

The configuration defines a tree of either generators or combiners of different
types. This tree should have a single root generator or combiner, which,
depending on the type, may have one or more children. The following types are
available:

| Type                          | Usage     | Description                                                                      |
| ----------------------------- | --------- | -------------------------------------------------------------------------------- |
| [`<Markov>`](#markov)         | Generator | Generates words based on a Markov chain built from a list of words.              |
| [`<Number>`](#number)         | Generator | Selects a random number from a range with a variety of formatting options.       |
| [`<Words>`](#words)           | Generator | Selects a random word from a list.                                               |
| [`<Capitalize>`](#capitalize) | Combiner  | Changes the capitalization of the output of its child.                           |
| [`<Concat>`](#concat)         | Combiner  | Connects the output of two or more children together with a separator character. |
| [`<Option>`](#option)         | Combiner  | Runs its child only a certain percentage of the time.                            |
| [`<Repeat>`](#repeat)         | Combiner  | Runs its child a random number of times within a specified range.                |
| [`<Switch>`](#switch)         | Combiner  | Randomly selects only one of its children to run.                                |

Elements
--------

### `<Markov>` ###

```xml
<Markov [target_len="integer"]
        [cutoff_len="integer"]
        [uniform="boolean"]
        [reject_training="boolean"]>
  ...
</Markov>
```

#### Attributes ####

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

#### Description ####

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
on `/`. This requires the user to have some intuition about the structure of the
and place the split characters accordingly.

```xml
<SspTokenizer>
  <Class rank="5">...</Class>
  <Class rank="4">...</Class>
  <Class rank="3">...</Class>
  <Class rank="2">...</Class>
  <Class rank="1">...</Class>
</SspTokenizer>
```

The Sonority Sequencing Principle tokenizer is the most sophisticated option. It
breaks words into their syllable-like based on phonetic rules. This requires
some understanding of the phonetics of the  language being trained on, but it
can produce more natural-sounding results.

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

### `<Number>` ###

```xml
<Number [min="integer"]
        [max="integer"]
        [style="string"] />
```

#### Attributes ####

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

#### Description ####

The `<Number>` generator selects a random number from the specified range and
formats it according to the specified style. The range is inclusive of both `min`
and `max`. If `min` is greater than `max`, then a parse error occurs.

### `<Words>` ###

```xml
<Words>
  ...
</Words>
```

#### Description ####

The `<Words>` generator selects a random word from a list. The element should be
populated with a list of words to select from, separated by whitespace.


### `<Capitalize>` ###

```xml
<Capitalize [mode="string"]>
  ...
</Capitalize>
```

#### Attributes ####

- `mode` (optional): The capitalization mode to apply to the output of the child
  element. Available modes are:
  - `FirstUpper` (default): Capitalizes the first letter of the output and lowercases
    the rest (e.g. `Smith`).
  - `AllUpper`: Capitalizes all letters of the output (e.g. `SMITH`).
  - `AllLower`: Lowercases all letters of the output (e.g. `smith`).

#### Description ####

The `<Capitalize>` combiner changes the capitalization of the output of its
child element according to the specified mode. It should have exactly one child
element, which can be any generator or combiner.

### `<Concat>` ###

```xml
<Concat [joiner="string"]>
```

#### Attributes ####

- `joiner` (optional): A single character to intersperse between the outputs of
  the child elements. If not specified, then the outputs will be concatenated
  with no separator.

#### Description ####

The `<Concat>` element concatenates (joins together) the outputs of its child
elements, optionally with a separator character in between. It can have two or
more child elements, which can be any generators or combiners.

### `<Option>` ###

```xml
<Option probability="real">
  ...
</Option>
```

#### Attributes ####

- `probability` (required): A number between 0.0 and 1.0 that specifies the
  probability that the child element will run. An error occurs if this attribute
  is not between 0.0 and 1.0.

#### Description ####

The `<Option>` element runs its child element only a certain percentage of the
time, as determined by the `probability` attribute. It should have exactly one
child element, which can be any generator or combiner.

### `<Repeat>` ###

```xml
<Repeat [min="integer"] [max="integer"]>
  ...
</Repeat>
```

#### Attributes ####

- `min` (optional): The minimum number of times to run the child element. Default
  value is `1`.
- `max` (optional): The maximum number of times to run the child element. Default
  value is `2`.

#### Description ####

The `<Repeat>` element runs its child element a random number of times within
the specified range. It should be placed in a [`<Concat>`](#concat) in order to
to be effective. The range is inclusive of both `min` and `max`. If `min` is
greater than `max`, then a parse error occurs. It should have exactly one child
element, which can be any generator or combiner.

### `<Switch>` ###

```xml
<Switch>
  ...
</Switch>
```

#### Description ####

The `<Switch>` element randomly selects only one of its child elements to run.
It can have two or more child elements, which can be any generators or
combiners. It selects from its children with equal likelihood.
