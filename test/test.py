# top = main

from spade import SpadeExt
import cocotb
from cocotb.triggers import Timer

@cocotb.test()
async def test_fetch(dut):
    s = SpadeExt(dut)
    clk = dut.clk_i
    datapath = dut.datapath_0
    ram = datapath.ram_0.ram

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