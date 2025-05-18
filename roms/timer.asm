; Counts time on screen, in seconds. Maximum value = 127, then overflow
; V0 = temp register (set to DT, read from BCD)
; V9 = time passed since the start of the program (in seconds)
; VA = x coordinate
; VB = y coordinate

CLS
LD V9, 0
mainloop:
    CALL draw

    LD V0, 60  ; 60 ticks = 1 second
    LD DT, V0
    CALL wait_dt

    CALL draw  ; erase everything
    ADD V9, 1
    JP mainloop

; Draw the value stored in V9 to the screen
draw:
    LD VA, 0
    LD VB, 0
    LD I, bcd
    LD B, V9
    
    LD I, hundereds
    LD V0, [I]
    LD F, V0  ; set I = location of sprite for digit V0
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

; Wait until DT is 0
wait_dt:
    LD V1, DT
    SE V1, 0
    JP wait_dt
    RET

; Reserve space for BCD representation
bcd:
    hundereds:
        .byte 0x00
    tens:
        .byte 0x00
    ones:
        .byte 0x00
