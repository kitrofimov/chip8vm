CLS
mainloop:
    CALL new_sprite
    CALL draw_screen
    LD V1, 60
    LD DT, V1
    CALL wait_dt
    JP mainloop

draw_column:
    LD V1, 0
    draw_column_loop:
        DRW V0, V1, 15
        ADD V1, 15
        SE V1, 45
        JP draw_column_loop
    RET

draw_screen:
    LD V0, 0
    draw_screen_loop:
        CALL draw_column
        ADD V0, 8
        SE V0, 64
        JP draw_screen_loop
    RET

new_sprite:
    LD I, registers
    LD [I], VF
    RND V0, 0xFF
    RND V1, 0xFF
    RND V2, 0xFF
    RND V3, 0xFF
    RND V4, 0xFF
    RND V5, 0xFF
    RND V6, 0xFF
    RND V7, 0xFF
    RND V8, 0xFF
    RND V9, 0xFF
    RND VA, 0xFF
    RND VB, 0xFF
    RND VC, 0xFF
    RND VD, 0xFF
    RND VE, 0xFF
    RND VF, 0xFF
    LD I, sprite
    LD [I], VF
    LD I, registers
    LD VF, [I]
    LD I, sprite
    RET

wait_dt:
    LD V1, DT
    SE V1, 0
    JP wait_dt
    RET

sprite:
.space 16

registers:
.space 15
