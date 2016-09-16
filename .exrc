" aligning
filetype plugin indent on
set expandtab
set tabstop=4
set shiftwidth=4

" auto setf
autocmd BufNewFile,BufRead *.asm call AssemblySettings()

" auto format on save
autocmd BufWritePost *.asm :silent %s:^\s\{4}:\t:g

function AssemblySettings()
    set expandtab
    set tabstop=4
    set shiftwidth=4
    set syntax=nasm
endfunction
