to launch surfer, run this in a normal wsl bash/fish terminal 
powershell.exe -NoProfile -ExecutionPolicy Bypass -File ./surfer.ps1

to pass args to the cocotb use an env var 
env PROGRAM=asm/progs/collatz.sasm swim -j 1 test

# todo 
* stack/ call ret 
* testing programs
* 


# done
* sasm parsing
* cjmp/ cflags 
* collatz 