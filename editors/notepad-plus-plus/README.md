# Nexium for Notepad++

`nexium.udl.xml` is a User Defined Language: keywords, control flow,
constants, types, builtins, effects (`!allocates`), intrinsics
(`@typeName`), binary pattern modifiers, numbers with `0x`/`0b`/`0o`
prefixes and `_` separators, strings and chars with escapes, `//` comments,
and folding on braces.

## Install

Copy the file into `%AppData%\Notepad++\userDefineLangs\` (create the
directory if it is not there) and restart Notepad++. Every `.nx` file then
opens as Nexium; the language is also under *Language → Nexium*.

Or import it: *Language → User Defined Language → Define your language... →
Import...* and choose the file.

The colors are set in the file (a UDL does not follow the theme); *Define
your language...* edits them.

## Running nx

*Run → Run...* (F5) with

```
cmd /k cd /d "$(CURRENT_DIRECTORY)" && nx run "$(FULL_CURRENT_PATH)"
```

*Save...* in that dialog keeps it under the Run menu with a shortcut. The
NppExec plugin does the same with a console inside Notepad++.
