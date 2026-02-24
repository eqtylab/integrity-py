#!/usr/bin/env python3
"""Script to automatically generate _rust.pyi stub file from Rust source code.
Run this after making changes to PyO3 functions in the Rust codebase.
"""

import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class RustTypeMapper:
    """Maps Rust types to Python types in stub files."""

    TYPE_MAP = {
        "String": "str",
        "&str": "str",
        "str": "str",
        "PathBuf": "PathLike[str]",
        "bool": "bool",
        "u8": "int",
        "u16": "int",
        "u32": "int",
        "u64": "int",
        "usize": "int",
        "i32": "int",
        "i64": "int",
        "f32": "float",
        "f64": "float",
        "Bytes": "bytes",
        "PyAny": "Any",
        "PyObject": "Any",
        "Py<PyAny>": "Any",
        "PyBytes": "bytes",
        "Uuid": "uuid.UUID",
        "uuid::Uuid": "uuid.UUID",
        "PyResult<()>": "None",
    }

    @classmethod
    def map_type(cls, rust_type: str, class_map: Dict[str, str]) -> str:
        """Convert Rust type to Python type annotation."""
        rust_type = rust_type.strip()
        if not rust_type:
            return "Any"

        rust_type = rust_type.replace("mut ", "").strip()
        rust_type = cls._strip_reference(rust_type)

        if rust_type == "Vec<u8>":
            return "bytes"

        if rust_type == "[u8]":
            return "bytes"

        if rust_type in cls.TYPE_MAP:
            return cls.TYPE_MAP[rust_type]

        if rust_type in class_map:
            return class_map[rust_type]

        if rust_type == "()":
            return "None"

        inner = cls._extract_generic(rust_type, "PyResult")
        if inner is not None:
            if inner.strip() == "()":
                return "None"
            return cls.map_type(inner, class_map)

        inner = cls._extract_generic(rust_type, "Option")
        if inner is not None:
            return f"Optional[{cls.map_type(inner, class_map)}]"

        inner = cls._extract_generic(rust_type, "Vec")
        if inner is not None:
            return f"List[{cls.map_type(inner, class_map)}]"

        inner = cls._extract_generic(rust_type, "HashMap")
        if inner is not None:
            key_val = cls._split_generic_args(inner)
            if len(key_val) == 2:
                key_type = cls.map_type(key_val[0], class_map)
                val_type = cls.map_type(key_val[1], class_map)
                return f"Dict[{key_type}, {val_type}]"
            return "Dict[Any, Any]"

        inner = cls._extract_generic(rust_type, "Result")
        if inner is not None:
            parts = cls._split_generic_args(inner)
            if parts:
                return cls.map_type(parts[0], class_map)

        inner = cls._extract_generic(rust_type, "Py")
        if inner is not None:
            return cls.map_type(inner, class_map)

        inner = cls._extract_generic(rust_type, "Bound")
        if inner is not None:
            parts = cls._split_generic_args(inner)
            if parts:
                return cls.map_type(parts[-1], class_map)

        if rust_type.startswith("(") and rust_type.endswith(")"):
            inner = rust_type[1:-1]
            parts = cls._split_generic_args(inner)
            mapped = ", ".join(cls.map_type(part, class_map) for part in parts if part)
            return f"Tuple[{mapped}]" if mapped else "Tuple[Any, ...]"

        return "Any"

    @staticmethod
    def _extract_generic(rust_type: str, name: str) -> Optional[str]:
        prefix = f"{name}<"
        if rust_type.startswith(prefix) and rust_type.endswith(">"):
            return rust_type[len(prefix) : -1]
        return None

    @staticmethod
    def _split_generic_args(arg_str: str) -> List[str]:
        args = []
        current = ""
        angle_depth = 0
        paren_depth = 0
        for char in arg_str:
            if char == "," and angle_depth == 0 and paren_depth == 0:
                if current.strip():
                    args.append(current.strip())
                current = ""
                continue
            current += char
            if char == "<":
                angle_depth += 1
            elif char == ">":
                angle_depth -= 1
            elif char == "(":
                paren_depth += 1
            elif char == ")":
                paren_depth -= 1
        if current.strip():
            args.append(current.strip())
        return args

    @staticmethod
    def _strip_reference(rust_type: str) -> str:
        if rust_type.startswith("&"):
            rust_type = rust_type[1:].strip()
            rust_type = re.sub(r"^'[\w_]+\s*", "", rust_type).strip()
        return rust_type


class RustStubParser:
    """Parses PyO3 modules, classes, and functions from Rust sources."""

    def __init__(self, src_dir: Path):
        self.src_dir = src_dir
        self.classes: Dict[str, dict] = {}
        self.functions: Dict[str, dict] = {}
        self.modules: Dict[str, dict] = {}
        self.class_name_map: Dict[str, str] = {}

    def parse_all_files(self):
        rust_files = list(self.src_dir.rglob("*.rs"))
        for rust_file in rust_files:
            content = rust_file.read_text()
            self._extract_classes(content)

        for rust_file in rust_files:
            content = rust_file.read_text()
            self._extract_pymethods(content)
            self._extract_pyfunctions(content)
        for rust_file in rust_files:
            content = rust_file.read_text()
            self._extract_pymodules(content)

        return self

    def _extract_classes(self, content: str):
        class_pattern = (
            r"#\[pyclass(?P<attrs>[^\]]*)\](?:\s*#\[[^\]]+\])*\s*"
            r"(?:pub\s+)?(?P<kind>struct|enum)\s+(?P<name>\w+)"
        )
        for match in re.finditer(class_pattern, content):
            rust_name = match.group("name")
            attrs = match.group("attrs")
            py_name = self._parse_pyclass_name(attrs) or rust_name
            doc = self._extract_docstring(content, match.start())

            class_info = self.classes.get(
                rust_name,
                {
                    "rust_name": rust_name,
                    "name": py_name,
                    "doc": doc,
                    "properties": {},
                    "methods": [],
                    "classattrs": {},
                },
            )
            class_info["name"] = py_name
            if doc and not class_info.get("doc"):
                class_info["doc"] = doc

            block, _ = self._extract_block(content, match.end())
            if block:
                if match.group("kind") == "enum":
                    variants = self._parse_enum_variants(block)
                    for variant in variants:
                        class_info["classattrs"].setdefault(variant, py_name)
                else:
                    properties = self._parse_struct_getters(block)
                    for prop_name, prop_type, prop_doc in properties:
                        existing = class_info["properties"].get(prop_name)
                        if not existing:
                            class_info["properties"][prop_name] = {
                                "type": prop_type,
                                "doc": prop_doc,
                            }
                        elif not existing.get("doc") and prop_doc:
                            existing["doc"] = prop_doc

            self.classes[rust_name] = class_info
            self.class_name_map[rust_name] = py_name

    def _parse_struct_getters(self, block: str) -> List[Tuple[str, str, str]]:
        properties = []
        doc_lines: List[str] = []
        has_getter = False
        for line in block.splitlines():
            stripped = line.strip()
            if stripped.startswith("///"):
                doc_lines.append(stripped[3:].strip())
                continue
            if stripped.startswith("#[pyo3(") and "get" in stripped:
                has_getter = True
                continue
            field_match = re.match(r"(?:pub\s+)?(\w+)\s*:\s*([^,]+),", stripped)
            if field_match:
                if has_getter:
                    name = field_match.group(1)
                    rust_type = field_match.group(2).strip()
                    prop_type = RustTypeMapper.map_type(rust_type, self.class_name_map)
                    doc = " ".join(doc_lines).strip()
                    properties.append((name, prop_type, doc))
                has_getter = False
                doc_lines = []
                continue
            if not stripped:
                doc_lines = []
        return properties

    @staticmethod
    def _parse_enum_variants(block: str) -> List[str]:
        variants = []
        for line in block.splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("///") or stripped.startswith("#"):
                continue
            match = re.match(r"(\w+)\s*(?:=|,)", stripped)
            if match:
                variants.append(match.group(1))
        return variants

    def _extract_pymethods(self, content: str):
        impl_pattern = r"#\[pymethods\][\s\S]*?impl\s+(\w+)\s*\{"
        for match in re.finditer(impl_pattern, content):
            rust_name = match.group(1)
            class_info = self.classes.get(rust_name)
            if not class_info:
                continue

            block, _ = self._extract_block(content, match.end() - 1)
            if not block:
                continue

            for const_match in re.finditer(r"#\[classattr\]\s*const\s+(\w+)\s*:\s*([^=]+)=", block):
                const_name = const_match.group(1)
                rust_type = const_match.group(2).strip()
                const_type = RustTypeMapper.map_type(rust_type, self.class_name_map)
                class_info["classattrs"][const_name] = const_type

            func_pattern = (
                r"(?P<attrs>(?:\s*#\[[^\]]+\]\s*)*)\s*"
                r"(?:pub\s+)?fn\s+(?P<name>\w+)(?:<[^>]*>)?\s*"
                r"\((?P<params>.*?)\)\s*(?:->\s*(?P<return>[^{]+))?\s*\{"
            )
            for func_match in re.finditer(func_pattern, block, re.DOTALL):
                attrs = func_match.group("attrs") or ""
                rust_method_name = func_match.group("name")
                return_type = func_match.group("return")
                return_type = return_type.strip() if return_type else "PyResult<()>"

                is_new = "#[new]" in attrs
                is_static = "#[staticmethod]" in attrs
                is_getter = "#[getter]" in attrs

                py_name_override, signature = self._parse_pyo3_attrs(attrs)
                method_name = py_name_override or rust_method_name

                params = self._parse_parameters(func_match.group("params"))
                doc = self._extract_docstring(content, match.start() + func_match.start())

                if is_getter:
                    prop_type = RustTypeMapper.map_type(return_type, self.class_name_map)
                    existing = class_info["properties"].get(method_name)
                    if not existing:
                        class_info["properties"][method_name] = {
                            "type": prop_type,
                            "doc": doc,
                        }
                    elif not existing.get("doc") and doc:
                        existing["doc"] = doc
                    continue

                return_type_mapped = return_type
                if "Self" in return_type_mapped:
                    return_type_mapped = re.sub(r"\bSelf\b", rust_name, return_type_mapped)

                class_info["methods"].append(
                    {
                        "name": method_name,
                        "params": params,
                        "signature": signature,
                        "return_type": RustTypeMapper.map_type(
                            return_type_mapped, self.class_name_map
                        ),
                        "doc": doc,
                        "is_static": is_static,
                        "is_new": is_new,
                    }
                )

    def _extract_pyfunctions(self, content: str):
        func_pattern = (
            r"#\[pyfunction\](?P<attrs>(?:\s*#\[[^\]]+\])*)\s*"
            r"(?:pub\s+)?fn\s+(?P<name>\w+)(?:<[^>]*>)?\s*"
            r"\((?P<params>.*?)\)\s*(?:->\s*(?P<return>[^{]+))?\s*\{"
        )
        for match in re.finditer(func_pattern, content, re.DOTALL):
            attrs = match.group("attrs") or ""
            rust_name = match.group("name")
            return_type = match.group("return")
            return_type = return_type.strip() if return_type else "PyResult<()>"

            py_name_override, signature = self._parse_pyo3_attrs(attrs)
            name = py_name_override or rust_name
            params = self._parse_parameters(match.group("params"))
            doc = self._extract_docstring(content, match.start())

            self.functions[rust_name] = {
                "name": name,
                "params": params,
                "signature": signature,
                "return_type": RustTypeMapper.map_type(return_type, self.class_name_map),
                "doc": doc,
            }

    def _extract_pymodules(self, content: str):
        module_pattern = r"#\[pymodule\]\s*(?:pub\s+)?fn\s+(\w+)\s*\("
        for match in re.finditer(module_pattern, content):
            module_name = match.group(1)
            module_info = self.modules.setdefault(
                module_name, {"functions": [], "classes": [], "submodules": []}
            )

            block, _ = self._extract_block(content, match.end())
            if not block:
                continue

            for class_match in re.finditer(r"add_class::<\s*([\w:]+)\s*>", block):
                rust_class = class_match.group(1).split("::")[-1]
                class_info = self.classes.get(rust_class)
                if not class_info:
                    continue
                class_name = class_info["name"]
                if class_name not in module_info["classes"]:
                    module_info["classes"].append(class_name)

            for fn_match in re.finditer(r"wrap_pyfunction!\(\s*([\w:]+)\s*,", block):
                rust_fn = fn_match.group(1).split("::")[-1]
                func_info = self.functions.get(rust_fn)
                if not func_info:
                    continue
                module_info["functions"].append(func_info)

            for sub_match in re.finditer(r"wrap_pymodule!\(\s*([\w:]+)\s*\)", block):
                submodule_name = sub_match.group(1).split("::")[-1]
                if submodule_name not in module_info["submodules"]:
                    module_info["submodules"].append(submodule_name)
                self.modules.setdefault(
                    submodule_name, {"functions": [], "classes": [], "submodules": []}
                )

    def _parse_parameters(self, param_str: str) -> List[dict]:
        params = []
        raw_params = self._split_parameters(param_str)

        for param in raw_params:
            param = param.strip()
            if not param:
                continue
            if ":" not in param:
                continue

            name_part, type_part = param.split(":", 1)
            name = name_part.strip().replace("mut ", "")
            rust_type = type_part.strip().replace("mut ", "")

            if name in {"py", "_py", "slf"}:
                continue
            if "Python" in rust_type or "PyRef" in rust_type:
                continue

            rust_type = RustTypeMapper._strip_reference(rust_type)
            rust_type = rust_type.replace("mut ", "").strip()

            is_optional = "Option<" in rust_type

            params.append(
                {
                    "name": name,
                    "type": RustTypeMapper.map_type(rust_type, self.class_name_map),
                    "optional": is_optional,
                }
            )

        return params

    @staticmethod
    def _split_parameters(param_str: str) -> List[str]:
        params = []
        current = ""
        paren_depth = 0
        angle_depth = 0

        for char in param_str:
            if char == "," and paren_depth == 0 and angle_depth == 0:
                if current.strip():
                    params.append(current.strip())
                current = ""
                continue
            current += char
            if char == "(":
                paren_depth += 1
            elif char == ")":
                paren_depth -= 1
            elif char == "<":
                angle_depth += 1
            elif char == ">":
                angle_depth -= 1

        if current.strip():
            params.append(current.strip())

        return params

    @staticmethod
    def _extract_docstring(content: str, func_start: int) -> str:
        lines_before = content[:func_start].split("\n")
        doc_lines = []

        for line in reversed(lines_before[-10:]):
            line = line.strip()
            if line.startswith("///"):
                doc_lines.insert(0, line[3:].strip())
            elif line.startswith("//"):
                continue
            elif line == "" or line.startswith("#"):
                continue
            else:
                break

        return " ".join(doc_lines) if doc_lines else ""

    @staticmethod
    def _parse_pyclass_name(attrs: str) -> Optional[str]:
        match = re.search(r"name\s*=\s*\"([^\"]+)\"", attrs)
        if match:
            return match.group(1)
        return None

    @staticmethod
    def _parse_pyo3_attrs(attrs: str) -> Tuple[Optional[str], Optional[str]]:
        py_name = None
        signature = None
        for attr_match in re.finditer(r"#\[pyo3\(([^\]]+)\)\]", attrs, re.DOTALL):
            attr_body = attr_match.group(1)
            name_match = re.search(r"name\s*=\s*\"([^\"]+)\"", attr_body)
            if name_match:
                py_name = name_match.group(1)
            sig_match = re.search(r"signature\s*=\s*\(([^\)]*)\)", attr_body, re.DOTALL)
            if sig_match:
                signature = sig_match.group(1).strip()
        return py_name, signature

    @staticmethod
    def _extract_block(content: str, start_pos: int) -> Tuple[Optional[str], int]:
        brace_pos = content.find("{", start_pos)
        if brace_pos == -1:
            return None, -1
        depth = 0
        for idx in range(brace_pos, len(content)):
            if content[idx] == "{":
                depth += 1
            elif content[idx] == "}":
                depth -= 1
                if depth == 0:
                    return content[brace_pos + 1 : idx], idx
        return None, -1


class StubGenerator:
    """Generates Python stub file from parsed Rust functions."""

    def __init__(self, class_names: Optional[List[str]] = None):
        self.class_names = set(class_names or [])

    def generate_stub_file(
        self, modules: Dict[str, dict], classes: Dict[str, dict], output_path: Path
    ):
        lines = []

        lines.append('"""Type stubs for the eqty_sdk._rust module."""')
        lines.append("import eqty_sdk")
        lines.append("from pathlib import Path")
        lines.append("from typing import Any, Dict, List, Optional, Tuple, Union")
        lines.append("import uuid")
        lines.append("from os import PathLike")
        lines.append("")

        rust_module = modules.get("_rust")
        if rust_module:
            for func in rust_module["functions"]:
                lines.extend(self._generate_function_stub(func, is_module_function=False))
                lines.append("")

        lines.extend(self._generate_class_definitions(classes))

        for module_name in sorted(modules.keys()):
            if module_name == "_rust":
                continue

            module_info = modules[module_name]
            lines.append(f"# {module_name.title()} module")
            lines.append(f"class {module_name}:")

            if module_info["classes"]:
                for class_name in module_info["classes"]:
                    lines.append(f"    {class_name}: type[{class_name}]")
                lines.append("")

            if module_info["functions"]:
                for func in module_info["functions"]:
                    lines.extend(
                        self._generate_function_stub(func, indent="    ", qualify_types=True)
                    )
                    lines.append("")
            elif not module_info["classes"]:
                lines.append("    ...")

            lines.append("")

        output_path.write_text("\n".join(lines))
        print(f"Generated stub file: {output_path}")

    def _generate_function_stub(
        self,
        func: dict,
        is_module_function: bool = True,
        indent: str = "",
        qualify_types: bool = False,
    ) -> List[str]:
        lines = []

        if is_module_function:
            lines.append(f"{indent}@staticmethod")

        param_str = self._build_param_str(func, qualify_types=qualify_types)
        return_type = func["return_type"]
        if qualify_types:
            return_type = self._qualify_type(return_type)

        lines.append(f"{indent}def {func['name']}({param_str}) -> {return_type}:")
        doc = func.get("doc") or ""
        if doc:
            lines.append(f'{indent}    """{doc}"""')
        lines.append(f"{indent}    ...")

        return lines

    def _build_param_str(self, func: dict, qualify_types: bool = False) -> str:
        if func.get("signature"):
            params = self._parse_signature_params(
                func["signature"], func["params"], qualify_types=qualify_types
            )
            return ", ".join(params)
        return ", ".join(self._format_param(p, qualify_types=qualify_types) for p in func["params"])

    def _parse_signature_params(
        self, signature: str, rust_params: List[dict], qualify_types: bool = False
    ) -> List[str]:
        params = []
        type_map = {p["name"]: p for p in rust_params}
        for raw in signature.split(","):
            token = raw.strip()
            if not token:
                continue
            if token == "*":
                params.append("*")
                continue
            if token.startswith("**"):
                name = token[2:].strip() or "kwargs"
                params.append(f"**{name}: Any")
                continue
            optional = False
            name = token
            if "=" in token:
                name, _ = token.split("=", 1)
                name = name.strip()
                optional = True

            param_info = type_map.get(name)
            if param_info:
                param_type = param_info["type"]
                if optional and not param_type.startswith("Optional["):
                    param_type = f"Optional[{param_type}]"
            else:
                param_type = "Any"

            if qualify_types:
                param_type = self._qualify_type(param_type)
            param_str = f"{name}: {param_type}"
            if optional:
                param_str += " = None"
            params.append(param_str)

        return params

    def _format_param(self, param: dict, qualify_types: bool = False) -> str:
        param_type = param["type"]
        if qualify_types:
            param_type = self._qualify_type(param_type)
        param_str = f"{param['name']}: {param_type}"
        if param["optional"] and not param_str.endswith(" = None"):
            param_str += " = None"
        return param_str

    def _qualify_type(self, type_str: str) -> str:
        qualified = type_str
        for class_name in sorted(self.class_names, key=len, reverse=True):
            pattern = rf"(?<![\w.]){re.escape(class_name)}(?![\w])"
            replacement = f"eqty_sdk._rust.{class_name}"
            qualified = re.sub(pattern, replacement, qualified)
        return qualified

    def _generate_class_definitions(self, classes: Dict[str, dict]) -> List[str]:
        lines = []

        for class_info in sorted(classes.values(), key=lambda c: c["name"]):
            lines.append(f"class {class_info['name']}:")
            if class_info.get("doc"):
                lines.append(f'    """{class_info["doc"]}"""')

            if class_info["classattrs"]:
                for name, attr_type in class_info["classattrs"].items():
                    lines.append(f"    {name}: {attr_type}")
                lines.append("")

            if class_info["properties"]:
                for prop_name, prop_info in class_info["properties"].items():
                    lines.append("    @property")
                    lines.append(f"    def {prop_name}(self) -> {prop_info['type']}:")
                    if prop_info.get("doc"):
                        lines.append(f'        """{prop_info["doc"]}"""')
                    lines.append("        ...")
                    lines.append("")

            if class_info["methods"]:
                for method in class_info["methods"]:
                    lines.extend(self._generate_method_stub(method))
                    lines.append("")

            if (
                not class_info["classattrs"]
                and not class_info["properties"]
                and not class_info["methods"]
                and not class_info.get("doc")
            ):
                lines.append("    ...")

            lines.append("")

        return lines

    def _generate_method_stub(self, method: dict) -> List[str]:
        lines = []
        indent = "    "
        name = "__init__" if method["is_new"] else method["name"]
        return_type = "None" if method["is_new"] else method["return_type"]

        if method["is_static"]:
            lines.append(f"{indent}@staticmethod")
            param_str = self._build_param_str(method)
        else:
            param_str = self._build_param_str(method)
            if param_str:
                param_str = f"self, {param_str}"
            else:
                param_str = "self"

        lines.append(f"{indent}def {name}({param_str}) -> {return_type}:")
        doc = method.get("doc") or ""
        if doc:
            lines.append(f'{indent}    """{doc}"""')
        lines.append(f"{indent}    ...")

        return lines


def main():
    """Main function to generate stub file."""
    script_dir = Path(__file__).parent.parent
    src_dir = script_dir / "src"
    output_file = script_dir / "eqty_sdk" / "_rust.pyi"

    if not src_dir.exists():
        print(f"Error: Source directory not found: {src_dir}")
        return 1

    parser = RustStubParser(src_dir).parse_all_files()

    if not parser.modules:
        print("Warning: No PyO3 modules found in Rust source files")
        return 1

    generator = StubGenerator([info["name"] for info in parser.classes.values()])
    generator.generate_stub_file(parser.modules, parser.classes, output_file)

    extra_stubs = script_dir / "eqty_sdk" / "_rust_extra.pyi"
    if extra_stubs.exists():
        output_file.write_text(
            output_file.read_text().rstrip() + "\n\n# -- extra stubs --\n" + extra_stubs.read_text()
        )

    total_functions = sum(len(m["functions"]) for m in parser.modules.values())
    print(f"Successfully generated {total_functions} function stubs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
