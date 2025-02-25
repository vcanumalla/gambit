// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >0.7.0;
pragma experimental ABIEncoderV2;

contract UnaryOperatorExample {
    function myBitwiseNeg(uint256 x) public pure returns (uint256) {
	return ~ x;
    }

    function myPrefixIncr(uint256 x) public pure returns (uint256) {
	return ++x;
    }

    function myPrefixDecr(uint256 x) public pure returns (uint256) {
	return --x;
    }

    function mySuffixIncr(uint256 x) public pure returns (uint256) {
	x++;
	return x;
    }

    function mySuffixDecr(uint256 x) public pure returns (uint256) {
	x--;
	return x;
    }
}
