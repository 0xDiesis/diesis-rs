import copy
import unittest
from typegen_inputs import order_overloads, parameter_type


class OverloadOrderTests(unittest.TestCase):
    def test_selector_order_preserves_aliases_and_original_artifact(self):
        # Signature sorting gives the opposite order for these resolver methods.
        short = {"type": "function", "name": "setAddr", "inputs": [{"type": "bytes32"}, {"type": "address"}]}
        long = {"type": "function", "name": "setAddr", "inputs": [{"type": "bytes32"}, {"type": "uint256"}, {"type": "bytes"}]}
        artifact = {"abi": [short, long], "rawMetadata": "exact compiler bytes", "methodIdentifiers": {"setAddr(bytes32,address)": "d5fa2b00", "setAddr(bytes32,uint256,bytes)": "8b95dd71"}}
        original = copy.deepcopy(artifact)
        ordered = order_overloads(artifact)
        self.assertEqual(ordered["abi"], [long, short])
        self.assertEqual(artifact, original)
        self.assertEqual(ordered["rawMetadata"], artifact["rawMetadata"])
        self.assertEqual(order_overloads(ordered), ordered)
        for invalid in ({}, {"setAddr(bytes32,address)": "oops"}):
            artifact["methodIdentifiers"] = invalid
            with self.assertRaises(ValueError):
                order_overloads(artifact)

    def test_nested_tuple_array_signature(self):
        self.assertEqual(parameter_type({"type": "tuple[][2]", "components": [{"type": "address"}, {"type": "tuple", "components": [{"type": "uint256"}]}]}), "(address,(uint256))[][2]")


if __name__ == "__main__":
    unittest.main()
