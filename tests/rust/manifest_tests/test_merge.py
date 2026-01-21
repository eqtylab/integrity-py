import json
import unittest

from eqty_sdk._rust import manifest


class TestManifestMerge(unittest.TestCase):
    def test_merge_empty_manifests(self):
        """Test merging two empty manifests."""
        manifest_a = json.dumps({"statements": {}, "blobs": {}, "version": "2.0", "contexts": {}})

        manifest_b = json.dumps({"statements": {}, "blobs": {}, "version": "2.0", "contexts": {}})

        result = manifest.merge(manifest_a, manifest_b)

        self.assertIsInstance(result, str)
        merged = json.loads(result)
        self.assertIn("statements", merged)
        self.assertEqual(len(merged["statements"]), 0)
        self.assertIn("blobs", merged)
        self.assertEqual(len(merged["blobs"]), 0)

    def test_merge_manifests_with_statements(self):
        """Test merging manifests with different statements."""
        statement1 = {
            "@context": "urn:cid:bafkr4iagb4u7jqlwqrftw4mn3l634wmgatmpvvzqgntgxaaerzljhggvdu",
            "@id": "urn:cid:bagb6qaq6edqb5xwzjktlnlpcwmpgganigqj2o74x5kbv6uuung37khoer52ky",
            "@type": "MetadataRegistration",
            "subject": "urn:cid:bafkr4ibcl6e7kiy2pggcohuygv64wjudxkllx7tq664tbl2ehycl3hvd4m",
            "metadata": "urn:cid:baga6yaq6ectq47a5vn2lc76qooxwx4imcc23dzs3v2wd5ww6scizj7555epwc",
            "registeredBy": "did:key:zDnaeuYuGvB3ox3MSqA5K1axfqQu5U1Jz1JXya5Dh9F5ZhnqF",
            "timestamp": "2025-08-19T00:06:56Z",
        }
        statement2 = {
            "@context": "urn:cid:bafkr4iagb4u7jqlwqrftw4mn3l634wmgatmpvvzqgntgxaaerzljhggvdu",
            "@id": "urn:cid:bagb6qaq6ebzxk44r6d53nm4uo6x6s2lfdhff5lqcqbjybsrr4eewzt2cognfu",
            "@type": "MetadataRegistration",
            "subject": "urn:cid:bagaachradlrhaifv5s36ni22vh7kexo5zb2z3r7phfycgbsw3ubtxut4dczq",
            "metadata": "urn:cid:baga6yaq6eahlafu4zlfltyb4kcv256ceiz4n4cneku2yexrebtpmtqw3pxqzs",
            "registeredBy": "did:key:zDnaeuYuGvB3ox3MSqA5K1axfqQu5U1Jz1JXya5Dh9F5ZhnqF",
            "timestamp": "2025-08-18T23:56:31Z",
        }

        manifest_a = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {"urn:cid:test1": statement1},
                "blobs": {},
            }
        )

        manifest_b = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {"urn:cid:test2": statement2},
                "blobs": {},
            }
        )

        result = manifest.merge(manifest_a, manifest_b)

        self.assertIsInstance(result, str)
        merged = json.loads(result)
        self.assertIn("statements", merged)
        # Should contain statements from both manifests
        self.assertEqual(len(merged["statements"]), 2)
        self.assertEqual(merged["statements"]["urn:cid:test1"], statement1)
        self.assertEqual(merged["statements"]["urn:cid:test2"], statement2)

    def test_merge_manifests_with_blobs(self):
        """Test merging manifests with different blobs."""
        import base64

        blob1_content = base64.b64encode(b"blob1 content").decode()
        blob2_content = base64.b64encode(b"blob2 content").decode()

        manifest_a = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {},
                "blobs": {"blob1_cid": blob1_content},
            }
        )

        manifest_b = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {},
                "blobs": {"blob2_cid": blob2_content},
            }
        )

        result = manifest.merge(manifest_a, manifest_b)

        self.assertIsInstance(result, str)
        merged = json.loads(result)
        self.assertIn("blobs", merged)
        self.assertEqual(merged["blobs"]["blob1_cid"], blob1_content)
        self.assertEqual(merged["blobs"]["blob2_cid"], blob2_content)

    def test_merge_overlapping_manifests(self):
        """Test merging manifests with overlapping content."""
        common_statement = {
            "@context": "urn:cid:bafkr4iagb4u7jqlwqrftw4mn3l634wmgatmpvvzqgntgxaaerzljhggvdu",
            "@id": "urn:cid:bagb6qaq6ebzxk44r6d53nm4uo6x6s2lfdhff5lqcqbjybsrr4eewzt2cognfu",
            "@type": "MetadataRegistration",
            "subject": "urn:cid:bagaachradlrhaifv5s36ni22vh7kexo5zb2z3r7phfycgbsw3ubtxut4dczq",
            "metadata": "urn:cid:baga6yaq6eahlafu4zlfltyb4kcv256ceiz4n4cneku2yexrebtpmtqw3pxqzs",
            "registeredBy": "did:key:zDnaeuYuGvB3ox3MSqA5K1axfqQu5U1Jz1JXya5Dh9F5ZhnqF",
            "timestamp": "2025-08-18T23:56:31Z",
        }

        manifest_a = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {
                    "urn:cid:shared": common_statement,
                    "urn:cid:unique_a": {
                        "@context": "urn:cid:a",
                        "@id": "urn:cid:baga",
                        "@type": "MetadataRegistration",
                        "subject": "urn:cid:bagaac",
                        "metadata": "urn:cid:baga6",
                        "registeredBy": "did:key:zDnae",
                        "timestamp": "2025-08-18T23:56:31Z",
                    },
                },
                "blobs": {},
            }
        )

        manifest_b = json.dumps(
            {
                "version": "2.0",
                "contexts": {},
                "statements": {
                    "urn:cid:shared": common_statement,
                    "urn:cid:unique_b": {
                        "@context": "urn:cid:b",
                        "@id": "urn:cid:unique_b",
                        "registeredBy": "did:key:zDnae",
                        "timestamp": "2025-08-18T23:56:31Z",
                        "@type": "DataRegistration",
                        "data": ["urn:cid:data_b"],
                    },
                },
                "blobs": {},
            }
        )

        result = manifest.merge(manifest_a, manifest_b)

        self.assertIsInstance(result, str)
        merged = json.loads(result)
        self.assertIn("statements", merged)
        self.assertEqual(len(merged["statements"]), 3, "Should be 3 statements")
        self.assertEqual(
            merged["statements"]["urn:cid:shared"], common_statement, "Common Statement is wrong"
        )

    def test_merge_invalid_json_a(self):
        """Test merging with invalid JSON in first manifest."""
        invalid_manifest = "invalid json"
        valid_manifest = json.dumps({"statements": {}, "blobs": {}})

        with self.assertRaises(Exception):
            manifest.merge(invalid_manifest, valid_manifest)

    def test_merge_invalid_json_b(self):
        """Test merging with invalid JSON in second manifest."""
        valid_manifest = json.dumps({"statements": {}, "blobs": {}, "version": "1", "contexts": {}})
        invalid_manifest = "invalid json"

        with self.assertRaises(Exception):
            manifest.merge(valid_manifest, invalid_manifest)

    def test_merge_malformed_manifest(self):
        """Test merging with malformed manifest structure."""
        malformed_manifest = json.dumps(
            {
                "wrong_field": {}
                # Missing statements and blobs
            }
        )

        valid_manifest = json.dumps({"statements": {}, "blobs": {}})

        with self.assertRaises(Exception):
            manifest.merge(malformed_manifest, valid_manifest)


if __name__ == "__main__":
    unittest.main()
