"""Keep numbered overload aliases stable across Solidity compilers."""
from collections import defaultdict
from contextlib import contextmanager
import json
from pathlib import Path
import re
from tempfile import TemporaryDirectory


def parameter_type(parameter):
    name = parameter["type"]
    if name.startswith("tuple"):
        return "(" + ",".join(parameter_type(p) for p in parameter["components"]) + ")" + name[5:]
    return name


def order_overloads(artifact):
    # Copy through JSON so no caller or original compiler artifact is mutated.
    artifact = json.loads(json.dumps(artifact))
    groups = defaultdict(list)
    for index, entry in enumerate(artifact["abi"]):
        if entry["type"] == "function":
            groups[entry["name"]].append(index)
    for indices in groups.values():
        if len(indices) < 2:
            continue
        entries = []
        for index in indices:
            entry = artifact["abi"][index]
            signature = entry["name"] + "(" + ",".join(parameter_type(p) for p in entry["inputs"]) + ")"
            selector = artifact.get("methodIdentifiers", {}).get(signature, "")
            if not re.fullmatch("[0-9a-fA-F]{8}", selector):
                raise ValueError("missing compiler method identifier: " + signature)
            entries.append((int(selector, 16), entry))
        if len({selector for selector, _ in entries}) != len(entries):
            raise ValueError("ambiguous overload selectors")
        for index, (_, entry) in zip(indices, sorted(entries, key=lambda pair: pair[0])):
            artifact["abi"][index] = entry
    return artifact


@contextmanager
def ordered_artifacts(artifacts, names):
    """Project verified ABIs into temporary inputs; retain original metadata."""
    with TemporaryDirectory(prefix="diesis-overloads-") as temporary:
        output = Path(temporary)
        for name in names:
            matches = list(Path(artifacts).rglob(name + ".json"))
            if len(matches) != 1:
                raise ValueError("expected exactly one compiler artifact: " + name)
            artifact = order_overloads(json.loads(matches[0].read_text()))
            target = output / (name + ".sol") / (name + ".json")
            target.parent.mkdir()
            target.write_text(json.dumps(artifact))
        yield output
