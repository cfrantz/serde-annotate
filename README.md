# Serde-Annotate: Bridging Human and Machine Readability

## Introduction

`serde-annotate` is a serde serializer that can emit documents with
improved human readability in a number of formats.

## Background
### Criticism of Current Formats

All current text serialization formats are awful:

- JSON: No comments, no multi-line strings, no hex literals, trailing-comma
  stupidity with aggregates.
- HJSON: No hex literals; I don't like the triple single quote (`'''`) to
  designate multiline strings.  I'm ambivalent about quoteless strings.
- JSON5: The multiline-string syntax is bad:  the C-like line continuations
  are clumsy.
- YAML: Too loosey-goosey: white-space has meaning and lack of punctuation
  around maps, lists and strings.
- Google Text Proto: undocumented.

At least to my personal tastes, JSON5 and YAML come closest to what I
want: namely support for multi-line strings and hexadecimal literals.

While I like YAML for its minimalism, the use of whitespace to convey
document structure is not to my liking.  Given the use of whitespace to
convey structure, I do like YAML's approach to multiline string blocks:
The magic sequences `|-` or `|+` start a multiline string block which
is then just inserted into the document.

JSON5 is marred by its syntax for multi-line string blocks.  It does,
however permit hex literals and quoteless strings.  While I generally
don't care for quoteless strings, I believe they are sensible when their
use is restricted to keys in a key-value mapping.

To avoid [xkcd 927](https://xkcd.com/927/), I am not going to propose a
new standard.

### Human Readability

All of the current standards are text based and purport to be human-readble.
While it is true that the current formats can be read by a human, their
lack of expressivity means that the emitted documents are not necessarily
readable in practice.

A particular use case I have in mind are diagnostic tools: I want to be
able to write tools which emit outputs useful to human readers.  I _also_
want the outputs to be in a machine readable form so that the tools can be
chained together or used in automated tests.  I do _not_ want to write two
output emitters in the tool: I want to emit a single document which is
comprehensible to both human and machine consumers.

### Rust's `serde` library

I find Rust's `serde` library a joy to use:  It gives the ability to
define arbitrary data structures in Rust's data model and convert those
data structures to a multitude of serialized forms.  `Serde` is like
having [Google Protobuf](https://developers.google.com/protocol-buffers)
built into your programming language.

In achieving combined human and machine readability, I find `serde`
somewhat lacking: I do not have as tight of control over the emitted
forms as I would like.  Specifically, I would like to be able to control
the serializer's choice in numeric bases, the string forms used in the output
(e.g. quoted, unquoted, multi-line block) and to add comments into the
output which can improve human comprehensibility of the emitted document.

### Example

Perhaps this is best demonstrated with a practical example.  It is common
for embedded firmware engineers to have to work with SPI Flash EEPROM chips.
SPI EEPROMs have a table of parameters known as the Serial Flash Discoverable
Parameters (aka SFDP) that can be used to identify their
size, timing characteristics and feature set.

This table starts with the following header (describe as a Rust struct):
```
struct SfdpHeader {
    signature: u32,
    minor: u8,
    major: u8,
    num_param_headers: u8,
    reserved: u8,
}
```

You could imagine a diagnostic program reading this header from an EEPROM
and printing its values:
```
header: {
    signature: 1346651731,
    minor: 6,
    major: 1,
    num_param_headers: 2,
    reserved: 255
}
```

As an expert reading such a table, you might know what each of these values
mean; as an astute novice, you might have a pretty good guess; as a beginner,
you'd have no idea.

Moreover, even as an expert, would you remember the correct decimal value
for the `signature` field?  Would you know that the `num_param_headers`
value of `2` means there are three headers?

Suppose instead the output contained comments with some helpful reminders and
the numbers were expressed in a base more appropriate to their use case:
```
header:
    # Signature value='SFDP' (should be 'SFDP')
    signature: 0x50444653,
    # SFDP version number
    minor: 6,
    major: 1,
    # Number of parameter headers (minus 1)
    num_param_headers: 2,
    # Reserved field should be all ones
    reserved: 0b11111111
```

This second version conveys exactly the same information as the first
version, but is clearly more readable.  The `signature` field is expressed
in hex and the comment includes the ASCII representation of the hex
bytes along with a reminder of what they should be.  The comment for
`num_param_headers` helpfully reminds the reader that the number shown is
one less than the true value.  Lastly, the `reserved` field is represented
in binary and the comment reminds the viewer of what the value should be.

Not only is this document _more_ human readable, it is also a valid YAML
document and can be parsed by any tooling with a YAML parser.

## Enter `serde-annotate`

`serde-annotate` is a serde serializer that can emit more readable documents
in several existing formats including `json`, `json5`, `hjson` and `yaml`.
`serde-annotate` allows you to control the bases used to express integers,
the style of strings used in the output document, the compactness of
certain structures and the comments emitted for each field.

For JSON-style documents, `serde-annotate` allows the programmer to control
the degree of adherence to the JSON specification and how to handle certain
deviations from strict JSON.  These include:
- Whether comments are permitted.
- Whether bare keys are permitted in maps.
- Whether integers are limited to the range -2^53 to +2^53.  By default,
  integers outside of the range are emitted as quoted strings.
- Which integer bases are permitted.  Disallowed bases default to decimal.
- Which integer bases are permitted for literals.  An allowed base not permitted
  to be expressed as a literal is emitted as a quoted string.
- Which style of multi-line strings are permitted.

### Using `serde-annotate`

Serde-annotate includes a procedural macro to annotate your structs and enums
with with your preferred data representation and comments.

```
#[derive(Serialize, Annotate, ...)]
struct SfdpHeader {
    #[annotate(format=hex, comment=_signature())]
    signature: u32,
    #[annotate(comment = "SFDP version number")]
    minor: u8,
    major: u8,
    #[annotate(comment = "Number of parameter headers (minus 1)")]
    num_param_headers: u8,
    #[annotate(format=bin, comment = "Reserved field should be all ones")]
    reserved: u8,
}

impl SfdpHeader {
    pub fn _signature(&self) -> Option<String> { ... }
}
```

You can then use `serde_annotate::serialize()` to serialize your struct
and convert it to your chosen document type:

```
    let doc = serde_annotate::serialize(&sfdp_hdr)?.to_yaml();
    println!("{}", doc);
```

There are predefined document profiles using `to_json`, `to_json5`, `to_hjson`
and `to_yaml`.  The `json` style is rather customizable; for example, the
`json5` style is:

```
    pub fn to_json5(self) -> Json {
        self.to_json()
            .comment(Comment::SlashSlash)
            .literals(&[Base::Hex])
            .multiline(Multiline::Json5)
            .bare_keys(true)
    }
```
