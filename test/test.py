# top = main

from spade import SpadeExt
import cocotb
from cocotb.triggers import Timer
import os
import subprocess
from pathlib import Path

@cocotb.test()
async def test_fetch(dut):
    s = SpadeExt(dut)
    clk = dut.clk_i
    root = Path(os.environ["SWIM_ROOT"])
    source = (root / os.environ["PROGRAM"]).resolve()
    output = Path.cwd() / "program.ram"  # This test's build directory.

    subprocess.run(
        [
            "cargo", "run", "--quiet", "-p", "asm", "--",
            "asm",
            "--input", str(source),
            "--output", str(output),
        ],
        cwd=root,
        check=True,
    )

    image = output.read_bytes()
    ram = dut.datapath_0.ram_0.ram
    assert len(image) <= len(ram), "Program exceeds RAM capacity"

    clk.value = 0
    await Timer(1, unit="ns")  # Let HDL initialization finish.

    for address in range(len(ram)):
        ram[address].value = image[address] if address < len(image) else 0

    await Timer(1, unit="ns")  # Let memory writes settle.



    clk.value = 0
    step = 0
    hlt = False
    # One reset edge, then four fetch edges.
    while not hlt and (step < 1e3):
        await Timer(5, "ns")
        clk.value = 1
        await Timer(5, "ns")
        hlt = s.o.control.hlt.is_eq(True)
        
        clk.value = 0
        step += 1 
    print(f"num steps run: {step}")
    
    # assert 