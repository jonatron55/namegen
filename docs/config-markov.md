`<Markov>` element
==================

```xml
<Markov [id="string"]
        [target_len="integer"]
        [cutoff_len="integer"]
        [uniform="boolean"]
        [reject_training="boolean"]>

  <!-- Optional tokenizer. Exactly one of: -->
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

Attributes
----------

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

Description
-----------

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

### Input tokenization ###

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

Constraining
------------

The Markov generator can be constrained by providing a prefix string. The
generator will break the prefix into tokens using the configured tokenizer and
match only states that are consistent with those tokens until the prefix is
exhausted. In simpler terms, this forces the output to start with the given
prefix.

If the prefix cannot be matched after 100 attempts, then the generator aborts.
