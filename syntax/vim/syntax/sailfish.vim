runtime! syntax/html.vim
unlet b:current_syntax

syn include @rustSyntax syntax/rust.vim

syn region sailfishCodeBlock matchgroup=sailfishTag start=/<%/ keepend end=/%>/ contains=@rustSyntax
syn region sailfishBufferBlock matchgroup=sailfishTag start=/<%=/ keepend end=/%>/ contains=@rustSyntax
syn region sailfishCommentBlock start=/<%#/ end=/%>/

" Redefine htmlTag so that it can contain jspExpr
syn clear htmlTag
syn region htmlTag start=+<[^/%]+ end=+>+ fold contains=htmlTagN,htmlString,htmlArg,htmlValue,htmlTagError,htmlEvent,htmlCssDefinition,@htmlPreproc,@htmlArgCluster,sailfishBufferBlock

hi default link sailfishTag htmlTag
hi default link sailfishCommentBlock htmlComment

let b:current_syntax = "sailfish"
