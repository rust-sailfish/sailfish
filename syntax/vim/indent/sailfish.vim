" Vim indent file
" Language:	Sailfish template language
" Maintainer: Ryohei Machida <orcinus4627@gmail.com>
" Last Change:	2020 May 29

" Only load this indent file when no other was loaded.
if exists("b:did_indent")
  finish
endif

" Use HTML formatting rules.
runtime! indent/html.vim
