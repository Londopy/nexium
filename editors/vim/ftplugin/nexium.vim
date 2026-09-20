" Vim filetype plugin for Nexium
if exists("b:did_ftplugin")
  finish
endif
let b:did_ftplugin = 1

setlocal commentstring=//\ %s
setlocal comments=:///,://!,://
setlocal formatoptions-=t formatoptions+=croql
setlocal expandtab shiftwidth=4 softtabstop=4 tabstop=4
setlocal suffixesadd=.nx

" :NxRun, :NxTest, :NxCheck, :NxFmt on the current file
command! -buffer NxRun   execute '!nx run ' . shellescape(expand('%'))
command! -buffer NxTest  execute '!nx test ' . shellescape(expand('%'))
command! -buffer NxCheck execute '!nx check ' . shellescape(expand('%'))
command! -buffer NxFmt   execute 'silent !nx fmt ' . shellescape(expand('%')) | edit

" `nx check` as the compiler for :make
setlocal makeprg=nx\ check\ %
setlocal errorformat=error:\ %m,%*[\ ]-->\ %f:%l:%c

let b:undo_ftplugin = "setl cms< com< fo< et< sw< sts< ts< sua< mp< efm<"
