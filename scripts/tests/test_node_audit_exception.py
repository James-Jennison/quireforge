from __future__ import annotations
import copy, unittest
from scripts.validate_node_audit_exception import path_set_sha256, validate_report
PATHS = ["apps__desktop>eslint>minimatch>brace-expansion"]
EXCEPTION = {"githubAdvisoryId":"GHSA-example","severity":"high","package":"brace-expansion","version":"1.1.16","pathCount":1,"pathSetSha256":path_set_sha256(PATHS),"developmentOnly":True}
REPORT = {"advisories":{"1":{"github_advisory_id":"GHSA-example","severity":"high","module_name":"brace-expansion","findings":[{"version":"1.1.16","dev":True,"paths":PATHS}]}}}
class NodeAuditExceptionTests(unittest.TestCase):
    def test_exact_reviewed_finding_is_accepted(self): self.assertEqual(validate_report(REPORT, [EXCEPTION]), [])
    def test_new_advisory_is_rejected(self):
        report=copy.deepcopy(REPORT); report["advisories"]["2"]=copy.deepcopy(report["advisories"]["1"]); report["advisories"]["2"]["github_advisory_id"]="GHSA-new"; self.assertTrue(validate_report(report,[EXCEPTION]))
    def test_changed_details_are_rejected(self):
        for field, value in (("severity","critical"),("version","1.1.17")):
            report=copy.deepcopy(REPORT); target=report["advisories"]["1"]; (target["findings"][0] if field=="version" else target)[field]=value; self.assertTrue(validate_report(report,[EXCEPTION]))
        report=copy.deepcopy(REPORT); report["advisories"]["1"]["findings"][0]["paths"].append("new>path"); self.assertTrue(validate_report(report,[EXCEPTION]))
