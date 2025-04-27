.macro testmacro a b
    CLS
    LD $a, $b
.endmacro

RET

testmacro V0, 62

LD V0, 22
LD V1, 6
DRW V0, V1, 15

testmacro DT, V0
