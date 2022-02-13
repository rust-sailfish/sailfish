" Sailfish syntax
" Language:	Sailfish template language
" Maintainer: Ryohei Machida <orcinus4627@gmail.com>
" License: MIT

runtime! syntax/html.vim
unlet b:current_syntax

syn include @rustSyntax syntax/rust.vim

syn region sailfishCodeBlock matchgroup=sailfishTag start=/<%/ keepend end=/%>/ contains=@rustSyntax
syn region sailfishBufferBlock matchgroup=sailfishTag start=/<%=/ keepend end=/%>/ contains=@rustSyntax
syn region sailfishCommentBlock start=/<%#/ end=/%>/

" Redefine htmlTag so that it can contain jspExpr
syn clear htmlString
syn region  htmlString   contained start=+"+ end=+"+ contains=htmlSpecialChar,javaScriptExpression,@htmlPreproc,sailfishCodeBlock,sailfishBufferBlock
syn region  htmlString   contained start=+'+ end=+'+ contains=htmlSpecialChar,javaScriptExpression,@htmlPreproc,sailfishCodeBlock,sailfishBufferBlock

syn clear htmlTag
syn region htmlTag start=+<[^/%]+ end=+>+ fold contains=htmlTagN,htmlString,htmlArg,htmlValue,htmlTagError,htmlEvent,htmlCssDefinition,@htmlPreproc,@htmlArgCluster,sailfishBufferBlock

hi default link sailfishTag htmlPreProc
hi default link sailfishCommentBlock htmlComment

let b:current_syntax = "sailfish"
