# Prevent a build-dependency backport from silently changing native/FFI code.
import hashlib
import json
import tomllib
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class BackportProvenance(unittest.TestCase):
    def test_only_reviewed_files_differ_from_original_crate(self):
        folder = ROOT / "third-party/sherpa-onnx-sys"
        manifest = json.loads((folder / "BACKPORT.json").read_text(encoding="utf-8"))
        self.assertEqual(
            manifest["archive_sha256"],
            "14bbeabb73f73f1c1f4a278af0ee9ae90a19f9727482027047fb9422fa14f8fe",
        )
        self.assertEqual(manifest["local_version"], "1.13.8+tinystt.zip9.1")
        for file, digest in manifest["original_files_sha256"].items():
            if file in ["Cargo.toml", "build.rs", "README.md"]:
                continue
            self.assertEqual(
                hashlib.sha256((folder / file).read_bytes()).hexdigest(),
                digest,
                file,
            )
        crate = tomllib.loads((folder / "Cargo.toml").read_text(encoding="utf-8"))
        self.assertEqual(crate["package"]["version"], "1.13.8+tinystt.zip9.1")
        self.assertEqual(crate["build-dependencies"]["zip"]["version"], "=9.1.3")
        build = (folder / "build.rs").read_text(encoding="utf-8")
        self.assertIn('const UPSTREAM_NATIVE_VERSION: &str = "1.13.8";', build)
        self.assertNotIn('env!("CARGO_PKG_VERSION")', build)
        lock = tomllib.loads((ROOT / "Cargo.lock").read_text(encoding="utf-8"))
        self.assertEqual(
            [item["version"] for item in lock["package"] if item["name"] == "zip"],
            ["9.1.3"],
        )
        self.assertEqual(
            [item["version"] for item in lock["package"] if item["name"] == "sherpa-onnx"],
            ["1.13.8"],
        )
        self.assertEqual(
            [item["version"] for item in lock["package"] if item["name"] == "rustls"],
            ["0.23.46"],
        )


if __name__ == "__main__":
    unittest.main()
