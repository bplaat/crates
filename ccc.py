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
from typing import Dict, List, Optional


# MARK: Types
@dataclass
class Field:
    """Class field metadata"""

    name: str
    type: str
    default: Optional[str]
    attributes: Dict[str, List[str]]
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
    arguments: List[Argument]
    class_: str
    origin_class: str


class Class:
    """Class metadata"""

    name: str
    snake_name: str
    parent_name: Optional[str]
    is_abstract: bool
    fields: Dict[str, Field]
    methods: Dict[str, Method]
    interface_names: List[str]

    def __init__(self, name: str, parent_name: Optional[str]) -> None:
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
    parent_names: List[str]
    methods: Dict[str, Method]
    default_bodies: Dict[str, str]

    def __init__(self, name: str, id_: int) -> None:
        self.name = name
        self.snake_name = to_snake_case(name)
        self.id = id_
        self.parent_names = []
        self.methods = {}
        self.default_bodies = {}


# MARK: Globals
include_paths: List[str] = []
classes: Dict[str, Class] = {}
interfaces: Dict[str, Interface] = {}
next_interface_id: int = 1


# MARK: Utils
def file_read(path: str) -> str:
    """Read file"""
    with open(path, "r", encoding="utf-8") as file:
        return file.read()


def file_write(path: str, text: str) -> None:
    """Write file"""
    with open(path, "w", encoding="utf-8") as file:
        file.write(text)


def to_snake_case(camel_case: str) -> str:
    """Camel case to snake case"""
    string = "".join(["_" + char.lower() if char.isupper() else char for char in camel_case])
    return string[1:] if string.startswith("_") else string


def find_class_for_method(class_: Class, method_name: str) -> Class:
    """Find first class that implements method"""
    if class_.methods[method_name].class_ == class_.name:
        return class_
    if class_.parent_name is None:
        logging.error("No class implements method: %s", method_name)
        sys.exit(1)
    return find_class_for_method(classes[class_.parent_name], method_name)


# MARK: Convert include
class ConvertInclude:
    """Convert class"""

    processed_includes: List[str] = []

    def __init__(self, current_path: str) -> None:
        self.current_path = current_path

    def __call__(self, match: re.Match[str]) -> str:
        """Convert include"""
        base_path = f"{match.groups()[0]}.hh"
        if base_path in ConvertInclude.processed_includes:
            return ""
        ConvertInclude.processed_includes.append(base_path)

        for include_path in include_paths:
            complete_path = f"{include_path}/{base_path}"
            if os.path.exists(complete_path):
                is_header = base_path.endswith(".hh") and os.path.abspath(complete_path) != os.path.abspath(
                    self.current_path.replace(".cc", ".hh")
                )
                return transpile_text(complete_path, is_header, file_read(complete_path))
        logging.error("Can't find include: %s", base_path)
        sys.exit(1)


# MARK: Convert interface
class ConvertInterface:
    """Convert interface declaration"""

    def __init__(self, is_header: bool) -> None:
        self.is_header = is_header

    def __call__(self, iface_name: str, supers_raw: Optional[str], contents: str) -> str:
        global next_interface_id
        snake_name = to_snake_case(iface_name)

        # Create interface object
        iface = Interface(iface_name, next_interface_id)
        next_interface_id += 1
        interfaces[iface_name] = iface

        # Inherit parent interface methods (multi-parent support)
        if supers_raw is not None:
            for name in supers_raw.strip().lstrip(":").split(","):
                name = name.strip()
                if not name:
                    continue
                if name not in interfaces:
                    logging.error("Can't find parent interface %s for %s", name, iface_name)
                    sys.exit(1)
                iface.parent_names.append(name)
                parent_iface = interfaces[name]
                for method in parent_iface.methods.values():
                    iface.methods[method.name] = method

        # Index methods (all methods in an interface are implicitly virtual)
        for method_parts in re.findall(
            r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*(=\s*0\s*)?;",
            contents,
        ):
            return_type, name, arguments_str, _ = method_parts

            # Strip optional 'virtual' keyword (interface methods are always virtual)
            return_type = return_type.replace("virtual ", "").strip()

            arguments = []
            if arguments_str.strip() != "":
                for argument_str in arguments_str.split(","):
                    argument_parts = re.search(
                        r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)",
                        argument_str,
                    )
                    if argument_parts is not None:
                        arguments.append(Argument(argument_parts[2].strip(), argument_parts[1]))

            iface.methods[name] = Method(name, return_type.strip(), False, True, arguments, iface_name, iface_name)

        # ==== Codegen ====
        c = ""

        c += f"// interface {iface_name}\n"
        c += f"#define _{iface_name}_ID {iface.id}\n\n"

        # Interface vtable struct (parent methods first, with group comments)
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

        # Interface fat-pointer struct
        c += f"typedef struct {iface_name} {{\n"
        c += "    void* obj;\n"
        c += f"    const {iface_name}Vtbl* vtbl;\n"
        c += f"}} {iface_name};\n\n"

        # Method dispatch macros (fat pointer by value: use . not ->)
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


# MARK: Convert class
class ConvertClass:
    """Convert class"""

    def __init__(self, is_header: bool) -> None:
        self.is_header = is_header

    def __call__(self, class_name: str, supers_raw: Optional[str], contents: str) -> str:
        # ==== Indexing ====

        # Parse the unified `: Parent, Iface1, Iface2` list
        parent_name = None if class_name == "Object" else "Object"
        explicit_interfaces: List[str] = []

        if supers_raw is not None:
            for name in supers_raw.strip().lstrip(":").split(","):
                name = name.strip()
                if not name:
                    continue
                if name in classes:
                    parent_name = name
                elif name in interfaces:
                    if name not in explicit_interfaces:
                        explicit_interfaces.append(name)
                else:
                    logging.error("Unknown class or interface '%s' in class %s", name, class_name)
                    sys.exit(1)

        parent_class = None
        if parent_name is not None:
            if parent_name not in classes:
                logging.error("Can't find parent class %s for %s", parent_name, class_name)
                sys.exit(1)
            parent_class = classes[parent_name]

        # Create class object
        class_ = Class(class_name, parent_name)
        if parent_class is not None:
            class_.fields = copy.deepcopy(parent_class.fields)
            class_.methods = copy.deepcopy(parent_class.methods)
            class_.interface_names = list(parent_class.interface_names)

        # Add explicitly declared interfaces (and their ancestors)
        def add_interface(iface_name: str) -> None:
            if iface_name in class_.interface_names:
                return
            class_.interface_names.append(iface_name)
            # Recurse into parent interfaces (multi-parent support)
            iface = interfaces[iface_name]
            for parent_name in iface.parent_names:
                add_interface(parent_name)

        for iface_name in explicit_interfaces:
            add_interface(iface_name)

        classes[class_.name] = class_

        # Index fields
        for field_parts in re.findall(r"(.+[^=][\s|\*])([_A-Za-z][_A-Za-z0-9]*)\s*(=\s*[^;]+)?;", contents):
            attributes_and_type_str, name, default_str = field_parts

            # Parse attributes
            attributes = {}

            def convert_attribute(match: re.Match[str]) -> str:
                name, arguments = match.groups()
                if arguments is not None:
                    attributes[name] = [  # pylint: disable=cell-var-from-loop
                        argument.strip() for argument in arguments[1 : len(arguments) - 1].split(",")
                    ]
                else:
                    attributes[name] = []  # pylint: disable=cell-var-from-loop
                return ""

            field_type = re.sub(
                r"@([_A-Za-z][_A-Za-z0-9]*)(\([^\)]*\))?",
                convert_attribute,
                attributes_and_type_str,
            ).strip()

            # Add field to class
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

            arguments = []
            if arguments_str != "":
                for argument_str in arguments_str.split(","):
                    argument_parts = re.search(
                        r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)",
                        argument_str,
                    )
                    if argument_parts is not None:
                        arguments.append(Argument(argument_parts[2].strip(), argument_parts[1]))

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
            else:
                class_.methods[name] = Method(
                    name, return_type, is_return_self, is_virtual, arguments, class_.name, class_.name
                )

        # ==== Generate missing methods ====
        g = ""
        if parent_class is not None:
            # Init method
            field_needs_init = next(
                (field for field in class_.fields.values() if "init" in field.attributes or field.default is not None),
                None,
            )
            if class_.methods["init"].class_ != class_.name and field_needs_init is not None:
                class_.methods["init"].class_ = class_.name
                class_.methods["init"].arguments = [
                    Argument(field.name, field.type) for field in class_.fields.values() if "init" in field.attributes
                ]

                init_method = class_.methods["init"]
                parent_init_method = classes[parent_class.name].methods["init"]

                g += f"void _{class_.snake_name}_init("
                g += ", ".join(
                    [f"{class_.name}* this"]
                    + [f"{argument.type} {argument.name}" for argument in init_method.arguments]
                )
                g += ") {\n"
                g += f"    {parent_class.snake_name}_init("
                g += ", ".join(["this"] + [argument.name for argument in parent_init_method.arguments])
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

            # Deinit method
            field_needs_deinit = False
            for field in class_.fields.values():
                if field.class_ == class_.name:
                    if "deinit" in field.attributes:
                        field_needs_deinit = True
                        break
            if field_needs_deinit:
                class_.methods["deinit"].class_ = class_.name

                g += f"void _{class_.snake_name}_deinit({class_.name}* this) {{\n"
                for field in class_.fields.values():
                    if field.class_ == class_.name and "deinit" in field.attributes:
                        if len(field.attributes["deinit"]) > 0:
                            g += f"    {field.attributes['deinit'][0]}(this->{field.name});\n"
                        else:
                            for other_class in classes.values():
                                if field.type.startswith(other_class.name):
                                    g += f"    {other_class.snake_name}_free(this->{field.name});\n"
                                    break
                            else:
                                g += f"    free(this->{field.name});\n"
                class_with_deinit = find_class_for_method(classes[parent_class.name], "deinit")
                g += f"    _{class_with_deinit.snake_name}_deinit(({class_with_deinit.name}*)this);\n"
                g += "}\n\n"

            # Get attribute
            for field in class_.fields.values():
                if field.class_ == class_.name and ("get" in field.attributes or "prop" in field.attributes):
                    method_name = f"get_{field.name}"
                    class_.methods[method_name] = Method(
                        method_name, field.type, False, False, [], class_.name, class_.name
                    )

                    g += f"{field.type} _{class_.snake_name}_get_{field.name}({class_.name}* this) {{\n"
                    g += f"    return this->{field.name};\n"
                    g += "}\n\n"

            # Set attribute
            for field in class_.fields.values():
                if field.class_ == class_.name and ("set" in field.attributes or "prop" in field.attributes):
                    method_name = f"set_{field.name}"
                    class_.methods[method_name] = Method(
                        method_name, "void", False, False, [Argument(field.name, field.type)], class_.name, class_.name
                    )

                    g += f"void _{class_.snake_name}_set_{field.name}({class_.name}* this, "
                    g += f"{field.type} {field.name}) {{\n"
                    g += f"    this->{field.name} = {field.name};\n"
                    g += "}\n\n"

            # New method
            if not class_.is_abstract:
                init_method = class_.methods["init"]
                g += f"{class_.name}* {class_.snake_name}_new("
                if len(init_method.arguments) > 0:
                    g += ", ".join([f"{argument.type} {argument.name}" for argument in init_method.arguments])
                else:
                    g += "void"
                g += (
                    ") {\n"
                    + f"    {class_.name}* this = malloc(sizeof({class_.name}));\n"
                    + f"    this->vtbl = &_{class_.name}Vtbl;\n"
                    + f"    {class_.snake_name}_init("
                    + ", ".join(["this"] + [argument.name for argument in init_method.arguments])
                    + ");\n"
                    + "    return this;\n"
                    + "}\n\n"
                )

        # ==== Codegen ====

        # Class Vtbl struct
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
                    [f"{method.class_}* this"] + [f"{argument.type} {argument.name}" for argument in method.arguments]
                )
                c += ");\n"

        c += f"}} {class_.name}Vtbl;\n\n"

        # Extern vtbl instance declaration (needed for instanceof<> checks)
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

        # Class method forward defines
        if not class_.is_abstract:
            init_method = class_.methods["init"]
            c += f"{class_.name}* {class_.snake_name}_new("
            if len(init_method.arguments) > 0:
                c += ", ".join([f"{argument.type} {argument.name}" for argument in init_method.arguments])
            else:
                c += "void"
            c += ");\n"
        for method in class_.methods.values():
            if method.class_ == class_.name:
                c += (
                    f"{method.return_type} _{class_.snake_name}_{method.name}("
                    + ", ".join(
                        [f"{class_.name}* this"] + [f"{argument.type} {argument.name}" for argument in method.arguments]
                    )
                    + ");\n"
                )
        c += "\n"

        # Class Vtbl instance
        if not self.is_header and not class_.is_abstract:
            # Per-interface vtable instances and slot array
            if class_.interface_names:
                for iface_name in class_.interface_names:
                    iface = interfaces[iface_name]
                    c += f"static const {iface_name}Vtbl _{class_.name}{iface_name}Vtbl = {{\n"
                    for method in iface.methods.values():
                        c += f"    ({method.return_type}(*)(void*"
                        for argument in method.arguments:
                            c += f", {argument.type}"
                        c += "))"
                        if method.name in class_.methods:
                            # Class (or ancestor) provides the implementation
                            impl_class = to_snake_case(class_.methods[method.name].class_)
                            c += f"&_{impl_class}_{method.name},\n"
                        elif method.name in iface.default_bodies:
                            # Fall back to interface default
                            c += f"&_{iface.snake_name}_{method.name},\n"
                        else:
                            logging.error(
                                "Class %s implements %s but does not provide '%s' and there is no default",
                                class_.name,
                                iface_name,
                                method.name,
                            )
                            sys.exit(1)
                    c += "};\n\n"

                c += f"static const _InterfaceSlot _{class_.name}Interfaces[] = {{\n"
                for iface_name in class_.interface_names:
                    iface = interfaces[iface_name]
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

        # Class macro method wrappers
        for method in class_.methods.values():
            return_cast = ""
            if method.is_return_self:
                return_cast = f"({class_.name}*)"

            target = f"_{to_snake_case(method.class_)}_{method.name}"
            if method.is_virtual:
                target = f"(({class_.name}*)(this))->vtbl->{method.name}"

            c += f"#define {class_.snake_name}_{method.name}("
            c += ", ".join(["this"] + [argument.name for argument in method.arguments])
            c += f") {return_cast}{target}(({method.class_}*)(this)"
            for argument in method.arguments:
                for other_class in classes.values():
                    if argument.type.startswith(other_class.name):
                        c += f", ({other_class.name}*)({argument.name})"
                        break
                else:
                    c += f", ({argument.name})"
            c += ")\n"
        c += "\n"

        # Generated methods
        if not self.is_header:
            c += g

        return c


# MARK: Convert method
def convert_method(match: re.Match[str]) -> str:
    """Convert method define"""
    return_type, class_name, method_name, arguments = match.groups()

    if class_name not in classes:
        logging.error("Can't find class: %s", class_name)
        sys.exit(1)
    class_ = classes[class_name]
    method = class_.methods.get(method_name)
    if method is None:
        logging.error("Can't find method: %s::%s", class_name, method_name)
        sys.exit(1)

    if method.is_return_self:
        return_type = return_type.replace("Self", class_.name)

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


# MARK: Convert super call
def convert_super_call(match: re.Match[str]) -> str:
    """Convert super call"""
    parent_class_name, method_name, arguments = match.groups()

    if parent_class_name not in classes:
        logging.error("Can't find class: %s", parent_class_name)
        sys.exit(1)
    parent_class = classes[parent_class_name]
    method = parent_class.methods.get(method_name)
    if method is None:
        logging.error("Can't find method: %s::%s", parent_class_name, method_name)
        sys.exit(1)

    return (
        f"_{to_snake_case(method.class_)}_{method.name}(({method.class_}*)(this)"
        + (f", {arguments}" if len(arguments) > 0 else "")
        + ");"
    )


# MARK: Transpile text
def transpile_text(path: str, is_header: bool, text: str) -> str:
    """Transpile text"""

    if not is_header:
        # Add prelude
        text = '// @generated\n#include "prelude.h"\n#include "Object.hh"\n' + text

    # Remove #pragma once
    text = re.sub(r"#pragma once\n", "", text)
    # Convert includes
    text = re.sub(
        r"#include\s*[\"<](.+).hh[\">]",
        ConvertInclude(path),
        text,
    )

    # Convert @"..." string literals to string_new("...") early (any context)
    text = re.sub(r'@"([^"]*)"', lambda sm: f'string_new("{sm.group(1)}")', text)

    # ── Phase 1: Convert interface declarations (class IXxx, where X is uppercase) ──
    convert_iface = ConvertInterface(is_header)
    while True:
        m = re.search(
            r"class\s+(I[A-Z][_A-Za-z0-9]*)(\s*:\s*[_A-Za-z][_A-Za-z0-9,\s]*)?\s*\{",
            text,
        )
        if not m:
            break
        iface_name = m.group(1)
        supers_raw = m.group(2)
        # Find matching closing brace
        start = m.end() - 1  # points at opening {
        depth = 0
        pos = start
        while pos < len(text):
            if text[pos] == "{":
                depth += 1
            elif text[pos] == "}":
                depth -= 1
                if depth == 0:
                    break
            pos += 1
        body = text[start + 1 : pos]
        end = pos + 1
        if end < len(text) and text[end] == ";":
            end += 1
        replacement = convert_iface(iface_name, supers_raw, body)
        text = text[: m.start()] + replacement + text[end:]

    if not is_header:
        # ── Phase 2: Pre-scan interface default method bodies to populate default_bodies flags ──
        # This must run before Phase 3 (class declarations) so vtable construction knows which
        # methods have defaults. Both IFoo::method and Foo::method syntax are accepted.
        for iface_name, iface in interfaces.items():
            impl_names = [iface_name]
            # Also accept Foo::method as shorthand for IFoo::method
            if iface_name.startswith("I") and len(iface_name) > 1 and iface_name[1].isupper():
                impl_names.append(iface_name[1:])
            for impl_name in impl_names:
                for dm in re.finditer(
                    rf"[_A-Za-z][_A-Za-z0-9 ]*[\**|\s+]\s*{re.escape(impl_name)}::([_A-Za-z][_A-Za-z0-9]*)\(",
                    text,
                ):
                    method_name = dm.group(1)
                    if method_name in iface.methods:
                        iface.default_bodies[method_name] = ""  # flag: default exists

    # ── Phase 3: Convert class declarations ──
    convert_class = ConvertClass(is_header)
    while True:
        m = re.search(
            r"class\s+([_A-Za-z][_A-Za-z0-9]*)(\s*:\s*[_A-Za-z][_A-Za-z0-9,\s]*)?\s*\{",
            text,
        )
        if not m:
            break
        class_name = m.group(1)
        supers_raw = m.group(2)
        # Find matching closing brace
        start = m.end() - 1
        depth = 0
        pos = start
        while pos < len(text):
            if text[pos] == "{":
                depth += 1
            elif text[pos] == "}":
                depth -= 1
                if depth == 0:
                    break
            pos += 1
        body = text[start + 1 : pos]
        end = pos + 1
        if end < len(text) and text[end] == ";":
            end += 1
        replacement = convert_class(class_name, supers_raw, body)
        text = text[: m.start()] + replacement + text[end:]

    if not is_header:
        # ── Phase 4: Replace interface default method bodies with static functions (in-place) ──
        # Supports both IFoo::method_name and Foo::method_name (shorthand for IFoo).
        # The replacement happens at the same text position so static functions always
        # appear before any vtable inits that reference them.
        while True:
            found = False
            for cur_iface_name, cur_iface in interfaces.items():
                impl_names = [cur_iface_name]
                if cur_iface_name.startswith("I") and len(cur_iface_name) > 1 and cur_iface_name[1].isupper():
                    impl_names.append(cur_iface_name[1:])
                for impl_name in impl_names:
                    dm = re.search(
                        rf"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*{re.escape(impl_name)}::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*\{{",
                        text,
                    )
                    if dm:
                        ret_type, method_name, arguments_str = dm.group(1), dm.group(2), dm.group(3)
                        if method_name not in cur_iface.methods:
                            logging.error("Interface %s has no method '%s'", cur_iface_name, method_name)
                            sys.exit(1)
                        # Find matching closing brace
                        dstart = dm.end() - 1
                        ddepth = 0
                        dpos = dstart
                        while dpos < len(text):
                            if text[dpos] == "{":
                                ddepth += 1
                            elif text[dpos] == "}":
                                ddepth -= 1
                                if ddepth == 0:
                                    break
                            dpos += 1
                        dend = dpos + 1
                        body_text = text[dstart + 1 : dpos]

                        # Parse arguments
                        def_arguments = []
                        if arguments_str.strip():
                            for arg_str in arguments_str.split(","):
                                arg_parts = re.search(
                                    r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)",
                                    arg_str,
                                )
                                if arg_parts:
                                    def_arguments.append(Argument(arg_parts[2].strip(), arg_parts[1]))

                        snake_iface = cur_iface.snake_name
                        fn_code = f"static {ret_type.strip()} _{snake_iface}_{method_name}(void* this"
                        for arg in def_arguments:
                            fn_code += f", {arg.type} {arg.name}"
                        fn_code += ") {\n"
                        # Inject vtbl lookup
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
                        # Transform method(this, ...) calls to vtbl dispatch in body
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
                        break
                if found:
                    break
            if not found:
                break

        # ── Phase 5: Convert method defines ──
        text = re.sub(
            r"([_A-Za-z][_A-Za-z0-9 ]*[\**|\s+])\s*([_A-Za-z][_A-Za-z0-9]*)::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*{",
            convert_method,
            text,
        )

        # ── Phase 6: Convert super calls ──
        text = re.sub(
            r"([_A-Za-z][_A-Za-z0-9]*)::([_A-Za-z][_A-Za-z0-9]*)\(([^\)]*)\)\s*;",
            convert_super_call,
            text,
        )

        # ── Phase 7: Convert cast<Iface>(x) to inline fat-pointer construction ──
        # Collect all interfaces used so we can emit per-interface lookup functions.
        ifaces_for_lookup: Dict[str, str] = {}
        for cast_m in re.finditer(r"cast<([_A-Za-z][_A-Za-z0-9]*)>", text):
            cast_iface_name = cast_m.group(1)
            if cast_iface_name in interfaces and cast_iface_name not in ifaces_for_lookup:
                ifaces_for_lookup[cast_iface_name] = interfaces[cast_iface_name].snake_name

        if ifaces_for_lookup:
            lookup_code = ""
            for lk_iface_name, lk_snake in ifaces_for_lookup.items():
                lookup_code += f"static {lk_iface_name} _cast_{lk_iface_name}(void* obj) {{\n"
                lookup_code += "    Object* _obj = (Object*)obj;\n"
                lookup_code += "    const _InterfaceSlot* s = _obj->vtbl->interfaces;\n"
                lookup_code += f"    const {lk_iface_name}Vtbl* vtbl = NULL;\n"
                lookup_code += "    if (s) for (; s->id; s++) {\n"
                lookup_code += (
                    f"        if (s->id == _{lk_iface_name}_ID) {{ vtbl = (const {lk_iface_name}Vtbl*)s->vtbl; break; }}\n"
                )
                lookup_code += "    }\n"
                lookup_code += f"    return ({lk_iface_name}){{ .obj = _obj, .vtbl = vtbl }};\n"
                lookup_code += "}\n\n"
            insert_marker = re.search(r"(}\n\n)(?=[a-zA-Z#])", text)
            if insert_marker:
                ins_pos = insert_marker.end()
                text = text[:ins_pos] + lookup_code + text[ins_pos:]
            else:
                text += lookup_code

        # cast<Iface>(expr) - brace-aware to handle nested parens in expr
        while True:
            cast_m = re.search(r"cast<([_A-Za-z][_A-Za-z0-9]*)>\(", text)
            if not cast_m:
                break
            cast_iface_name = cast_m.group(1)
            if cast_iface_name not in interfaces:
                break
            cstart = cast_m.end() - 1
            cdepth = 0
            cpos = cstart
            while cpos < len(text):
                if text[cpos] == "(":
                    cdepth += 1
                elif text[cpos] == ")":
                    cdepth -= 1
                    if cdepth == 0:
                        break
                cpos += 1
            obj_expr = text[cstart + 1 : cpos].strip()
            cend = cpos + 1
            cast_repl = f"_cast_{cast_iface_name}((void*)({obj_expr}))"
            text = text[: cast_m.start()] + cast_repl + text[cend:]

        # ── Phase 8: Convert instanceof<X>(expr) to bool check ──
        types_for_instanceof: List[str] = []
        for inst_m2 in re.finditer(r"instanceof<([_A-Za-z][_A-Za-z0-9]*)>", text):
            type_name = inst_m2.group(1)
            if type_name not in types_for_instanceof:
                types_for_instanceof.append(type_name)

        if types_for_instanceof:
            instanceof_code = ""
            # Add forward declarations and extern declarations for class vtables
            for type_name in types_for_instanceof:
                if type_name in classes:
                    instanceof_code += f"typedef struct {type_name}Vtbl {type_name}Vtbl;\n"
                    instanceof_code += f"extern {type_name}Vtbl _{type_name}Vtbl;\n"
            instanceof_code += "\n"

            for type_name in types_for_instanceof:
                if type_name in interfaces:
                    instanceof_code += f"static bool _instanceof_{type_name}(void* obj) {{\n"
                    instanceof_code += "    Object* _obj = (Object*)obj;\n"
                    instanceof_code += "    const _InterfaceSlot* s = _obj->vtbl->interfaces;\n"
                    instanceof_code += "    if (!s) return false;\n"
                    instanceof_code += "    for (; s->id; s++)\n"
                    instanceof_code += f"        if (s->id == _{type_name}_ID) return true;\n"
                    instanceof_code += "    return false;\n"
                    instanceof_code += "}\n\n"
                elif type_name in classes:
                    instanceof_code += f"static bool _instanceof_{type_name}(void* obj) {{\n"
                    instanceof_code += "    Object* _obj = (Object*)obj;\n"
                    instanceof_code += f"    return _obj->vtbl == (void*)&_{type_name}Vtbl;\n"
                    instanceof_code += "}\n\n"
                else:
                    logging.warning(f"Type '{type_name}' used in instanceof<> is not defined as a class or interface")

            # Try to find a good insertion point:
            # 1. If there's a main function, insert before it
            # 2. Otherwise, look for the first function definition after struct definitions
            main_marker = re.search(r"\bint\s+main\s*\(", text)
            if main_marker:
                ins_pos2 = main_marker.start()
            else:
                # Find first function definition (starts with a type name followed by * and function name)
                # This pattern looks for lines that might be function implementations
                func_marker = re.search(r"\n(?:typedef|extern|#|struct|}\s*;)", text)
                if func_marker:
                    # Find the next function definition after this marker
                    rest = text[func_marker.end():]
                    # Look for patterns like: return_type function_name( or return_type *function_name(
                    func_impl = re.search(r"\n[_A-Za-z*][_A-Za-z0-9*\s]*\s+\**[_A-Za-z][_A-Za-z0-9]*\s*\(", rest)
                    if func_impl:
                        ins_pos2 = func_marker.end() + func_impl.start()
                    else:
                        ins_pos2 = len(text)
                else:
                    ins_pos2 = len(text)
            text = text[:ins_pos2] + instanceof_code + text[ins_pos2:]

        # instanceof<X>(expr) - brace-aware
        while True:
            inst_m = re.search(r"instanceof<([_A-Za-z][_A-Za-z0-9]*)>\(", text)
            if not inst_m:
                break
            inst_type_name = inst_m.group(1)
            if inst_type_name not in interfaces and inst_type_name not in classes:
                logging.error(f"Type '{inst_type_name}' used in instanceof<> is not defined as a class or interface")
                text = text[:inst_m.start()] + "false" + text[inst_m.end():]
                continue
            istart = inst_m.end() - 1
            idepth = 0
            ipos = istart
            while ipos < len(text):
                if text[ipos] == "(":
                    idepth += 1
                elif text[ipos] == ")":
                    idepth -= 1
                    if idepth == 0:
                        break
                ipos += 1
            inst_expr = text[istart + 1 : ipos].strip()
            iend = ipos + 1
            text = text[: inst_m.start()] + f"_instanceof_{inst_type_name}({inst_expr})" + text[iend:]

    return text


# MARK: Main
if __name__ == "__main__":
    logging.basicConfig(level=logging.INFO, format="[%(levelname)s] %(message)s")

    # Parse arguments
    parser = argparse.ArgumentParser()
    parser.add_argument("file", help="cContinue file", nargs="*")
    parser.add_argument("--output", "-o", help="Output file", required=False)
    parser.add_argument("--include", "-I", help="Include headers", action="append", default=[])
    parser.add_argument("--source", "-S", help="Only run transpile step", action="store_true")
    parser.add_argument("--compile", "-c", help="Only run transpile and compile steps", action="store_true")
    parser.add_argument("--run", "-r", help="Run linked binary", action="store_true")
    parser.add_argument("--run-leaks", "-R", help="Run linked binary with memory leaks checks", action="store_true")
    args = parser.parse_args()

    # Get paths
    cc = os.environ["CC"] if "CC" in os.environ else "gcc"
    include_paths = [".", f"{os.path.dirname(__file__)}/std"] + args.include
    source_paths = args.file
    if not args.source and not args.compile:
        for path in os.listdir(f"{os.path.dirname(__file__)}/std"):
            if path.endswith(".c") or path.endswith(".cc"):
                source_paths.append(f"{os.path.dirname(__file__)}/std/{path}")

    # Compile objects
    object_paths = []
    for path in source_paths:
        # Skip already compiled object files
        if path.endswith(".o"):
            object_paths.append(path)
            continue

        # Transpile file
        source_path = path
        if path.endswith(".hh") or path.endswith(".cc"):
            source_path = (
                args.output
                if args.output is not None
                else path.replace(".cc", ".c").replace(".hh", ".h") if args.source else tempfile.mktemp(".c")
            )
            ConvertInclude.processed_includes = []
            # Reset global state so each file gets consistent interface IDs
            next_interface_id = 1  # type: ignore[assignment]
            interfaces = {}  # type: ignore[assignment]
            classes = {}  # type: ignore[assignment]
            file_write(source_path, transpile_text(path, path.endswith(".hh"), file_read(path)))
            if args.source:
                sys.exit(0)

        # Compile file
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

    # Link objects
    exe_path = (
        args.output
        if args.output is not None
        else args.file[0].replace(".cc", ".exe" if platform.system() == "Windows" else "")
    )
    subprocess.run(
        [cc] + object_paths + ["-o", exe_path],
        check=True,
    )

    # Run executable
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
