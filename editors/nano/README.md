# Nexium for nano

`nexium.nanorc` colors keywords, types, calls, effects, intrinsics, error
names, variants, labels, numbers, strings, binary pattern modifiers and
comments, and sets `//` for `M-3` (comment) and four spaces for Tab.

## Install

```sh
mkdir -p ~/.nano
cp nexium.nanorc ~/.nano/
echo 'include "~/.nano/nexium.nanorc"' >> ~/.nanorc
```

Or, system wide, copy it next to the other definitions in
`/usr/share/nano/` and `include "/usr/share/nano/nexium.nanorc"` in
`/etc/nanorc`.

nano 5 or later (the `comment`, `tabgives` and `\<` `\>` word boundaries).
