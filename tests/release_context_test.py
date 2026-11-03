"""Exercise the publisher's actual version/tag validation with isolated fixtures."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location(
    "release_context", Path(__file__).resolve().parents[1] / "scripts/release_context.py"
)
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseContextTest(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "Cargo.toml").write_text('[package]\nname="truenas-exporter"\nversion="0.5.9"\n')
        (self.root / "Cargo.lock").write_text('[[package]]\nname="truenas-exporter"\nversion="0.5.9"\n')
        (self.root / ".release-please-manifest.json").write_text(json.dumps({".": "0.5.9"}))

    def test_main_build_is_not_a_release(self):
        self.assertEqual(module.release_context(self.root, ""), {
            "is_release": "false", "version": "0.5.9", "tag": ""
        })

    def test_matching_release_is_accepted(self):
        self.assertEqual(module.release_context(self.root, "v0.5.9"), {
            "is_release": "true", "version": "0.5.9", "tag": "v0.5.9"
        })

    def test_component_release_is_accepted(self):
        context = module.release_context(self.root, "truenas-exporter-rs-v0.5.9")
        self.assertEqual(context["tag"], "truenas-exporter-rs-v0.5.9")
        self.assertEqual(context["is_release"], "true")

    def test_wrong_version_or_tag_format_is_rejected(self):
        for tag in ["v0.5.10", "truenas-exporter-rs-v0.5.10", "other-v0.5.9", "v0.5.9-rc.1", "main", "v0.5.9\nextra=true", "v0.5.9;echo bad"]:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                module.release_context(self.root, tag)

    def test_manifest_drift_is_rejected(self):
        (self.root / ".release-please-manifest.json").write_text(json.dumps({".": "0.5.10"}))
        with self.assertRaisesRegex(ValueError, "must agree"):
            module.release_context(self.root, "v0.5.9")

    def test_lockfile_drift_is_rejected(self):
        (self.root / "Cargo.lock").write_text('[[package]]\nname="truenas-exporter"\nversion="0.5.8"\n')
        with self.assertRaisesRegex(ValueError, "must agree"):
            module.release_context(self.root, "v0.5.9")


if __name__ == "__main__":
    unittest.main()
