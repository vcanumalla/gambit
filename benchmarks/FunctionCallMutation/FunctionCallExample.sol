// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >0.7.0;
pragma experimental ABIEncoderV2;

contract FunctionCallExample {
    function myAddition(uint256 x, uint256 y) public pure returns (uint256) {
	return x + y;
    }

    function myOtherAddition(uint256 x, uint256 y) public pure returns (uint256) {
	return myAddition(x, y);
    }
}
