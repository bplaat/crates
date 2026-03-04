#!/usr/bin/env python3
"""cContinue Transpiler v0.1.0"""

import argparse
import copy
from dataclasses import dataclass
import logging
import os
import platform
import re
import subprocess
import sys
import tempfile


# MARK: Types
@dataclass
class Field:
    """Class field metadata"""

    name: str
    type: str
    default: str | None
    attributes: dict[str, list[str]]
    class_: str


@dataclass
class Argument:
    """Argument metadata"""

    name: str
    type: str


@dataclass
class Method:
    """Class method metadata"""

    name: str
    return_type: str
    is_return_self: bool
    is_virtual: bool
    is_static: bool
    arguments: list[Argument]
    class_: str
    origin_class: str


class Class:
    """Class metadata"""

    name: str
    snake_name: str
    parent_name: str | None
    is_abstract: bool
    fields: dict[str, Field]
    methods: dict[str, Method]
    interface_names: list[str]

    def __init__(self, name: str, parent_name: str | None) -> None:
        self.name = name
        self.snake_name = to_snake_case(name)
        self.parent_name = parent_name
        self.is_abstract = False
        self.fields = {}
        self.methods = {}
        self.interface_names = []


class Interface:
    """Interface metadata"""

    name: str
    snake_name: str
    id: int
    parent_names: list[str]
    methods: dict[str, Method]
    default_bodies: dict[str, str]

    def __init__(self, name: str, id_: int) -> None:
        self.name = name
        self.snake_name = to_snake_case(name)
        self.id = id_
        self.parent_names = []
        self.methods = {}
        self.default_bodies = {}


# MARK: Utils
def file_read(path: str) -> str:
    with open(path, "r", encoding="utf-8") as file:
        return file.read()


def file_write(path: str, text: str) -> None:
    with open(path, "w", encoding="utf-8") as file:
        file.write(text)


def to_snake_case(camel_case: str) -> str:
    string = "".join(["_" + char.lower() if char.isupper() else char for char in camel_case])
    return string[1:] if string.startswith("_") else string


def find_matching_close(text: str, start: int) -> int:
    """Return index of the closing char that matches the opening char at text[start]"""
    open_char = text[start]
    close_char = "}" if open_char == "{" else ")"
    depth = 0
    pos = start
    while pos < len(text):
        if text[pos] == open_char:
            depth += 1
        elif text[pos] == close_char:
            depth -= 1
            if depth == 0:
                return pos
        pos += 1
    return pos


def parse_arguments(arguments_str: str) -> list[Argument]:
    """Parse a comma-separated argument list string into Argument objects"""
    arguments = []
    if arguments_str.strip():
        for argument_str in arguments_str.split(","):
            argument_parts = re.search(
                r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)",
                argument_str,
            )
            if argument_parts is not None:
                arguments.append(Argument(argument_parts[2].strip(), argument_parts[1]))
    return arguments


# MARK: Transpiler
class Transpiler:
    """Transpiles cContinue source to C"""

    def __init__(self, include_paths: list[str]) -> None:
        self.include_paths = include_paths
        self.classes: dict[str, Class] = {}
        self.interfaces: dict[str, Interface] = {}
        self.next_interface_id: int = 1
        self.processed_includes: list[str] = []

    def reset(self) -> None:
        """Reset per-file state"""
        self.next_interface_id = 1
        self.interfaces = {}
        self.classes = {}
        self.processed_includes = []

    # MARK: Helpers
    def _find_class_for_method(self, class_: Class, method_name: str) -> Class:
        """Find first class in the hierarchy that implements method_name"""
        if class_.methods[method_name].class_ == class_.name:
            return class_
        if class_.parent_name is None:
            logging.error("No class implements method: %s", method_name)
            sys.exit(1)
        return self._find_class_for_method(self.classes[class_.parent_name], method_name)

    def _concrete_subclasses(self, target_name: str) -> list[str]:
        """Return names of all concrete classes that are equal to or inherit from target_name"""
        result = []
        for cls in self.classes.values():
            if cls.is_abstract:
                continue
            cur: Class | None = cls
            while cur is not None:
                if cur.name == target_name:
                    result.append(cls.name)
                    break
                cur = self.classes.get(cur.parent_name) if cur.parent_name else None
        return result

    def _static_method_signature(self, class_: Class, method: Method) -> str:
        """Build the shared signature string for a static method (no trailing punctuation)"""
        sig = f"{method.return_type} {class_.snake_name}_{method.name}("
        if method.arguments:
            sig += ", ".join(f"{a.type} {a.name}" for a in method.arguments)
        else:
            sig += "void"
        sig += ")"
        return sig

    def _codegen_static_method_declaration(self, class_: Class, method: Method) -> str:
        return self._static_method_signature(class_, method) + ";\n"

    def _codegen_static_method_definition(self, class_: Class, method: Method) -> str:
        code = self._static_method_signature(class_, method) + " {\n"
        if method.name == "new":
            code += f"    {class_.name}* this = malloc(sizeof({class_.name}));\n"
            code += f"    this->vtbl = &_{class_.name}Vtbl;\n"
            code += f"    {class_.snake_name}_init("
            code += ", ".join(["this"] + [a.name for a in method.arguments])
            code += ");\n"
            code += "    return this;\n"
        code += "}\n\n"
        return code

    # MARK: Convert includes
    def _convert_include(self, current_path: str, match: re.Match[str]) -> str:
        base_path = f"{match.groups()[0]}.hh"
        if base_path in self.processed_includes:
            return ""
        self.processed_includes.append(base_path)
        for include_path in self.include_paths:
            complete_path = f"{include_path}/{base_path}"
            if os.path.exists(complete_path):
                is_header = base_path.endswith(".hh") and os.path.abspath(complete_path) != os.path.abspath(
                    current_path.replace(".cc", ".hh")
                )
                return self.transpile(complete_path, is_header, file_read(complete_path))
        logging.error("Can't find include: %s", base_path)
        sys.exit(1)

    # MARK: Convert interfaces
    def _convert_interface(self, iface_name: str, supers_raw: str | None, contents: str) -> str:
        """Convert an interface declaration to C structs and dispatch macros"""
        snake_name = to_snake_case(iface_name)
        iface = Interface(iface_name, self.next_interface_id)
        self.next_interface_id += 1
        self.interfaces[iface_name] = iface

        if supers_raw is not None:
            for name in supers_raw.strip().lstrip(":").split(","):
                name = name.strip()
                if not name:
                    continue
                if name not in self.interfaces:
                    logging.error("Can't find parent interface %s for %s", name, iface_name)
                    sys.exit(1)
                iface.parent_names.append(name)
                for method in self.interfaces[name].methods.values():
                    iface.methods[method.name] = method

        for method_parts in re.findall(
            r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*(=\s*0\s*)?;",
            contents,
        ):
            return_type, name, arguments_str, _ = method_parts
            return_type = return_type.replace("virtual ", "").strip()
            arguments = parse_arguments(arguments_str)
            iface.methods[name] = Method(
                name, return_type.strip(), False, True, False, arguments, iface_name, iface_name
            )

        c = f"// interface {iface_name}\n"
        c += f"#define _{iface_name}_ID {iface.id}\n\n"

        c += f"typedef struct {iface_name}Vtbl {{\n"
        current_origin = ""
        for method in iface.methods.values():
            if method.origin_class != current_origin:
                c += f"    // {method.origin_class}\n"
                current_origin = method.origin_class
            c += f"    {method.return_type} (*{method.name})(void* this"
            for argument in method.arguments:
                c += f", {argument.type} {argument.name}"
            c += ");\n"
        c += f"}} {iface_name}Vtbl;\n\n"

        c += f"typedef struct {iface_name} {{\n"
        c += "    void* obj;\n"
        c += f"    const {iface_name}Vtbl* vtbl;\n"
        c += f"}} {iface_name};\n\n"

        for method in iface.methods.values():
            c += f"#define {snake_name}_{method.name}(iface"
            for argument in method.arguments:
                c += f", {argument.name}"
            c += f") ((iface).vtbl->{method.name}((iface).obj"
            for argument in method.arguments:
                c += f", ({argument.name})"
            c += "))\n"
        c += "\n"

        return c

    # MARK: Convert classes: indexing
    def _index_class(
        self, class_name: str, supers_raw: str | None, contents: str
    ) -> tuple[Class, "Class | None"]:
        """Parse class body, build and register the Class object. Returns (class_, parent_class)."""
        parent_name: str | None = None if class_name == "Object" else "Object"
        explicit_interfaces: list[str] = []

        if supers_raw is not None:
            for name in supers_raw.strip().lstrip(":").split(","):
                name = name.strip()
                if not name:
                    continue
                if name in self.classes:
                    parent_name = name
                elif name in self.interfaces:
                    if name not in explicit_interfaces:
                        explicit_interfaces.append(name)
                else:
                    logging.error("Unknown class or interface '%s' in class %s", name, class_name)
                    sys.exit(1)

        parent_class: Class | None = None
        if parent_name is not None:
            if parent_name not in self.classes:
                logging.error("Can't find parent class %s for %s", parent_name, class_name)
                sys.exit(1)
            parent_class = self.classes[parent_name]

        class_ = Class(class_name, parent_name)
        if parent_class is not None:
            class_.fields = copy.deepcopy(parent_class.fields)
            class_.methods = copy.deepcopy(parent_class.methods)
            class_.interface_names = list(parent_class.interface_names)

        def add_interface(iface_name: str) -> None:
            if iface_name in class_.interface_names:
                return
            class_.interface_names.append(iface_name)
            for pname in self.interfaces[iface_name].parent_names:
                add_interface(pname)

        for iface_name in explicit_interfaces:
            add_interface(iface_name)

        self.classes[class_.name] = class_

        # Index fields
        for field_parts in re.findall(r"(.+[^=][\s|\*])([_A-Za-z][_A-Za-z0-9]*)\s*(=\s*[^;]+)?;", contents):
            attributes_and_type_str, name, default_str = field_parts
            attributes: dict[str, list[str]] = {}
            for attr_m in re.finditer(r"@([_A-Za-z][_A-Za-z0-9]*)(\([^\)]*\))?", attributes_and_type_str):
                attr_name, attr_args = attr_m.groups()
                if attr_args is not None:
                    attributes[attr_name] = [a.strip() for a in attr_args[1 : len(attr_args) - 1].split(",")]
                else:
                    attributes[attr_name] = []
            field_type = re.sub(r"@([_A-Za-z][_A-Za-z0-9]*)(\([^\)]*\))?", "", attributes_and_type_str).strip()
            if name in class_.fields:
                logging.error("Can't inherit field: %s", name)
                sys.exit(1)
            class_.fields[name] = Field(
                name, field_type, default_str[1:].strip() if default_str != "" else None, attributes, class_.name
            )

        # Index methods
        for method_parts in re.findall(
            r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*(=\s*0)?;",
            contents,
        ):
            return_type, name, arguments_str, is_zero = method_parts
            arguments = parse_arguments(arguments_str)

            is_static = False
            if "static " in return_type:
                return_type = return_type.replace("static ", "")
                is_static = True

            is_virtual = False
            if "virtual " in return_type:
                return_type = return_type.replace("virtual ", "")
                is_virtual = True

            is_return_self = False
            if re.match(r"Self\s*\*", return_type):
                is_return_self = True
                return_type = return_type.replace("Self", class_.name)

            if is_zero != "":
                if is_virtual:
                    class_.is_abstract = True
                else:
                    logging.error("Only virtual methods can be set to zero: %s", name)
                    sys.exit(1)

            if name in class_.methods:
                class_.methods[name].return_type = return_type
                class_.methods[name].arguments = arguments
                class_.methods[name].class_ = class_.name
                class_.methods[name].is_static = is_static
            else:
                class_.methods[name] = Method(
                    name, return_type, is_return_self, is_virtual, is_static, arguments, class_.name, class_.name
                )

        return class_, parent_class

    # MARK: Convert classes: codegen
    def _codegen_missing_methods(self, class_: Class, parent_class: "Class | None", is_header: bool) -> str:
        """Generate auto-derived methods (init/deinit/getters/setters/new) and return their C code.
        Also registers the generated methods in class_.methods."""
        g = ""

        if parent_class is not None:
            # Auto init
            field_needs_init = next(
                (f for f in class_.fields.values() if "init" in f.attributes or f.default is not None),
                None,
            )
            if class_.methods["init"].class_ != class_.name and field_needs_init is not None:
                class_.methods["init"].class_ = class_.name
                class_.methods["init"].arguments = [
                    Argument(f.name, f.type) for f in class_.fields.values() if "init" in f.attributes
                ]
                init_method = class_.methods["init"]
                parent_init_method = self.classes[parent_class.name].methods["init"]

                g += f"void _{class_.snake_name}_init("
                g += ", ".join(
                    [f"{class_.name}* this"] + [f"{a.type} {a.name}" for a in init_method.arguments]
                )
                g += ") {\n"
                g += f"    {parent_class.snake_name}_init("
                g += ", ".join(["this"] + [a.name for a in parent_init_method.arguments])
                g += ");\n"
                for field in class_.fields.values():
                    if field.class_ == class_.name:
                        if field.default is not None:
                            g += f"    this->{field.name} = {field.default};\n"
                        if "init" in field.attributes:
                            if len(field.attributes["init"]) > 0:
                                g += f"    this->{field.name} = {field.attributes['init'][0]}({field.name});\n"
                            else:
                                g += f"    this->{field.name} = {field.name};\n"
                g += "}\n\n"

            # Auto deinit
            field_needs_deinit = any(
                f.class_ == class_.name and "deinit" in f.attributes for f in class_.fields.values()
            )
            if field_needs_deinit:
                class_.methods["deinit"].class_ = class_.name
                g += f"void _{class_.snake_name}_deinit({class_.name}* this) {{\n"
                for field in class_.fields.values():
                    if field.class_ == class_.name and "deinit" in field.attributes:
                        if len(field.attributes["deinit"]) > 0:
                            g += f"    {field.attributes['deinit'][0]}(this->{field.name});\n"
                        else:
                            for other_class in self.classes.values():
                                if field.type.startswith(other_class.name):
                                    g += f"    {other_class.snake_name}_free(this->{field.name});\n"
                                    break
                            else:
                                g += f"    free(this->{field.name});\n"
                class_with_deinit = self._find_class_for_method(self.classes[parent_class.name], "deinit")
                g += f"    _{class_with_deinit.snake_name}_deinit(({class_with_deinit.name}*)this);\n"
                g += "}\n\n"

            # Auto getters
            for field in class_.fields.values():
                if field.class_ == class_.name and ("get" in field.attributes or "prop" in field.attributes):
                    method_name = f"get_{field.name}"
                    class_.methods[method_name] = Method(
                        method_name, field.type, False, False, False, [], class_.name, class_.name
                    )
                    g += f"{field.type} _{class_.snake_name}_get_{field.name}({class_.name}* this) {{\n"
                    g += f"    return this->{field.name};\n"
                    g += "}\n\n"

            # Auto setters
            for field in class_.fields.values():
                if field.class_ == class_.name and ("set" in field.attributes or "prop" in field.attributes):
                    method_name = f"set_{field.name}"
                    class_.methods[method_name] = Method(
                        method_name, "void", False, False, False,
                        [Argument(field.name, field.type)], class_.name, class_.name,
                    )
                    g += f"void _{class_.snake_name}_set_{field.name}({class_.name}* this, "
                    g += f"{field.type} {field.name}) {{\n"
                    g += f"    this->{field.name} = {field.name};\n"
                    g += "}\n\n"

        # New method (all non-abstract classes, including those without parents)
        if not class_.is_abstract:
            init_method = class_.methods["init"]
            class_.methods["new"] = Method(
                "new", f"{class_.name}*", False, False, True, init_method.arguments, class_.name, class_.name
            )
            if not is_header:
                g += self._codegen_static_method_definition(class_, class_.methods["new"])

        return g

    def _codegen_class_struct(self, class_: Class) -> str:
        """Generate typedef, vtbl struct, extern decl, and struct body."""
        c = f"typedef struct {class_.name} {class_.name};\n\n"
        c += f"typedef struct {class_.name}Vtbl {{\n"
        c += "    const _InterfaceSlot* interfaces;\n"
        current_class_name = ""
        for method in class_.methods.values():
            if method.is_virtual:
                if method.origin_class != current_class_name:
                    c += f"    // {method.origin_class}\n"
                    current_class_name = method.origin_class
                c += f"    {method.return_type} (*{method.name})("
                c += ", ".join(
                    [f"{method.class_}* this"] + [f"{a.type} {a.name}" for a in method.arguments]
                )
                c += ");\n"
        c += f"}} {class_.name}Vtbl;\n\n"
        if not class_.is_abstract:
            c += f"extern {class_.name}Vtbl _{class_.name}Vtbl;\n\n"
        c += f"struct {class_.name} {{\n"
        c += f"    {class_.name}Vtbl* vtbl;\n"
        current_class_name = ""
        for field in class_.fields.values():
            if field.class_ != current_class_name:
                c += f"    // {field.class_}\n"
                current_class_name = field.class_
            c += f"    {field.type} {field.name};\n"
        c += "};\n\n"
        return c

    def _codegen_class_forward_decls(self, class_: Class) -> str:
        """Generate forward declarations for all class methods."""
        c = ""
        if not class_.is_abstract and "new" in class_.methods:
            c += self._codegen_static_method_declaration(class_, class_.methods["new"])
        for method in class_.methods.values():
            if method.class_ == class_.name and method.name != "new":
                if method.is_static:
                    c += self._codegen_static_method_declaration(class_, method)
                else:
                    c += (
                        f"{method.return_type} _{class_.snake_name}_{method.name}("
                        + ", ".join(
                            [f"{class_.name}* this"]
                            + [f"{a.type} {a.name}" for a in method.arguments]
                        )
                        + ");\n"
                    )
        c += "\n"
        return c

    def _codegen_class_vtbl_instance(self, class_: Class) -> str:
        """Generate per-interface vtbl instances and the main class vtbl instance."""
        c = ""
        if class_.interface_names:
            for iface_name in class_.interface_names:
                iface = self.interfaces[iface_name]
                c += f"static const {iface_name}Vtbl _{class_.name}{iface_name}Vtbl = {{\n"
                for method in iface.methods.values():
                    c += f"    ({method.return_type}(*)(void*"
                    for argument in method.arguments:
                        c += f", {argument.type}"
                    c += "))"
                    if method.name in class_.methods:
                        impl_class = to_snake_case(class_.methods[method.name].class_)
                        c += f"&_{impl_class}_{method.name},\n"
                    elif method.name in iface.default_bodies:
                        c += f"&_{iface.snake_name}_{method.name},\n"
                    else:
                        logging.error(
                            "Class %s implements %s but does not provide '%s' and there is no default",
                            class_.name, iface_name, method.name,
                        )
                        sys.exit(1)
                c += "};\n\n"

            c += f"static const _InterfaceSlot _{class_.name}Interfaces[] = {{\n"
            for iface_name in class_.interface_names:
                iface = self.interfaces[iface_name]
                c += f"    {{ _{iface_name}_ID, &_{class_.name}{iface_name}Vtbl }},\n"
            c += "    { 0, NULL }\n"
            c += "};\n\n"

        c += f"{class_.name}Vtbl _{class_.name}Vtbl = {{\n"
        if class_.interface_names:
            c += f"    _{class_.name}Interfaces,\n"
        else:
            c += "    NULL,\n"
        current_class_name = ""
        for method in class_.methods.values():
            if method.is_virtual:
                if method.origin_class != current_class_name:
                    c += f"    // {method.origin_class}\n"
                    current_class_name = method.origin_class
                c += f"    &_{to_snake_case(method.class_)}_{method.name},\n"
        c += "};\n\n"
        return c

    def _codegen_class_macros(self, class_: Class) -> str:
        """Generate #define method dispatch macros for all non-static methods."""
        c = ""
        for method in class_.methods.values():
            if method.is_static:
                continue
            return_cast = f"({class_.name}*)" if method.is_return_self else ""
            target = f"_{to_snake_case(method.class_)}_{method.name}"
            if method.is_virtual:
                target = f"(({class_.name}*)(this))->vtbl->{method.name}"
            c += f"#define {class_.snake_name}_{method.name}("
            c += ", ".join(["this"] + [a.name for a in method.arguments])
            c += f") {return_cast}{target}(({method.class_}*)(this)"
            for argument in method.arguments:
                for other_class in self.classes.values():
                    if argument.type.startswith(other_class.name):
                        c += f", ({other_class.name}*)({argument.name})"
                        break
                else:
                    c += f", ({argument.name})"
            c += ")\n"
        c += "\n"
        return c

    def _convert_class(self, is_header: bool, class_name: str, supers_raw: str | None, contents: str) -> str:
        """Convert a class declaration to C."""
        class_, parent_class = self._index_class(class_name, supers_raw, contents)
        g = self._codegen_missing_methods(class_, parent_class, is_header)

        c = self._codegen_class_struct(class_)
        c += self._codegen_class_forward_decls(class_)
        if not is_header and not class_.is_abstract:
            c += self._codegen_class_vtbl_instance(class_)
        c += self._codegen_class_macros(class_)
        if not is_header:
            c += g

        return c

    # MARK: Convert method / super calls
    def _convert_method(self, match: re.Match[str]) -> str:
        """Convert a ClassName::method_name(...) { definition header to C"""
        return_type, class_name, method_name, arguments = match.groups()
        if class_name not in self.classes:
            logging.error("Can't find class: %s", class_name)
            sys.exit(1)
        class_ = self.classes[class_name]
        method = class_.methods.get(method_name)
        if method is None:
            logging.error("Can't find method: %s::%s", class_name, method_name)
            sys.exit(1)
        if method.is_return_self:
            return_type = return_type.replace("Self", class_.name)
        if method.is_static:
            c = f"{return_type.strip()} {class_.snake_name}_{method_name}({arguments if arguments.strip() else 'void'}) {{"
        else:
            arguments_str = f", {arguments}" if len(arguments) > 0 else ""
            c = f"{return_type.strip()} _{class_.snake_name}_{method_name}({class_name}* this{arguments_str}) {{"
        if method_name == "init":
            for field in class_.fields.values():
                if field.class_ == class_.name and field.default is not None:
                    c += f"\n    this->{field.name} = {field.default};"
        if method_name == "deinit":
            for field in class_.fields.values():
                if field.class_ == class_.name and "deinit" in field.attributes:
                    if len(field.attributes["deinit"]) > 0:
                        c += f"\n    {field.attributes['deinit'][0]}(this->{field.name});"
                    else:
                        c += f"\n    free(this->{field.name});"
        return c

    def _convert_super_call(self, match: re.Match[str]) -> str:
        """Convert a SuperClass::method(args); super call to C"""
        parent_class_name, method_name, arguments = match.groups()
        if parent_class_name not in self.classes:
            logging.error("Can't find class: %s", parent_class_name)
            sys.exit(1)
        parent_class = self.classes[parent_class_name]
        method = parent_class.methods.get(method_name)
        if method is None:
            logging.error("Can't find method: %s::%s", parent_class_name, method_name)
            sys.exit(1)
        return (
            f"_{to_snake_case(method.class_)}_{method.name}(({method.class_}*)(this)"
            + (f", {arguments}" if len(arguments) > 0 else "")
            + ");"
        )

    # MARK: Transpile steps
    def _step_prelude_and_includes(self, path: str, is_header: bool, text: str) -> str:
        """Add prelude, strip #pragma once, expand #includes, rewrite literal syntax."""
        if not is_header:
            text = '// @generated\n#include "prelude.h"\n#include "Object.hh"\n' + text
        text = re.sub(r"#pragma once\n", "", text)
        text = re.sub(r"#include\s*[\"<](.+).hh[\">]", lambda m: self._convert_include(path, m), text)
        text = re.sub(r'@"([^"]*)"', lambda sm: f'string_new("{sm.group(1)}")', text)
        text = re.sub(r"@(true|false)(?![_A-Za-z0-9])", lambda sm: f"bool_new({sm.group(1)})", text)
        text = re.sub(r"@([0-9]+\.[0-9]+)", lambda sm: f"float_new({sm.group(1)})", text)
        text = re.sub(r"@([0-9]+)", lambda sm: f"int_new({sm.group(1)})", text)
        return text

    def _step_interfaces(self, text: str) -> str:
        """Replace interface class declarations with C structs."""
        while True:
            m = re.search(
                r"class\s+(I[A-Z][_A-Za-z0-9]*)(\s*:\s*[_A-Za-z][_A-Za-z0-9,\s]*)?\s*\{",
                text,
            )
            if not m:
                break
            start = m.end() - 1
            pos = find_matching_close(text, start)
            body = text[start + 1 : pos]
            end = pos + 1
            if end < len(text) and text[end] == ";":
                end += 1
            replacement = self._convert_interface(m.group(1), m.group(2), body)
            text = text[: m.start()] + replacement + text[end:]
        return text

    def _step_prescan_default_bodies(self, text: str) -> None:
        """Flag interface default method bodies before class codegen."""
        for iface_name, iface in self.interfaces.items():
            for dm in re.finditer(
                rf"[_A-Za-z][_A-Za-z0-9 ]*[\**|\s+]\s*{re.escape(iface_name)}::([_A-Za-z][_A-Za-z0-9]*)\(",
                text,
            ):
                method_name = dm.group(1)
                if method_name in iface.methods:
                    iface.default_bodies[method_name] = ""

    def _step_classes(self, text: str, is_header: bool) -> str:
        """Convert class forward decls and full class declarations."""
        text = re.sub(r"class\s+([_A-Za-z][_A-Za-z0-9]*)\s*;", r"typedef struct \1 \1;", text)
        while True:
            m = re.search(
                r"class\s+([_A-Za-z][_A-Za-z0-9]*)(\s*:\s*[_A-Za-z][_A-Za-z0-9,\s]*)?\s*\{",
                text,
            )
            if not m:
                break
            start = m.end() - 1
            pos = find_matching_close(text, start)
            body = text[start + 1 : pos]
            end = pos + 1
            if end < len(text) and text[end] == ";":
                end += 1
            replacement = self._convert_class(is_header, m.group(1), m.group(2), body)
            text = text[: m.start()] + replacement + text[end:]
        return text

    def _step_default_body_implementations(self, text: str) -> str:
        """Replace interface default method bodies with static C functions."""
        while True:
            found = False
            for cur_iface_name, cur_iface in self.interfaces.items():
                dm = re.search(
                    rf"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*{re.escape(cur_iface_name)}::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*\{{",
                    text,
                )
                if dm:
                    ret_type, method_name, arguments_str = dm.group(1), dm.group(2), dm.group(3)
                    if method_name not in cur_iface.methods:
                        logging.error("Interface %s has no method '%s'", cur_iface_name, method_name)
                        sys.exit(1)
                    dstart = dm.end() - 1
                    dpos = find_matching_close(text, dstart)
                    dend = dpos + 1
                    body_text = text[dstart + 1 : dpos]
                    def_arguments = parse_arguments(arguments_str)

                    snake_iface = cur_iface.snake_name
                    fn_code = f"static {ret_type.strip()} _{snake_iface}_{method_name}(void* this"
                    for arg in def_arguments:
                        fn_code += f", {arg.type} {arg.name}"
                    fn_code += ") {\n"
                    fn_code += f"    const {cur_iface_name}Vtbl* _vtbl;\n"
                    fn_code += "    {\n"
                    fn_code += (
                        "        const _InterfaceSlot* _s = *(const _InterfaceSlot* const*)*(void* const*)this;\n"
                    )
                    fn_code += "        _vtbl = NULL;\n"
                    fn_code += "        if (_s) for (; _s->id; _s++) {\n"
                    fn_code += f"            if (_s->id == _{cur_iface_name}_ID) {{ _vtbl = (const {cur_iface_name}Vtbl*)_s->vtbl; break; }}\n"
                    fn_code += "        }\n"
                    fn_code += "    }\n"
                    transformed_body = body_text
                    for m_name in cur_iface.methods:
                        transformed_body = re.sub(
                            rf"\b{re.escape(m_name)}\(this\b",
                            f"_vtbl->{m_name}(this",
                            transformed_body,
                        )
                    fn_code += transformed_body
                    fn_code += "}\n\n"

                    cur_iface.default_bodies[method_name] = fn_code
                    text = text[: dm.start()] + fn_code + text[dend:]
                    found = True
                if found:
                    break
            if not found:
                break
        return text

    def _step_methods_and_super_calls(self, text: str) -> str:
        """Convert method definitions and super calls."""
        text = re.sub(
            r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*{",
            self._convert_method,
            text,
        )
        text = re.sub(
            r"([_A-Za-z][_A-Za-z0-9]*)::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*;",
            self._convert_super_call,
            text,
        )
        return text

    def _step_for_in(self, text: str) -> str:
        """Expand for (Type* var in expr) { body } loops."""
        counter = 0
        while True:
            fm = re.search(
                r"for\s*\(\s*([_A-Za-z][_A-Za-z0-9 \*]*\*?)\s+([_A-Za-z][_A-Za-z0-9]*)\s+in\s+([^\)]+)\)\s*\{",
                text,
            )
            if not fm:
                break
            var_type = fm.group(1).strip()
            var_name = fm.group(2).strip()
            iterable_expr = fm.group(3).strip()
            iter_var = f"_iter_{counter}"
            counter += 1
            fstart = fm.end() - 1
            fpos = find_matching_close(text, fstart)
            body = text[fstart + 1 : fpos]
            fend = fpos + 1
            replacement = (
                f"{{\n"
                f"    IIterator {iter_var} = i_iterable_iterator(cast<IIterable>({iterable_expr}));\n"
                f"    while (i_iterator_has_next({iter_var})) {{\n"
                f"        {var_type} {var_name} = ({var_type})i_iterator_next({iter_var});\n"
                f"        {body.strip()}\n"
                f"    }}\n"
                f"    object_free((Object*){iter_var}.obj);\n"
                f"}}"
            )
            text = text[: fm.start()] + replacement + text[fend:]
        return text

    def _step_cast(self, text: str) -> str:
        """Convert cast<Iface>(expr) to fat-pointer construction."""
        ifaces_used: list[str] = []
        for cast_m in re.finditer(r"cast<([_A-Za-z][_A-Za-z0-9]*)>", text):
            name = cast_m.group(1)
            if name in self.interfaces and name not in ifaces_used:
                ifaces_used.append(name)

        if ifaces_used:
            lookup_code = ""
            for lk_iface_name in ifaces_used:
                lookup_code += f"static {lk_iface_name} _cast_{lk_iface_name}(void* obj) {{\n"
                lookup_code += "    Object* _obj = (Object*)obj;\n"
                lookup_code += "    const _InterfaceSlot* s = _obj->vtbl->interfaces;\n"
                lookup_code += f"    const {lk_iface_name}Vtbl* vtbl = NULL;\n"
                lookup_code += "    if (s) for (; s->id; s++) {\n"
                lookup_code += f"        if (s->id == _{lk_iface_name}_ID) {{ vtbl = (const {lk_iface_name}Vtbl*)s->vtbl; break; }}\n"
                lookup_code += "    }\n"
                lookup_code += f"    return ({lk_iface_name}){{ .obj = _obj, .vtbl = vtbl }};\n"
                lookup_code += "}\n\n"
            cast_main_marker = re.search(r"\bint\s+main\s*\(", text)
            if cast_main_marker:
                ins_pos = cast_main_marker.start()
            else:
                insert_marker = re.search(r"(}\n\n)(?=[a-zA-Z#])", text)
                ins_pos = insert_marker.end() if insert_marker else len(text)
            text = text[:ins_pos] + lookup_code + text[ins_pos:]

        while True:
            cast_m = re.search(r"cast<([_A-Za-z][_A-Za-z0-9]*)>\(", text)
            if not cast_m:
                break
            cast_iface_name = cast_m.group(1)
            if cast_iface_name not in self.interfaces:
                break
            cstart = cast_m.end() - 1
            cpos = find_matching_close(text, cstart)
            obj_expr = text[cstart + 1 : cpos].strip()
            cend = cpos + 1
            text = text[: cast_m.start()] + f"_cast_{cast_iface_name}((void*)({obj_expr}))" + text[cend:]

        return text

    def _step_instanceof(self, text: str) -> str:
        """Convert instanceof<X>(expr) to bool checks."""
        types_for_instanceof: list[str] = []
        for inst_m2 in re.finditer(r"instanceof<([_A-Za-z][_A-Za-z0-9]*)>", text):
            type_name = inst_m2.group(1)
            if type_name not in types_for_instanceof:
                types_for_instanceof.append(type_name)

        if types_for_instanceof:
            instanceof_code = ""
            declared_vtbls: list[str] = []
            for type_name in types_for_instanceof:
                if type_name in self.classes:
                    for sub in self._concrete_subclasses(type_name):
                        if sub not in declared_vtbls:
                            instanceof_code += f"typedef struct {sub}Vtbl {sub}Vtbl;\n"
                            instanceof_code += f"extern {sub}Vtbl _{sub}Vtbl;\n"
                            declared_vtbls.append(sub)
            instanceof_code += "\n"

            for type_name in types_for_instanceof:
                if type_name in self.interfaces:
                    instanceof_code += f"static bool _instanceof_{type_name}(void* obj) {{\n"
                    instanceof_code += "    Object* _obj = (Object*)obj;\n"
                    instanceof_code += "    const _InterfaceSlot* s = _obj->vtbl->interfaces;\n"
                    instanceof_code += "    if (!s) return false;\n"
                    instanceof_code += "    for (; s->id; s++)\n"
                    instanceof_code += f"        if (s->id == _{type_name}_ID) return true;\n"
                    instanceof_code += "    return false;\n"
                    instanceof_code += "}\n\n"
                elif type_name in self.classes:
                    subs = self._concrete_subclasses(type_name)
                    instanceof_code += f"static bool _instanceof_{type_name}(void* obj) {{\n"
                    instanceof_code += "    Object* _obj = (Object*)obj;\n"
                    if subs:
                        checks = " ||\n        ".join(f"_obj->vtbl == (void*)&_{s}Vtbl" for s in subs)
                        instanceof_code += f"    return {checks};\n"
                    else:
                        instanceof_code += "    return false;\n"
                    instanceof_code += "}\n\n"
                else:
                    logging.warning(
                        "Type '%s' used in instanceof<> is not defined as a class or interface", type_name
                    )

            main_marker = re.search(r"\bint\s+main\s*\(", text)
            if main_marker:
                ins_pos2 = main_marker.start()
            else:
                func_marker = re.search(r"\n(?:typedef|extern|#|struct|}\s*;)", text)
                if func_marker:
                    rest = text[func_marker.end() :]
                    func_impl = re.search(r"\n[_A-Za-z*][_A-Za-z0-9*\s]*\s+\**[_A-Za-z][_A-Za-z0-9]*\s*\(", rest)
                    if func_impl:
                        ins_pos2 = func_marker.end() + func_impl.start()
                    else:
                        ins_pos2 = len(text)
                else:
                    ins_pos2 = len(text)
            text = text[:ins_pos2] + instanceof_code + text[ins_pos2:]

        while True:
            inst_m = re.search(r"instanceof<([_A-Za-z][_A-Za-z0-9]*)>\(", text)
            if not inst_m:
                break
            inst_type_name = inst_m.group(1)
            if inst_type_name not in self.interfaces and inst_type_name not in self.classes:
                logging.error(
                    "Type '%s' used in instanceof<> is not defined as a class or interface", inst_type_name
                )
                text = text[: inst_m.start()] + "false" + text[inst_m.end() :]
                continue
            istart = inst_m.end() - 1
            ipos = find_matching_close(text, istart)
            inst_expr = text[istart + 1 : ipos].strip()
            iend = ipos + 1
            text = text[: inst_m.start()] + f"_instanceof_{inst_type_name}({inst_expr})" + text[iend:]

        return text

    def transpile(self, path: str, is_header: bool, text: str) -> str:
        """Run all transpilation steps on the given source text."""
        text = self._step_prelude_and_includes(path, is_header, text)
        text = self._step_interfaces(text)
        if not is_header:
            self._step_prescan_default_bodies(text)
        text = self._step_classes(text, is_header)
        if not is_header:
            text = self._step_default_body_implementations(text)
            text = self._step_methods_and_super_calls(text)
            text = self._step_for_in(text)
            text = self._step_cast(text)
            text = self._step_instanceof(text)
        return text


# MARK: Main
def main() -> None:
    logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")

    parser = argparse.ArgumentParser()
    parser.add_argument("file", help="cContinue file", nargs="*")
    parser.add_argument("--output", "-o", help="Output file", required=False)
    parser.add_argument("--include", "-I", help="Include headers", action="append", default=[])
    parser.add_argument("--source", "-S", help="Only run transpile step", action="store_true")
    parser.add_argument("--compile", "-c", help="Only run transpile and compile steps", action="store_true")
    parser.add_argument("--run", "-r", help="Run linked binary", action="store_true")
    parser.add_argument("--run-leaks", "-R", help="Run linked binary with memory leaks checks", action="store_true")
    args = parser.parse_args()

    cc = os.environ.get("CC", "gcc")
    script_dir = os.path.dirname(__file__)
    include_paths = [".", f"{script_dir}/std"] + args.include
    source_paths = list(args.file)
    if not args.source and not args.compile:
        for path in os.listdir(f"{script_dir}/std"):
            if path.endswith(".c") or path.endswith(".cc"):
                source_paths.append(f"{script_dir}/std/{path}")

    transpiler = Transpiler(include_paths)
    object_paths: list[str] = []
    for path in source_paths:
        if path.endswith(".o"):
            object_paths.append(path)
            continue

        source_path = path
        if path.endswith(".hh") or path.endswith(".cc"):
            source_path = (
                args.output
                if args.output is not None
                else path.replace(".cc", ".c").replace(".hh", ".h") if args.source else tempfile.mktemp(".c")
            )
            transpiler.reset()
            file_write(source_path, transpiler.transpile(path, path.endswith(".hh"), file_read(path)))
            if args.source:
                sys.exit(0)

        object_path = (
            (args.output if args.output is not None else path.replace(".cc", ".o").replace(".c", ".o"))
            if args.compile
            else tempfile.mktemp(".o")
        )
        object_paths.append(object_path)
        subprocess.run(
            [cc]
            + ["--std=c23", "-Wall", "-Wextra", "-Wpedantic", "-Werror"]
            + [f"-I{include_path}" for include_path in include_paths]
            + ["-c", source_path, "-o", object_path],
            check=True,
        )
        if args.compile:
            sys.exit(0)

    exe_path = (
        args.output
        if args.output is not None
        else args.file[0].replace(".cc", ".exe" if platform.system() == "Windows" else "")
    )
    subprocess.run([cc] + object_paths + ["-o", exe_path], check=True)

    if args.run:
        os.system(f"./{exe_path}")
    elif args.run_leaks:
        if platform.system() == "Darwin":
            os.system(f"leaks --atExit -- ./{exe_path}")
        elif platform.system() == "Linux":
            os.system(f"valgrind --leak-check=full --show-leak-kinds=all --track-origins=yes ./{exe_path}")
        else:
            logging.error("Memory leak checks are not supported on this platform")
            sys.exit(1)


if __name__ == "__main__":
    main()
