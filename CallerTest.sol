pragma solidity ^0.8.6;

interface Callee {
    function setGreeting(bytes32) external;
    function greeting() external view returns (bytes32);
}
contract Caller {
    address public callee;

    function setCalleeTarget(address _target) public returns (address) {
        callee = _target;
        return callee;
    }

    function callCalleeGreeting() public returns (bytes32) {
        Callee _callee = Callee(callee);
        bytes32 _greeting = _callee.greeting();
        return _greeting;
    }

    function callCalleeSetGreeting() public {
        Callee _callee = Callee(callee);
        _callee.setGreeting(bytes32("hello_callee"));
    }

}
