pragma solidity ^0.8.6;

contract Callee {
    bytes32 public greeting;

    function setGreeting(bytes32 _greeting) public {
        greeting = _greeting;
    }

    function testGreeting() public {
        require(greeting == bytes32("hello_callee"), "greeting not hello");
    }

    function getExpected() public returns (bytes32) {
        return bytes32("hello");
    }

}
