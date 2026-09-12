#!/usr/bin/env python3
"""Verify sibling API, protocol, generated client and controller checkouts together."""

import argparse
import concurrent.futures
import json
import os
from pathlib import Path
import re
import subprocess


def verify_identity(root, modules):
    results = {}
    old_name = re.compile(r"kdubbo|dxgate|dxsvc", re.I)
    for name in ("transit", "api", "xds-api", "client-go", "tools"):
        path = root / name
        origin = subprocess.check_output(["rtk", "proxy", "git", "remote", "get-url", "origin"], cwd=path, text=True).strip()
        expected = f"github.com/dubml/{name}"
        if origin.removesuffix(".git").replace("git@github.com:", "github.com/").removeprefix("https://") != expected:
            raise RuntimeError(f"{name}: origin must identify {expected}, got {origin}")
        files = subprocess.check_output(["rtk", "proxy", "git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"], cwd=path).decode().split("\0")
        checked = 0
        for relative in sorted(set(files) - {""}):
            file = path / relative
            if not file.is_file() or file.resolve() == Path(__file__).resolve():
                continue
            if old_name.search(relative):
                raise RuntimeError(f"{name}: obsolete file name: {relative}")
            data = file.read_bytes()
            if b"\0" in data:
                continue
            text = data.decode("utf-8", errors="replace")
            if old_name.search(text):
                raise RuntimeError(f"{name}: obsolete identity in {relative}")
            checked += 1
        results[name] = {"origin": origin, "checked_text_files": checked}
    for name, (path, expected) in modules.items():
        source = (path / "go.mod").read_text()
        match = re.search(r"^module\s+(\S+)", source, re.M)
        if not match or match.group(1) != expected:
            raise RuntimeError(f"{name}: module identity must be {expected}")
    for directory, module in ((root / "api", "github.com/dubml/api"), (root / "xds-api", "github.com/dubml/xds-api"), (root / "transit/crates/xds/proto", "github.com/dubml/xds-api")):
        for file in directory.rglob("*.proto"):
            relative = file.relative_to(directory)
            # Google definitions and dubbod's activation contract retain upstream identity.
            if relative.parts[0] in ("google", "activation"):
                continue
            match = re.search(r'option go_package = "([^";]+)', file.read_text())
            expected = module + "/" + str(relative.parent)
            if not match or match.group(1) != expected:
                raise RuntimeError(f"{file}: go_package must be {expected}")
    command_doc = root / "md/command.md"
    if old_name.search(command_doc.read_text()) or "github.com/apache/dubbo-kubernetes/" in command_doc.read_text():
        raise RuntimeError(f"{command_doc}: stale module identity")
    crds = (root / "api/kubernetes/customresourcedefinitions.gen.yaml").read_text()
    transit_crds = [part for part in re.split(r"^---\s*$", crds, flags=re.M) if re.search(r"^  name: transitservices\.networking\.dubbo\.apache\.org$", part, re.M)]
    bundle = root / "transit/controller/install/crds/transitservices.yaml"
    header = "# Generated from dubml/api kubernetes/customresourcedefinitions.gen.yaml.\n"
    if len(transit_crds) != 1 or bundle.read_text() != header + transit_crds[0].strip() + "\n":
        raise RuntimeError(f"{bundle}: bundled CRD differs from the API source")
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--workspace", type=Path, default=Path(__file__).resolve().parents[2])
    parser.add_argument("--evidence", type=Path, required=True, help="Directory for complete Go test JSON logs")
    args = parser.parse_args()
    root = args.workspace.resolve()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    modules = {
        "api": (root / "api", "github.com/dubml/api"),
        "xds-api": (root / "xds-api", "github.com/dubml/xds-api"),
        "client-go": (root / "client-go", "github.com/dubml/client-go"),
        "tools": (root / "tools", "github.com/dubml/tools"),
        "controller": (root / "transit/controller", "github.com/dubml/transit/controller"),
    }
    identities = verify_identity(root, modules)
    (evidence / "identities.json").write_text(json.dumps(identities, indent=2) + "\n")
    def verify(item):
        name, (path, _) = item
        env = os.environ.copy()
        env["GOWORK"] = "off"
        env["GOMAXPROCS"] = "4"
        log = evidence / f"{name}.jsonl"
        with log.open("w") as output:
            result = subprocess.run(["rtk", "proxy", "go", "test", "-p=2", "-json", "./..."], cwd=path, env=env, stdout=output, stderr=subprocess.STDOUT)
        tests = []
        packages = set()
        failures = []
        for line in log.read_text().splitlines():
            try:
                event = json.loads(line)
            except json.JSONDecodeError:
                continue
            if event.get("Action") == "pass" and event.get("Test"):
                tests.append(event["Package"] + "/" + event["Test"])
            if event.get("Package"):
                packages.add(event["Package"])
            if event.get("Action") == "fail":
                failures.append(event.get("Test", event.get("Package")))
        return name, {"exit_code": result.returncode, "packages": len(packages), "passed_test_entries": tests, "failures": failures, "log": str(log)}

    with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool:
        results = dict(pool.map(verify, modules.items()))
    (evidence / "summary.json").write_text(json.dumps(results, indent=2) + "\n")
    for name, result in results.items():
        print(f'{name}: {"PASS" if result["exit_code"] == 0 else "FAIL"}; {result["packages"]} packages; {len(result["passed_test_entries"])} passed test entries')
    return int(any(result["exit_code"] != 0 for result in results.values()))


if __name__ == "__main__":
    raise SystemExit(main())
