#!/bin/bash

cp ../../target/release/interpreter .
cp ../../target/release/assembler .
cp ../../target/release/disassembler .

cp ../../roms/* .

# https://github.com/paxtonhare/demo-magic
. demo-magic.sh

# Be careful!
export PATH="$PATH:."

clear

pei "assembler random.asm random.ch8"
pei "interpreter random.ch8"
wait
pei "interpreter ibmlogo.ch8"
