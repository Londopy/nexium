" Vim indent file for Nexium: braces open and close blocks, like C
if exists("b:did_indent")
  finish
endif
let b:did_indent = 1

setlocal indentexpr=GetNexiumIndent()
setlocal indentkeys=0{,0},0),!^F,o,O,e
setlocal nosmartindent

function! GetNexiumIndent()
  let lnum = prevnonblank(v:lnum - 1)
  if lnum == 0
    return 0
  endif
  let prev = getline(lnum)
  let ind = indent(lnum)
  " strip comments and strings before counting braces
  let code = substitute(prev, '//.*$', '', '')
  let code = substitute(code, '"\%(\.\|[^"\]\)*"', '', 'g')
  let opens = len(substitute(code, '[^{(\[]', '', 'g'))
  let closes = len(substitute(code, '[^})\]]', '', 'g'))
  if opens > closes
    let ind += shiftwidth()
  endif
  let cur = getline(v:lnum)
  if cur =~ '^\s*[})\]]'
    let ind -= shiftwidth()
  endif
  return ind < 0 ? 0 : ind
endfunction
