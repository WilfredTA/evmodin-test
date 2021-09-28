pragma solidity ^0.8.6;


contract Basic {
    uint public myval;
    uint public otherval;
    constructor() public {
        otherval = 1;
    }
    function setVal(uint val) public {
        myval = val;
    }

//    function testVal() public {
//        require(myval == "x", "myval not x");
//    }

}



contract BasicCreate {
    Basic public basic;

    function setUp() public {
        basic = new Basic();
    }

//    function testVal() public {
//        require(myval == "x", "myval not x");
//    }

}