#!/usr/bin/env python3
"""Script to automatically generate _rust.pyi stub file from Rust source code.
Run this after making changes to PyO3 functions in the Rust codebase.
"""

import re
from pathlib import Path
from typing import Dict, List, Optional


class RustTypeMapper:
    """Maps Rust types to Python types in stub files."""

    TYPE_MAP = {
        "String": "str",
        "&str": "str",
        "PathBuf": "PathLike[str]",
        "Vec<String>": "List[str]",
        "Vec<PyObject>": "List[Any]",
        "Vec<u8>": "bytes",
        "Vec<&[u8]>": "List[bytes]",
        "Option<String>": "Optional[str]",
        "Option<&str>": "Optional[str]",
        "Option<bool>": "Optional[bool]",
        "Option<PyObject>": "Optional[Any]",
        "bool": "bool",
        "u8": "int",
        "u16": "int",
        "u32": "int",
        "u64": "int",
        "i32": "int",
        "i64": "int",
        "f32": "float",
        "f64": "float",
        "&[u8]": "bytes",
        "&PyDict": "Dict[str, Any]",
        "PyResult<()>": "None",
        "PyResult<String>": "str",
        "PyResult<bool>": "bool",
        "PyResult<Vec<String>>": "List[str]",
        "PyResult<DirCidResult>": "DirCidResult",
        "PyResult<CidResult>": "CidResult",
        "PyResult<Py<PySigner>>": "PySigner",
        "PyResult<(PyObject, PyObject)>": "Tuple[List[Any], Any]",
        "PyResult<HashMap<String, &PyBytes>>": "Dict[str, bytes]",
        "HashMap<String, bool>": "Dict[str, bool]",
        "HashMap<String, &PyBytes>": "Dict[str, bytes]",
        "&PyAny": "Any",
        "PyObject": "Any",
    }

    @classmethod
    def map_type(cls, rust_type: str) -> str:
        """Convert Rust type to Python type annotation."""
        # Clean up the type string
        rust_type = rust_type.strip()

        # Special case for Vec<u8> which should be bytes, not List[int]
        if rust_type == "Vec<u8>":
            return "bytes"

        # Handle PyDict types (with or without lifetime parameters)
        if "PyDict" in rust_type:
            return "Dict[str, Any]"

        # Handle generic types like Vec<T>
        if rust_type.startswith("Vec<") and rust_type.endswith(">"):
            inner_type = rust_type[4:-1]
            mapped_inner = cls.map_type(inner_type)
            return f"List[{mapped_inner}]"

        # Handle Option<T>
        if rust_type.startswith("Option<") and rust_type.endswith(">"):
            inner_type = rust_type[7:-1]
            mapped_inner = cls.map_type(inner_type)
            return f"Optional[{mapped_inner}]"

        # Handle PyResult<T>
        if rust_type.startswith("PyResult<") and rust_type.endswith(">"):
            inner_type = rust_type[9:-1]
            if inner_type == "()":
                return "None"
            return cls.map_type(inner_type)

        # Direct mapping
        return cls.TYPE_MAP.get(rust_type, "Any")


class PyFunctionParser:
    """Parses PyO3 function definitions from Rust source files."""

    def __init__(self, src_dir: Path):
        self.src_dir = src_dir
        self.modules = {}
        self.classes = set()

    def parse_all_files(self) -> Dict[str, List[dict]]:
        """Parse all Rust files and extract PyO3 function definitions."""
        # Find all .rs files
        rust_files = list(self.src_dir.rglob("*.rs"))

        for rust_file in rust_files:
            self._parse_file(rust_file)

        return self.modules

    def _parse_file(self, file_path: Path):
        """Parse a single Rust file for PyO3 functions."""
        try:
            content = file_path.read_text()

            # Find module name from pymodule attribute
            module_match = re.search(r"#\[pymodule\]\s*pub fn (\w+)", content)
            if module_match:
                module_name = module_match.group(1)
                if module_name not in self.modules:
                    self.modules[module_name] = []
            else:
                # Use parent directory name for statements submodules
                if "statements" in str(file_path):
                    module_name = "statements"
                else:
                    return

            # Find all pyfunction definitions
            functions = self._extract_functions(content)

            if module_name and functions:
                if module_name not in self.modules:
                    self.modules[module_name] = []
                self.modules[module_name].extend(functions)

            # Extract pyclass definitions
            self._extract_classes(content)

        except Exception as e:
            print(f"Warning: Could not parse {file_path}: {e}")

    def _skip_pyo3_attributes(self, content: str, start_pos: int) -> int:
        """Skip over any #[pyo3(...)] attributes after #[pyfunction], handling nested brackets."""
        pos = start_pos
        while pos < len(content):
            # Skip whitespace
            while pos < len(content) and content[pos] in " \t\n\r":
                pos += 1

            # Check if we have a #[pyo3 attribute
            if content[pos : pos + 7] == "#[pyo3(":
                pos += 7
                # Find the matching closing ]
                depth = 1  # We're inside [pyo3(
                while pos < len(content) and depth > 0:
                    if content[pos] == "(":
                        depth += 1
                    elif content[pos] == ")":
                        depth -= 1
                    pos += 1
                # Skip the closing ]
                if pos < len(content) and content[pos] == "]":
                    pos += 1
            else:
                # No more pyo3 attributes
                break

        return pos

    def _extract_functions(self, content: str) -> List[dict]:
        """Extract pyfunction definitions from file content."""
        functions = []

        # Find all #[pyfunction] markers
        pyfunction_pattern = r"#\[pyfunction\]"

        for match in re.finditer(pyfunction_pattern, content):
            start_pos = match.end()

            # Skip over any #[pyo3(...)] attributes
            pos_after_attrs = self._skip_pyo3_attributes(content, start_pos)
            section = content[pos_after_attrs:]

            # Match the function definition
            # Simplified pattern that just looks for fn name() -> return_type {
            func_pattern = r"^\s*(?:\/\/\/[^\n]*)?\s*(?:pub\s+)?fn\s+(\w+)(?:<[^>]*>)?\s*\(.*?\)\s*(?:->\s*([^{]+?))?\s*\{"
            func_match = re.match(func_pattern, section, re.MULTILINE | re.DOTALL)

            if not func_match:
                continue

            func_name = func_match.group(1)
            return_type = func_match.group(2).strip() if func_match.group(2) else "PyResult<()>"

            # Extract full function signature
            func_start = match.start()

            # Find the complete function definition
            func_def = self._extract_full_function(content, func_start)
            if func_def:
                params = self._parse_parameters(func_def)

                # Extract docstring from comments
                doc = self._extract_docstring(content, func_start)

                functions.append(
                    {
                        "name": func_name,
                        "params": params,
                        "return_type": RustTypeMapper.map_type(return_type),
                        "doc": doc,
                    }
                )

        return functions

    def _extract_full_function(self, content: str, start_pos: int) -> Optional[str]:
        """Extract the complete function definition including parameters."""
        lines = content[start_pos:].split("\n")
        func_lines = []
        brace_count = 0
        in_params = False

        for line in lines:
            if "fn " in line:
                in_params = True

            if in_params:
                func_lines.append(line)
                if "{" in line:
                    break

        return "\n".join(func_lines) if func_lines else None

    def _parse_parameters(self, func_def: str) -> List[dict]:
        """Parse function parameters from function definition."""
        params = []

        # Extract parameter list - handle multi-line and nested parentheses
        param_match = re.search(r"fn\s+\w+(?:<[^>]*>)?\s*\((.*?)\)", func_def, re.DOTALL)
        if not param_match:
            return params

        param_str = param_match.group(1)

        # Split parameters, handling nested generics
        raw_params = self._split_parameters(param_str)

        for param in raw_params:
            param = param.strip()
            if not param or param.startswith("_py:") or param.startswith("py:"):
                continue

            # Parse parameter name and type
            if ":" in param:
                name_part, type_part = param.split(":", 1)
                name = name_part.strip()
                rust_type = type_part.strip()

                # Handle references, mutability, and lifetime parameters
                name = name.replace("mut ", "").replace("&", "").strip()
                # Remove lifetime parameters and references from types
                rust_type = re.sub(r"&'?\w*\s*", "", rust_type).strip()
                rust_type = rust_type.replace("mut ", "").strip()

                # Check if parameter is optional
                is_optional = "Option<" in rust_type

                params.append(
                    {
                        "name": name,
                        "type": RustTypeMapper.map_type(rust_type),
                        "optional": is_optional,
                    }
                )

        return params

    def _split_parameters(self, param_str: str) -> List[str]:
        """Split parameter string, handling nested generics properly."""
        params = []
        current_param = ""
        paren_depth = 0
        angle_depth = 0

        for char in param_str:
            if char == "," and paren_depth == 0 and angle_depth == 0:
                if current_param.strip():
                    params.append(current_param.strip())
                current_param = ""
            else:
                current_param += char
                if char == "(":
                    paren_depth += 1
                elif char == ")":
                    paren_depth -= 1
                elif char == "<":
                    angle_depth += 1
                elif char == ">":
                    angle_depth -= 1

        if current_param.strip():
            params.append(current_param.strip())

        return params

    def _extract_docstring(self, content: str, func_start: int) -> str:
        """Extract docstring from comments before function."""
        lines_before = content[:func_start].split("\n")
        doc_lines = []

        # Look backwards for doc comments
        for line in reversed(lines_before[-10:]):  # Check last 10 lines
            line = line.strip()
            if line.startswith("///"):
                doc_lines.insert(0, line[3:].strip())
            elif line.startswith("//"):
                continue
            elif line == "" or line.startswith("#["):
                continue
            else:
                break

        return " ".join(doc_lines) if doc_lines else ""

    def _extract_classes(self, content: str):
        """Extract pyclass definitions."""
        class_pattern = r"#\[pyclass\]\s*(?:pub\s+)?(?:struct|enum)\s+(\w+)"
        matches = re.findall(class_pattern, content)
        for class_name in matches:
            self.classes.add(class_name)


class StubGenerator:
    """Generates Python stub file from parsed Rust functions."""

    def __init__(self):
        self.imports = {
            "pathlib": ["Path"],
            "typing": ["Any", "Dict", "List", "Optional", "Tuple", "Union"],
            "os": ["PathLike"],
        }

    def generate_stub_file(self, modules: Dict[str, List[dict]], classes: set, output_path: Path):
        """Generate the complete stub file."""
        lines = []

        # Header and imports
        lines.append('"""Type stubs for the eqty_sdk._rust module."""')
        lines.append("from pathlib import Path")
        lines.append("from typing import Any, Dict, List, Optional, Tuple, Union")
        lines.append("from os import PathLike")
        lines.append("")

        # Top-level functions (from lib.rs)
        if "_rust" in modules:
            for func in modules["_rust"]:
                lines.extend(self._generate_function_stub(func, is_module_function=False))
                lines.append("")

        # Module functions that appear at top level
        top_level_funcs = []
        for module_name, functions in modules.items():
            if module_name in ["lib", "_rust"]:
                top_level_funcs.extend(functions)

        # Generate class definitions for known classes
        lines.extend(self._generate_class_definitions(classes))

        # Generate module classes
        for module_name, functions in modules.items():
            if module_name in ["_rust", "lib"]:
                continue

            lines.append(f"# {module_name.title()} module")
            lines.append(f"class {module_name}:")

            if not functions:
                lines.append("    ...")
            else:
                for func in functions:
                    lines.extend(self._generate_function_stub(func, indent="    "))
                    lines.append("")

            lines.append("")

        # Write to file
        output_path.write_text("\n".join(lines))
        print(f"Generated stub file: {output_path}")

    def _generate_function_stub(
        self, func: dict, is_module_function: bool = True, indent: str = ""
    ) -> List[str]:
        """Generate stub for a single function."""
        lines = []

        if is_module_function:
            lines.append(f"{indent}@staticmethod")

        # Build parameter list
        params = []
        for param in func["params"]:
            param_str = f"{param['name']}: {param['type']}"
            if param["optional"] and not param_str.endswith(" = None"):
                param_str += " = None"
            params.append(param_str)

        param_str = ", ".join(params)

        # Handle special signature cases
        if func["name"] == "create_data_statement":
            # Fix the keyword-only arguments
            param_parts = []
            required_params = []
            keyword_params = []

            for param in func["params"]:
                if param["name"] in ["graph_id", "timestamp"]:
                    keyword_params.append(f"{param['name']}: {param['type']} = None")
                else:
                    required_params.append(f"{param['name']}: {param['type']}")

            if keyword_params:
                param_str = ", ".join(required_params) + ", *, " + ", ".join(keyword_params)
            else:
                param_str = ", ".join(required_params)

        lines.append(f"{indent}def {func['name']}({param_str}) -> {func['return_type']}:")

        # Add docstring
        doc = func["doc"] or f"{func['name'].replace('_', ' ').title()}."
        lines.append(f'{indent}    """{doc}"""')
        lines.append(f"{indent}    ...")

        return lines

    def _generate_class_definitions(self, classes: set) -> List[str]:
        """Generate class definitions for PyO3 classes."""
        lines = []

        # Known class structures
        class_definitions = {
            "Canon": {"attributes": ["RDFC1", "JSONJCS"], "doc": "Canonicalization options."},
            "CidResult": {
                "properties": [
                    ("cid", "str", "Get the CID string."),
                    ("blob", "bytes", "Get the binary blob data."),
                ],
                "doc": "Result of CID computation.",
            },
            "DirCidResult": {
                "properties": [
                    ("collection", "CidResult", "Get the collection CID result."),
                    ("meta", "CidResult", "Get the metadata CID result."),
                    ("file_hashes", "List[Tuple[str, str]]", "Get list of (filename, CID) tuples."),
                ],
                "doc": "Result of directory CID computation.",
            },
            "PySigner": {
                "properties": [
                    ("name", "str", "Get the signer name."),
                    ("did_key", "str", "Get the DID key."),
                ],
                "doc": "Python wrapper for Rust signer.",
            },
        }

        for class_name in sorted(classes):
            if class_name in class_definitions:
                definition = class_definitions[class_name]
                lines.append(f"class {class_name}:")
                lines.append(f'    """{definition["doc"]}"""')

                # Add attributes
                if "attributes" in definition:
                    for attr in definition["attributes"]:
                        lines.append(f"    {attr}: {class_name}")

                # Add properties
                if "properties" in definition:
                    for prop_name, prop_type, prop_doc in definition["properties"]:
                        lines.append("    @property")
                        lines.append(f"    def {prop_name}(self) -> {prop_type}:")
                        lines.append(f'        """{prop_doc}"""')
                        lines.append("        ...")
                        lines.append("")

                if "attributes" in definition and "properties" not in definition:
                    lines.append("")

                lines.append("")

        return lines


def main():
    """Main function to generate stub file."""
    # Paths
    script_dir = Path(__file__).parent
    src_dir = script_dir / "src"
    output_file = script_dir / "eqty_sdk" / "_rust.pyi"

    if not src_dir.exists():
        print(f"Error: Source directory not found: {src_dir}")
        return 1

    # Parse Rust files
    parser = PyFunctionParser(src_dir)
    modules = parser.parse_all_files()

    if not modules:
        print("Warning: No PyO3 functions found in Rust source files")
        return 1

    # Generate stub file
    generator = StubGenerator()
    generator.generate_stub_file(modules, parser.classes, output_file)

    print(f"Successfully generated {len(sum(modules.values(), []))} function stubs")
    return 0


if __name__ == "__main__":
    exit(main())
