import hashlib, importlib.util, io, json, os, stat, tempfile, unittest
from pathlib import Path

ROOT=Path(__file__).resolve().parents[2]
SPEC=importlib.util.spec_from_file_location("installed_helper", ROOT/"scripts/quireforge_validate_deb.py")
helper=importlib.util.module_from_spec(SPEC); SPEC.loader.exec_module(helper)

class Result:
 def __init__(self, stdout): self.stdout=stdout
class HelperTest(unittest.TestCase):
 def request(self): return {"schema_version":1,"session_id":"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10","nonce":"a"*64,"expected_application_version":"0.1.0-beta.46","expected_debian_version":"0.1.0~beta.46"}
 def fixture(self):
  root=tempfile.TemporaryDirectory(); base=Path(root.name)
  for fixed in helper.PROTECTED:
   path=base/fixed.lstrip("/"); path.parent.mkdir(parents=True,exist_ok=True); path.write_text("fixture")
  def run(args):
   self.assertTrue(args[0].startswith("/usr/")); self.assertNotIn("sh",args)
   if args[0]==helper.DPKG_QUERY and args[1]=="--showformat=${db:Status-Status}\\n${Version}\\n": return Result("installed\n0.1.0~beta.46\n")
   if args[0]==helper.DPKG_QUERY: return Result(f"quireforge: {args[-1]}\n")
   if args==[helper.DPKG,"--verify",helper.PACKAGE]: return Result("")
   raise AssertionError(args)
  return root,base,run
 def test_valid_simulated_installed_state_and_digest(self):
  temporary,root,run=self.fixture()
  try:
   facts=helper.validate(self.request(),run,root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
   value=helper.result(self.request(),"passed",facts); self.assertEqual(value["outcome"],"passed"); self.assertEqual(value["result_sha256"],hashlib.sha256(helper.canonical({k:v for k,v in value.items() if k!="result_sha256"})).hexdigest())
  finally: temporary.cleanup()
 def test_failures_and_unavailable_are_closed(self):
  temporary,root,run=self.fixture()
  try:
   for mutate in (lambda args: Result("installed\nwrong\n") if args[0]==helper.DPKG_QUERY and len(args)>1 and args[1].startswith("--showformat") else run(args), lambda args: Result("changed") if args==[helper.DPKG,"--verify",helper.PACKAGE] else run(args)):
    with self.assertRaises(helper.Failed): helper.validate(self.request(),mutate,root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
   with self.assertRaises(helper.Unavailable): helper.validate(self.request(),lambda args: (_ for _ in ()).throw(FileNotFoundError()),root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
  finally: temporary.cleanup()
 def test_unsafe_files_symlinks_and_package_ownership_fail(self):
  temporary,root,run=self.fixture()
  try:
   unsafe=lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o777,"st_uid":1000})()
   with self.assertRaises(helper.Failed): helper.validate(self.request(),run,root,unsafe)
   target=root/helper.PROTECTED[0].lstrip("/"); target.unlink(); target.symlink_to(root/"outside")
   with self.assertRaises(helper.Failed): helper.validate(self.request(),run,root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
   target.unlink(); target.write_text("fixture")
   def wrong_owner(args): return Result("other: "+args[-1]) if args[0]==helper.DPKG_QUERY and args[1]=="--search" else run(args)
   with self.assertRaises(helper.Failed): helper.validate(self.request(),wrong_owner,root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
  finally: temporary.cleanup()
 def test_request_and_sudoers_are_closed(self):
  request=json.dumps(self.request()).encode(); self.assertEqual(helper.read_request(io.BytesIO(request)),self.request())
  invalid_uuid={**self.request(),"session_id":"x"*36}
  invalid_nonce={**self.request(),"nonce":"G"*64}
  invalid_application={**self.request(),"expected_application_version":"bad version"}
  invalid_debian={**self.request(),"expected_debian_version":"bad version"}
  missing={key:value for key,value in self.request().items() if key!="nonce"}
  for bad in (b"{}",b"{",request+b"{}",b"x"*4097,json.dumps({**self.request(),"path":"forbidden"}).encode(),json.dumps(missing).encode(),json.dumps(invalid_uuid).encode(),json.dumps(invalid_nonce).encode(),json.dumps(invalid_application).encode(),json.dumps(invalid_debian).encode()):
   with self.assertRaises(helper.Failed): helper.read_request(io.BytesIO(bad))
  out=io.StringIO(); err=io.StringIO(); self.assertEqual(helper.main(["bad"],io.BytesIO(request),out,err),2)
  value=helper.result(self.request(),"unavailable")
  self.assertEqual(set(value),{"schema_version","session_id","nonce","outcome","facts","result_sha256"}); self.assertIsNone(value["facts"])
  self.assertFalse(any(word in json.dumps(value) for word in ("path","filename","command","output","metadata.sqlite3")))
  policy=(ROOT/"packaging/linux/quireforge-validate-deb.sudoers.example").read_text(); self.assertIn('%quireforge-package-validation ALL=(root)',policy); self.assertIn('/usr/local/sbin/quireforge-validate-deb ""',policy); self.assertIn("NOSETENV",policy); self.assertNotIn("*",policy); self.assertNotIn("python",policy.lower()); self.assertNotIn("%sudo",policy); self.assertNotIn("%wheel",policy); self.assertNotIn("SETENV:",policy.replace("NOSETENV:",""))
  self.assertNotIn("metadata.sqlite3",(ROOT/"scripts/quireforge_validate_deb.py").read_text())
 def test_version_mapping_and_stderr_are_bounded(self):
  self.assertEqual(helper.application_version_from_debian("0.1.0~beta.46"),"0.1.0-beta.46")
  temporary,root,run=self.fixture()
  try:
   bad={**self.request(),"expected_application_version":"0.1.0-beta.47"}
   with self.assertRaises(helper.Failed): helper.validate(bad,run,root,lambda path: type("S",(),{"st_mode":stat.S_IFREG|0o755,"st_uid":0})())
  finally: temporary.cleanup()
  out=io.StringIO(); err=io.StringIO(); self.assertEqual(helper.main([],io.BytesIO(b"{"),out,err),2); self.assertEqual(err.getvalue(),"invalid request\n"); self.assertEqual(out.getvalue(),"")
 def test_snake_case_golden_request_and_result_vectors(self):
  request=self.request()
  self.assertEqual(helper.canonical(request).decode(),'{"expected_application_version":"0.1.0-beta.46","expected_debian_version":"0.1.0~beta.46","nonce":"' + 'a'*64 + '","schema_version":1,"session_id":"019d4e3c-3b14-7a2b-8c91-3f27d4f7aa10"}')
  self.assertEqual(helper.read_request(io.BytesIO(helper.canonical(request))),request)
  passed=helper.result(request,"passed",{"kind":"installed-host","schema_version":1,"package_state":"installed","version_match":True,"ownership_verified":True,"permissions_safe":True,"package_integrity_verified":True})
  self.assertEqual(passed["result_sha256"],"f717187b6bee5c97948ebe75196b6c19921711b1feebb60e17b967443348e4a8")
  self.assertEqual(helper.result(request,"failed")["result_sha256"],"41810f1ac1fa8e58224b9bc7f87c1986d6d2a1676c4a6518c3c2ca0f7c50e95b")
  self.assertEqual(helper.result(request,"unavailable")["result_sha256"],"b406376fa2ecd097ebf7fca0cc9b75b3552e3a6a6041c889655efc33f4630074")
  with self.assertRaises(helper.Failed): helper.read_request(io.BytesIO(b'{"schemaVersion":1}'))
