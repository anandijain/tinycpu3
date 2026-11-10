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

    # One reset edge, then four fetch edges.
    for _ in range(20):
        await Timer(5, "ns")
        clk.value = 1
        await Timer(5, "ns")
        clk.value = 0
