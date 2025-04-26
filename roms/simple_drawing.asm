CLS
LD I, sprite

LD V0, 22
LD V1, 6
DRW V0, V1, 15

loop:
JP loop

.space 5
.fill 3, 0xFF
.space 5
; .text "Hello, world!"
; .space 5

sprite:
.byte 0b11110000
.byte 0b00001111
.word 0b1111000000001111
.word 0b1111000000001111
.word 0b1111000000001111
.word 0b1111000000001111
.word 0b1111000000001111
.word 0b1111000000001111
.word 0b1111000000001111
