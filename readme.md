# Ley
Ley is a markup language designed to let you focus on functionality rather than layout.

# Format
Ley files are made up of `ley lines` which describe functionality. A ley line is of the format

```ley
!name: type {
    contents
}
```

where:
- `name` describes a short piece of text for the ley line, such as a heading or link uri
- `type` describes the functionality itself, ie. a *section* of text, a *link* or an *image*
- `contents` describes further ley lines

Additionally, a ley line may be replaced by direct text.

No whitespace is preserved.
When double quotes (`"`) are used, all text within is escaped. Two double quotes (`""`) may be used to the same effect, preserving the outer quotes and allowing the usage of single quotes within.

## Types
- `link` a link to other content, an html anchor
  - `name` the url
  - `contents` the content to anchor
- `image` an image
  - `name` the image location url
  - `contents` the image alt text
- `section` a section of text
  - `name` the heading of the section
  - `contents` the nested contents