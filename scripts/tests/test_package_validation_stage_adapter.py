import hashlib, json, subprocess, sys, tempfile, unittest
from pathlib import Path
from unittest import mock
ROOT=Path(__file__).resolve().parents[2]
ADAPTER=ROOT/"scripts/package_validation_stage_adapter.py"
sys.path.insert(0, str(ROOT / "scripts"))
import package_validation_stage_adapter as adapter
import release_contract
class AdapterTest(unittest.TestCase):
 def candidate(self, root):
  records=[]
  for kind,name,content in [("deb","desktop.deb",b"desktop"),("sandboxd-deb","sandbox.deb",b"sandbox")]:
   (root/name).write_bytes(content)
   records.append({"format":kind,"filename":name,"architecture":"x86_64","packageVersion":"0.1.0~beta.46","sha256":hashlib.sha256(content).hexdigest(),"size":len(content)})
  (root/"release-manifest.json").write_text(json.dumps({"schemaVersion":3,"state":"release-candidate","version":"0.1.0-beta.46","artifacts":records}),encoding="utf-8")
 def stage(self, root, stage):
  out=root/"result.json"
  subprocess.run(["python3",str(ADAPTER),"--stage",stage,"--session-id","019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10","--nonce","a"*64,"--candidate-root",str(root),"--result-file",str(out)],check=True)
  return json.loads(out.read_text())
 def test_missing_candidate_is_closed_and_redacted(self):
  with tempfile.TemporaryDirectory() as d:
   out=Path(d)/"result.json"
   subprocess.run(["python3",str(ADAPTER),"--stage","manifest","--session-id","019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10","--nonce","a"*64,"--candidate-root",d,"--result-file",str(out)],check=True)
   value=json.loads(out.read_text()); self.assertEqual(value["outcome"],"unavailable"); self.assertNotIn("path",value)
 def test_manifest_and_checksum_are_distinct_from_unavailable_operational_stages(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   self.assertEqual(self.stage(root,"manifest")["outcome"],"passed")
   self.assertEqual(self.stage(root,"checksum")["outcome"],"passed")
   self.assertEqual(self.stage(root,"abi")["outcome"],"failed")
   self.assertEqual(self.stage(root,"provenance")["outcome"],"failed")
 def test_manifest_does_not_claim_checksum(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root); (root/"desktop.deb").write_bytes(b"changed")
   self.assertEqual(self.stage(root,"manifest")["outcome"],"passed")
   self.assertEqual(self.stage(root,"checksum")["outcome"],"failed")

 def test_package_validation_abi_requires_both_validators_and_matching_evidence(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   manifest=json.loads((root/"release-manifest.json").read_text())
   manifest["abi"]={"baseline":"GLIBC_2.35","highestRequired":"GLIBC_2.34","binaries":[{"format":"deb","highestRequired":"GLIBC_2.34"},{"format":"sandboxd-deb","highestRequired":"GLIBC_2.33"}]}
   (root/"release-manifest.json").write_text(json.dumps(manifest))
   with mock.patch.object(adapter,"validate_debian",return_value=(2,34)) as deb, mock.patch.object(adapter,"validate_sandboxd",return_value=(2,33)) as sandboxd, mock.patch.object(adapter,"validate_abi_evidence") as evidence:
    self.assertEqual(adapter.abi(root),{"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.34"})
   deb.assert_called_once(); sandboxd.assert_called_once(); evidence.assert_called_once_with(manifest,(2,34),(2,33))

 def test_package_validation_abi_incompatibility_missing_capability_and_disagreement_do_not_pass(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   for failure in (RuntimeError("incompatible"), FileNotFoundError("readelf")):
    with mock.patch.object(adapter,"validate_debian",side_effect=failure), mock.patch.object(adapter,"validate_sandboxd"):
     with self.assertRaises(type(failure)): adapter.abi(root)
   with mock.patch.object(adapter,"validate_debian",return_value=(2,34)), mock.patch.object(adapter,"validate_sandboxd",return_value=(2,34)), mock.patch.object(adapter,"validate_abi_evidence",side_effect=RuntimeError("evidence")):
    with self.assertRaises(RuntimeError): adapter.abi(root)
   with mock.patch.object(adapter,"validate_debian",return_value=(2,34)), mock.patch.object(adapter,"validate_sandboxd",side_effect=RuntimeError("sandbox incompatible")):
    with self.assertRaises(RuntimeError): adapter.abi(root)

 def test_package_validation_abi_maps_tool_failures_without_generic_success(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"abi",return_value={"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.34"}):
    self.assertEqual(adapter.evaluate_stage(root,"abi")[3],"passed")
   with mock.patch.object(adapter,"abi",side_effect=RuntimeError("nonzero validator exit")):
    self.assertEqual(adapter.evaluate_stage(root,"abi")[3],"failed")
   with mock.patch.object(adapter,"abi",side_effect=FileNotFoundError("readelf")):
    self.assertEqual(adapter.evaluate_stage(root,"abi")[3],"unavailable")

 def test_package_validation_abi_uses_fixed_readelf_and_redacts_tool_output(self):
  with mock.patch.object(release_contract,"run",return_value=mock.Mock(stdout="GLIBC_2.34\n")) as run:
   self.assertEqual(release_contract.glibc_requirement(Path("fixture-binary")),(2,34))
  run.assert_called_once_with(["readelf","--version-info",str(Path("fixture-binary"))],capture=True)
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"abi",return_value={"kind":"abi","schema_version":1,"glibc_baseline":"GLIBC_2.35","highest_required":"GLIBC_2.34"}):
    _,_,_,outcome,facts=adapter.evaluate_stage(root,"abi")
   self.assertEqual(outcome,"passed")
   self.assertEqual(set(facts),{"kind","schema_version","glibc_baseline","highest_required"})
   self.assertNotIn("output",facts)

 def test_package_validation_abi_rejects_candidate_path_and_symlink_escape(self):
  with tempfile.TemporaryDirectory() as d, tempfile.TemporaryDirectory() as outside:
   root=Path(d); self.candidate(root)
   manifest=json.loads((root/"release-manifest.json").read_text())
   manifest["artifacts"][0]["filename"]="../escape.deb"
   (root/"release-manifest.json").write_text(json.dumps(manifest))
   with self.assertRaises(ValueError): adapter.abi(root)
   self.candidate(root)
   (root/"desktop.deb").unlink()
   (root/"desktop.deb").symlink_to(Path(outside)/"artifact.deb")
   (Path(outside)/"artifact.deb").write_bytes(b"desktop")
   with self.assertRaises(ValueError): adapter.abi(root)

 def test_package_validation_provenance_requires_fixed_authority_and_returns_redacted_facts(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"validate_manifest_provenance") as manifest, mock.patch.object(adapter,"validate_sandboxd_provenance") as sandboxd:
    facts=adapter.provenance(root)
   self.assertEqual(facts,{"kind":"provenance","schema_version":1,"evidence_state":"pinned-release-candidate","artifact_coverage":2,"identity_consistent":True})
   manifest.assert_called_once(); sandboxd.assert_called_once()
   self.assertEqual(set(facts),{"kind","schema_version","evidence_state","artifact_coverage","identity_consistent"})
   self.assertFalse(any(item in json.dumps(facts) for item in ("commit","path","filename","command","output")))

 def test_package_validation_provenance_maps_invalid_and_missing_evidence_closed(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"provenance",side_effect=RuntimeError("contradictory provenance")):
    self.assertEqual(adapter.evaluate_stage(root,"provenance")[3],"failed")
   with mock.patch.object(adapter,"provenance",side_effect=FileNotFoundError("validator unavailable")):
    self.assertEqual(adapter.evaluate_stage(root,"provenance")[3],"unavailable")
   self.assertEqual(adapter.evaluate_stage(root,"manifest")[3],"passed")
   self.assertEqual(adapter.evaluate_stage(root,"checksum")[3],"passed")
   self.assertEqual(adapter.evaluate_stage(root,"provenance")[3],"failed")

 def test_package_validation_visible_launch_requires_fixed_smoke_and_redacts_facts(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"smoke_packages") as smoke:
    facts=adapter.visible_launch(root)
   smoke.assert_called_once()
   self.assertEqual(facts,{"kind":"visible-launch","schema_version":1,"launch_state":"visible-window-confirmed","artifact_coverage":1,"visibility_confirmed":True,"lifecycle_clean":True})
   self.assertEqual(set(facts),{"kind","schema_version","launch_state","artifact_coverage","visibility_confirmed","lifecycle_clean"})
   self.assertFalse(any(item in json.dumps(facts) for item in ("path","filename","display", "screenshot","pid","command","output","commit")))

 def test_package_validation_visible_launch_maps_failure_and_missing_capability_closed(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d); self.candidate(root)
   with mock.patch.object(adapter,"visible_launch",side_effect=RuntimeError("no stable visible window")):
    self.assertEqual(adapter.evaluate_stage(root,"visible-launch")[3],"failed")
   with mock.patch.object(adapter,"visible_launch",side_effect=FileNotFoundError("xvfb-run")):
    self.assertEqual(adapter.evaluate_stage(root,"visible-launch")[3],"unavailable")
   self.assertEqual(adapter.evaluate_stage(root,"manifest")[3],"passed")
   self.assertEqual(adapter.evaluate_stage(root,"checksum")[3],"passed")
