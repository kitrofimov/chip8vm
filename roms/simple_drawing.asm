CLS
LD I, sprite

LD V0, 0x01
LD V1, 0x10
DRW V0, V1, 16

LD V0, 0x3D
LD V1, 0x1D
DRW V0, V1, 16

loop:
JP loop

sprite:
.fill 16, 0xFF