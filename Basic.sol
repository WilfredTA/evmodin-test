pragma solidity ^0.8.6;


contract Basic {
    bytes32 public myval;

    function setUp(bytes32 val) public {
        myval = val;
    }

    function testVal() public {
        require(myval == "x", "myval not x");
    }

}