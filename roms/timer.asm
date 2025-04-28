CLS
LD V9, 0
mainloop:
    CALL draw

    LD V0, 10
    LD DT, V0
    CALL wait_dt

    CALL draw  ; erase everything
    ADD V9, 1
    JP mainloop

draw:
    LD VA, 0
    LD VB, 0
    LD I, bcd
    LD B, V9
    
    LD I, hundereds
    LD V0, [I]
    LD F, V0
    DRW VA, VB, 5
    ADD VA, 5

    LD I, tens
    LD V0, [I]
    LD F, V0
    DRW VA, VB, 5
    ADD VA, 5

    LD I, ones 
    LD V0, [I]
    LD F, V0
    DRW VA, VB, 5

    RET

wait_dt:
    LD V1, DT
    SE V1, 0
    JP wait_dt
    RET

bcd:
    hundereds:
        .byte 0x00
    tens:
        .byte 0x00
    ones:
        .byte 0x00
